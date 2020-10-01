#![deny(missing_docs)]

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
//!     let conv = Convert::<Gray>::new();
//!     let mut dest = image.new_like_with_color::<Gray>();
//!     dest.apply(conv, &[&image]);
//!
//!     // This is equivalent to:
//!     let conv = Convert::<Gray>::new();
//!     let dest: Image<f32, Gray> = image.run(conv, None);
//!
//!     // Save an image to disk
//!     dest.save("test.jpg")?;
//!
//!     Ok(())
//! }
//!
//! ```

/// 16-bit float
pub use half::f16;

mod color;
mod error;
mod geom;
mod hash;
mod histogram;
mod image;
mod meta;
mod pixel;
mod r#type;

#[cfg(feature = "halide")]
mod halide_wrapper;

/// Halide bindings
#[cfg(feature = "halide")]
pub use halide_runtime as halide;

/// Composable image filters
pub mod filter;

/// Image input/output
pub mod io;

/// Convolutions kernels
pub mod kernel;

/// Image transforms
pub mod transform;

pub use crate::meta::Meta;
pub use color::{Channel, Cmyk, Color, Convert, Gray, Hsv, Rgb, Rgba, Xyz};
pub use error::Error;
pub use filter::{AndThen, Filter, Join};
pub use geom::{Point, Region, Size};
pub use hash::Hash;
pub use histogram::Histogram;
pub use image::Image;
pub use kernel::Kernel;
pub use pixel::Pixel;
pub use r#type::Type;

/// User interface
#[cfg(feature = "ui")]
pub mod ui;

#[cfg(test)]
mod tests;
