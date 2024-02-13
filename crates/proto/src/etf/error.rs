use std::fmt;

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
    TryFromBigInt(String),
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Msg(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Msg(msg.to_string())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self {
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
            Error::InvalidPrimitiveNumber => "Invalid primitive number",
            Error::ExpectedString => "Expected a string",
            Error::ExpectedBytes => "Expected a bytes",
            Error::ExpectedType => "Expect a field name or there is a invalid field name",
            Error::InvalidMapPair => "Invalid map",
            Error::InvalidEnumTuple => "Invalid enum tuple",
            Error::ExpectedBigInt => "Expected a bigint",
            Error::ExpectedTuple => "Expected a tuple",
            Error::TryFromBigInt(ref msg) => msg,
        };

        formatter.write_str(err_msg)
    }
}
