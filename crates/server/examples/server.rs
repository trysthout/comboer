use motore::Service;
use proto::{etf::term, term::PidOrAtom, Ctrl, CtrlMsg, ProcessKind, SendSender, SpawnReply};
use server::NodeAsServer;

#[derive(Debug, Clone)]
struct C(term::NewPid);

impl Service<server::ProcessContext, CtrlMsg> for C {
    type Response = bool;
    type Error = server::Error;
    async fn call<'s, 'cx>(
        &'s self,
        cx: &'cx mut server::ProcessContext,
        req: CtrlMsg,
    ) -> Result<Self::Response, Self::Error> {
        let Some(PidOrAtom::Pid(pid)) = req.ctrl.get_to_pid_atom() else {
            return Ok(false);
        };

        if pid != self.0 {
            return Ok(false);
        }

        let ctrl = SendSender::try_from(req.ctrl).unwrap();
        let ctrl = SendSender {
            from: ctrl.to,
            to: ctrl.from,
        };

        let msg = term::SmallTuple {
            arity: 2,
            elems: vec![
                term::SmallAtomUtf8("from_rust".to_string()).into(),
                term::SmallAtomUtf8("rust node".to_string()).into(),
            ],
        };

        let ctrl_msg = CtrlMsg::new(ctrl.into(), Some(msg.into()));
        cx.send(ctrl_msg);

        Ok(true)
    }
}

#[derive(Clone)]
struct B;

impl Service<server::ProcessContext, CtrlMsg> for B {
    type Response = bool;
    type Error = server::Error;
    async fn call<'s, 'cx>(
        &'s self,
        cx: &'cx mut server::ProcessContext,
        req: CtrlMsg,
    ) -> Result<Self::Response, Self::Error> {
        let Ctrl::SpawnRequest(req) = req.ctrl else {
            return Ok(false);
        };

        let result_process = cx.make_process();
        cx.add_matcher(C(result_process.get_pid()));

        let ctrl = SpawnReply {
            refe: req.refe,
            to: req.from,
            flags: term::SmallInteger(0),
            result: result_process.get_pid_ref().into(),
        };

        let ctrl_msg = CtrlMsg::new(ctrl.into(), None);
        cx.send(ctrl_msg);

        Ok(true)
    }
}

#[tokio::main]
async fn main() {
    let node = NodeAsServer::new("rust".to_string(), "aaa".to_string()).add_matcher(B);
    node.listen("127.0.0.1:4369").await.unwrap();
}
