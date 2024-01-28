use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

use byteorder::{BigEndian, ByteOrder};

use proto::{
    etf::term::{self}, handshake::{HandshakeCodec, HandshakeVersion, Status}, Ctrl, CtrlMsg, Encoder, EpmdClient, Len
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpListener, TcpStream, ToSocketAddrs,
    },
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
};

use crate::node::get_short_hostname;

use super::{AsServer, Error, Handler};

#[derive(Debug, Clone)]
pub struct NodeAsClient {
    pub is_tls: bool,
    pub handshaked: bool,
    pub node_name: String,
    pub cookie: String,
    pub creation: u32,
    internal_tx: Option<UnboundedSender<(CtrlMsg, oneshot::Sender<()>)>>,
}

impl NodeAsClient {
    pub fn new(node_name: String, cookie: String) -> Self {
        let creation = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        Self {
            node_name,
            cookie,
            creation,
            handshaked: false,
            is_tls: false,
            internal_tx: None,
        }
    }

    pub async fn connect_local_by_name<H: Handler + Send + 'static, A: ToSocketAddrs + Send>(
        mut self,
        epmd_addr: A,
        remote_node_name: &str,
        handler: H,
    ) -> Result<(Self, oneshot::Receiver<H::Error>), Error> {
        let mut epmd_client = EpmdClient::new(epmd_addr).await?;
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
        self.client_handshake(handshake_codec, &mut stream).await?;
        let (internal_tx, internal_rx) = unbounded_channel::<(CtrlMsg, oneshot::Sender<()>)>();
        self.internal_tx = Some(internal_tx.clone());
        let node = NodePrimitive::new(self.node_name.clone(), self.creation, internal_tx);
        let (err_tx, err_rx) = oneshot::channel::<H::Error>();
        tokio::spawn(async move {
            if let Err(err) = Session::new(internal_rx)
                .run(stream, handler, Some(node))
                .await
            {
                let _ = err_tx.send(err);
            }
        });

        Ok((self, err_rx))
    }

    async fn client_handshake(
        &mut self,
        mut handshake_codec: HandshakeCodec,
        stream: &mut TcpStream,
    ) -> Result<(), Error> {
        let mut buf = vec![0; 512];
        let n = handshake_codec.encode_v6_name(&mut &mut buf[..]);
        stream.write_all(&buf[..n]).await?;

        let header_length = header_length(self.is_tls, self.handshaked);

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
                    self.handshaked = true;
                    return Ok(());
                }
                x => return Err(Error::UnsupportedTag(x)),
            }
        }
    }

    pub async fn send(&mut self, dist: CtrlMsg) -> Result<(), Error> {
        let (wait_tx, wait_rx) = oneshot::channel::<()>();
        self.internal_tx
            .as_mut()
            .unwrap()
            .send((dist, wait_tx))
            .map_err(Error::ChannelSendError)?;
        let _ = wait_rx.await;
        Ok(())
    }
}

#[derive(Debug)]
pub struct NodeAsServer {
    pub is_tls: bool,
    pub handshaked: bool,
    pub node_name: String,
    pub cookie: String,
    pub creation: u32,
}

impl NodeAsServer {
    pub fn new(node_name: String, cookie: String) -> Self {
        let creation = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        Self {
            node_name,
            cookie,
            creation,
            handshaked: false,
            is_tls: false,
        }
    }

    pub async fn listen<A, S>(&mut self, epmd_addr: A, server: S) -> Result<(), Error>
    where
        A: ToSocketAddrs,
        S: AsServer + Send + Clone + 'static,
        <S as AsServer>::Handler: Clone + Sync,
        <<S as AsServer>::Handler as Handler>::Error: Into<Error>,
    {
        let listener = TcpListener::bind("0.0.0.0:0").await?;
        let port = listener.local_addr()?.port();
        let mut epmd_client = EpmdClient::new(epmd_addr).await?;
        let nodes = epmd_client.req_names().await?.nodes;
        let is_exists = nodes
            .iter()
            .any(|n| n.name == self.node_name);

        if !is_exists {
            let resp = epmd_client
                    .register_node(port, &self.node_name)
                    .await?;

            if resp.result != 0 {
                return Err(Error::Anyhow(anyhow::anyhow!(
                    "Faild register node to epmd"
                )));
            }
        }

        self.node_name.push('@');
        self.node_name.push_str(&get_short_hostname());
        let handshake_codec = HandshakeCodec::new(self.node_name.clone(), self.cookie.clone());

        let (error_tx, mut error_rx) = unbounded_channel::<Error>();
        loop {
            let node_name = self.node_name.clone();
            let is_tls = self.is_tls;
            let creation = self.creation;
            let handshake_codec = handshake_codec.clone();
            let error_tx = error_tx.clone();
            let mut server = server.clone();

            tokio::select! {
                res = listener.accept() => {
                    match res {
                        Ok((mut stream, _)) => {
                            tokio::spawn(
                                async move {
                                    if let Err(err) = Self::server_handshake(is_tls, handshake_codec.clone(), &mut stream).await {
                                        let _ = error_tx.send(err);
                                        return;
                                    }

                                    let handler = match server.new_session() {
                                        Ok(handler) => handler,
                                        Err(err) => {
                                            let _ = error_tx.send(err);
                                            return;
                                        }
                                    };

                                    let (internal_tx, internal_rx) = unbounded_channel::<(CtrlMsg, oneshot::Sender<()>)>();
                                    let node = NodePrimitive::new(node_name, creation, internal_tx);
                                    if let Err(err) = Session::new(internal_rx).run(stream, handler, Some(node)).await {
                                        let _ = error_tx.send(err.into());
                                    }
                                }
                            );
                        },

                        Err(err) => {
                            let _ = error_tx.send(Error::from(err));
                        }
                    }
                },
                Some(res) = error_rx.recv() => {
                    server.handle_error(res);
                }
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
                    if handshake_codec.decode_challenge_reply(&buf[..length]) {
                        let n = handshake_codec.encode_challenge_ack(&mut &mut buf[..]);
                        stream.write_all(&buf[..n]).await?;
                        return Ok(());
                    } else {
                        return Err(Error::HandshakeFailed("invalid reply".to_string()));
                    }
                }
                x => return Err(Error::UnsupportedTag(x)),
            }
        }
    }
}

