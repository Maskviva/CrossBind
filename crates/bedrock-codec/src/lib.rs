#![allow(missing_docs)]

pub mod reader;
pub mod types;
pub mod wrapper;
pub mod writer;

pub use reader::Reader;
pub use wrapper::PacketWrapper;
pub use writer::Writer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Eof { needed: usize, remaining: usize },
    VarIntTooLong,
    LengthLimit { got: usize, limit: usize },
    BadUtf8,
    BadDiscriminant { what: &'static str, value: i64 },
    Invalid(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Eof { needed, remaining } => {
                write!(
                    f,
                    "unexpected end of packet: wanted {needed}, {remaining} left"
                )
            }
            Error::VarIntTooLong => f.write_str("varint longer than its target type"),
            Error::LengthLimit { got, limit } => {
                write!(f, "length {got} exceeds limit {limit}")
            }
            Error::BadUtf8 => f.write_str("string field is not valid UTF-8"),
            Error::BadDiscriminant { what, value } => {
                write!(f, "unknown {what} discriminant: {value}")
            }
            Error::Invalid(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Codec {
    type Value;

    fn read(r: &mut Reader<'_>) -> Result<Self::Value>;
    fn write(w: &mut Writer, value: &Self::Value);
}

pub mod prelude {
    pub use crate::types::*;
    pub use crate::{Codec, Error, PacketWrapper, Reader, Result, Writer};
}
