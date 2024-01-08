
use async_trait::async_trait;
use proto::dist::*;
use proto::{etf::term, RegSend, SendCtrl};
use tokio::net::{TcpStream, ToSocketAddrs};

pub mod primitive;
pub use primitive::*;

#[async_trait]
pub trait AsClient  {
    // Connect remote node
    async fn connect_local_by_name<A: ToSocketAddrs + Send>(&mut self, epmd_addr: A, remote_node_name: &str) -> Result<NodeAsClient, Error>;
}

#[async_trait]
pub trait AsServer<H: Handler> {
    // Accept a new connection
    async fn listen<A: ToSocketAddrs + Send>(&mut self, epmd_addr: A, handler: H) -> Result<(), H::Error>;
}

#[async_trait]
pub trait Handler: Sized {
    type Error: From<Error> + Send;

    async fn handle_error(self, _err: Self::Error)  {}

    async fn link(self, _stream: &mut TcpStream, _ctrl: Link) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn send(self, _stream: &mut TcpStream, _ctrl: SendCtrl, msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn exit(self, _stream: &mut TcpStream, _ctrl: Exit) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn unlink(self, _stream: &mut TcpStream, _ctrl: UnLink) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn node_link(self, _stream: &mut TcpStream, _ctrl: NodeLink) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn reg_send(self, _stream: &mut TcpStream, _ctrl: RegSend, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn group_leader(self, _stream: &mut TcpStream, _ctrl: GroupLeader) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn exit2(self, _stream: &mut TcpStream, _ctrl: Exit2) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn send_tt(self, _stream: &mut TcpStream, _ctrl: SendTT, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn exit_tt(self, _stream: &mut TcpStream, _ctrl: ExitTT) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn reg_send_tt(self, _stream: &mut TcpStream, _ctrl: RegSendTT, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn exit2_tt(self, _stream: &mut TcpStream, _ctrl: Exit2TT) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn mointor_p(self, _stream: &mut TcpStream, _ctrl: MonitorP) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn de_monitor_p(self, _stream: &mut TcpStream, _ctrl: DeMonitorP) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn monitor_p_exit(self, _stream: &mut TcpStream, _ctrl: MonitorPExit) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn send_sender(self, _stream: &mut TcpStream, _ctrl: SendSender, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn send_sender_tt(self, _stream: &mut TcpStream, _ctrl: SendSenderTT, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn payload_exit(self, _stream: &mut TcpStream, _ctrl: PayloadExit, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn payload_exit_tt(self, _stream: &mut TcpStream, _ctrl: PayloadExitTT, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn payload_exit2(self, _stream: &mut TcpStream, _ctrl: PayloadExit2, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn payload_exit2_tt(self, _stream: &mut TcpStream, _ctrl: PayloadExit2TT, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn payload_monitor_p_exit(self, _stream: &mut TcpStream, _ctrl: PayloadMonitorPExit, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn spawn_request(self, _stream: &mut TcpStream, _ctrl: SpawnRequest, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn spawn_request_tt(self, _stream: &mut TcpStream, _ctrl: SpawnRequestTT, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn spawn_reply(self, _stream: &mut TcpStream, _ctrl: SpawnReply) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn spawn_reply_tt(self, _stream: &mut TcpStream, _ctrl: SpawnReplyTT) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn unlink_id(self, _stream: &mut TcpStream, _ctrl: UnLinkId) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn unlink_id_ack(self, _stream: &mut TcpStream, _ctrl: UnLinkIdAck) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn alias_send(self, _stream: &mut TcpStream, _ctrl: AliasSend, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn alias_send_tt(self, _stream: &mut TcpStream, _ctrl: AliasSendTT, _msg: term::Term) -> Result<Self, Self::Error> {
        Ok(self)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("handshake failed: {0}")]
    HandshakeFailed(String),
    #[error("unsupported tag: {0}")]
    UnsupportedTag(u8),
    #[error("client wait timeout 1s")]
    WaitTimeout,

    #[error(transparent)]
    IO(#[from] std::io::Error), 

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}



//impl Node {
//    pub fn new(is_tls: bool) -> Node {
//        Node { 
//            is_tls, 
//            handshake_codec: HandshakeCodec::new(),
//            creation: 0,
//            handshaked: false,
//            header_buf: BytesMut::with_capacity(4),
//            buf: BytesMut::with_capacity(1024),
//            group_pid: None,
//        }
//    }
//
//    pub async fn init(stream:TcpStream, port: u16) -> Result<(), anyhow::Error> {
//        let mut epmd_client = EpmdClient::new("127.0.0.1:4369").await?;
//        epmd_client.register_node(port, "rust").await?;
//        Ok(())
//    }
//
//    pub async fn get_names(&self, ) {
//
//    }
//
//    pub async fn start(&mut self, mut stream: TcpStream) -> Result<(), anyhow::Error> {
//        let err_ctx = "node start";
//        self.handshake_codec.creation = self.creation;
//        self.buf.clear();
//        let n = self.handshake_codec.encode_v6_name(&mut self.buf);
//        stream.write_all(&self.buf[..n]).await.with_context(|| err_ctx).map_err(anyhow::Error::msg)?;
//        
//        loop {
//            stream.read_exact(self.init_hedader_buf()).await.with_context(||err_ctx)?;
//            println!("ss {:?}", self.header_buf);
//            let length = self.read_length();
//            // ERLANG_TICK
//            if length == 0 {
//                println!("v {:?} {:?}", std::time::SystemTime::now(), &self.header_buf);
//                stream.write(&[0;4]).await.with_context(|| err_ctx)?;
//                continue;
//            }
//
//            stream.read_exact(self.init_buf(length)).await.with_context(||err_ctx).map_err(anyhow::Error::msg)?;
//            match self.buf[0] {
//                b's' => {
//                    self.handshake_codec.decode_status(&mut self.buf);
//                    if self.handshake_codec.status == Status::NotAllowed || self.handshake_codec.status == Status::Nok {
//                        return Err(anyhow::anyhow!("Remote node return Status::NowAllowed or Status::Nok"));
//                    }
//                    self.buf.clear();
//                }
//                b'n' => {
//                    self.handshake_codec.decode_v5_challenge(&mut self.buf);
//                    self.buf.clear();
//                    let n = self.handshake_codec.encode_challenge_reply(&mut self.buf);
//                    stream.write(&self.buf[..n]).await.with_context(|| err_ctx).map_err(anyhow::Error::msg)?;
//                }
//                b'N' => {
//                    self.handshake_codec.decode_v6_challenge(&mut self.buf);
//                    self.buf.clear();
//                    
//                    if self.handshake_codec.version == HandshakeVersion::V5 {
//                        let n = self.handshake_codec.encode_complement(&mut self.buf);
//                        stream.write(&self.buf[..n]).await.with_context(|| err_ctx).map_err(anyhow::Error::msg)?;
//                        self.buf.clear();
//                    }
//
//                    let  n = self.handshake_codec.encode_challenge_reply(&mut self.buf);
//                    stream.write(&self.buf[..n]).await.with_context(|| err_ctx).map_err(anyhow::Error::msg)?;
//
//                }
//                b'a' => {
//                    let is_valid = self.handshake_codec.decode_challenge_ack(&mut self.buf);
//                    println!("{:?}", is_valid);
//                    if !is_valid {
//                        return Err(anyhow::anyhow!("handshake incorrect digest").context(err_ctx));
//                    }
//                    self.handshaked = true;
//                }
//                _ => {}
//             }
//        } 
//    }
//
//    pub async fn accept(&mut self, mut stream: TcpStream) -> Result<(), anyhow::Error> {
//        println!("{:?}", gethostname::gethostname());
//        let err_ctx = "node accept";
//        let buf = vec![0;1024];
//        let mut buf = BytesMut::from(&buf[..]); 
//        buf.reserve(1024);
//        let sleep = tokio::time::sleep(tokio::time::Duration::from_secs(60));
//        tokio::pin!(sleep);
//        loop {
//            //let n = stream.read(&mut buf).await.with_context(||err_ctx).map_err(anyhow::Error::msg)?;
//            //if n == 0 || n <= 2 {
//            //    continue;
//            //}
//            tokio::select! {
//                res = self.handle_data(&mut stream) =>  res?,
//                _ = &mut sleep => {
//                    stream.write(&[0,0,0,0]).await?;
//                    println!("sleep");
//                    sleep.as_mut().reset(tokio::time::Instant::now() + tokio::time::Duration::from_secs(60));
//                }
//            }
//            
//        }
//    }
//
//    async fn handle_data(&mut self, stream: &mut TcpStream) -> Result<(), anyhow::Error> {
//            let err_ctx = "hande data";
//            stream.read_exact(self.init_hedader_buf()).await.with_context(||err_ctx).unwrap();
//            let length = self.read_length();
//            // ERLANG_TICK
//            if length == 0 {
//                println!("v {:?} {:?}", std::time::SystemTime::now(), &self.header_buf);
//                return Ok(())
//            }
//            stream.read_exact(self.init_buf(length)).await.with_context(||err_ctx).map_err(anyhow::Error::msg)?;
//            match self.buf[0] {
//                b's' => {
//                    self.handshake_codec.decode_status(&mut self.buf);
//                    if self.handshake_codec.status == Status::NotAllowed || self.handshake_codec.status == Status::Nok {
//                        return Ok(())
//                    }
//                    
//                }
//                b'n' => {
//                    self.handshake_codec.decode_v5_name(&mut self.buf);
//                    let n = self.handshake_codec.encode_status(&mut self.buf);
//                    stream.write(&self.buf[..n]).await.with_context(|| err_ctx).map_err(anyhow::Error::msg)?;
//                    self.buf.clear();
//                    let n = if self.handshake_codec.version == HandshakeVersion::V6 {
//                        self.handshake_codec.encode_v6_challenge(&mut self.buf)
//                    } else {
//                        self.handshake_codec.encode_v5_challenge(&mut self.buf)
//                    };
//                    stream.write(&self.buf[..n]).await.with_context(|| err_ctx).map_err(anyhow::Error::msg)?;
//                }
//                b'N' => {
//                    self.handshake_codec.decode_v6_name(&mut self.buf);
//                    println!("ssssss {:?}", self.buf);
//                    let n = self.handshake_codec.encode_status(&mut self.buf);
//                    stream.write(&self.buf[..n]).await.with_context(|| err_ctx).map_err(anyhow::Error::msg)?;
//                    self.buf.clear();
//
//                    let n = self.handshake_codec.encode_v6_challenge(&mut self.buf);
//                    stream.write(&self.buf[..n]).await.with_context(|| err_ctx).map_err(anyhow::Error::msg)?;
//                }
//                b'c' => {
//                    self.handshake_codec.decode_complement(&mut self.buf);
//                }
//                b'r' => {
//                    if self.handshake_codec.decode_challenge_reply(&mut self.buf) {
//                        let n = self.handshake_codec.encode_challenge_ack(&mut self.buf);
//                        stream.write(&self.buf[..n]).await.with_context(|| err_ctx).map_err(anyhow::Error::msg)?;
//                        self.handshaked = true;
//                    } else {
//                        return Err(anyhow::anyhow!("hanshake invalid reply").context(err_ctx));
//                    }
//                }
//                112 => {
//                    // 112 
//                    self.buf.get_u8();
//                    //// 131
//                    self.buf.get_u8();
//                    //println!("self.buf {:?}", &self.buf[..]);
//                    let dist = Dist::try_from(&self.buf[..])?;
//                    println!("dist {:?}", dist);
//                    //if let CtrlMsg::SpawnRequest(req) = dist.ctrl_msg {
//                    //    println!("req {:?}", req);
//                    //}
//
//                     if self.buf[3] == 29 {
//                         let creation = fastrand::u32(..);
//                         let result_pid = term::NewPid{
//                             node: term::SmallAtomUtf8("rust@fedora".to_string()),
//                             id: 0,
//                             serial: 1,
//                             creation,
//                         };
//
//                         let req = term::Term::from(&self.buf[..]);
//                         println!("{:?}", req);
//                         let tuple: term::SmallTuple = req.try_into().unwrap();
//  
//                         let refer = tuple.elems[1].clone();
//                         let from_pid = tuple.elems[2].clone();
//                         self.group_pid = Some(tuple.elems[3].clone());
//
//                         let mut buf = vec![];
//                         
//                         let flags = term::SmallInteger(0);
//                         //let result = term::SmallAtomUtf8("RUST".to_string());
//                         let tuple = term::SmallTuple{
//                             arity: 5,
//                             elems: vec![
//                                 term::Term::SmallInteger(term::SmallInteger(31)),
//                                 refer,
//                                 from_pid.clone(),
//                                 term::Term::SmallInteger(flags),
//                                 term::Term::NewPid(result_pid.clone()),
//                             ],
//                         };
//                         
//                         buf.put_u32(2+tuple.len() as u32);
//                         buf.put_u8(112);
//                         buf.put_u8(131);
//                         tuple.encode(&mut buf);
//
//                         println!("buffffffffffffffff {:?}", &buf);
//                         stream.write_all(&buf).await.unwrap();
//
//                         buf.clear();
//
//                         //let tuple = term::SmallTuple{
//                         //    arity: 3,
//                         //    elems: vec![
//                         //        term::Term::SmallInteger(term::SmallInteger(22)),
//                         //        term::Term::NewPid(result_pid.clone()),
//                         //        self.group_pid.as_ref().unwrap().clone(),
//                         //    ],
//                         //};
//
//                         //let refer = term::NewerReference{
//                         //    length: 2,
//                         //    node: term::SmallAtomUtf8("rust@fedora".to_string()),
//                         //    creation: fastrand::u32(..),
//                         //    id: vec![
//                         //        fastrand::u32(..),
//                         //        fastrand::u32(..),
//                         //    ],
//                         //};
//                         
//                         //let res = term::SmallTuple{
//                         //    arity: 4,
//                         //    elems: vec![
//                         //        term::Term::SmallAtomUtf8(term::SmallAtomUtf8("io_request".to_string())),
//                         //        term::Term::NewPid(result_pid.clone()),
//                         //        term::Term::NewerReference(refer),
//                         //        term::Term::SmallTuple(term::SmallTuple{
//                         //            arity:3,
//                         //            elems: vec![
//                         //                term::Term::SmallAtomUtf8(term::SmallAtomUtf8("put_chars".to_string())),
//                         //                term::Term::SmallAtomUtf8(term::SmallAtomUtf8("unicode".to_string())),
//                         //                term::Term::Binary(term::Binary { length: 2, data: vec![0x61, 0x0a] })
//                         //            ],
//                         //        }),
//                         //    ],
//                         //};
//
//                         //buf.put_u32(3+ tuple.len() as u32 + res.len() as u32);
//                         //buf.put_u8(112);
//                         //buf.put_u8(131);
//                         //tuple.encode(&mut buf);
//                         //println!("buff22222222222222 {:?}", &buf);
//                         //buf.put_u8(131);
//                         //res.encode(&mut buf);
//                         //stream.write_all(&buf).await.unwrap();
//                         //buf.clear();
//                     }
//                }
//               _ => {
//                    println!("1111 {:?}", self.buf.chunk());
//               } 
//            }
//            Ok(())
//    }
//
//    fn init_hedader_buf(&mut self) -> &mut BytesMut {
//        let length = if self.is_tls || self.handshaked {
//            4
//        } else {
//            2
//        };
//        if self.header_buf.capacity() < length {
//            self.header_buf.reserve(length);
//        }
//        
//        unsafe { self.header_buf.set_len(length) };
//        println!("aa {:?}", &self.header_buf[..]);
//        &mut self.header_buf
//    }
//}

fn get_short_hostname() -> String {
    let hostname = gethostname::gethostname();
    let hostname = hostname.to_string_lossy();
    hostname.split(".").next().unwrap().to_string()
}



pub trait PatternMatch {
    type Pattern;
    type Output;
    fn pattern_match(&self, pattern: &Self::Pattern) -> Self::Output;
}

#[derive(Debug, Clone)]
pub struct TuplePattern(Vec<TuplePatternInner>);
impl TuplePattern {
    pub fn new(len: usize) -> Self {
        Self(Vec::with_capacity(len))
    }
    
    pub fn with(mut self, idx: usize, term: term::Term) -> Self {
        self.0.push(TuplePatternInner {
            idx,
            term,
        });
        self
    } 
}

#[derive(Debug, Clone)]
pub struct TuplePatternInner {
    pub idx: usize,
    pub term: term::Term
}

impl PatternMatch for term::SmallTuple {
    type Pattern = TuplePattern;
    type Output = bool;

    fn pattern_match(&self, pattern: &Self::Pattern) -> Self::Output {
        pattern.0.iter().all(|x| {
            match self.elems.get(x.idx) {
                Some(e) => x.term.eq(e),
                None => false,
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct AtomPattern(pub String);

impl PatternMatch for term::SmallAtomUtf8 {
    type Pattern = AtomPattern;
    type Output = bool;

    fn pattern_match(&self, pattern: &Self::Pattern) -> Self::Output {
        self.0 == pattern.0
    }
}



#[cfg(test)]
mod test {
    use proto::etf::term::*;

    use super::{PatternMatch, TuplePattern};

    #[test]
    fn pattern_match() {
        let tuple = SmallTuple::from(vec![
            Term::SmallAtomUtf8(SmallAtomUtf8("a".to_string())),
            Term::SmallAtomUtf8(SmallAtomUtf8("b".to_string())),
        ]);

        let pattern = TuplePattern::new(10).with(0, SmallAtomUtf8("a".to_string()).into());
        assert!(tuple.pattern_match(&pattern));

        let pattern = TuplePattern::new(10).with(0, SmallAtomUtf8("b".to_string()).into())
            .with(1, SmallAtomUtf8("b".to_string()).into());
        assert!(!tuple.pattern_match(&pattern));
    }
}