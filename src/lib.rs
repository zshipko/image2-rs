#![feature(test)]
#![feature(stdsimd)]

extern crate num;
extern crate rayon;
extern crate euclid;

#[cfg(feature = "ser")]
extern crate serde;

#[cfg(feature = "ser")]
#[macro_use]
extern crate serde_derive;

extern crate test;

#[cfg(feature = "v4l")]
extern crate rscam;

#[cfg(feature = "raw")]
extern crate rawloader;

#[cfg(test)]
mod tests;

#[macro_use]
mod image;
#[macro_use]
pub mod filter;
pub mod color;
mod error;
pub mod io;
mod kernel;
mod pixel;
pub mod transform;
mod ty;
mod image_buf;

pub use color::{Color, Gray, Rgb, Rgba};
pub use error::Error;
pub use filter::Filter;
pub use image::Image;
pub use image_buf::ImageBuf;
pub use kernel::Kernel;
pub use pixel::Pixel;
pub use ty::Type;
