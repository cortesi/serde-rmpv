use std::fmt::{self, Display};

use serde::{de, ser};

pub type RResult<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Type mismatch error
    TypeError(String),
    /// Data format error
    Format(String),
    /// Unsupported type
    UnsupportedType,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Format(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Format(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::TypeError(msg) => write!(formatter, "invalid type: {}", msg),
            Error::Format(msg) => write!(formatter, "{}", msg),
            Error::UnsupportedType => write!(formatter, "unsupported type"),
        }
    }
}