struct Session {
    // receive internal process message, send to outside
    internal_rx: UnboundedReceiver<(CtrlMsg, oneshot::Sender<()>)>,
}

impl Session {
    fn new(internal_rx: UnboundedReceiver<(CtrlMsg, oneshot::Sender<()>)>) -> Self {
        Self { internal_rx }
    }

    async fn run<H: Handler + Send>(
        mut self,
        mut stream: TcpStream,
        mut handler: H,
        mut node: Option<NodePrimitive>,
    ) -> Result<(), H::Error> {
        let mut buf = vec![0; 512];
        let (mut reader, mut writer) = stream.split();
        let handle_data = Self::handle_data(reader, buf, handler, node.take().unwrap());
        tokio::pin!(handle_data);

        loop {
            tokio::select! {
                res = &mut handle_data => {
                    let res = res?;
                    if res.0 {
                        writer.write(&[0;4]).await.map_err(Error::from)?;
                    }

                    reader = res.1;
                    buf = res.2;
                    node = Some(res.3);
                    handler = res.4;
                    handle_data.set(Self::handle_data(reader, buf, handler, node.take().unwrap()));

                },
                msg = self.internal_rx.recv() => {
                    self.send_to_outside(msg, &mut writer).await?;
                },
            }
        }
    }

    async fn handle_data<H: Handler + Send>(
        mut stream: ReadHalf<'_>,
        mut buf: Vec<u8>,
        mut handler: H,
        mut node: NodePrimitive,
    ) -> Result<(bool, ReadHalf<'_>, Vec<u8>, NodePrimitive, H), H::Error> {
        stream
            .read_exact(&mut buf[..4])
            .await
            .map_err(Error::from)?;
        let length = BigEndian::read_u32(&buf[..4]) as usize;
        // Erlang Tick
        if length == 0 {
            return Ok((true, stream, buf, node, handler));
        }

        if length > buf.len() {
            buf.resize(length, 0)
        }

        stream
            .read_exact(&mut buf[..length])
            .await
            .map_err(Error::from)?;
        if buf[0] == 112 {
            let dist = CtrlMsg::try_from(&buf[..length]).map_err(Error::from)?;
            handler =
                Self::handle_ctrl(dist.ctrl.clone(), dist.msg.clone(), handler, &mut node)
                    .await?;
        }

        Ok((false, stream, buf, node, handler))
    }

