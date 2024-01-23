use proto::{etf::term, Dist, RegSend, SendCtrl, SendSender};
use server::{AsServer, Handler, NodeAsServer, NodePrimitive};

#[derive(Debug, Clone)]
struct A;

impl AsServer for A {
    type Handler = Self;

    fn new_session(&mut self) -> Result<Self::Handler, server::Error> {
        Ok(self.clone())
    }
}

#[async_trait::async_trait]
impl Handler for A {
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
                let dist = Dist {
                    ctrl_msg: reg.into(),
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
        let dist = Dist {
            ctrl_msg: ctrl.clone().into(),
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

        Ok(self)
    }
}

#[tokio::main]
async fn main() {
    let mut node = NodeAsServer::new("rust".to_string(), "aaa".to_string());
    node.listen("127.0.0.1:4369", A).await.unwrap();
}
