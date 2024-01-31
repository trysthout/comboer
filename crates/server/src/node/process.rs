use proto::{term, Ctrl, CtrlMsg, OpCode};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    oneshot,
};

use crate::ProcessHandler;

#[derive(Debug)]
pub struct Process {
    pid: term::NewPid,
    receiver: UnboundedReceiver<(Ctrl, Option<term::Term>)>,
    sender: UnboundedSender<(CtrlMsg, oneshot::Sender<()>)>,
}

impl Process {
    pub fn new(
        pid: term::NewPid,
        receiver: UnboundedReceiver<(Ctrl, Option<term::Term>)>,
        sender: UnboundedSender<(CtrlMsg, oneshot::Sender<()>)>,
    ) -> Self {
        Self {
            pid,
            receiver,
            sender,
        }
    }

    pub async fn send(&self, dist: CtrlMsg) {
        let (wait_tx, wait_rx) = oneshot::channel::<()>();
        let _ = self.sender.send((dist, wait_tx));
        let _ = wait_rx.await;
    }

    pub async fn recv_origin(&mut self) -> Option<(Ctrl, Option<term::Term>)> {
        if let Some((ctrl, msg)) = self.receiver.recv().await {
            return Some((ctrl, msg));
        }

        None
    }

    pub async fn recv<C>(&mut self) -> Result<Option<(C, Option<term::Term>)>, anyhow::Error>
    where
        C: OpCode + TryFrom<Ctrl, Error = anyhow::Error>,
    {
        if let Some((ctrl, msg)) = self.receiver.recv().await {
            let c = C::try_from(ctrl)?;
            return Ok(Some((c, msg)));
        }

        Ok(None)
    }

    pub async fn recv_with_call<P, C>(
        &mut self,
        mut handler: P,
    ) -> Option<(Ctrl, Option<term::Term>)>
    where
        P: ProcessHandler<C>,
        C: OpCode + TryFrom<Ctrl, Error = anyhow::Error>,
    {
        if let Some((ctrl, msg)) = self.receiver.recv().await {
            let c = C::try_from(ctrl).unwrap();
            let _ = handler.call(c, msg).await;
        }

        None
    }

    pub fn get_pid(&self) -> term::NewPid {
        self.pid.clone()
    }

    pub fn get_pid_ref(&self) -> &term::NewPid {
        &self.pid
    }
}
