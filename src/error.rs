use std::io::Error as IOError;
use io;

#[derive(Debug)]
pub enum Error {
    Magick(io::magick::Error),
    IO(IOError),
    Message(String)
}
