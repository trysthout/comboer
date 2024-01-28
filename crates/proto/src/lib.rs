pub mod dist;
pub use dist::*;

pub mod etf;
pub use etf::*;

pub trait Encoder {
    type Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), Self::Error>;
}

#[allow(clippy::len_without_is_empty)]
pub trait Len {
    fn len(&self) -> usize;
}
