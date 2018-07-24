#![feature(test)]

extern crate num;
extern crate test;

#[macro_use]
pub mod filter;
mod util;
mod image;
mod ty;
mod color;
mod kernel;
pub mod io;
mod error;

pub use error::Error;
pub use util::Angle;
pub use image::{Image, ImageBuffer};
pub use ty::Type;
pub use color::{Color, Gray, Rgb, Rgba};
pub use filter::Filter;
pub use kernel::Kernel;
