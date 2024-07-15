use futures_util::{stream, StreamExt};
use motore::Service;
use tokio_stream::wrappers::ReceiverStream;

use proto::{
    CtrlMsg, etf::term, ProcessKind, SendSender, SpawnReply, SpawnRequest, term::PidOrAtom,
};
use server::{BoxStream, EmptyBoxCx, NodeAsServer, ProcessContext, Request, Response};

#[derive(Debug, Clone)]
struct C(term::NewPid);

impl Service<ProcessContext<EmptyBoxCx>, Request<CtrlMsg<SendSender, term::SmallTuple>>> for C {
    // type Response = Response<Option<CtrlMsg<SendSender, term::SmallTuple>>>;
    type Response = Response<Option<BoxStream<'static, CtrlMsg<SendSender, term::SmallTuple>>>>;
    type Error = server::Error;
    async fn call<'s, 'cx>(
        &'s self,
        _cx: &'cx mut ProcessContext<EmptyBoxCx>,
        req: Request<CtrlMsg<SendSender, term::SmallTuple>>,
    ) -> Result<Self::Response, Self::Error> {
        let ctrl = &req.get_msg().ctrl;
        let Some(PidOrAtom::Pid(pid)) = ctrl.get_to_pid_atom() else {
            return Ok(Response::new(None));
        };

        if pid != self.0 {
            return Ok(Response::new(None));
        }

        let ctrl = SendSender {
            from: ctrl.to.clone(),
            to: ctrl.from.clone(),
        };

        let msg: term::SmallTuple = vec![
            ctrl.from.clone().into(),
            term::SmallAtomUtf8("from_rust0".to_string()).into(),
            term::SmallAtomUtf8("rust node".to_string()).into(),
        ]
        .into();

        let msg1: term::SmallTuple = vec![
            term::SmallAtomUtf8("EXIT".to_string()).into(),
            term::SmallAtomUtf8("rust node exit".to_string()).into(),
        ]
        .into();

        let (tx, rx) = tokio::sync::mpsc::channel(210);
        let ctrl_msg = CtrlMsg::new(ctrl.clone(), Some(msg));
        let mut resp = stream::repeat(ctrl_msg).take(10);
        let exit_msg = CtrlMsg::new(ctrl.clone(), Some(msg1));

        tokio::spawn(async move {
            while let Some(data) = resp.next().await {
                let _ = tx.send(data).await;
            }
            tx.send(exit_msg).await.unwrap();
        });

        Ok(Response::new(Some(Box::pin(ReceiverStream::new(rx)))))
    }
}

#[derive(Clone)]
struct B;

impl Service<ProcessContext<EmptyBoxCx>, Request<CtrlMsg<SpawnRequest, term::Nil>>> for B {
    type Response = Response<Option<BoxStream<'static, CtrlMsg<SpawnReply, term::Nil>>>>;
    type Error = server::Error;
    async fn call<'s, 'cx>(
        &'s self,
        cx: &'cx mut ProcessContext<EmptyBoxCx>,
        req: Request<CtrlMsg<SpawnRequest, term::Nil>>,
    ) -> Result<Self::Response, Self::Error> {
        let ctrl = &req.get_msg().ctrl;
        let result_process = cx.make_process();
        let c = C(result_process.get_pid());
        let c = server::ServiceBuilder::new(c).build();
        cx.add_matcher(c);

        let ctrl = SpawnReply {
            refe: ctrl.refe.clone(),
            to: ctrl.from.clone(),
            flags: term::SmallInteger(0),
            result: result_process.get_pid_ref().into(),
        };

        let ctrl_msg = CtrlMsg::new(ctrl, None);
        Ok(Response::new(Some(Box::pin(futures::stream::once(
            async { ctrl_msg },
        )))))
    }
}

#[tokio::main]
async fn main() {
    let s = server::ServiceBuilder::new(B).build();
    let node =
        NodeAsServer::new("rust".to_string(), "aaa".to_string(), "127.0.0.1:4369").add_matcher(s);
    node.listen().await.unwrap();
}
