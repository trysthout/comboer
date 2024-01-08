use std::net::{SocketAddrV4, Ipv4Addr};

use async_trait::async_trait;
use byteorder::{BigEndian, ByteOrder};
use proto::{EpmdClient, etf::term, Dist, handshake::{HandshakeCodec, HandshakeVersion, Status}, CtrlMsg, Encoder, Len};
use tokio::{net::{ToSocketAddrs, TcpStream, TcpListener}, time::sleep, sync::mpsc::unbounded_channel, io::{AsyncReadExt, AsyncWriteExt}};

use crate::node::get_short_hostname;

use super::{AsClient, Error, Handler, AsServer};

#[derive(Debug, Clone)]
pub struct NodePrimitive {
    pub is_tls: bool,
    pub handshaked: bool,
    pub node_name: String,
    pub cookie: String

}

impl NodePrimitive {
    pub fn new(node_name: String, cookie: String) -> Self {
        Self { 
            is_tls: false, 
            handshaked: false, 
            node_name,
            cookie
        }
    }
    async fn client_handshake(&mut self, mut handshake_codec: HandshakeCodec, stream: &mut TcpStream) -> Result<(), Error> {
        let mut buf = vec![0;512];
        let n = handshake_codec.encode_v6_name(&mut &mut buf[..]);
        stream.write_all(&buf[..n]).await?;
        
        let header_length = self.header_length();
        
        loop {
            stream.read_exact(&mut buf[0..header_length]).await?;
            let length = self.read_length(&buf[0..header_length]);
            // ERLANG_TICK
            if length == 0 {
                println!("v {:?} {:?}", std::time::SystemTime::now(), &buf);
                stream.write(&[0;4]).await?;
                continue;
            }

            if length > buf.len() {
                buf.resize(length, 0);
            }

            stream.read_exact(&mut buf[0..length]).await?;
            match buf[0] {
                b's' => {
                    handshake_codec.decode_status(&mut buf[..length]);
                    if handshake_codec.status == Status::NotAllowed || handshake_codec.status == Status::Nok {
                        return Err(Error::HandshakeFailed("Remote node return Status::NowAllowed or Status::Nok".to_string()));
                    }
                }
                b'n' => {
                    handshake_codec.decode_v5_challenge(&mut buf[..length]);
                    let n = handshake_codec.encode_challenge_reply(&mut &mut buf[..]);
                    stream.write(&buf[..n]).await?;
                }
                b'N' => {
                    handshake_codec.decode_v6_challenge(&mut buf[..length]);
                    
                    if handshake_codec.version == HandshakeVersion::V5 {
                        let n = handshake_codec.encode_complement(&mut &mut buf[..]);
                        stream.write(&buf[..n]).await?;
                    }

                    let  n = handshake_codec.encode_challenge_reply(&mut &mut buf[..]);
                    stream.write(&buf[..n]).await?;

                }
                b'a' => {
                    let is_valid = handshake_codec.decode_challenge_ack(&mut buf[..length]);
                    println!("{:?}", is_valid);
                    if !is_valid {
                        return Err(Error::HandshakeFailed("incorrect digest".to_string()));
                    }
                    self.handshaked = true;
                    return Ok(());
                }
                x => return Err(Error::UnsupportedTag(x))
             }
        } 
    }

    async fn server_handshake(&mut self, mut handshake_codec: HandshakeCodec, stream: &mut TcpStream) -> Result<(), Error> {
        let mut buf = vec![0;512];
        let header_length = self.header_length();
        loop {
            stream.read_exact(&mut buf[0..header_length]).await?;
            let length = self.read_length(&buf);
            // ERLANG_TICK
            if length == 0 {
                println!("v {:?} {:?}", std::time::SystemTime::now(), &buf);
                return Ok(())
            }
            if length > buf.len() {
                buf.resize(length, 0);
            }
            stream.read_exact(&mut buf[0..length]).await?;
            match buf[0] {
                b's' => {
                    handshake_codec.decode_status(&mut buf[..length]);
                    if handshake_codec.status == Status::NotAllowed || handshake_codec.status == Status::Nok {
                        return Ok(())
                    }
                }
                b'n' => {
                    handshake_codec.decode_v5_name(&mut buf[..length]);
                    let n = handshake_codec.encode_status(&mut &mut buf[..]);
                    stream.write(&buf[..n]).await?;
                    let n = if handshake_codec.version == HandshakeVersion::V6 {
                        handshake_codec.encode_v6_challenge(&mut &mut buf[..])
                    } else {
                        handshake_codec.encode_v5_challenge(&mut &mut buf[..])
                    };
                    stream.write(&buf[..n]).await?;
                }
                b'N' => {
                    handshake_codec.decode_v6_name(&mut buf[..length]);
                    let n = handshake_codec.encode_status(&mut &mut buf[..]);
                    stream.write(&buf[..n]).await?;

                    let n = handshake_codec.encode_v6_challenge(&mut &mut buf[..]);
                    stream.write(&buf[..n]).await?;
                }
                b'c' => {
                    handshake_codec.decode_complement(&mut buf[..length]);
                }
                b'r' => {
                    if handshake_codec.decode_challenge_reply(&mut buf[..length]) {
                        let n = handshake_codec.encode_challenge_ack(&mut &mut buf[..]);
                        stream.write(&buf[..n]).await?;
                        self.handshaked = true;
                        return Ok(());
                    } else {
                        return Err(Error::HandshakeFailed("invalid reply".to_string()));
                    }
                }
                x => return Err(Error::UnsupportedTag(x)),
            }
        }
    }

