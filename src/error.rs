use io;
use std::io::Error as IOError;

#[derive(Debug)]
pub enum Error {
    Magick(io::magick::Error),
    IO(IOError),
    Message(String),
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::Message(s)
    }
}
