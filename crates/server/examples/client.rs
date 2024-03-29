use motore::Service;
use proto::{etf::term, RegSend};
use proto::{Ctrl, CtrlMsg};
use server::{EmptyBoxCx, NodeAsClient, ProcessContext};

#[derive(Clone)]
struct A;

impl Service<ProcessContext<EmptyBoxCx>, CtrlMsg> for A {
    type Response = bool;
    type Error = server::Error;
    async fn call<'s, 'cx>(
        &'s self,
        _cx: &'cx mut ProcessContext<EmptyBoxCx>,
        _req: CtrlMsg,
    ) -> Result<Self::Response, Self::Error> {
        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let node = NodeAsClient::new(
        "rust@fedora".to_string(),
        "aaa".to_string(),
        "127.0.0.1:4369",
    )
    .add_matcher(A);
    let mut conn = node.connect_local_by_name("a").await?;

    let process = conn.get_cx().make_process();
    let from = process.get_pid();

    let ctrl: Ctrl = RegSend {
        from: from.clone(),
        unused: term::SmallAtomUtf8("".to_string()),
        to_name: term::SmallAtomUtf8("ss".to_string()),
    }
    .into();

    let msg: term::Term = term::SmallTuple {
        arity: 2,
        elems: vec![
            term::SmallAtomUtf8("call".to_string()).into(),
            from.clone().into(),
        ],
    }
    .into();

    conn.get_cx().send(CtrlMsg::new(ctrl.clone(), Some(msg)));

    let msg: term::Term = term::SmallTuple {
        arity: 2,
        elems: vec![
            term::SmallAtomUtf8("from rust client".to_string()).into(),
            from.clone().into(),
        ],
    }
    .into();
    conn.get_cx().send(CtrlMsg::new(ctrl, Some(msg)));

    loop {
        tokio::select! {
            res = &mut conn => {
                println!("result {:?}", res)

            }
        }
    }
}
