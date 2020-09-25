//! image2 - a high-performance image processing library with wide support for a variety of file
//! formats and data types
//!
//! OpenImageIO is used for encoding and decoding images, it should be installed before
//! building `image2`.
//!
//! ```rust,no_run
//! use image2::*;
//!
//! fn main() -> Result<(), Error> {
//!     // Load an image from disk
//!     let image = Image::<f32, Rgb>::open("images/A.exr")?;
//!
//!     // Apply a `Filter`, in this case using the `Convert` filter to
//!     // convert from `Rgb` to `Gray`
//!     let mut dest = image.new_like_with_color::<Gray>();
//!     dest.apply(Convert::<Gray>::new(), &[&image]);
//!
//!     // This is the same as:
//!     let dest: Image<f32, Gray> = image.run(Convert::<Gray>::new(), None);
//!
//!     // Save an image to disk
//!     dest.save("test.jpg")?;
//!
//!     Ok(())
//! }
//!
//! ```

pub use half::f16;

mod color;
mod error;
mod histogram;
mod image;
mod pixel;
mod r#type;

#[cfg(feature = "halide")]
mod halide_wrapper;

#[cfg(feature = "halide")]
pub use halide_runtime as halide;

pub mod filter;
pub mod io;
pub mod kernel;
pub mod transform;

pub use color::{Cmyk, Color, Convert, Gray, Hsv, Rgb, Rgba, Xyz};
pub use error::Error;
pub use filter::Filter;
pub use histogram::Histogram;
pub use image::{Hash, Image, Meta, Region};
pub use kernel::Kernel;
pub use pixel::Pixel;
pub use r#type::Type;

#[cfg(test)]
mod tests;

#[cfg(feature = "ui")]
pub mod ui;

#[cfg(feature = "ui")]
pub use bevy;
