use bytes::Buf;

use crate::{dist::OpCode, etf::term::*, Encoder, Len};

macro_rules! define_ctrl {
    (
        $(#[$outer:meta])*
        $sn:ident { $($f:ident:$ft:ty),* }, $op:expr, $num:expr) => {
        $(#[$outer])*
        #[derive(Debug, Clone)]
        pub struct $sn {
            $(pub $f: $ft),*
        }

        impl OpCode for $sn {
            const CODE: u8 = $op;
            const ARITY: u8 = $num;
        }

        impl From<&[u8]> for $sn {
            fn from(mut value: &[u8]) -> Self {
                // SmallTuple 104
                value.get_u8();
                // SmallTuple arity
                value.get_u8();

                let code  = SmallInteger::from(value);
                value.advance(code.len());

                $(
                    let $f = <$ft>::from(value);
                    value.advance($f.len());
                )*

                Self {
                    $( $f, )*
                }
            }
        }

        impl Encoder for $sn {
            type Error = anyhow::Error;
            fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), Self::Error> {
                w.write_all(&crate::etf::SMALL_TUPLE_EXT.to_be_bytes())?;
                w.write_all(&Self::ARITY.to_be_bytes())?;
                SmallInteger(Self::CODE).encode(w)?;

                $(
                    self.$f.encode(w)?;
                )*

                Ok(())
            }
        }

        impl Len for $sn {
            fn len(&self) -> usize {
               #[allow(unused_mut)]
               // SmallInteger length, SmallTuple includes tag and arity length
               let mut total = 2 + 2;
               $(total += self.$f.len();)*
               total
            }
       }
    };
}

define_ctrl!(
    #[doc = "
{1, FromPid, ToPid}
This signal is sent by FromPid in order to create a link between FromPid and ToPid.
    "]
    Link {
        from: NewPid,
        to: NewPid
    },
    1,
    3
);

define_ctrl!(
    #[doc = "
 SEND
 {2, Unused, ToPid}
 Followed by Message.

    "]
    SendCtrl {
        unused: SmallAtomUtf8,
        to: NewPid
    },
    2,
    3
);

define_ctrl!(
    #[doc = "
 EXIT
 {3, FromPid, ToPid, Reason}
 This signal is sent when a link has been broken
    "]
    Exit {
        from: NewPid,
        to: NewPid,
        reason: SmallAtomUtf8
    },
    3,
    4
);

define_ctrl!(
    #[doc = "
 UNLINK (obsolete)
 {4, FromPid, ToPid}
    "]
    UnLink {
        from: NewPid,
        to: NewPid
    },
    4,
    3
);

define_ctrl!(
    #[doc = "
NODE_LINK
{5}
    "]
    NodeLink {},
    5,
    1
);

define_ctrl!(
    #[doc = "
REG_SEND
{6, FromPid, Unused, ToName}
Followed by Message.
Unused is kept for backward compatibility.
    "]
    RegSend {
        from: NewPid,
        unused: SmallAtomUtf8,
        to_name: SmallAtomUtf8
    },
    6,
    4
);

define_ctrl!(
    #[doc = "
GROUP_LEADER
{7, FromPid, ToPid}
    "]
    GroupLeader {
        from: NewPid,
        to: NewPid
    },
    7,
    3
);

define_ctrl!(
    #[doc = "
EXIT2
{8, FromPid, ToPid, Reason}
    "]
    Exit2 {
        from: NewPid,
        to: NewPid,
        reason: SmallAtomUtf8
    },
    8,
    4
);

define_ctrl!(
    #[doc = "
SEND_TT
{12, Unused, ToPid, TraceToken}
Followed by Message.
Unused is kept for backward compatibility.
    "]
    SendTT {
        unused: SmallAtomUtf8,
        to: NewPid,
        trace_token: SmallAtomUtf8
    },
    12,
    4
);

define_ctrl!(
    #[doc = "
// EXIT_TT
// {13, FromPid, ToPid, TraceToken, Reason}
    "]
    ExitTT {
        from: NewPid,
        to: NewPid,
        trace_token: SmallAtomUtf8,
        reason: SmallAtomUtf8
    },
    13,
    5
);

define_ctrl!(
    #[doc = "
REG_SEND_TT
{16, FromPid, Unused, ToName, TraceToken}

Followed by Message.

Unused is kept for backward compatibility.
    "]
    RegSendTT {
        from: NewPid,
        unused: SmallAtomUtf8,
        to_name: SmallAtomUtf8,
        trace_token: SmallAtomUtf8
    },
    16,
    5
);

define_ctrl!(
    #[doc = "
EXIT2_TT
{18, FromPid, ToPid, TraceToken, Reason}
    "]
    Exit2TT {
        from: NewPid,
        to: NewPid,
        trace_token: SmallAtomUtf8,
        reason: SmallAtomUtf8
    },
    18,
    5
);

//
define_ctrl!(
    #[doc = "
MONITOR_P
{19, FromPid, ToProc, Ref}, where FromPid = monitoring process and ToProc = monitored process pid or name (atom)
    "]
    MonitorP {
        from: NewPid,
        to_proc: PidOrAtom,
        refe: NewerReference
    },
    19,
    4
);

define_ctrl!(
    #[doc = "
DEMONITOR_P
{20, FromPid, ToProc, Ref}, where FromPid = monitoring process and ToProc = monitored process pid or name (atom)

We include FromPid just in case we want to trace this.
    "]
    DeMonitorP {
        from: NewPid,
        to_proc: PidOrAtom,
        refe: NewerReference
    },
    20,
    4
);

define_ctrl!(
    #[doc = "
MONITOR_P_EXIT
{21, FromProc, ToPid, Ref, Reason}, where FromProc = monitored process pid or name (atom), ToPid = monitoring process, and Reason = exit reason for the monitored process
    "]
    MonitorPExit {
        from_proc: PidOrAtom,
        to: NewPid,
        refe: NewerReference,
        reason: SmallAtomUtf8
    },
    21,
    5
);

//

//
define_ctrl!(
    #[doc = "
SEND_SENDER
{22, FromPid, ToPid}

Followed by Message.
This control message replaces the SEND control message and will be sent when the distribution flag DFLAG_SEND_SENDER has been negotiated in the connection setup handshake.
    "]
    SendSender {
        from: NewPid,
        to: NewPid
    },
    22,
    3
);

//

//
define_ctrl!(
    #[doc = "
SEND_SENDER_TT
{23, FromPid, ToPid, TraceToken}
Followed by Message.
This control message replaces the SEND_TT control message and will be sent when the distribution flag DFLAG_SEND_SENDER has been negotiated in the connection setup handshake.
    "]
    SendSenderTT {
        from: NewPid,
        to: NewPid,
        trace_token: SmallAtomUtf8
    },
    23,
    4
);

//
define_ctrl!(
    #[doc = "
PAYLOAD_EXIT
{24, FromPid, ToPid}

Followed by Reason.

This control message replaces the EXIT control message and will be sent when the distribution flag DFLAG_EXIT_PAYLOAD has been negotiated in the connection setup handshake.
    "]
    PayloadExit {
        from: NewPid,
        to: NewPid
    },
    24,
    3
);

//
define_ctrl!(
    #[doc = "
PAYLOAD_EXIT_TT
{25, FromPid, ToPid, TraceToken}

Followed by Reason.

This control message replaces the EXIT_TT control message and will be sent when the distribution flag DFLAG_EXIT_PAYLOAD has been negotiated in the connection setup handshake.
    "]
    PayloadExitTT {
        from: NewPid,
        to: NewPid,
        trace_token: SmallAtomUtf8
    },
    25,
    4
);

define_ctrl!(
    #[doc = "
PAYLOAD_EXIT2
{26, FromPid, ToPid}

Followed by Reason.

This control message replaces the EXIT2 control message and will be sent when the distribution flag DFLAG_EXIT_PAYLOAD has been negotiated in the connection setup handshake.
    "]
    PayloadExit2 {
        from: NewPid,
        to: NewPid
    },
    26,
    3
);

//
define_ctrl!(
    #[doc = "
PAYLOAD_EXIT2_TT
{27, FromPid, ToPid, TraceToken}

Followed by Reason.

This control message replaces the EXIT2_TT control message and will be sent when the distribution flag DFLAG_EXIT_PAYLOAD has been negotiated in the connection setup handshake.
    "]
    PayloadExit2TT {
        from: NewPid,
        to: NewPid,
        trace_token: SmallAtomUtf8
    },
    27,
    4
);

define_ctrl!(
    #[doc = "
PAYLOAD_MONITOR_P_EXIT
{28, FromProc, ToPid, Ref}

Followed by Reason.

This control message replaces the MONITOR_P_EXIT control message and will be sent when the distribution flag DFLAG_EXIT_PAYLOAD has been negotiated in the connection setup handshake.
    
    "]
    PayloadMonitorPExit {
        from_proc: PidOrAtom,
        to: NewPid,
        refe: NewerReference
    },
    28,
    4
);

#[derive(Debug, Clone)]
pub struct MFA {
    pub module: SmallAtomUtf8,
    pub fun: SmallAtomUtf8,
    pub arity: SmallInteger,
}

impl Encoder for MFA {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), Self::Error> {
        let tuple = SmallTuple {
            arity: 3,
            elems: vec![
                self.module.clone().into(),
                self.fun.clone().into(),
                self.arity.clone().into(),
            ],
        };

        tuple.encode(w)?;

        Ok(())
    }
}

