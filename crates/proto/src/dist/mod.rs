use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone)]
    pub struct DistFlags: u64 {
        //The node is to be published and part of the global namespace.
        const PUBLISHED = 0x01;
        //The node implements an atom cache (obsolete).
        const ATOM_CACHE = 0x02;
        //The node implements extended (3 × 32 bits) references. This flag is mandatory. If not present, the connection is refused.
        const EXTENDED_REFERENCES = 0x04;
        //The node implements distributed process monitoring.
        const DIST_MONITOR = 0x08;
        //The node uses separate tags for funs (lambdas) in the distribution protocol. This flag is mandatory. If not present, the connection is refused.
        const FUN_TAGS = 0x10;
        //The node implements distributed named process monitoring.
        const DIST_MONITOR_NAME = 0x20;
        //The (hidden) node implements atom cache (obsolete).
        const HIDDEN_ATOM_CACHE = 0x40;
        //The node understands the NEW_FUN_EXT tag. This flag is mandatory. If not present, the connection is refused.
        const NEW_FUN_TAGS = 0x80;
        //The node can handle extended pids and ports. This flag is mandatory. If not present, the connection is refused.
        const EXTENDED_PIDS_PORTS = 0x100;
        //The node understands the EXPORT_EXT tag. This flag is mandatory. If not present, the connection is refused.
        const EXPORT_PTR_TAG = 0x200;
        //The node understands the BIT_BINARY_EXT tag. This flag is mandatory. If not present, the connection is refused.
        const BIT_BINARIES = 0x400;
        //The node understands the NEW_FLOAT_EXT tag. This flag is mandatory. If not present, the connection is refused.
        const NEW_FLOATS = 0x800;
        const UNICODE_IO = 0x1000;
        //The node implements atom cache in distribution header.
        const DIST_HDR_ATOM_CACHE = 0x2000;
        //The node understands the SMALL_ATOM_EXT tag.
        const SMALL_ATOM_TAGS = 0x4000;
        //The node understands UTF-8 atoms encoded with ATOM_UTF8_EXT and SMALL ATOM_UTF8_EXT. This flag is mandatory. If not present, the connection is refused.
        const UTF8_ATOMS = 0x10000;
        //The node understands the map tag MAP_EXT. This flag is mandatory. If not present, the connection is refused.
        const MAP_TAG = 0x20000;
        //The node understands big node creation tags NEW_PID_EXT, NEW_PORT_EXT and NEWER_REFERENCE_EXT. This flag is mandatory. If not present, the connection is refused.
        const BIG_CREATION = 0x40000;
        //Use the SEND_SENDER control message instead of the SEND control message and use the SEND_SENDER_TT control message instead of the SEND_TT control message.
        const SEND_SENDER = 0x80000;
        //The node understands any term as the seqtrace label.
        const BIG_SEQTRACE_LABELS = 0x100000;
        //Use the PAYLOAD_EXIT, PAYLOAD_EXIT_TT, PAYLOAD_EXIT2, PAYLOAD_EXIT2_TT and PAYLOAD_MONITOR_P_EXIT control messages instead of the non-PAYLOAD variants.
        const EXIT_PAYLOAD = 0x400000;
        //Use fragmented distribution messages to send large messages.
        const FRAGMENTS = 0x800000;
        //The node supports the new connection setup handshake (version 6) introduced in OTP 23. This flag is mandatory (from OTP 25). If not present, the connection is refused.
        const HANDSHAKE_23 = 0x1000000;
        //Use the new link protocol.
        const UNLINK_ID = 0x2000000;
        //This flag is mandatory as of OTP 26.
        //The node supports all capabilities that are mandatory in OTP 25. Introduced in OTP 25.
        const MANDATORY_25_DIGEST = 1 << 36;
        //This flag will become mandatory in OTP 27.
        //Set if the SPAWN_REQUEST, SPAWN_REQUEST_TT, SPAWN_REPLY, SPAWN_REPLY_TT control messages are supported.
        const SPAWN = 1 << 32;
        //Dynamic node name. This is not a capability but rather used as a request from the connecting node to receive its node name from the accepting node as part of the handshake.
        const NAME_ME = 1 << 33;
        //The node accepts a larger amount of data in pids, ports and references (node container types version 4). In the pid case full 32-bit ID and Serial fields in NEW_PID_EXT, in the port case a 64-bit integer in V4_PORT_EXT, and in the reference case up to 5 32-bit ID words are now accepted in NEWER_REFERENCE_EXT. This flag was introduced in OTP 24 and became mandatory in OTP 26.
        const V4_NC = 1 << 34;
        //The node supports process alias and can by this handle the ALIAS_SEND and ALIAS_SEND_TT control messages. Introduced in OTP 24.
        const ALIAS = 1 << 35;
    }
}

impl Default for DistFlags {
    fn default() -> Self {
        DistFlags::PUBLISHED
            | DistFlags::HIDDEN_ATOM_CACHE
            | DistFlags::EXTENDED_REFERENCES
            | DistFlags::EXTENDED_PIDS_PORTS
            | DistFlags::DIST_MONITOR
            | DistFlags::DIST_MONITOR_NAME
            | DistFlags::NEW_FLOATS
            | DistFlags::FUN_TAGS
            | DistFlags::NEW_FUN_TAGS
            | DistFlags::UNICODE_IO
            | DistFlags::HANDSHAKE_23
            | DistFlags::MAP_TAG
            | DistFlags::UTF8_ATOMS
            | DistFlags::SMALL_ATOM_TAGS
            | DistFlags::EXPORT_PTR_TAG
            | DistFlags::BIT_BINARIES
            | DistFlags::SPAWN
            | DistFlags::V4_NC
            | DistFlags::BIG_CREATION
            | DistFlags::UNLINK_ID
            | DistFlags::SEND_SENDER
    }
}

trait OpCode {
    const CODE: u8;
    /// The number of elements in the tuple representing control message.
    const ARITY: u8;
}

pub trait ProcessKind {
    fn get_from_pid_atom(&self) -> NewPid;
    fn get_to_pid_atom(&self) -> NewPid;
}

pub mod dist;
pub use dist::*;
pub mod epmd;
pub use epmd::*;

use crate::etf::term::NewPid;
pub mod handshake;
