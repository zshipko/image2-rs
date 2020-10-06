/// Enumerates possible errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Pixel is out of bounds
    #[error("Out of bounds: ({0}, {1})")]
    OutOfBounds(usize, usize),

    /// An image file cannot be opened
    #[error("Unable to open image: {0}")]
    UnableToOpenImage(String),

    /// An image file cannot be written
    #[error("Unable to write image: {0}")]
    UnableToWriteImage(String),

    /// Image data is invalid
    #[error("Cannot read image: {0}")]
    CannotReadImage(String),

    /// Invalid image dimensions
    #[error("Invalid image dimensions: width={0}, height={1}, channels={2}")]
    InvalidDimensions(usize, usize, usize),

    /// Colorspace conversion failed
    #[error("Failed color conversion from {0} to {1}")]
    FailedColorConversion(String, String),

    /// Unable to write an additional image to a single image file
    #[error("Multiple images not supported in image: {0}")]
    MultipleImagesNotSupported(String),

    /// Invalid image data type
    #[error("Invalid data type")]
    InvalidType,

    /// Magick I/O error type
    #[cfg(not(feature = "oiio"))]
    #[error("Magick: {0}")]
    Magick(#[from] crate::io::magick::Error),

    /// Window creation error
    #[cfg(feature = "window")]
    #[error("Glutin window creation: {0}")]
    GlutinWindowCreation(#[from] glutin::CreationError),

    /// Glutin context error
    #[cfg(feature = "window")]
    #[error("Glutin context: {0}")]
    GlutinContext(#[from] glutin::ContextError),
}
