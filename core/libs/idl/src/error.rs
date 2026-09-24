use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnexpectedEnd,
    InvalidEncapsulation,
    InvalidBool,
    Utf8,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd => formatter.write_str("unexpected end of CDR payload"),
            Self::InvalidEncapsulation => formatter.write_str("invalid CDR encapsulation header"),
            Self::InvalidBool => formatter.write_str("invalid bool encoding"),
            Self::Utf8 => formatter.write_str("invalid utf-8 string"),
        }
    }
}

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
impl std::error::Error for Error {}
