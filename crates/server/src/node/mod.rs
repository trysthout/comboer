use async_trait::async_trait;
use proto::dist::*;
use proto::{etf::term, RegSend, SendCtrl};
use tokio::net::ToSocketAddrs;

pub mod primitive;
pub use primitive::*;
use tokio::sync::mpsc::error::{SendError, TryRecvError};
use tokio::sync::oneshot;

#[async_trait]
pub trait AsClient {
    // Connect remote node
    async fn connect_local_by_name<A: ToSocketAddrs + Send>(
        mut self,
        epmd_addr: A,
        remote_node_name: &str,
    ) -> Result<NodeAsClient, Error>;
}

#[async_trait]
pub trait AsServer {
    type Handler: Handler + Send;
    fn new_session(&mut self) -> Result<Self::Handler, Error>;

    fn handle_error(&mut self, _err: Error) {}
}

#[async_trait]
pub trait Handler: Sized {
    type Error: From<Error> + Send;

    async fn link(self, _node: &mut NodePrimitive, _ctrl: Link) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn send(
        self,
        _node: &mut NodePrimitive,
        _ctrl: SendCtrl,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn exit(self, _node: &mut NodePrimitive, _ctrl: Exit) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn unlink(self, _node: &mut NodePrimitive, _ctrl: UnLink) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn node_link(
        self,
        _node: &mut NodePrimitive,
        _ctrl: NodeLink,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn reg_send(
        self,
        _node: &mut NodePrimitive,
        _ctrl: RegSend,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn group_leader(
        self,
        _node: &mut NodePrimitive,
        _ctrl: GroupLeader,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn exit2(self, _node: &mut NodePrimitive, _ctrl: Exit2) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn send_tt(
        self,
        _node: &mut NodePrimitive,
        _ctrl: SendTT,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn exit_tt(self, _node: &mut NodePrimitive, _ctrl: ExitTT) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn reg_send_tt(
        self,
        _node: &mut NodePrimitive,
        _ctrl: RegSendTT,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn exit2_tt(
        self,
        _node: &mut NodePrimitive,
        _ctrl: Exit2TT,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn mointor_p(
        self,
        _node: &mut NodePrimitive,
        _ctrl: MonitorP,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn de_monitor_p(
        self,
        _node: &mut NodePrimitive,
        _ctrl: DeMonitorP,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn monitor_p_exit(
        self,
        _node: &mut NodePrimitive,
        _ctrl: MonitorPExit,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn send_sender(
        self,
        _node: &mut NodePrimitive,
        _ctrl: SendSender,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn send_sender_tt(
        self,
        _node: &mut NodePrimitive,
        _ctrl: SendSenderTT,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn payload_exit(
        self,
        _node: &mut NodePrimitive,
        _ctrl: PayloadExit,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn payload_exit_tt(
        self,
        _node: &mut NodePrimitive,
        _ctrl: PayloadExitTT,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn payload_exit2(
        self,
        _node: &mut NodePrimitive,
        _ctrl: PayloadExit2,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn payload_exit2_tt(
        self,
        _node: &mut NodePrimitive,
        _ctrl: PayloadExit2TT,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn payload_monitor_p_exit(
        self,
        _node: &mut NodePrimitive,
        _ctrl: PayloadMonitorPExit,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn spawn_request(
        self,
        _node: &mut NodePrimitive,
        _ctrl: SpawnRequest,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn spawn_request_tt(
        self,
        _node: &mut NodePrimitive,
        _ctrl: SpawnRequestTT,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn spawn_reply(
        self,
        _node: &mut NodePrimitive,
        _ctrl: SpawnReply,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn spawn_reply_tt(
        self,
        _node: &mut NodePrimitive,
        _ctrl: SpawnReplyTT,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn unlink_id(
        self,
        _node: &mut NodePrimitive,
        _ctrl: UnLinkId,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn unlink_id_ack(
        self,
        _node: &mut NodePrimitive,
        _ctrl: UnLinkIdAck,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn alias_send(
        self,
        _node: &mut NodePrimitive,
        _ctrl: AliasSend,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        Ok(self)
    }
    async fn alias_send_tt(
        self,
        _node: &mut NodePrimitive,
        _ctrl: AliasSendTT,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
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
    ChannelTryRecv(#[from] TryRecvError),
    #[error(transparent)]
    ChannelSendError(#[from] SendError<(Dist, oneshot::Sender<()>)>),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

fn get_short_hostname() -> String {
    let hostname = gethostname::gethostname();
    let hostname = hostname.to_string_lossy();
    hostname.split('.').next().unwrap().to_string()
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
        self.0.push(TuplePatternInner { idx, term });
        self
    }
}

#[derive(Debug, Clone)]
pub struct TuplePatternInner {
    pub idx: usize,
    pub term: term::Term,
}

impl PatternMatch for term::SmallTuple {
    type Pattern = TuplePattern;
    type Output = bool;

    fn pattern_match(&self, pattern: &Self::Pattern) -> Self::Output {
        pattern.0.iter().all(|x| match self.elems.get(x.idx) {
            Some(e) => x.term.eq(e),
            None => false,
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

        let pattern = TuplePattern::new(10)
            .with(0, SmallAtomUtf8("b".to_string()).into())
            .with(1, SmallAtomUtf8("b".to_string()).into());
        assert!(!tuple.pattern_match(&pattern));
    }
}