impl From<&[u8]> for MFA {
    fn from(value: &[u8]) -> Self {
        let tuple = SmallTuple::from(value);
        let module: SmallAtomUtf8 = tuple.elems[0].clone().try_into().unwrap();
        let fun: SmallAtomUtf8 = tuple.elems[1].clone().try_into().unwrap();
        let arity: SmallInteger = tuple.elems[2].clone().try_into().unwrap();
        Self { module, fun, arity }
    }
}

impl Len for MFA {
    fn len(&self) -> usize {
        1 + 1 + self.module.len() + self.fun.len() + self.arity.len()
    }
}

define_ctrl!(
    #[doc = "
SPAWN_REQUEST
{29, ReqId, From, GroupLeader, {Module, Function, Arity}, OptList}

Followed by ArgList.

This signal is sent by the spawn_request() BIF.

ReqId :: reference()
Request identifier. Also used as monitor reference in case the monitor option has been passed.

From :: pid()
Process identifier of the process making the request. That is, the parent process to be.

GroupLeader :: pid()
Process identifier of the group leader of the newly created process.

{Module :: atom(), Function :: atom(), Arity :: integer() >= 0}
Entry point for the new process.

OptList :: [term()]
A proper list of spawn options to use when spawning.

ArgList :: [term()]
A proper list of arguments to use in the call to the entry point.

Only supported when the DFLAG_SPAWN distribution flag has been passed.
    "]
    SpawnRequest {
        refe: NewerReference,
        from: NewPid,
        group_leader: NewPid,
        mfa: MFA,
        opts: Term
    },
    29,
    6
);

