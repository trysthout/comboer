use std::{sync::Arc, time::SystemTime};

use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BufMut};
use dashmap::DashMap;
use regex::bytes::Regex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

///
/// 1	2	    1	        1	        2	            2	            2	    Nlen	    2	    Elen
/// 120	PortNo	NodeType	Protocol	HighestVersion	LowestVersion	Nlen	NodeName	Elen	Extra
/// PortNo
/// The port number on which the node accept connection requests.
///
/// NodeType
/// 77 = normal Erlang node, 72 = hidden node (C-node), ...
///
/// Protocol
/// 0 = TCP/IPv4, ...
///
/// HighestVersion
/// The highest distribution protocol version this node can handle. The value in OTP 23 and later is 6. Older nodes only support version 5.
///
/// LowestVersion
/// The lowest distribution version that this node can handle. The value in OTP 25 and later is 6 as support for connections to nodes older than OTP 23 has been dropped.
///
/// Nlen
/// The length (in bytes) of field NodeName.
///
/// NodeName
/// The node name as an UTF-8 encoded string of Nlen bytes.
///
/// Elen
/// The length of field Extra.
///
/// Extra
/// Extra field of Elen bytes.
///
#[allow(dead_code)]
const ALIVE2_REQ: u8 = 120;
#[allow(dead_code)]
const ALIVE2_X_RESP: u8 = 118;
#[allow(dead_code)]
const ALIVE2_RESP: u8 = 121;
#[allow(dead_code)]
const PORT_PLEASE2_REQ: u8 = 1;
#[allow(dead_code)]
const PORT2_RESP: u8 = 119;
#[allow(dead_code)]
const NAMES_REQ: u8 = 110;
#[allow(dead_code)]
const KILL_REQ: u8 = 107;
#[allow(dead_code)]
const STOP_REQ: u8 = 115;

#[derive(Debug)]
pub struct RegisterNodeReq {
    pub port_no: u16,
    pub hidden: bool,
    pub protocol: u8,
    pub highest_version: u16,
    pub lowest_version: u16,
    pub node_name: String,
    pub extra: Vec<u8>,
}

// Result = 0 -> ok, result > 0 -> error.
#[derive(Debug)]
pub struct RegisterNodeXResp {
    pub result: u8,
    pub creation: u32,
}

// Result = 0 -> ok, result > 0 -> error.
pub struct RegisterNodeResp {
    pub result: u8,
    pub creation: u16,
}

// A node unregisters itself from the EPMD by closing the TCP connection to EPMD established when the node was registered.
pub struct UnregisterNode;

pub struct GetDistPortReq(Vec<u8>);

pub struct GetDistPortResp {
    pub result: u8,
    pub info: Option<RegisterNodeReq>,
}

pub struct GetAllRegisteredNamesReq;

#[derive(Debug)]
pub struct GetAllRegisteredNamesResp {
    pub epmd_port: u32,
    pub nodes: Vec<NodeInfo>,
}

#[derive(Debug)]
pub struct NodeInfo {
    pub name: String,
    pub port: u16,
}

#[derive(Debug)]
pub struct EpmdClient {
    stream: TcpStream,
}

impl EpmdClient {
    pub async fn new<T: ToSocketAddrs>(addr: T) -> Result<Self, anyhow::Error> {
        let stream = TcpStream::connect(addr).await?;
        let _ = stream.set_nodelay(true);

        Ok(Self { stream })
    }

    pub async fn req_names(&mut self) -> Result<GetAllRegisteredNamesResp, anyhow::Error> {
        self.stream.write_all(&[0, 1, NAMES_REQ]).await?;
        let mut buf = vec![0; 512];
        // epmd port
        self.stream.read_exact(&mut buf[..4]).await?;
        let epmd_port = BigEndian::read_u32(&buf[..4]);
        let mut nodes = Vec::with_capacity(10);
        let re = Regex::new(r"name (\S+) at port (\d+)\n")?;

        loop {
            let n = self.stream.read(&mut buf).await?;
            if n == 0 {
                break;
            }

            for (_, [name, port]) in re.captures_iter(&buf[..n]).map(|m| m.extract()) {
                nodes.push(NodeInfo {
                    name: String::from_utf8_lossy(name).to_string(),
                    port: String::from_utf8_lossy(port).parse::<u16>().unwrap(),
                });
            }
        }

        Ok(GetAllRegisteredNamesResp { epmd_port, nodes })
    }

    pub async fn register_node(
        &mut self,
        port: u16,
        node_name: &str,
    ) -> Result<RegisterNodeXResp, anyhow::Error> {
        let length = 1 + 2 + 1 + 1 + 2 + 2 + 2 + node_name.len() + 2;
        let mut buf = Vec::with_capacity(2 + length);
        buf.put_u16(length as u16);
        buf.put_u8(120);
        buf.put_u16(port);
        buf.put_u8(77);
        buf.put_u8(0);
        buf.put_u16(6);
        buf.put_u16(6);
        buf.put_u16(node_name.len() as u16);
        buf.extend_from_slice(node_name.as_bytes());
        buf.put_u16(0);

        // Send request
        self.stream.write_all(&buf).await?;

        //// Wait resp
        self.stream.read_exact(&mut buf[0..6]).await?;
        // result
        let result = buf[1];
        // creation
        let creation = BigEndian::read_u32(&buf[2..6]);
        Ok(RegisterNodeXResp { result, creation })
    }
}

#[derive(Debug, Clone)]
pub struct EpmdServer {
    nodes: Arc<DashMap<String, RegisterNodeReq>>,
}

