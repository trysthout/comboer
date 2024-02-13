use proto::*;

pub mod primitive;
pub use primitive::*;
pub mod process;
pub use process::*;

use tokio::sync::mpsc::error::{SendError, TryRecvError};

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
    ChannelSendError(#[from] SendError<CtrlMsg>),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

impl From<BoxError> for Error {
    fn from(value: BoxError) -> Self {
        Self::Anyhow(anyhow::anyhow!(value))
    }
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

pub trait NamedMatcher {
    const NAME: &'static str;
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