    async fn handle_data<H: Handler + Send>(&mut self, stream: &mut TcpStream, mut handler: H) -> Result<(), H::Error> {
        let mut buf = vec![0;512];
        loop {
            stream.read_exact(&mut buf[..4]).await.map_err(Error::from)?;
            let length = BigEndian::read_u32(&buf[..4]) as usize;
            // Erlang Tick
            if length == 0 {
                stream.write(&[0;4]).await.map_err(Error::from)?;
                continue;
            }

            if length > buf.len() {
                buf.resize(length, 0);
            }

            stream.read_exact(&mut buf[..length]).await.map_err(Error::from)?;
            match buf[0] {
                112 => {
                    let dist = Dist::try_from(&buf[..length]).map_err(Error::from)?;
                    println!("dist {:?}", dist);
                    let msg = dist.msg;
                    match dist.ctrl_msg {
                        CtrlMsg::RegSend(ctrl) => {
                            handler = handler.reg_send(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::AliasSend(ctrl) => {
                            handler = handler.alias_send(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::AliasSendTT(ctrl) => {
                            handler = handler.alias_send_tt(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::MonitorP(ctrl) => {
                            handler = handler.mointor_p(stream, ctrl).await?;
                        },
                        CtrlMsg::DeMonitorP(ctrl) => {
                            handler = handler.de_monitor_p(stream, ctrl).await?;
                        },
                        CtrlMsg::MonitorPExit(ctrl) => {
                            handler = handler.monitor_p_exit(stream, ctrl).await?;
                        },
                        CtrlMsg::Exit(ctrl) => {
                            handler = handler.exit(stream, ctrl).await?;
                        },
                        CtrlMsg::ExitTT(ctrl) => {
                            handler = handler.exit_tt(stream, ctrl).await?;
                        },
                        CtrlMsg::Exit2(ctrl) => {
                            handler = handler.exit2(stream, ctrl).await?;
                        },
                        CtrlMsg::Exit2TT(ctrl) => {
                            handler = handler.exit2_tt(stream, ctrl).await?;
                        },
                        CtrlMsg::GroupLeader(ctrl) => {
                            handler = handler.group_leader(stream, ctrl).await?;
                        },
                        CtrlMsg::Link(ctrl) => {
                            handler = handler.link(stream, ctrl).await?;
                        },
                        CtrlMsg::NodeLink(ctrl) => {
                            handler = handler.node_link(stream, ctrl).await?;
                        },
                        CtrlMsg::PayloadExit(ctrl) => {
                            handler = handler.payload_exit(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::PayloadExitTT(ctrl) => {
                            handler = handler.payload_exit_tt(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::PayloadExit2(ctrl) => {
                            handler = handler.payload_exit2(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::PayloadExit2TT(ctrl) => {
                            handler = handler.payload_exit2_tt(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::PayloadMonitorPExit(ctrl) => {
                            handler = handler.payload_monitor_p_exit(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::SendCtrl(ctrl) => {
                            handler = handler.send(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::UnLink(ctrl) => {
                            handler = handler.unlink(stream, ctrl).await?;
                        },
                        CtrlMsg::SendTT(ctrl) => {
                            handler = handler.send_tt(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::RegSendTT(ctrl) => {
                            handler = handler.reg_send_tt(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::SendSender(ctrl) => {
                            handler = handler.send_sender(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::SendSenderTT(ctrl) => {
                            handler = handler.send_sender_tt(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::SpawnRequest(ctrl) => {
                            handler = handler.spawn_request(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::SpawnRequestTT(ctrl) => {
                            handler = handler.spawn_request_tt(stream, ctrl, msg.unwrap()).await?;
                        },
                        CtrlMsg::SpawnReply(ctrl) => {
                            handler = handler.spawn_reply(stream, ctrl).await?;
                        },
                        CtrlMsg::SpawnReplyTT(ctrl) => {
                            handler = handler.spawn_reply_tt(stream, ctrl).await?;
                        },
                        CtrlMsg::UnLinkId(ctrl) => {
                            handler = handler.unlink_id(stream, ctrl).await?;
                        },
                        CtrlMsg::UnLinkIdAck(ctrl) => {
                            handler = handler.unlink_id_ack(stream, ctrl).await?;
                        },
                    }
                }
                _ => {}
            }
        }
    }

    #[inline]
    fn header_length(&mut self) -> usize {
        if self.is_tls || self.handshaked {
            4
        } else {
            2
        }
    }

    fn read_length(&self, buf: &[u8]) -> usize {
        if self.is_tls || self.handshaked {
            BigEndian::read_u32(buf) as usize
        } else {
            BigEndian::read_u16(buf) as usize
        }
    }
}

#[async_trait]
impl AsClient for NodePrimitive {
    async fn connect_local_by_name<A: ToSocketAddrs + Send>(&mut self, epmd_addr: A, remote_node_name: &str) -> Result<NodeAsClient, Error> {
        let mut epmd_client = EpmdClient::new(epmd_addr).await?;
        let nodes = epmd_client.req_names().await?.nodes;
        let node = nodes.iter().find(|&n|n.name == remote_node_name).ok_or_else(|| anyhow::anyhow!("Not found node_name {:?}", remote_node_name))?;

        println!("node {:?}", node);

        //let mut node = Node::new(false);
        let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), node.port);
        let mut stream = TcpStream::connect(addr).await.unwrap();
        let _ = stream.set_nodelay(true);
        let handshake_codec = HandshakeCodec::new(self.node_name.clone(), self.cookie.clone());
        self.client_handshake(handshake_codec, &mut stream).await?;
        Ok(NodeAsClient::new(stream))
    }
}

#[async_trait]
impl<H: Handler + Send + Clone + 'static> AsServer<H> for NodePrimitive {
    async fn listen<A: ToSocketAddrs + Send>(&mut self, epmd_addr: A, handler: H) -> Result<(), H::Error> {
        let listener = TcpListener::bind("0.0.0.0:0").await.map_err(Error::from)?;
        let port = listener.local_addr().map_err(Error::from)?.port();
        let mut epmd_client = EpmdClient::new(epmd_addr).await.map_err(Error::from)?;
        let resp = epmd_client.register_node(port, &self.node_name).await.map_err(Error::from)?;
        println!("{:?}", resp);
        if resp.result != 0 {
            return Err(Error::Anyhow(anyhow::anyhow!("Faild register node to epmd")).into());
        }

        self.node_name.push('@');
        self.node_name.push_str(&get_short_hostname());
        
        let (error_tx, mut error_rx) = unbounded_channel::<H::Error>();
        loop {
            let mut me = self.clone();
            let handler = handler.clone();
            tokio::select! {
                res = listener.accept() => {
                    match res {
                        Ok((mut stream, _)) => {
                            
                            
                            let error_tx = error_tx.clone();
                            tokio::spawn(
                                async move {
                                    let handshake_codec = HandshakeCodec::new(me.node_name.clone(), me.cookie.clone());
                                    if let Err(err) = me.server_handshake(handshake_codec, &mut stream).await {
                                        let _ = error_tx.send(err.into());
                                        return ;
                                    }

                                    if let Err(err) = me.handle_data(&mut stream, handler).await {
                                        let _ = error_tx.send(err);
                                    }

                                }
                            );
                        },

                        Err(err) => {
                            let _ = error_tx.send(Error::from(err).into());
                        }
                    }
                },
                Some(res) = error_rx.recv() => {
                    handler.handle_error(res).await;
                }
            }
        }

    }
}

#[derive(Debug)]
pub struct NodeAsClient {
    stream: TcpStream,
}

impl NodeAsClient {
    fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub async fn send(&mut self, ctrl: CtrlMsg, msg: term::Term) -> Result<(), Error> 
    {

        let dist = Dist::new(ctrl, Some(msg));

        let mut buf = Vec::with_capacity(dist.len());
        dist.encode(&mut buf)?;
        self.stream.write(&buf).await?;

        Ok(())
    }

    pub async fn send_and_wait<F>(&mut self, ctrl: CtrlMsg, msg: term::Term, pattern_fn: F) -> Result<Option<Dist>, Error> 
    where 
        F: FnMut(&term::Term) -> bool 
    {
        self.send(ctrl, msg).await?;
        let sleep = sleep(tokio::time::Duration::from_secs(1));
        tokio::select! {
            res = self.wait(pattern_fn) => res,
            _ = sleep => Ok(None),
        }
    }

    pub async fn wait<F>(&mut self, mut pattern_fn: F) -> Result<Option<Dist>, Error> 
    where 
        F: FnMut(&term::Term) -> bool
    {
        let mut buf = vec![0;512];
        loop {
            self.stream.read_exact(&mut buf[..4]).await?;
            let length = BigEndian::read_u32(&buf[..4]) as usize;
            // Erlang Tick
            if length == 0 {
                self.stream.write(&[0;4]).await?;
                continue;
            }

            if length > buf.len() {
                buf.resize(length, 0);
            }

            self.stream.read_exact(&mut buf[..length]).await?;
            match buf[0] {
                112 => {
                    let dist = Dist::try_from(&buf[..length])?;
                    println!("dist {:?}", dist);
                    if let Some(ref msg) = dist.msg {
                        if pattern_fn(msg) {
                            return Ok(Some(dist));
                        }
                    }
                },
                _ => {}
            }
        }

    }
}