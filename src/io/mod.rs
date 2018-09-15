pub mod magick;

#[cfg(feature = "v4l")]
pub mod v4l;

#[cfg(feature = "raw")]
pub mod raw;

pub mod jpg;
pub mod png;

use std::ffi::OsStr;
use std::path::Path;

use color::Color;
use error::Error;
use image::Image;
use image_buf::ImageBuf;
use ty::Type;

pub fn read<P: AsRef<Path>, T: Type, C: Color>(filename: P) -> Result<ImageBuf<T, C>, Error> {
    match png::read(filename.as_ref()) {
        Ok(image) => {
            let mut dest = ImageBuf::new(image.width(), image.height());
            image.convert_type(&mut dest);
            Ok(dest)
        }
        Err(_) => match jpg::read(filename.as_ref()) {
            Ok(image) => {
                let mut dest = ImageBuf::new(image.width(), image.height());
                image.convert_type(&mut dest);
                Ok(dest)
            }
            Err(_) => Ok(magick::read(filename)?),
        },
    }
}

pub fn write<P: AsRef<Path>, T: Type, C: Color, I: Image<T, C>>(
    filename: P,
    image: &I,
) -> Result<(), Error> {
    let filename = filename.as_ref();
    match filename.extension() {
        Some(ext) => {
            if ext == OsStr::new("png") || ext == OsStr::new("PNG") {
                let mut dest = ImageBuf::new(image.width(), image.height());
                image.convert_type(&mut dest);
                png::write(filename, &dest)
            } else {
                Ok(magick::write(filename, image)?)
            }
        }
        None => Err(Error::Message("Invalid extension".to_owned())),
    }
}
