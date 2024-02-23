use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use dashmap::DashMap;
use motore::{BoxCloneService, Service};
use proto::{
    term::{self},
    CtrlMsg,
};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug)]
pub struct Process {
    pid: term::NewPid,
    sender: UnboundedSender<CtrlMsg>,
}

impl Process {
    pub fn new(pid: term::NewPid, sender: UnboundedSender<CtrlMsg>) -> Self {
        Self { pid, sender }
    }

    pub fn get_pid(&self) -> term::NewPid {
        self.pid.clone()
    }

    pub fn get_pid_ref(&self) -> &term::NewPid {
        &self.pid
    }

    pub fn send(&self, msg: CtrlMsg) {
        let _ = self.sender.send(msg);
    }
}

static UNIQ_ID: AtomicU64 = AtomicU64::new(1);
static PID_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct ProcessContext {
    node_name: String,
    creation: u32,
    sender: UnboundedSender<CtrlMsg>,
    dispatcher: Dispatcher,
    curr_match_id: MatchId,
}

impl ProcessContext {
    pub fn with_dispathcer(
        node_name: String,
        creation: u32,
        dispatcher: Dispatcher,
        sender: UnboundedSender<CtrlMsg>,
    ) -> Self {
        Self {
            node_name,
            creation,
            curr_match_id: MatchId(0),
            dispatcher,
            sender,
        }
    }

    pub fn new(node_name: String, creation: u32, sender: UnboundedSender<CtrlMsg>) -> Self {
        Self {
            node_name,
            creation,
            curr_match_id: MatchId(0),
            dispatcher: Dispatcher::new(),
            sender,
        }
    }

    pub fn send(&self, ctrl_msg: CtrlMsg) {
        let _ = self.sender.send(ctrl_msg);
    }

    pub fn get_matcher(&self) -> Dispatcher {
        self.dispatcher.clone()
    }

    pub fn add_matcher<M>(&mut self, matcher: M) -> &mut Self
    where
        M: Service<ProcessContext, CtrlMsg, Response = bool, Error = crate::Error>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        let matcher_id = MatchId::next();
        self.dispatcher.add_matcher(matcher_id, matcher);
        self
    }

    fn set_match_id(&mut self, id: MatchId) {
        self.curr_match_id = id;
    }

    pub fn make_process(&self) -> Process {
        let pid_id = PID_ID.fetch_add(1, Ordering::Relaxed);
        let pid = term::NewPid {
            node: term::SmallAtomUtf8(self.node_name.clone()),
            id: (pid_id & 0x7fff) as u32,
            serial: ((pid_id >> 15) & 0x1fff) as u32,
            creation: self.creation,
        };

        Process::new(pid, self.sender.clone())
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

impl Service<ProcessContext, CtrlMsg> for ProcessContext {
    type Response = bool;
    type Error = crate::Error;
    async fn call<'s, 'cx>(
        &'s self,
        cx: &'cx mut ProcessContext,
        req: CtrlMsg,
    ) -> Result<Self::Response, Self::Error> {
        self.dispatcher.call(cx, req).await
    }
}

static MATCH_ID: AtomicU32 = AtomicU32::new(0);
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MatchId(u32);

impl MatchId {
    pub fn next() -> Self {
        let id = MATCH_ID.fetch_add(1, Ordering::Relaxed);
        Self(id)
    }
}

#[derive(Debug, Clone)]
pub struct Dispatcher {
    matchers: DashMap<MatchId, BoxCloneService<ProcessContext, CtrlMsg, bool, crate::Error>>,
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            matchers: DashMap::new(),
        }
    }

    pub fn add_matcher<M>(&self, matcher_id: MatchId, matcher: M)
    where
        M: Service<ProcessContext, CtrlMsg, Response = bool, Error = crate::Error>
            + Send
            + Sync
            + Clone
            + 'static,
    {
        self.matchers
            .insert(matcher_id, BoxCloneService::new(matcher));
    }
}

impl Service<ProcessContext, CtrlMsg> for Dispatcher {
    type Response = bool;
    type Error = crate::Error;
    async fn call<'s, 'cx>(
        &'s self,
        cx: &'cx mut ProcessContext,
        req: CtrlMsg,
    ) -> Result<Self::Response, Self::Error> {
        for matcher in self.matchers.iter() {
            let req = req.clone();
            let is_match = matcher.call(cx, req).await?;
            if is_match {
                cx.set_match_id(*matcher.key());
                return Ok(true);
            }
        }
        Ok(false)
    }
}
