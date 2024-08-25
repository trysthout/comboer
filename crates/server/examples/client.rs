use std::sync::Arc;

use motore::Service;
use tokio::sync::mpsc::UnboundedSender;

use proto::term::{SmallAtomUtf8, SmallTuple};
use proto::{etf::term, CtrlMsg, Encoder, RegSend, SendSender};
use server::{
    BoxStream, NodeAsClient, Process, ProcessContext, Request, Response,
    ServiceBuilder,
};

struct A {
    sender: UnboundedSender<Vec<u8>>,
}

#[derive(Debug, Clone)]
struct BoxCx {
    from: Process,
    ctrl: RegSend,
}

impl Service<ProcessContext<BoxCx>, Request<CtrlMsg<SendSender, SmallAtomUtf8>>> for A {
    type Response = Response<Option<BoxStream<'static, CtrlMsg<RegSend, term::Nil>>>>;
    type Error = server::Error;
    async fn call<'s, 'cx>(
        &'s self,
        cx: &'cx mut ProcessContext<BoxCx>,
        req: Request<CtrlMsg<SendSender, SmallAtomUtf8>>,
    ) -> Result<Self::Response, Self::Error> {
        println!("req {:?}", req.get_msg());
        if req.get_msg().msg == Some(SmallAtomUtf8("hi".to_string())) {
            let box_cx = cx.get_box_cx().unwrap();
            let ctrl = &box_cx.ctrl;
            let from = &box_cx.from;
            let msg = SmallTuple {
                arity: 2,
                elems: vec![
                    SmallAtomUtf8("i am rust client".to_string()).into(),
                    from.get_pid().into(),
                ],
            };

            let mut buf = Vec::new();
            let _ = CtrlMsg::new(ctrl, Some(msg)).encode(&mut buf);
            let _ = self.sender.send(buf);
        }
        Ok(Response::new(None))
    }
}

#[tokio::main]
async fn main() -> Result<(), server::Error> {
    let mut conn = NodeAsClient::new(
        "rust@fedora".to_string(),
        "aaa".to_string(),
        "127.0.0.1:4369",
        None,
    )
    .connect_local_by_name("a")
    .await?;

    let sender = conn.get_sender();
    conn.get_cx()
        .add_matcher(ServiceBuilder::new(Arc::new(A { sender })).build());

    let process = conn.get_cx().make_process();
    let from = process.get_pid();

    let ctrl = RegSend {
        from: from.clone(),
        unused: SmallAtomUtf8("".to_string()),
        to_name: SmallAtomUtf8("ss".to_string()),
    };

    let msg = SmallTuple {
        arity: 2,
        elems: vec![
            SmallAtomUtf8("call".to_string()).into(),
            from.clone().into(),
        ],
    };

    let _ = conn.send_req_data(CtrlMsg::new(&ctrl, Some(msg))).await?;
    conn.get_cx().set_box_cx(BoxCx {
        from: process,
        ctrl: ctrl.clone(),
    });

    let _ = conn.serving().await?;

    Ok(())
}
