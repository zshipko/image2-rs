extern crate num;
extern crate rayon;
extern crate euclid;
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

extern crate png;
extern crate jpeg_decoder as jpeg;

#[cfg(test)]
mod tests;

#[macro_use]
mod image;
#[macro_use]
pub mod filter;
pub mod color;
mod error;
pub mod io;
pub mod kernel;
mod pixel;
pub mod transform;
mod ty;
mod image_buf;
mod image_ref;

pub use color::{Color, Gray, Rgb, Rgba};
pub use error::Error;
pub use filter::Filter;
pub use image::{Layout, Image};
pub use image_buf::ImageBuf;
pub use image_ref::ImageRef;
pub use kernel::Kernel;
pub use pixel::{Pixel, PixelMut, PixelVec};
pub use ty::Type;
