#[cfg(feature = "io")]
use crate::io;
use std::io::Error as IOError;

#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "io")]
    Magick(io::magick::Error),
    IO(IOError),
    Message(String),
    InvalidColor,
    InvalidType,
}

#[cfg(feature = "io")]
impl From<io::magick::Error> for Error {
    fn from(err: io::magick::Error) -> Error {
        Error::Magick(err)
    }
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::Message(s)
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Error {
        Error::IO(err)
    }
}
