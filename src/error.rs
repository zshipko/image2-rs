#[derive(Debug)]
pub enum Error {
    OutOfBounds(usize, usize),
    UnableToOpenImage(String),
    UnableToWriteImage(String),
    CannotReadImage(String),
    InvalidDimensions(usize, usize, usize),
    FailedColorConversion(String, String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Error::*;
        match self {
            OutOfBounds(x, y) => write!(fmt, "out of bounds: {}, {}", x, y),
            UnableToOpenImage(filename) => write!(fmt, "unable to open image: {}", filename),
            UnableToWriteImage(filename) => write!(fmt, "unable to write image: {}", filename),
            CannotReadImage(filename) => write!(fmt, "cannot read image: {}", filename),
            InvalidDimensions(w, h, c) => {
                write!(fmt, "invalid image dimensions: {}x{}x{}", w, h, c)
            }
            FailedColorConversion(a, b) => write!(fmt, "failed color conversion: {} to {}", a, b),
        }
    }
}

impl std::error::Error for Error {}
