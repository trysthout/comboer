pub mod dist;
pub use dist::*;

pub mod etf;

pub trait Encoder {
    type Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), Self::Error>;
}

pub trait Len {
    fn len(&self) -> usize;
}