define_ctrl!(
    #[doc = "
SPAWN_REQUEST_TT
{30, ReqId, From, GroupLeader, {Module, Function, Arity}, OptList, Token}

Followed by ArgList.

Same as SPAWN_REQUEST, but also with a sequential trace Token.

Only supported when the DFLAG_SPAWN distribution flag has been passed.
    "]
    SpawnRequestTT {
        refe: NewerReference,
        from: NewPid,
        group_leader: NewPid,
        mfa: MFA,
        opts: Term,
        token: SmallAtomUtf8
    },
    30,
    7
);

define_ctrl!(
    #[doc = "
SPAWN_REPLY
{31, ReqId, To, Flags, Result}

This signal is sent as a reply to a process previously sending a SPAWN_REQUEST signal.

ReqId :: reference()
Request identifier. Also used as monitor reference in case the monitor option has been passed.

To :: pid()
Process identifier of the process making the spawn request.

Flags :: integer() >= 0
A bit flag field of bit flags bitwise or:ed together. Currently the following flags are defined:

1
A link between To and Result was set up on the node where Result resides.

2
A monitor from To to Result was set up on the node where Result resides.

Result :: pid() | atom()
Result of the operation. If Result is a process identifier, the operation succeeded and the process identifier is the identifier of the newly created process. If Result is an atom, the operation failed and the atom identifies failure reason.

Only supported when the DFLAG_SPAWN distribution flag has been passed.
    "]
    SpawnReply {
        refe: NewerReference,
        to: NewPid,
        flags: SmallInteger,
        result: PidOrAtom
    },
    31,
    5
);

