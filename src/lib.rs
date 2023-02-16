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
//!     let conv = filter::convert::<f32, Rgb, f32, Gray>();
//!     let mut dest = image.new_like_with_color::<Gray>();
//!     dest.apply(conv, &[&image]);
//!
//!     // This is equivalent to:
//!     let conv = filter::convert();
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
mod data;
mod error;
mod filters;
mod geom;
mod hash;
mod histogram;
mod image;
mod image_data;
mod meta;
mod pixel;
mod r#type;

/// OpenGL interop
#[cfg(feature = "opengl")]
pub mod texture;

/// Display images
#[cfg(feature = "window")]
pub mod window;

/// Text
#[cfg(feature = "text")]
pub mod text;

/// Image input/output
pub mod io;

/// Convolutions kernels
pub mod kernel;

/// Image transforms
pub mod transform;

pub use crate::meta::Meta;
pub use color::{Channel, Cmyk, Color, Gray, Hsv, Rgb, Rgba, Srgb, Srgba, Xyz, Yuv};
pub use data::{Data, DataMut};
pub use error::Error;
pub use filters::{
    filter, AsyncFilter, AsyncMode, AsyncPipeline, Filter, FilterExt, Input, Pipeline, Schedule,
};
pub use geom::{Point, Region, Size};
pub use hash::Hash;
pub use histogram::Histogram;
pub use image::Image;
pub use image_data::ImageData;
pub use kernel::Kernel;
pub use pixel::Pixel;
pub use r#type::Type;
pub use transform::Transform;

#[cfg(feature = "mmap")]
pub use image_data::mmap::Mmap;

#[cfg(test)]
mod tests;

#[cfg(feature = "parallel")]
pub use rayon::iter::ParallelIterator;
