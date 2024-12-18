use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    TypeError(String),
    Message(String),
    UnsupportedType,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::TypeError(msg) => write!(formatter, "invalid type: {}", msg),
            Error::Message(msg) => write!(formatter, "{}", msg),
            Error::UnsupportedType => write!(formatter, "unsupported type"),
        }
    }
}
