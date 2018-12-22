pub mod magick;
pub mod stb;

#[cfg(feature = "v4l")]
pub mod v4l;

#[cfg(feature = "raw")]
pub mod raw;

use std::ffi::OsStr;
use std::path::Path;

use crate::color::{Color, Rgb};
use crate::error::Error;
use crate::image::Image;
use crate::image_ptr::ImagePtr;
use crate::ty::Type;

pub use self::stb::*;

macro_rules! cstring {
    ($s:expr) => {
        format!("{}\0", $s);
    };
}

pub fn read_u8<'a, P: AsRef<Path>, C: Color>(path: P) -> Result<ImagePtr<'a, u8, C>, Error> {
    let filename = match path.as_ref().to_str() {
        Some(f) => cstring!(f),
        None => {
            return Err(Error::Message(format!(
                "Invalid filename: {:?}",
                path.as_ref()
            )));
        }
    };

    let mut width = 0;
    let mut height = 0;
    let mut channels = 0;

    let ptr = unsafe {
        stbi_load(
            filename.as_str().as_ptr() as *mut i8,
            &mut width,
            &mut height,
            &mut channels,
            C::channels() as i32,
        )
    };

    Ok(ImagePtr::new(width as usize, height as usize, ptr, None))
}
