use png;
use png::HasParameters;

use std::path::Path;
use std::fs::File;

use color::{Color, Rgb, Rgba, Gray, RgbToRgba, RgbaToRgb};
use filter::{ToColor, ToGrayscale, Filter};
use image_buf::ImageBuf;
use image::Image;
use error::Error;

pub fn read<P: AsRef<Path>, C: Color>(path: P) -> Result<ImageBuf<u8, C>, Error>  {
    let decoder = png::Decoder::new(File::open(path.as_ref())?);
    let (info, mut reader) = decoder.read_info()?;

    if info.bit_depth != png::BitDepth::Eight {
        return Err(Error::InvalidType)
    }

    match (C::name(), info.color_type) {
        ("gray", png::ColorType::Grayscale) => {
            let mut image: ImageBuf<u8, C> = ImageBuf::new(info.width as usize, info.height as usize);
            reader.next_frame(image.data_mut())?;
            Ok(image)
        },
        ("gray", png::ColorType::RGB) => {
            let mut image: ImageBuf<u8, Rgb> = ImageBuf::new(info.width as usize, info.height as usize);
            reader.next_frame(image.data_mut())?;
            let mut dest: ImageBuf<u8, C> = ImageBuf::new(info.width as usize, info.height as usize);
            ToGrayscale.eval(&mut dest, &[&image]);
            Ok(dest)
        },
        ("gray", png::ColorType::RGBA) => {
            let mut image: ImageBuf<u8, Rgba> = ImageBuf::new(info.width as usize, info.height as usize);
            reader.next_frame(image.data_mut())?;
            let mut dest: ImageBuf<u8, C> = ImageBuf::new(info.width as usize, info.height as usize);
            ToGrayscale.eval(&mut dest, &[&image]);
            Ok(dest)
        },
        ("rgb", png::ColorType::Grayscale) => {
            let mut image: ImageBuf<u8, Gray> = ImageBuf::new(info.width as usize, info.height as usize);
            reader.next_frame(image.data_mut())?;
            let mut dest: ImageBuf<u8, C> = ImageBuf::new(info.width as usize, info.height as usize);
            ToColor.eval(&mut dest, &[&image]);
            Ok(dest)
        },
        ("rgb", png::ColorType::RGB) => {
            let mut image: ImageBuf<u8, C> = ImageBuf::new(info.width as usize, info.height as usize);
            reader.next_frame(image.data_mut())?;
            Ok(image)
        },
        ("rgb", png::ColorType::RGBA) => {
            let mut image: ImageBuf<u8, Rgba> = ImageBuf::new(info.width as usize, info.height as usize);
            reader.next_frame(image.data_mut())?;
            let mut dest: ImageBuf<u8, C> = ImageBuf::new(info.width as usize, info.height as usize);
            RgbaToRgb.eval(&mut dest, &[&image]);
            Ok(dest)
        },
        ("rgba", png::ColorType::Grayscale) => {
            let mut image: ImageBuf<u8, Gray> = ImageBuf::new(info.width as usize, info.height as usize);
            reader.next_frame(image.data_mut())?;
            let mut dest: ImageBuf<u8, C> = ImageBuf::new(info.width as usize, info.height as usize);
            ToColor.eval(&mut dest, &[&image]);
            Ok(dest)
        },
        ("rgba", png::ColorType::RGB) => {
            let mut image: ImageBuf<u8, Rgb> = ImageBuf::new(info.width as usize, info.height as usize);
            reader.next_frame(image.data_mut())?;
            let mut dest: ImageBuf<u8, C> = ImageBuf::new(info.width as usize, info.height as usize);
            RgbToRgba.eval(&mut dest, &[&image]);
            Ok(dest)
        },
        ("rgba", png::ColorType::RGBA) => {
            let mut image: ImageBuf<u8, C> = ImageBuf::new(info.width as usize, info.height as usize);
            reader.next_frame(image.data_mut())?;
            Ok(image)
        },
        (_, _) => {
            Err(Error::InvalidColor)
        }
    }
}

pub fn write<P: AsRef<Path>, C: Color, I: Image<u8, C>>(filename: P, image: &I) -> Result<(), Error> {
    let f = File::create(filename)?;
    let mut encoder = png::Encoder::new(f, image.width() as u32, image.height() as u32);
    {
        let x = match C::name() {
            "gray" => encoder.set(png::ColorType::Grayscale),
            "rgb" => encoder.set(png::ColorType::RGB),
            "rgba" => encoder.set(png::ColorType::RGBA),
            _ => return Err(Error::InvalidColor),
        };
        x.set(png::BitDepth::Eight);
    }
    let mut writer = encoder.write_header().unwrap();
    Ok(writer.write_image_data(image.data())?)
}
