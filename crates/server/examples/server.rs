use proto::{ etf::term, SendCtrl, RegSend, Dist, Encoder, SendSender, Len};
use server::{NodePrimitive, AsServer, Handler };
use tokio::{net::TcpStream, io::AsyncWriteExt};

#[derive(Debug, Clone)]
struct A;
#[async_trait::async_trait]
impl Handler for A {
   type Error = anyhow::Error; 

   async fn reg_send(self, stream: &mut TcpStream, _ctrl: RegSend, msg: term::Term) -> Result<Self, Self::Error> {
        if let Ok(tuple) = term::SmallTuple::try_from(msg.clone()) {
            if let Ok(p) = term::NewPid::try_from(&tuple.elems[1]) {
                let mut data = vec![];
                let reg = SendCtrl {
                    unused: term::SmallAtomUtf8("".to_string()),
                    to: p,
                };

                let msg = term::SmallAtomUtf8("ccccccccccccccccc".to_string());
                let dist = Dist {
                    ctrl_msg: reg.into(),
                    msg: Some(msg.into()),
                };

                dist.encode(&mut data)?;
                println!("data1 {:?}", data);

                stream.write(&data).await?;
            }
        }

        Ok(self)
   }

   async fn send_sender(self, stream: &mut TcpStream, ctrl: SendSender, msg: term::Term) -> Result<Self, Self::Error> {
        let from = ctrl.from;
        let ctrl = SendCtrl {
            unused: term::SmallAtomUtf8("".to_string()),
            to: from,
        };
        let dist = Dist {
            ctrl_msg: ctrl.clone().into(),
            msg: Some(term::SmallTuple {
                arity: 2,
                elems: vec![
                    term::SmallAtomUtf8("from_rust".to_string()).into(),
                    term::SmallAtomUtf8("rust node".to_string()).into(),
                ],
            }.into()),
        };

        let mut buf = Vec::with_capacity(dist.len());
        dist.encode(&mut buf)?;
        stream.write(&buf).await?;

        Ok(self)
   }
}

#[tokio::main]
async fn main() {
    let mut node = NodePrimitive::new("rust".to_string(), "aaa".to_string());
    node.listen("127.0.0.1:4369", A).await.unwrap();
}