impl EpmdServer {
    pub async fn new() -> EpmdServer {
        Self {
            nodes: Arc::new(DashMap::new()),
        }
    }

    pub async fn accept<T: ToSocketAddrs>(&mut self, addr: T) -> Result<(), anyhow::Error> {
        let listener = TcpListener::bind(addr).await?;
        let me = &mut self.clone();
        loop {
            let (stream, _) = listener.accept().await?;
            let mut me = me.clone();
            tokio::spawn(async move { me.handle_connection(stream).await });
        }
    }

    async fn handle_connection(&mut self, mut stream: TcpStream) -> Result<(), anyhow::Error> {
        let mut buf = vec![0; 512];
        stream.read_exact(&mut buf[..2]).await?;
        let length = BigEndian::read_u16(&buf[..2]) as usize;
        if length > buf.len() {
            buf.resize(length, 0)
        }
        stream.read_exact(&mut buf[..length]).await?;
        match buf[0] {
            ALIVE2_REQ => {
                self.decode_alive_req(&buf);
                let n = self.encode_alive_x_resp(&mut buf[0..]);
                stream.write_all(&buf[..n]).await?;
                Ok(())
            }

            NAMES_REQ => {
                let n = self.encode_names_resp(&mut buf[0..]);
                stream.write_all(&buf[..n]).await?;
                Ok(())
            }

            x => Err(anyhow::anyhow!("Unsupported tag, got: {}", x)),
        }
    }

    fn decode_alive_req(&mut self, mut buf: &[u8]) {
        // 120
        buf.get_u8();

        let port_no = buf.get_u16();

        let hidden = buf.get_u8() == 72;

        let protocol = buf.get_u8();
        let highest_version = buf.get_u16();
        let lowest_version = buf.get_u16();
        // node_name length
        let n = buf.get_u16() as usize;
        let node_name = String::from_utf8_lossy(&buf[0..n]).to_string();
        buf.advance(n);
        // extra length
        let n = buf.get_u16() as usize;
        let extra_bytes = &buf[0..n];
        buf.advance(n);

        let req = RegisterNodeReq {
            port_no,
            hidden,
            protocol,
            highest_version,
            lowest_version,
            node_name: node_name.clone(),
            extra: extra_bytes.to_vec(),
        };

        self.nodes.insert(node_name, req);
    }

    fn encode_alive_x_resp(&self, mut buf: &mut [u8]) -> usize {
        buf.put_u8(ALIVE2_X_RESP);
        buf.put_u8(0);
        buf.put_u32(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
        );

        1 + 1 + 4
    }

    fn encode_names_resp(&self, mut buf: &mut [u8]) -> usize {
        // Default port is 4369
        // TODO: port as a parameter
        buf.put_u32(4369);

        let mut length = 1;
        for node in self.nodes.iter() {
            // The format is: io:format("name ~ts at port ~p~n", [NodeName, Port]).
            let s = format!("name {} at port {}\n", node.key(), node.port_no);
            buf.put_slice(s.as_bytes());
            length += s.len();
        }

        length
    }

    pub fn get_register_nodes(&self) -> Arc<DashMap<String, RegisterNodeReq>> {
        self.nodes.clone()
    }
}

#[derive(Debug, Default)]
pub struct EpmdCodec {
    nodes: Arc<DashMap<String, RegisterNodeReq>>,
    port: u16,
}

impl EpmdCodec {
    pub fn new(port: u16) -> EpmdCodec {
        Self {
            port,
            nodes: Arc::new(DashMap::new()),
        }
    }

    fn decode_alive_x_resp(&mut self, buf: &mut bytes::BytesMut) -> RegisterNodeXResp {
        // 118
        buf.get_u8();
        // result
        let result = buf.get_u8();
        // creation
        let creation = buf.get_u32();
        RegisterNodeXResp { result, creation }
    }

    fn encode_alive_resp(&self, code: u8, buf: &mut bytes::BytesMut) {
        buf.put_u8(ALIVE2_X_RESP);
        buf.put_u8(code);
        buf.put_u32(1)
    }

    fn encode_port_please2_req(&self, node: &RegisterNodeReq, buf: &mut bytes::BytesMut) {
        buf.put_u8(PORT_PLEASE2_REQ);
        buf.put_u8(0);
        buf.put_u16(node.port_no);
        let hidden = if node.hidden { 72 } else { 77 };
        buf.put_u8(hidden);
        buf.put_u16(node.highest_version);
        buf.put_u16(node.lowest_version);
        buf.put_u16(node.node_name.len() as u16);
        buf.extend_from_slice(node.node_name.as_bytes());
        buf.put_u16(node.extra.len() as u16);
        buf.extend_from_slice(&node.extra);
    }

    fn encode_names_req(&self, buf: &mut bytes::BytesMut) {
        buf.put_u8(NAMES_REQ);
    }

    fn decode_names_resp(&self, buf: &mut bytes::BytesMut) {
        let _port = buf.get_u32();
        let data = buf.split();
        println!("names {:?}", String::from_utf8_lossy(&data));
    }

    fn encode_names_resp(&self, port: u32, buf: &mut bytes::BytesMut) {
        buf.put_u32(port);

        for node in self.nodes.iter() {
            // The format is: io:format("name ~ts at port ~p~n", [NodeName, Port]).
            let s = format!("name {} at port {}\n", node.key(), node.port_no);
            buf.extend_from_slice(s.as_bytes());
        }
    }
}
