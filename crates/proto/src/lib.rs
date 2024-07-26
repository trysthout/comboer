pub use dist::*;
pub use etf::*;

pub mod dist;
pub mod etf;
pub trait Encoder {
    type Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), Self::Error>;
}

pub trait Decoder: Sized {
    type Error;
    fn decode<T: AsRef<[u8]>>(value: T) -> Result<Self, Self::Error>;
}

#[allow(clippy::len_without_is_empty)]
pub trait Len {
    fn len(&self) -> usize;
}