    async fn handle_ctrl<H: Handler + Send>(
        ctrl: Ctrl,
        msg: Option<term::Term>,
        handler: H,
        node: &mut NodePrimitive,
    ) -> Result<H, H::Error> {
        match ctrl {
            Ctrl::RegSend(ctrl) => handler.reg_send(node, ctrl, msg.unwrap()).await,
            Ctrl::AliasSend(ctrl) => handler.alias_send(node, ctrl, msg.unwrap()).await,
            Ctrl::AliasSendTT(ctrl) => handler.alias_send_tt(node, ctrl, msg.unwrap()).await,
            Ctrl::MonitorP(ctrl) => handler.mointor_p(node, ctrl).await,
            Ctrl::DeMonitorP(ctrl) => handler.de_monitor_p(node, ctrl).await,
            Ctrl::MonitorPExit(ctrl) => handler.monitor_p_exit(node, ctrl).await,
            Ctrl::Exit(ctrl) => handler.exit(node, ctrl).await,
            Ctrl::ExitTT(ctrl) => handler.exit_tt(node, ctrl).await,
            Ctrl::Exit2(ctrl) => handler.exit2(node, ctrl).await,
            Ctrl::Exit2TT(ctrl) => handler.exit2_tt(node, ctrl).await,
            Ctrl::GroupLeader(ctrl) => handler.group_leader(node, ctrl).await,
            Ctrl::Link(ctrl) => handler.link(node, ctrl).await,
            Ctrl::NodeLink(ctrl) => handler.node_link(node, ctrl).await,
            Ctrl::PayloadExit(ctrl) => handler.payload_exit(node, ctrl, msg.unwrap()).await,
            Ctrl::PayloadExitTT(ctrl) => handler.payload_exit_tt(node, ctrl, msg.unwrap()).await,
            Ctrl::PayloadExit2(ctrl) => handler.payload_exit2(node, ctrl, msg.unwrap()).await,
            Ctrl::PayloadExit2TT(ctrl) => {
                handler.payload_exit2_tt(node, ctrl, msg.unwrap()).await
            }
            Ctrl::PayloadMonitorPExit(ctrl) => {
                handler
                    .payload_monitor_p_exit(node, ctrl, msg.unwrap())
                    .await
            }
            Ctrl::SendCtrl(ctrl) => handler.send(node, ctrl, msg.unwrap()).await,
            Ctrl::UnLink(ctrl) => handler.unlink(node, ctrl).await,
            Ctrl::SendTT(ctrl) => handler.send_tt(node, ctrl, msg.unwrap()).await,
            Ctrl::RegSendTT(ctrl) => handler.reg_send_tt(node, ctrl, msg.unwrap()).await,
            Ctrl::SendSender(ctrl) => handler.send_sender(node, ctrl, msg.unwrap()).await,
            Ctrl::SendSenderTT(ctrl) => handler.send_sender_tt(node, ctrl, msg.unwrap()).await,
            Ctrl::SpawnRequest(ctrl) => handler.spawn_request(node, ctrl, msg.unwrap()).await,
            Ctrl::SpawnRequestTT(ctrl) => {
                handler.spawn_request_tt(node, ctrl, msg.unwrap()).await
            }
            Ctrl::SpawnReply(ctrl) => handler.spawn_reply(node, ctrl).await,
            Ctrl::SpawnReplyTT(ctrl) => handler.spawn_reply_tt(node, ctrl).await,
            Ctrl::UnLinkId(ctrl) => handler.unlink_id(node, ctrl).await,
            Ctrl::UnLinkIdAck(ctrl) => handler.unlink_id_ack(node, ctrl).await,
        }
    }

    async fn send_to_outside(
        &self,
        msg: Option<(CtrlMsg, oneshot::Sender<()>)>,
        stream: &mut WriteHalf<'_>,
    ) -> Result<(), Error> {
        if let Some((msg, wait_tx)) = msg {
            let mut buf = Vec::with_capacity(4 + msg.len());
            let _ = msg.encode(&mut buf);
            stream.write_all(&buf).await?;
            let _ = wait_tx.send(());
        }

        Ok(())
    }
}

#[inline]
fn header_length(is_tls: bool, _handshaked: bool) -> usize {
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

static UNIQ_ID: AtomicU64 = AtomicU64::new(1);
static PID_ID: AtomicU64 = AtomicU64::new(1);
#[derive(Debug, Clone)]
pub struct NodePrimitive {
    node_name: String,
    creation: u32,
    sender: UnboundedSender<(CtrlMsg, oneshot::Sender<()>)>,
}

impl NodePrimitive {
    pub fn new(
        node_name: String,
        creation: u32,
        sender: UnboundedSender<(CtrlMsg, oneshot::Sender<()>)>,
    ) -> Self {
        Self {
            node_name: node_name.clone(),
            creation,
            sender,
        }
    }

    pub async fn send(&self, dist: CtrlMsg) {
        let (wait_tx, wait_rx) = oneshot::channel::<()>();
        let _ = self.sender.send((dist, wait_tx));
        let _ = wait_rx.await;
    }

    pub fn make_pid(&self) -> term::NewPid {
        let pid_id = PID_ID.fetch_add(1, Ordering::Relaxed);
        term::NewPid {
            node: term::SmallAtomUtf8(self.node_name.clone()),
            id: (pid_id & 0x7fff) as u32,
            serial: ((pid_id >> 15) & 0x1fff) as u32,
            creation: self.creation,
        }
    }

    /// create ref. refer to https://github.com/erlang/otp/blob/master/lib/erl_interface/src/connect/ei_connect.c#L745
    pub fn make_ref(&self) -> term::NewerReference {
        let uniq_id = UNIQ_ID.fetch_add(1, Ordering::Relaxed);
        term::NewerReference {
            length: 3,
            node: term::SmallAtomUtf8(self.node_name.clone()),
            creation: self.creation,
            id: vec![
                (uniq_id & 0x3ffff) as u32,
                ((uniq_id >> 18) & 0xffffffff) as u32,
                ((uniq_id >> (18 + 32)) & 0xffffffff) as u32,
            ],
        }
    }
}
