//! `image2` is an image processing crate with a focus on ease-of-use, support for a wide range
//! of datatypes and composable operations.
//!
//! Getting started:
//! ```rust
//! use image2::{
//!    ImageBuf,
//!    Rgb, Gray,
//!    Type,
//!    io,
//!    Filter,
//!    filter::ToGrayscale
//! };
//!
//! fn main() {
//!    // Read an image using the default JPEG decoder (stb_image)
//!    let image: ImageBuf<f64, Rgb> = io::read("test/test.jpg").unwrap();
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
//!    // Save the image using the default PNG encoder (stb_image)
//!    io::write("example.png", &output).unwrap();
//!}
//!```
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
