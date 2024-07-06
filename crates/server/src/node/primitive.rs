use std::{
    fmt::Debug,
    net::{Ipv4Addr, SocketAddrV4},
    time::SystemTime,
};

use byteorder::{BigEndian, ByteOrder};
use motore::Service;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use proto::{
    EpmdClient,
    handshake::{HandshakeCodec, HandshakeVersion, Status},
};

use crate::{
    Dispatcher, MatchId, node::get_short_hostname, ProcessContext, RawMsg, Request, Response,
};
use crate::node::conn::Connection;

use super::Error;

#[derive(Debug, Clone)]
pub struct NodeAsClient<C> {
    pub is_tls: bool,
    pub node_name: String,
    pub cookie: String,
    pub creation: u32,
    epmd_addr: &'static str,
    // internal_tx: Option<UnboundedSender<CtrlMsg>>,
    dispatcher: Dispatcher<C, Vec<u8>>,
}

impl<C> NodeAsClient<C>
where
    C: Clone + Debug + Sync + Send,
{
    pub fn new(node_name: String, cookie: String, epmd_addr: &'static str) -> Self {
        let creation = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        Self {
            node_name,
            cookie,
            creation,
            is_tls: false,
            // internal_tx: None,
            dispatcher: Dispatcher::new(),
            epmd_addr,
        }
    }

    pub fn add_matcher<M>(self, matcher: M) -> Self
    where
        M: Service<ProcessContext<C>, Request<Vec<u8>>, Response = Response<RawMsg>, Error = Error>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        let matcher_id = MatchId::next();
        self.dispatcher.add_matcher(matcher_id, matcher);
        Self {
            is_tls: self.is_tls,
            node_name: self.node_name,
            cookie: self.cookie,
            creation: self.creation,
            // internal_tx: self.internal_tx,
            dispatcher: self.dispatcher,
            epmd_addr: self.epmd_addr,
        }
    }

    pub async fn connect_local_by_name(
        self,
        remote_node_name: &str,
    ) -> Result<Connection<TcpStream, C>, Error> {
        let mut epmd_client = EpmdClient::new(self.epmd_addr).await?;
        let nodes = epmd_client.req_names().await?.nodes;
        let node = nodes
            .iter()
            .find(|&n| n.name == remote_node_name)
            .ok_or_else(|| anyhow::anyhow!("Not found node_name {:?}", remote_node_name))?;

        println!("node {:?}", node);

        //let mut node = Node::new(false);
        let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), node.port);
        let mut stream = TcpStream::connect(addr).await.unwrap();
        let _ = stream.set_nodelay(true);
        let handshake_codec = HandshakeCodec::new(self.node_name.clone(), self.cookie.clone());
        let _ = self.client_handshake(handshake_codec, &mut stream).await?;
        // let (internal_tx, internal_rx) = unbounded_channel::<CtrlMsg>();
        // self.internal_tx = Some(internal_tx.clone());

        let cx = ProcessContext::with_dispatcher(
            self.node_name.clone(),
            self.creation,
            self.dispatcher.clone(),
        );

        let conn = Connection::new(stream, cx.clone());
        Ok(conn)
    }

    async fn client_handshake(
        &self,
        mut handshake_codec: HandshakeCodec,
        stream: &mut TcpStream,
    ) -> Result<(), Error> {
        let mut buf = vec![0; 512];
        let n = handshake_codec.encode_v6_name(&mut &mut buf[..]);
        stream.write_all(&buf[..n]).await?;

        let header_length = header_length(self.is_tls);

        loop {
            stream.read_exact(&mut buf[0..header_length]).await?;
            let length = read_length(header_length, &buf[0..header_length]);
            // ERLANG_TICK
            if length == 0 {
                stream.write_all(&[0; 4]).await?;
                continue;
            }

            if length > buf.len() {
                buf.resize(length, 0);
            }

            stream.read_exact(&mut buf[0..length]).await?;
            match buf[0] {
                b's' => {
                    handshake_codec.decode_status(&buf[..length]);
                    if handshake_codec.status == Status::NotAllowed
                        || handshake_codec.status == Status::Nok
                    {
                        return Err(Error::HandshakeFailed(
                            "Remote node return Status::NowAllowed or Status::Nok".to_string(),
                        ));
                    }
                }
                b'n' => {
                    handshake_codec.decode_v5_challenge(&buf[..length]);
                    let n = handshake_codec.encode_challenge_reply(&mut &mut buf[..]);
                    stream.write_all(&buf[..n]).await?;
                }
                b'N' => {
                    handshake_codec.decode_v6_challenge(&buf[..length]);

                    if handshake_codec.version == HandshakeVersion::V5 {
                        let n = handshake_codec.encode_complement(&mut &mut buf[..]);
                        stream.write_all(&buf[..n]).await?;
                    }

                    let n = handshake_codec.encode_challenge_reply(&mut &mut buf[..]);
                    stream.write_all(&buf[..n]).await?;
                }
                b'a' => {
                    let is_valid = handshake_codec.decode_challenge_ack(&buf[..length]);
                    if !is_valid {
                        return Err(Error::HandshakeFailed("incorrect digest".to_string()));
                    }
                    return Ok(());
                }
                x => return Err(Error::UnsupportedTag(x)),
            }
        }
    }
}

#[derive(Debug)]
pub struct ServerConfig {}

