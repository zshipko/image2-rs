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
impl std::error::Error for Error {
    fn description(&self) -> &str {
        use Error::*;
        match *self {
            #[cfg(feature = "io")]
            Magick(ref e) => "Magick error",
            IO(ref e) => e.description(),
            Message(ref s) => s.as_str(),
            InvalidColor => "Invalid color",
            InvalidType => "Invalid type",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        use Error::*;
        match *self {
            #[cfg(feature = "io")]
            Magick(ref e) => None,
            IO(ref e) => Some(e),
            Message(ref e) => None,
            InvalidColor => None,
            InvalidType => None,
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <Self as std::error::Error>::description(self))
    }
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
