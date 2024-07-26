pub use de::{from_term, Deserializer};
pub use ser::{to_term, Serializer};

mod de;
mod error;
mod ser;
pub mod term;

#[allow(dead_code)]
pub const ATOM_CACHE_REF: u8 = 82;
pub const SMALL_INTEGER_EXT: u8 = 97;
pub const INTEGER_EXT: u8 = 98;
pub const FLOAT_EXT: u8 = 99;
pub const PORT_EXT: u8 = 102;
pub const NEW_PORT_EXT: u8 = 89;
pub const V4_PORT_EXT: u8 = 120;
pub const PID_EXT: u8 = 103;
pub const NEW_PID_EXT: u8 = 88;
pub const SMALL_TUPLE_EXT: u8 = 104;
pub const LARGE_TUPLE_EXT: u8 = 105;
pub const MAP_EXT: u8 = 116;
pub const NIL_EXT: u8 = 106;
pub const STRING_EXT: u8 = 107;
pub const LIST_EXT: u8 = 108;
pub const BINARY_EXT: u8 = 109;
pub const SMALL_BIG_EXT: u8 = 110;
pub const LARGE_BIG_EXT: u8 = 111;
// deprecated
#[allow(dead_code)]
pub const REFERENCE_EXT: u8 = 101;
#[allow(dead_code)]
pub const NEW_REFERENCE_EXT: u8 = 114;
pub const NEWER_REFERENCE_EXT: u8 = 90;
pub const NEW_FUN_EXT: u8 = 112;
pub const EXPORT_EXT: u8 = 113;
pub const BIT_BINARY_EXT: u8 = 77;
pub const NEW_FLOAT_EXT: u8 = 70;
pub const ATOM_UTF8_EXT: u8 = 118;
pub const SMALL_ATOM_UTF8_EXT: u8 = 119;
// deprecated
#[allow(dead_code)]
pub const ATOM_EXT: u8 = 100;
// deprecated
#[allow(dead_code)]
pub const SMALL_ATOM_EXT: u8 = 115;
pub const LOCAL_EXT: u8 = 121;

#[derive(Debug, Clone, PartialEq)]
pub enum Sign {
    Positive,
    Negative,
}

impl From<u8> for Sign {
    fn from(value: u8) -> Self {
        match value {
            0 => Sign::Positive,
            1 => Sign::Negative,
            _ => unreachable!(),
        }
    }
}

impl From<&Sign> for u8 {
    fn from(value: &Sign) -> Self {
        match value {
            Sign::Positive => 0,
            Sign::Negative => 1,
        }
    }
}

impl From<&Sign> for num_bigint::Sign {
    fn from(value: &Sign) -> Self {
        match value {
            Sign::Positive => Self::Plus,
            Sign::Negative => Self::Minus,
        }
    }
}

//pub trait TermFromSlice: Sized {
//    type Error;
//    fn decode<T: AsRef<[u8]>>(slice: T) -> Result<Self, Self::Error>;
//}
