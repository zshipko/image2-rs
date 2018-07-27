#![feature(test)]
#![feature(stdsimd)]

extern crate num;
extern crate rayon;
extern crate euclid;
extern crate test;

#[cfg(feature = "v4l")]
extern crate rscam;

#[cfg(feature = "raw")]
extern crate rawloader;

#[cfg(test)]
mod tests;

#[macro_use]
pub mod filter;
pub mod color;
mod error;
#[macro_use]
mod image;
pub mod io;
mod kernel;
mod pixel;
pub mod transform;
mod ty;
mod util;

pub use color::{Color, Gray, Rgb, Rgba};
pub use error::Error;
pub use filter::Filter;
pub use image::{Image, ImageBuf};
pub use kernel::Kernel;
pub use pixel::Pixel;
pub use ty::Type;
pub use util::Angle;
