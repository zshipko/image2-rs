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

pub use half::f16;

mod color;
mod error;
mod geom;
mod histogram;
mod image;
mod meta;
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

pub use crate::meta::Meta;
pub use color::{Channel, Cmyk, Color, Convert, Gray, Hsv, Rgb, Rgba, Xyz};
pub use error::Error;
pub use filter::Filter;
pub use geom::{Point, Region, Size};
pub use histogram::Histogram;
pub use image::{Hash, Image};
pub use kernel::Kernel;
pub use pixel::Pixel;
pub use r#type::Type;

#[cfg(test)]
mod tests;

#[cfg(feature = "ui")]
pub mod ui;

#[cfg(feature = "ui")]
pub use bevy;
