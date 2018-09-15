use jpeg;

use std::fs::File;
use std::path::Path;

use color::{Color, Gray, Rgb};
use error::Error;
use filter::{Filter, ToColor, ToGrayscale};
use image::{Image, Layout};
use image_buf::ImageBuf;

pub fn read<P: AsRef<Path>, C: Color>(path: P) -> Result<ImageBuf<u8, C>, Error> {
    let mut decoder = jpeg::Decoder::new(File::open(path.as_ref())?);
    let pixels = decoder.decode()?;

    let info = match decoder.info() {
        Some(info) => info,
        None => return Err(Error::Message("Unable to read JPEG info".to_owned())),
    };

    match (C::name(), info.pixel_format) {
        ("gray", jpeg::PixelFormat::L8)
        | ("rgb", jpeg::PixelFormat::RGB24)
        | ("cmyk", jpeg::PixelFormat::CMYK32) => Ok(ImageBuf::new_from(
            info.width as usize,
            info.height as usize,
            Layout::Interleaved,
            pixels,
        )),
        ("gray", jpeg::PixelFormat::RGB24) => {
            let tmp: ImageBuf<u8, Rgb> = ImageBuf::new_from(
                info.width as usize,
                info.height as usize,
                Layout::Interleaved,
                pixels,
            );
            let mut dest: ImageBuf<u8, C> = ImageBuf::new(tmp.width(), tmp.height());
            ToGrayscale.eval(&mut dest, &[&tmp]);
            Ok(dest)
        }
        ("rgba", jpeg::PixelFormat::RGB24) => {
            let tmp: ImageBuf<u8, Rgb> = ImageBuf::new_from(
                info.width as usize,
                info.height as usize,
                Layout::Interleaved,
                pixels,
            );
            let mut dest: ImageBuf<u8, C> = ImageBuf::new(tmp.width(), tmp.height());
            ToColor.eval(&mut dest, &[&tmp]);
            Ok(dest)
        }
        ("rgb", jpeg::PixelFormat::L8) => {
            let tmp: ImageBuf<u8, Gray> = ImageBuf::new_from(
                info.width as usize,
                info.height as usize,
                Layout::Interleaved,
                pixels,
            );
            let mut dest: ImageBuf<u8, C> = ImageBuf::new(tmp.width(), tmp.height());
            ToColor.eval(&mut dest, &[&tmp]);
            Ok(dest)
        }
        ("rgba", jpeg::PixelFormat::L8) => {
            let tmp: ImageBuf<u8, Gray> = ImageBuf::new_from(
                info.width as usize,
                info.height as usize,
                Layout::Interleaved,
                pixels,
            );
            let mut dest: ImageBuf<u8, C> = ImageBuf::new(tmp.width(), tmp.height());
            ToColor.eval(&mut dest, &[&tmp]);
            Ok(dest)
        }
        (_, _) => Err(Error::InvalidColor),
    }
}