define_ctrl!(
    #[doc = "
SPAWN_REPLY_TT
{32, ReqId, To, Flags, Result, Token}

Same as SPAWN_REPLY, but also with a sequential trace Token.

Only supported when the DFLAG_SPAWN distribution flag has been passed.
    "]
    SpawnReplyTT {
        refe: NewerReference,
        to: NewPid,
        flags: SmallInteger,
        result: PidOrAtom,
        token: SmallAtomUtf8
    },
    32,
    6
);

define_ctrl!(
    #[doc = "
UNLINK_ID
{35, Id, FromPid, ToPid}

This signal is sent by FromPid in order to remove a link between FromPid and ToPid. This unlink signal replaces the UNLINK signal.
Besides process identifiers of the sender and receiver the UNLINK_ID signal also contains an integer identifier Id.
Valid range of Id is [1, (1 bsl 64) - 1]. Id is to be passed back to the sender by the receiver in an UNLINK_ID_ACK signal.
Id must uniquely identify the UNLINK_ID signal among all not yet acknowledged UNLINK_ID signals from FromPid to ToPid.

This signal is part of the new link protocol which became mandatory as of OTP 26.
    "]
    UnLinkId {
        id: SmallBig,
        from: NewPid,
        to: NewPid
    },
    35,
    4
);

define_ctrl!(
    #[doc = "
UNLINK_ID_ACK
{36, Id, FromPid, ToPid}

An unlink acknowledgement signal. This signal is sent as an acknowledgement of the reception of an UNLINK_ID signal.
The Id element should be the same Id as present in the UNLINK_ID signal. FromPid identifies the sender of the UNLINK_ID_ACK signal
and ToPid identifies the sender of the UNLINK_ID signal.

This signal is part of the new link protocol which became mandatory as of OTP 26.
    "]
    UnLinkIdAck {
        id: SmallBig,
        from: NewPid,
        to: NewPid
    },
    36,
    4
);

define_ctrl!(
    #[doc = "
New Ctrlmessages for Erlang/OTP 24
ALIAS_SEND
{33, FromPid, Alias}

Followed by Message.

This control message is used when sending the message Message to the process identified by the process alias Alias.
Nodes that can handle this control message sets the distribution flag DFLAG_ALIAS in the connection setup handshake.
    "]
    AliasSend {
        from: NewPid,
        alias: SmallAtomUtf8
    },
    33,
    3
);

define_ctrl!(
    #[doc = "
ALIAS_SEND_TT
{34, FromPid, Alias, Token}

Followed by Message.

Same as ALIAS_SEND, but also with a sequential trace Token

    "]
    AliasSendTT {
        from: NewPid,
        alias: SmallAtomUtf8,
        token: SmallAtomUtf8
    },
    34,
    4
);

macro_rules! impl_ctrl {
    ($($t:ident),+) => {
        #[derive(Debug, Clone)]
        pub enum Ctrl {
            $($t($t),)+
        }

        $(
            impl From<$t> for Ctrl {
                fn from(value: $t) -> Self {
                    Self::$t(value)
                }
            }

            impl From<&$t> for Ctrl {
                fn from(value: &$t) -> Self {
                    Self::$t(value.clone())
                }
            }
        )+

        $(
            impl TryFrom<Ctrl> for $t {
                type Error = anyhow::Error;
                fn try_from(value: Ctrl) -> Result<Self, Self::Error> {
                    if let Ctrl::$t(v) = value {
                        Ok(v)
                    } else {
                        Err(anyhow::anyhow!("cannot convert value type"))
                    }
                }
            }
        )+

        impl Encoder for Ctrl {
            type Error = anyhow::Error;
           fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), Self::Error> {
                match self {
                    $(
                        Self::$t(v) => v.encode(w),
                    )+
                }
           }
        }

        impl TryFrom<&[u8]> for Ctrl {
            type Error = anyhow::Error;
            fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
                match value[3] {
                    $($t::CODE => Ok(Self::$t($t::from(value))),)+
                    _ => Err(anyhow::anyhow!("cannot convert value type to CtrlMsg"))
                }
            }
        }

        impl Len for Ctrl {
            fn len(&self) -> usize {
                match self {
                    $(
                        Self::$t(v) => v.len(),
                    )+
                }
            }
        }
    }
}

