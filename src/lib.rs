//! `image2` is an image processing crate with a focus on ease-of-use, support for a wide range
//! of datatypes and composable operations.
//!
//! As you may notice, `image2` optionally depends on ImageMagick/GraphicsMagick for loading
//! images. `io::magick` defines methods for loading and saving images of many data/color types.
//!
//! Getting started:
//! ```rust
//! use image2::{
//!    ImageBuf,
//!    Rgb, Gray,
//!    Type,
//!    io::magick,
//!    Filter,
//!    filter::ToGrayscale
//! };
//!
//! fn main() {
//!    // Read an image using ImageMagick, `io::magick` is provided by default
//!    let image: ImageBuf<f64, Rgb> = magick::read("test/test.jpg").unwrap();
//!
//!    // Setup a filter
//!    let filter = ToGrayscale.and_then(|f| {
//!        f64::max_f() - f
//!    });
//!
//!    // Create an output image
//!    let mut output: ImageBuf<f64, Gray> = ImageBuf::new_like_with_color::<Gray>(&image);
//!
//!    // Execute the filter
//!    filter.eval(&mut output, &[&image]);
//!
//!    // Save the image using ImageMagick
//!    magick::write("inverted_grayscale.jpg", &output).unwrap();
//!}
//!```
extern crate euclid;
extern crate num;
extern crate rayon;
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "ser")]
extern crate serde;

#[cfg(feature = "ser")]
#[macro_use]
extern crate serde_derive;

#[cfg(feature = "v4l")]
extern crate rscam;

#[cfg(feature = "raw")]
extern crate rawloader;

#[cfg(test)]
mod tests;

#[macro_use]
pub mod image;
#[macro_use]
pub mod filter;
pub mod color;
mod error;
mod image_buf;
mod image_ptr;
mod image_ref;
pub mod io;
pub mod kernel;
mod pixel;
pub mod transform;
mod ty;

pub use self::color::{Color, Gray, Rgb, Rgba};
pub use self::error::Error;
pub use self::filter::Filter;
pub use self::image::Image;
pub use self::image_buf::ImageBuf;
pub use self::image_ptr::ImagePtr;
pub use self::image_ref::ImageRef;
pub use self::kernel::Kernel;
pub use self::pixel::{Pixel, PixelMut, PixelVec};
pub use self::ty::Type;
