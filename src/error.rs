use crate::io;
use std::io::Error as IOError;

#[derive(Debug)]
pub enum Error {
    Magick(io::magick::Error),
    FFmpeg(io::ffmpeg::Error),
    IO(IOError),
    Message(String),
    InvalidColor,
    InvalidType,
}

impl From<io::magick::Error> for Error {
    fn from(err: io::magick::Error) -> Error {
        Error::Magick(err)
    }
}

impl From<io::ffmpeg::Error> for Error {
    fn from(err: io::ffmpeg::Error) -> Error {
        Error::FFmpeg(err)
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
