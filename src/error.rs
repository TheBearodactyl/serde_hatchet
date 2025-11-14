use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Io(std::io::Error),
    Eof,
    InvalidUtf8,
    InvalidVarint,
    InvalidBool,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "{}", msg),
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Eof => write!(f, "Unexpected end of input"),
            Error::InvalidUtf8 => write!(f, "Invalid UTF-8"),
            Error::InvalidVarint => write!(f, "Invalid varint"),
            Error::InvalidBool => write!(f, "Invalid boolean value"),
        }
    }
}

impl std::error::Error for Error {}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
