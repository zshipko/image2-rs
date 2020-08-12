#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Out of bounds: ({0}, {1})")]
    OutOfBounds(usize, usize),

    #[error("Unable to open image: {0}")]
    UnableToOpenImage(String),

    #[error("Unable to write image: {0}")]
    UnableToWriteImage(String),

    #[error("Cannot read image: {0}")]
    CannotReadImage(String),

    #[error("Invalid image dimensions: width={0}, height={1}, channels={2}")]
    InvalidDimensions(usize, usize, usize),

    #[error("Failed color conversion from {0} to {1}")]
    FailedColorConversion(String, String),

    #[error("Multiple images not supported in image: {0}")]
    MultipleImagesNotSupported(String),

    #[error("Magick: {0}")]
    Magick(#[from] crate::io::magick::Error),
}
