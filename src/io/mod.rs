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
use crate::image_buf::ImageBuf;
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

pub fn read_u16<'a, P: AsRef<Path>, C: Color>(path: P) -> Result<ImagePtr<'a, u16, C>, Error> {
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
        stbi_load_16(
            filename.as_str().as_ptr() as *mut i8,
            &mut width,
            &mut height,
            &mut channels,
            C::channels() as i32,
        )
    };

    Ok(ImagePtr::new(width as usize, height as usize, ptr, None))
}

pub fn readf<'a, P: AsRef<Path>, C: Color>(path: P) -> Result<ImagePtr<'a, f32, C>, Error> {
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
        stbi_loadf(
            filename.as_str().as_ptr() as *mut i8,
            &mut width,
            &mut height,
            &mut channels,
            C::channels() as i32,
        )
    };

    Ok(ImagePtr::new(width as usize, height as usize, ptr, None))
}

pub fn read<'a, P: AsRef<Path>, T: Type, C: Color>(path: P) -> Result<ImageBuf<T, C>, Error> {
    let x = read_u16(path)?;
    let mut y = ImageBuf::new(x.width(), x.height());
    x.convert_type(&mut y);
    Ok(y)
}
