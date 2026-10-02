//! Typed errors returned by the CDR codec.

use core::fmt;

/// CDR codec failure while reading or writing a message payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A bool field was not `0` or `1`.
    InvalidBool,
    /// The four-byte CDR encapsulation prefix was missing or wrong.
    InvalidEncapsulation,
    /// A length prefix exceeds the remaining payload bytes.
    InvalidLength,
    /// The payload ended before a field could be read.
    UnexpectedEnd,
    /// A string field was not valid UTF-8.
    Utf8,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBool => formatter.write_str("invalid bool encoding"),
            Self::InvalidEncapsulation => formatter.write_str("invalid CDR encapsulation header"),
            Self::InvalidLength => formatter.write_str("invalid CDR length prefix"),
            Self::UnexpectedEnd => formatter.write_str("unexpected end of CDR payload"),
            Self::Utf8 => formatter.write_str("invalid utf-8 string"),
        }
    }
}

impl core::error::Error for Error {}
