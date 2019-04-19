#[cfg(feature = "io")]
use crate::io;
use std::io::Error as IOError;

#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "io")]
    Magick(io::magick::Error),
    #[cfg(feature = "io")]
    FFmpeg(io::ffmpeg::Error),
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

#[cfg(feature = "io")]
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