impl_ctrl!(
    Link,
    SendCtrl,
    Exit,
    UnLink,
    NodeLink,
    RegSend,
    GroupLeader,
    Exit2,
    SendTT,
    ExitTT,
    RegSendTT,
    Exit2TT,
    MonitorP,
    DeMonitorP,
    MonitorPExit,
    SendSender,
    SendSenderTT,
    PayloadExit,
    PayloadExitTT,
    PayloadExit2,
    PayloadExit2TT,
    PayloadMonitorPExit,
    SpawnRequest,
    SpawnRequestTT,
    SpawnReply,
    SpawnReplyTT,
    UnLinkId,
    UnLinkIdAck,
    AliasSend,
    AliasSendTT
);

impl Ctrl {
    pub fn get_from_pid_atom(&self) -> Option<PidOrAtom> {
        match self {
            Self::AliasSend(v) => Some((&v.from).into()),
            Self::AliasSendTT(v) => Some((&v.from).into()),
            Self::Link(v) => Some((&v.from).into()),
            Self::SendCtrl(_) => None,
            Self::Exit(v) => Some((&v.from).into()),
            Self::UnLink(v) => Some((&v.from).into()),
            Self::NodeLink(_) => None,
            Self::RegSend(v) => Some((&v.from).into()),
            Self::GroupLeader(v) => Some((&v.from).into()),
            Self::Exit2(v) => Some((&v.from).into()),
            Self::SendTT(_) => None,
            Self::ExitTT(v) => Some((&v.from).into()),
            Self::RegSendTT(v) => Some((&v.from).into()),
            Self::Exit2TT(v) => Some((&v.from).into()),
            Self::MonitorP(v) => Some((&v.from).into()),
            Self::DeMonitorP(v) => Some((&v.from).into()),
            Self::MonitorPExit(v) => Some(v.from_proc.clone()),
            Self::SendSender(v) => Some((&v.from).into()),
            Self::SendSenderTT(v) => Some((&v.from).into()),
            Self::PayloadExit(v) => Some((&v.from).into()),
            Self::PayloadExitTT(v) => Some((&v.from).into()),
            Self::PayloadExit2(v) => Some((&v.from).into()),
            Self::PayloadExit2TT(v) => Some((&v.from).into()),
            Self::PayloadMonitorPExit(v) => Some(v.from_proc.clone()),
            Self::SpawnRequest(v) => Some((&v.from).into()),
            Self::SpawnRequestTT(v) => Some((&v.from).into()),
            Self::SpawnReply(_) => None,
            Self::SpawnReplyTT(_) => None,
            Self::UnLinkId(v) => Some((&v.from).into()),
            Self::UnLinkIdAck(v) => Some((&v.from).into()),
        }
    }

