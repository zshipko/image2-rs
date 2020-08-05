//! image2 - a high-performance image processing library with wide support for a variety of file
//! formats and data type
//!
//! OpenImageIO is used for encoding and decoding images. It should be installed before
//! building `image2`
//!
//! ```rust,no_run
//! use image2::*;
//!
//! fn main() -> Result<(), Error> {
//!     /// Load an image from disk
//!     let image = Image::<f32, Rgb>::open("images/A.exr")?;
//!
//!     /// Apply a `Filter`, in this case using the `Convert` filter to
//!     /// convert from `Rgb` to `Gray`
//!     let dest = image.new_like_with_color::<Gray>();
//!     let dest = image.apply(filter::Convert, dest);
//!
//!     /// Save an image to disk
//!     dest.save("test.jpg")?;
//!
//!     Ok(())
//! }
//!
//! ```

pub use half::f16;

mod color;
mod convert;
mod error;
mod image;
mod oiio;
mod pixel;
mod r#type;

pub mod filter;
pub mod kernel;
pub mod transform;

pub use color::{Color, Gray, Rgb, Rgba, Xyz};
pub use convert::{Convert, ConvertColor};
pub use error::Error;
pub use filter::Filter;
pub use image::{Image, Meta};
pub use kernel::Kernel;
pub use pixel::Pixel;
pub use r#type::Type;

#[cfg(test)]
mod tests;
