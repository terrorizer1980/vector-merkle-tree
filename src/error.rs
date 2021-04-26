use std::error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidFormat,
    DuplicateTransferID,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidFormat => write!(f, "Invalid format for encoded core transfer app"),
            Error::DuplicateTransferID => write!(f, "Cannot insert duplicate transfer id"),
        }
    }
}