    pub fn get_to_pid_atom(&self) -> Option<PidOrAtom> {
        match self {
            Self::AliasSend(v) => Some((&v.alias).into()),
            Self::AliasSendTT(v) => Some((&v.alias).into()),
            Self::Link(v) => Some((&v.to).into()),
            Self::SendCtrl(_) => None,
            Self::Exit(v) => Some((&v.to).into()),
            Self::UnLink(v) => Some((&v.to).into()),
            Self::NodeLink(_) => None,
            Self::RegSend(v) => Some((&v.to_name).into()),
            Self::GroupLeader(v) => Some((&v.to).into()),
            Self::Exit2(v) => Some((&v.to).into()),
            Self::SendTT(_) => None,
            Self::ExitTT(v) => Some((&v.to).into()),
            Self::RegSendTT(v) => Some((&v.to_name).into()),
            Self::Exit2TT(v) => Some((&v.to).into()),
            Self::MonitorP(v) => Some(v.to_proc.clone()),
            Self::DeMonitorP(v) => Some(v.to_proc.clone()),
            Self::MonitorPExit(v) => Some((&v.to).into()),
            Self::SendSender(v) => Some((&v.to).into()),
            Self::SendSenderTT(v) => Some((&v.to).into()),
            Self::PayloadExit(v) => Some((&v.to).into()),
            Self::PayloadExitTT(v) => Some((&v.to).into()),
            Self::PayloadExit2(v) => Some((&v.to).into()),
            Self::PayloadExit2TT(v) => Some((&v.to).into()),
            Self::PayloadMonitorPExit(v) => Some((&v.to).into()),
            Self::SpawnRequest(v) => Some((&v.group_leader).into()),
            Self::SpawnRequestTT(v) => Some((&v.group_leader).into()),
            Self::SpawnReply(_) => None,
            Self::SpawnReplyTT(_) => None,
            Self::UnLinkId(v) => Some((&v.to).into()),
            Self::UnLinkIdAck(v) => Some((&v.to).into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CtrlMsg {
    pub ctrl: Ctrl,
    pub msg: Option<Term>,
}

impl CtrlMsg {
    pub fn new(ctrl: Ctrl, msg: Option<Term>) -> Self {
        Self { ctrl, msg }
    }
}

impl Encoder for CtrlMsg {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), Self::Error> {
        w.write_all(&(self.len() as u32).to_be_bytes())?;
        w.write_all(&112_u8.to_be_bytes())?;
        w.write_all(&131_u8.to_be_bytes())?;
        self.ctrl.encode(w)?;
        if let Some(term) = &self.msg {
            w.write_all(&131_u8.to_be_bytes())?;
            term.encode(w)?;
        }

        Ok(())
    }
}

impl TryFrom<&[u8]> for CtrlMsg {
    type Error = anyhow::Error;
    fn try_from(mut value: &[u8]) -> Result<Self, Self::Error> {
        // 112
        value.get_u8();
        // 131
        value.get_u8();
        let ctrl = Ctrl::try_from(value)?;
        value.advance(ctrl.len());
        let msg = (!value.is_empty()).then(|| {
            // 131
            value.get_u8();
            let term = Term::from(value);
            value.advance(term.len());
            term
        });

        Ok(Self { ctrl, msg })
    }
}

impl Len for CtrlMsg {
    fn len(&self) -> usize {
        2 + self.ctrl.len() + 1 + self.msg.as_ref().map(|t| t.len()).unwrap_or_default()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn send() {
        let buf = [
            104, 3, 97, 2, 119, 0, 88, 119, 8, 97, 64, 102, 101, 100, 111, 114, 97, 0, 0, 0, 116,
            0, 0, 0, 0, 101, 136, 14, 0,
        ];
        let s = SendCtrl::from(&buf[..]);
        assert_eq!(s.unused.0, "");
        assert_eq!(s.to.id, 116);
        assert_eq!(s.len(), 29);
        println!("{:?}", s.len());

        // dist
        let buf = vec![
            112, 131, 104, 3, 97, 2, 119, 0, 88, 119, 8, 97, 64, 102, 101, 100, 111, 114, 97, 0, 0,
            0, 116, 0, 0, 0, 0, 101, 136, 14, 0, 131, 104, 2, 119, 2, 104, 105, 88, 119, 8, 97, 64,
            102, 101, 100, 111, 114, 97, 0, 0, 0, 116, 0, 0, 0, 0, 101, 136, 14, 0,
        ];
        let dist = CtrlMsg::try_from(&buf[..]).unwrap();
        println!("dist {:?}", dist.len());
        assert!(matches!(dist.ctrl, Ctrl::SendCtrl(_)));
        assert!(dist.msg.is_some());
        assert!(matches!(dist.msg.as_ref().unwrap(), Term::SmallTuple(_)));
        assert_eq!(dist.len(), 61);

        let mut expeted = vec![];
        dist.encode(&mut expeted).unwrap();
        assert_eq!(buf, expeted[4..]);
        println!("{:?}", buf);
        println!("{:?}", expeted);
    }

    #[test]
    fn exit() {}

    //#[test]
    //fn send() {
    //    let buf = vec![
    //        0x61, 0x06, 0x58, 0x77, 0x08, 0x62, 0x40, 0x66, 0x65, 0x64, 0x6f, 0x72, 0x61, 0x00,
    //        0x00, 0x00, 0x39, 0x00, 0x00, 0x00, 0x00, 0x65, 0x88, 0x36, 0x16, 0x83, 0x68, 0x02,
    //        0x58, 0x77, 0x08, 0x62, 0x40, 0x66, 0x65, 0x64, 0x6f, 0x72, 0x61, 0x00, 0x00, 0x00,
    //        0x39, 0x00, 0x00, 0x00, 0x00, 0x65, 0x88, 0x36, 0x16, 0x77, 0x10, 0x66, 0x65, 0x61,
    //        0x74, 0x75, 0x72, 0x65, 0x73, 0x5f, 0x72, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74,
    //    ];
    //    let s = SendCtrl::from(&buf[..]);
    //    println!("{:?}", s);
    //}
}