#[derive(Debug)]
pub struct NodeAsServer<C> {
    pub is_tls: bool,
    pub node_name: String,
    pub cookie: String,
    pub creation: u32,
    epmd_addr: &'static str,
    dispatcher: Dispatcher<C>,
}

impl<C> NodeAsServer<C>
where
    C: Debug + Clone + Send + Sync + 'static,
{
    pub fn new(node_name: String, cookie: String, epmd_addr: &'static str) -> Self {
        let creation = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        Self {
            node_name,
            cookie,
            creation,
            is_tls: false,
            dispatcher: Dispatcher::new(),
            epmd_addr,
        }
    }

    pub fn add_matcher<M>(self, matcher: M) -> Self
    where
        M: Service<
                ProcessContext<C>,
                Request<Vec<u8>>,
                Response = Response<RawMsg>,
                Error = crate::Error,
            > + Clone
            + Send
            + Sync
            + 'static,
    {
        let matcher_id = MatchId::next();
        self.dispatcher.add_matcher(matcher_id, matcher);
        Self {
            node_name: self.node_name,
            cookie: self.cookie,
            creation: self.creation,
            is_tls: self.is_tls,
            dispatcher: self.dispatcher,
            epmd_addr: self.epmd_addr,
        }
    }

    pub async fn listen(mut self) -> Result<(), Error> {
        let listener = TcpListener::bind("0.0.0.0:0").await?;
        let port = listener.local_addr()?.port();
        let mut epmd_client = EpmdClient::new(self.epmd_addr).await?;
        let resp = epmd_client.register_node(port, &self.node_name).await?;

        if resp.result != 0 {
            return Err(Error::Anyhow(anyhow::anyhow!(
                "Faild register node to epmd, maybe {:?} is still in use",
                self.node_name
            )));
        }

        self.node_name.push('@');
        self.node_name.push_str(&get_short_hostname());
        let handshake_codec = HandshakeCodec::new(self.node_name.clone(), self.cookie.clone());

        loop {
            let node_name = self.node_name.clone();
            //let is_tls = self.is_tls;
            let creation = self.creation;
            let handshake_codec = handshake_codec.clone();
            let dispatcher = self.dispatcher.clone();

            tokio::select! {
                res = listener.accept() => {
                    match res {
                        Ok((mut stream, _)) => {
                            tokio::spawn(
                                async move {
                                    if let Err(err) = Self::server_handshake(self.is_tls, handshake_codec.clone(), &mut stream).await {
                                        println!("error {:?}", err);
                                        return;
                                    }

                                    let cx = ProcessContext::with_dispatcher(node_name, creation, dispatcher);
                                    let mut conn = Connection::new(&mut stream, cx);
                                    let _ = conn.serving().await;
                                }
                            );
                        },

                        Err(_err) => {
                        }
                    }
                },
            }
        }
    }

    async fn server_handshake(
        is_tls: bool,
        mut handshake_codec: HandshakeCodec,
        stream: &mut TcpStream,
    ) -> Result<(), Error> {
        let mut buf = vec![0; 512];
        let header_length = if is_tls { 4 } else { 2 };
        loop {
            stream.read_exact(&mut buf[0..header_length]).await?;
            let length = read_length(header_length, &buf);
            // ERLANG_TICK
            if length == 0 {
                return Ok(());
            }
            if length > buf.len() {
                buf.resize(length, 0);
            }
            stream.read_exact(&mut buf[0..length]).await?;
            match buf[0] {
                b's' => {
                    handshake_codec.decode_status(&buf[..length]);
                    if handshake_codec.status == Status::NotAllowed
                        || handshake_codec.status == Status::Nok
                    {
                        return Ok(());
                    }
                }
                b'n' => {
                    handshake_codec.decode_v5_name(&buf[..length]);
                    let n = handshake_codec.encode_status(&mut &mut buf[..]);
                    stream.write_all(&buf[..n]).await?;
                    let n = if handshake_codec.version == HandshakeVersion::V6 {
                        handshake_codec.encode_v6_challenge(&mut &mut buf[..])
                    } else {
                        handshake_codec.encode_v5_challenge(&mut &mut buf[..])
                    };
                    stream.write_all(&buf[..n]).await?;
                }
                b'N' => {
                    handshake_codec.decode_v6_name(&buf[..length]);
                    let n = handshake_codec.encode_status(&mut &mut buf[..]);
                    stream.write_all(&buf[..n]).await?;

                    let n = handshake_codec.encode_v6_challenge(&mut &mut buf[..]);
                    stream.write_all(&buf[..n]).await?;
                }
                b'c' => {
                    handshake_codec.decode_complement(&buf[..length]);
                }
                b'r' => {
                    return if handshake_codec.decode_challenge_reply(&buf[..length]) {
                        let n = handshake_codec.encode_challenge_ack(&mut &mut buf[..]);
                        stream.write_all(&buf[..n]).await?;
                        Ok(())
                    } else {
                        Err(Error::HandshakeFailed("invalid reply".to_string()))
                    }
                }
                x => return Err(Error::UnsupportedTag(x)),
            }
        }
    }
}

#[inline]
fn header_length(is_tls: bool) -> usize {
    if is_tls {
        4
    } else {
        2
    }
}

#[inline]
fn read_length(header_length: usize, buf: &[u8]) -> usize {
    if header_length == 4 {
        BigEndian::read_u32(buf) as usize
    } else {
        BigEndian::read_u16(buf) as usize
    }
}
