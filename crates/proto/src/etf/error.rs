use std::fmt::{self, Display};

use serde::{de, ser};

#[derive(Debug)]
pub enum Error {
    Msg(String),

    InvalidBoolean,
    ExpectedBoolean,
    ExpectedPrimitiveNumber,
    InvalidPrimitiveNumber,
    ExpectedString,
    ExpectedBytes,
    ExpectedType,
    ExpectedChar,
    ExpectedNil,
    ExpectedList,
    InvalidMapPair,
    ExpectedMap,
    ExpectedAtom,
    ExpectedAtomOrTuple,
    InvalidEnumTuple,
    ExpectedBigInt,
    ExpectedTuple,
    TryFromBigIntError(String),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Msg(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Msg(msg.to_string())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Msg(ref msg) => msg,
            Error::ExpectedBoolean => "Expected boolean, got something else",
            Error::InvalidBoolean => "Invalid boolean",
            Error::ExpectedChar => "Expected string of one character, got something else",
            Error::ExpectedNil => "Expected nil, got something else",
            Error::ExpectedList => "Expected list",
            Error::ExpectedMap => "Expected map, got something else",
            Error::ExpectedAtom => "Expected atom, got something else",
            Error::ExpectedAtomOrTuple => "Was expecting an atom or a tuple",
            Error::ExpectedPrimitiveNumber => "Expected a primitive number",
            Error::InvalidPrimitiveNumber => "11",
            Error::ExpectedString => "Expected a string",
            Error::ExpectedBytes => "33",
            Error::ExpectedType => "Expect a field name or there is a invalid field name",
            Error::InvalidMapPair => "55",
            Error::InvalidEnumTuple => "66",
            Error::ExpectedBigInt => "77",
            Error::ExpectedTuple => "88",
            Error::TryFromBigIntError(ref msg) => msg,
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&self.to_string())
    }
}
