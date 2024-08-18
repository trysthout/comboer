use std::time::SystemTime;

use bytes::{Buf, BufMut};
use md5::{Digest, Md5};

use crate::DistFlags;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum HandshakeVersion {
    V5,
    #[default]
    V6,
}

impl From<&HandshakeVersion> for u16 {
    fn from(value: &HandshakeVersion) -> Self {
        match value {
            HandshakeVersion::V5 => 5,
            HandshakeVersion::V6 => 6,
        }
    }
}

impl From<u16> for HandshakeVersion {
    fn from(value: u16) -> Self {
        match value {
            5 => HandshakeVersion::V5,
            6 => HandshakeVersion::V6,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Status {
    Ok,
    OkSimultaneous,
    Nok,
    NotAllowed,
    Alive,
    // nodename, creation
    #[allow(dead_code)]
    Named(String, u32),
    #[default]
    None,
}

impl From<&[u8]> for Status {
    fn from(value: &[u8]) -> Self {
        match value {
            b"ok" => Status::Ok,
            b"ok_simultaneous" => Status::OkSimultaneous,
            b"nok" => Status::Nok,
            b"not_allowed" => Status::NotAllowed,
            b"alive" => Status::Alive,
            x => todo!("{:?}", x),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct HandshakeCodec {
    pub version: HandshakeVersion,
    pub dflags: DistFlags,
    pub local_node_name: String,
    pub remote_node_name: String,
    pub creation: u32,
    pub is_tls: bool,
    pub status: Status,
    pub local_challenge: u32,
    pub remote_challenge: u32,
    pub cookie: String,
}

impl HandshakeCodec {
    pub fn new(node_name: String, cookie: String, is_tls: bool) -> HandshakeCodec {
        Self {
            version: HandshakeVersion::V6,
            dflags: DistFlags::default(),
            is_tls,
            local_node_name: node_name,
            remote_node_name: "".to_string(),
            cookie,
            creation: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
            local_challenge: fastrand::u32(..),
            remote_challenge: fastrand::u32(..),
            ..Default::default()
        }
    }

    pub fn encode_v5_name(&self, buf: &mut &mut [u8]) -> usize {
        let length = 1 + 2 + 4 + self.local_node_name.len();
        // https://www.erlang.org/doc/man/ei_connect
        // length if tls size is 4 byte otherwise is 2 byte
        let packet_length = self.encode_length(length, buf);
        buf.put_u8(b'n');
        buf.put_u16((&self.version).into());
        buf.put_u32(self.dflags.bits() as u32);
        buf.put_slice(self.local_node_name.as_bytes());
        packet_length
    }

    pub fn decode_v5_name(&mut self, mut buf: &[u8]) {
        // 'n'
        let _ = buf.get_u8();
        // version
        self.version = buf.get_u16().into();
        // flags
        self.dflags = DistFlags::from_bits(buf.get_u32() as u64).unwrap_or_default();
        if self.dflags.contains(DistFlags::HANDSHAKE_23) {
            self.version = HandshakeVersion::V6;
        }

        self.remote_node_name = String::from_utf8_lossy(buf.chunk()).to_string();
    }

    pub fn encode_v6_name(&self, buf: &mut &mut [u8]) -> usize {
        //let length = 1 + 8 + 4 + 2 + self.node_name.len();
        let length = 1 + 8 + 4 + 2 + self.local_node_name.len();
        let packet_length = self.encode_length(length, buf);
        buf.put_u8(b'N');
        buf.put_u64(self.dflags.bits());
        //buf.put_u64(0x0000000d07df7fbd);
        buf.put_u32(self.creation);
        buf.put_u16(self.local_node_name.len() as u16);
        //buf.extend_decode(self.node_name.as_bytes());
        buf.put_slice(self.local_node_name.as_bytes());
        packet_length
    }

    pub fn decode_v6_name(&mut self, mut buf: &[u8]) {
        // N
        buf.get_u8();

        // flags
        self.dflags = DistFlags::from_bits(buf.get_u64()).unwrap_or_default();
        // creation
        self.creation = buf.get_u32();
        // nodename length
        let n = buf.get_u16();
        self.remote_node_name = String::from_utf8_lossy(buf.chunk()).to_string();
        buf.advance(n as usize);
    }

    pub fn encode_status(&self, buf: &mut &mut [u8]) -> usize {
        let packet_length = self.encode_length(3, buf);
        //let packet_length = 5;
        //buf.put_u16(3);
        //buf[2..5].copy_decode(b"sok");
        buf.put_slice(b"sok");
        packet_length
    }

    pub fn decode_status(&mut self, mut buf: &[u8]) {
        // 's' tag
        buf.get_u8();
        self.status = buf.chunk().into();
    }

    pub fn encode_v5_challenge(&mut self, buf: &mut &mut [u8]) -> usize {
        let length = 1 + 2 + 4 + 4 + self.local_node_name.len();
        let packet_length = self.encode_length(length, buf);
        buf.put_u8(b'n');
        buf.put_u16(5);
        buf.put_u32(self.dflags.bits() as u32);
        buf.put_u32(self.local_challenge);
        buf.put_slice(self.local_node_name.as_bytes());
        packet_length
    }

    pub fn decode_v5_challenge(&mut self, mut buf: &[u8]) {
        // 'n'
        buf.get_u8();
        // version
        self.version = buf.get_u16().into();
        // flags
        self.dflags = DistFlags::from_bits(buf.get_u32() as u64).unwrap_or_default();
        if self.dflags.contains(DistFlags::HANDSHAKE_23) {
            self.version = HandshakeVersion::V6;
        }
        // challenge
        self.remote_challenge = buf.get_u32();
        self.remote_node_name = String::from_utf8_lossy(buf.chunk()).to_string();
    }

    pub fn encode_v6_challenge(&mut self, buf: &mut &mut [u8]) -> usize {
        //let node_name = "rust@fedora".to_string();
        let length = 1 + 8 + 4 + 4 + 2 + self.local_node_name.len();
        let packet_length = self.encode_length(length, buf);
        buf.put_u8(b'N');
        buf.put_u64(self.dflags.bits());
        buf.put_u32(self.local_challenge);
        buf.put_u32(self.creation);
        buf.put_u16(self.local_node_name.len() as u16);
        buf.put_slice(self.local_node_name.as_bytes());
        packet_length
    }

    pub fn decode_v6_challenge(&mut self, mut buf: &[u8]) {
        // 'N'
        buf.get_u8();
        self.dflags = DistFlags::from_bits(buf.get_u64()).unwrap_or_default();
        self.remote_challenge = buf.get_u32();
        self.creation = buf.get_u32();
        // node_name length
        let n = buf.get_u16();
        self.remote_node_name = String::from_utf8_lossy(&buf[..n as usize]).to_string();
    }

    pub fn encode_complement(&self, buf: &mut &mut [u8]) -> usize {
        let node_flags = self.dflags.bits() >> 32;
        let length = 1 + 4 + 4;
        let packet_length = self.encode_length(length, buf);
        buf.put_u8(b'c');
        buf.put_u32(node_flags as u32);
        buf.put_u32(self.creation);
        packet_length
    }

    pub fn decode_complement(&mut self, mut buf: &[u8]) {
        // 'c'
        buf.get_u8();
        self.dflags = DistFlags::from_bits((buf.get_u32() as u64) << 32).unwrap_or_default();
        self.creation = buf.get_u32();
    }

    pub fn encode_challenge_reply(&self, buf: &mut &mut [u8]) -> usize {
        let digest = gen_digest(self.remote_challenge, &self.cookie);
        let length = 1 + 4 + digest.len();
        let packet_length = self.encode_length(length, buf);
        buf.put_u8(b'r');
        buf.put_u32(self.local_challenge);
        buf.put_slice(&digest);
        packet_length
    }

    pub fn decode_challenge_reply(&mut self, mut buf: &[u8]) -> bool {
        // 'r'
        buf.get_u8();
        self.remote_challenge = buf.get_u32();
        let r_digest = &buf[0..16];
        let l_digest = gen_digest(self.local_challenge, &self.cookie);
        //self.challenge = challenge;

        r_digest.eq(&l_digest)
    }

    pub fn encode_challenge_ack(&self, buf: &mut &mut [u8]) -> usize {
        let length = 1 + 16;
        let packet_length = self.encode_length(length, buf);
        buf.put_u8(b'a');
        let digest = gen_digest(self.remote_challenge, &self.cookie);
        buf.put_slice(&digest);
        packet_length
    }

    pub fn decode_challenge_ack(&self, mut buf: &[u8]) -> bool {
        // a
        buf.get_u8();
        let digest = &buf[..16];
        buf.advance(16);
        let expected_digest = gen_digest(self.local_challenge, &self.cookie);

        digest.eq(&expected_digest)
    }

    fn encode_length(&self, length: usize, buf: &mut &mut [u8]) -> usize {
        if self.is_tls {
            buf.put_u32(length as u32);
            4 + length
        } else {
            buf.put_u16(length as u16);
            2 + length
        }
    }
}

pub fn gen_digest(challenge: u32, cookie: &String) -> Vec<u8> {
    let mut hasher = Md5::new();
    hasher.update(format!("{}{}", cookie, challenge));
    //hasher.update(challenge.to_be_bytes());
    hasher.finalize().to_vec()
}
