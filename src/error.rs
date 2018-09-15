use io;
use std::io::Error as IOError;

use jpeg;
use png;

#[derive(Debug)]
pub enum Error {
    Magick(io::magick::Error),
    IO(IOError),
    Message(String),
    InvalidColor,
    InvalidType,
    PNGDecoding(png::DecodingError),
    PNGEncoding(png::EncodingError),
    JPEG(jpeg::Error),
}

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

impl From<png::DecodingError> for Error {
    fn from(err: png::DecodingError) -> Error {
        Error::PNGDecoding(err)
    }
}

impl From<png::EncodingError> for Error {
    fn from(err: png::EncodingError) -> Error {
        Error::PNGEncoding(err)
    }
}

impl From<jpeg::Error> for Error {
    fn from(err: jpeg::Error) -> Error {
        Error::JPEG(err)
    }
}
