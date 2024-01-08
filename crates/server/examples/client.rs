use proto::{etf::term, RegSend, CtrlMsg, SendCtrl};
use server::{NodePrimitive, AsClient, AtomPattern, PatternMatch};


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut node = NodePrimitive::new("rust@fedora".to_string(), "aaa".to_string());
    let mut client = node.connect_local_by_name("127.0.0.1:4369", "b").await?;

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
    }.into();

    let msg = term::SmallTuple{
        arity: 2,
        elems: vec![
            term::SmallAtomUtf8("call".to_string()).into(),
            from.clone().into(),
        ]
    }.into();

    let pattern = AtomPattern("hi".to_string());
    let pattern_fn = |term: &term::Term| -> bool {
       term::SmallAtomUtf8::try_from(term).map(|t| t.pattern_match(&pattern)).ok().unwrap_or_default() 
    };
    //res.send(ctrl, msg).await?;
    let res = client.send_and_wait(ctrl, msg, pattern_fn).await?;
    if let Some(dist) = res {
        if let CtrlMsg::SendSender(ctrl) = dist.ctrl_msg {
            let from = ctrl.from;
            let ctrl = SendCtrl {
                unused: term::SmallAtomUtf8("".to_string()),
                to: from,
            }.into();
            let msg = term::SmallTuple {
                    arity: 2,
                    elems: vec![
                        term::SmallAtomUtf8("from_rust".to_string()).into(),
                        term::SmallAtomUtf8("rust node".to_string()).into(),
                    ],
                }.into();

            client.send(ctrl, msg).await?;
        }
    }

    Ok(())
}