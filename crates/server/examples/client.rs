use proto::etf::term::Term;
use proto::{etf::term, RegSend, SendCtrl};
use proto::{CtrlMsg, SendSender};
use server::{NodeAsClient, NodePrimitive};
use tokio::sync::mpsc;

#[derive(Debug)]
struct A {
    shutdown_tx: mpsc::Sender<()>,
}

#[async_trait::async_trait]
impl server::Handler for A {
    type Error = anyhow::Error;

    async fn reg_send(
        self,
        stream: &mut NodePrimitive,
        _ctrl: RegSend,
        msg: term::Term,
    ) -> Result<Self, Self::Error> {
        if let Ok(tuple) = term::SmallTuple::try_from(msg.clone()) {
            if let Ok(p) = term::NewPid::try_from(&tuple.elems[1]) {
                //let mut data = vec![];
                let reg = SendCtrl {
                    unused: term::SmallAtomUtf8("".to_string()),
                    to: p,
                };

                let msg = term::SmallAtomUtf8("ccccccccccccccccc".to_string());
                let dist = CtrlMsg {
                    ctrl: reg.into(),
                    msg: Some(msg.into()),
                };

                //dist.encode(&mut data)?;
                stream.send(dist).await;

                //stream.write_all(&data).await?;
            }
        }

        Ok(self)
    }

    async fn send_sender(
        self,
        stream: &mut NodePrimitive,
        ctrl: SendSender,
        _msg: term::Term,
    ) -> Result<Self, Self::Error> {
        let from = ctrl.from;
        let ctrl = SendCtrl {
            unused: term::SmallAtomUtf8("".to_string()),
            to: from,
        };
        let dist = CtrlMsg {
            ctrl: ctrl.clone().into(),
            msg: Some(
                term::SmallTuple {
                    arity: 2,
                    elems: vec![
                        term::SmallAtomUtf8("from_rust".to_string()).into(),
                        term::SmallAtomUtf8("rust node".to_string()).into(),
                    ],
                }
                .into(),
            ),
        };

        stream.send(dist).await;
        let _ = self.shutdown_tx.send(()).await;

        Ok(self)
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
    let handler = A { shutdown_tx };
    let node = NodeAsClient::new("rust@fedora".to_string(), "aaa".to_string());
    let mut client = node
        .connect_local_by_name("127.0.0.1:4369", "a", handler)
        .await?;

    let from = term::NewPid {
        node: term::SmallAtomUtf8("rust@fedora".to_string()),
        id: 0,
        serial: 0,
        creation: fastrand::u32(..),
    };

    let ctrl = RegSend {
        from: from.clone(),
        unused: term::SmallAtomUtf8("".to_string()),
        to_name: term::SmallAtomUtf8("ss".to_string()),
    }
    .into();

    let msg: Term = term::SmallTuple {
        arity: 2,
        elems: vec![
            term::SmallAtomUtf8("call".to_string()).into(),
            from.clone().into(),
        ],
    }
    .into();

    let _ = client.0.send(CtrlMsg::new(ctrl, Some(msg))).await;

    let _ = shutdown_rx.recv().await;

    Ok(())
}
