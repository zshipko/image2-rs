#[derive(Debug)]
pub enum Error {
    OutOfBounds,
    UnableToOpenImage,
    UnableToWriteImage,
    CannotReadImage,
    InvalidDimensions,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Error::*;
        let s = match self {
            OutOfBounds => "out of bounds",
            UnableToOpenImage => "unable to open image",
            UnableToWriteImage => "unable to write image",
            CannotReadImage => "cannot read image",
            InvalidDimensions => "invalid image dimensions",
        };
        write!(fmt, "{:?}", s)
    }
}

impl std::error::Error for Error {}
