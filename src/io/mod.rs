pub mod magick;
mod stb;

#[cfg(feature = "v4l")]
pub mod v4l;

use std::path::Path;

use crate::color::Color;
use crate::error::Error;
use crate::image::Image;
use crate::image_buf::ImageBuf;
use crate::image_ptr::{Free, ImagePtr};
use crate::ty::Type;

pub use self::stb::*;

macro_rules! cstring {
    ($s:expr) => {
        format!("{}\0", $s);
    };
}

macro_rules! path {
    ($p:expr) => {
        match $p.as_ref().to_str() {
            Some(f) => f,
            None => {
                return Err(Error::Message(format!(
                    "Invalid filename: {:?}",
                    $p.as_ref()
                )));
            }
        };
    };
}

/// Read an image with u8 components using stb_image
pub fn read_u8<'a, P: AsRef<Path>, C: Color>(path: P) -> Result<ImagePtr<'a, u8, C>, Error> {
    let f = path!(path);
    let filename = cstring!(f);

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

    Ok(ImagePtr::new(
        width as usize,
        height as usize,
        ptr,
        Free::Default,
    ))
}

/// Read an image with u16 components using stb_image
pub fn read_u16<'a, P: AsRef<Path>, C: Color>(path: P) -> Result<ImagePtr<'a, u16, C>, Error> {
    let f = path!(path);
    let filename = cstring!(f);

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

    Ok(ImagePtr::new(
        width as usize,
        height as usize,
        ptr,
        Free::Default,
    ))
}

/// Read an image with f32 components using stb_image
pub fn read_f32<'a, P: AsRef<Path>, C: Color>(path: P) -> Result<ImagePtr<'a, f32, C>, Error> {
    let f = path!(path);
    let filename = cstring!(f);

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

    Ok(ImagePtr::new(
        width as usize,
        height as usize,
        ptr,
        Free::Default,
    ))
}

/// Read any type of image using stb_image
pub fn read<'a, P: AsRef<Path>, T: Type, C: Color>(path: P) -> Result<ImageBuf<T, C>, Error> {
    let x = match read_u8(&path) {
        Ok(x) => x,
        Err(_) => magick::read(path)?.to_image_ptr(),
    };
    let mut y = ImageBuf::new(x.width(), x.height());
    x.convert_type(&mut y);
    Ok(y)
}

/// Decode an image with u8 components from memory
pub fn decode_u8<'a, Data: AsRef<[u8]>, C: Color>(
    data: Data,
) -> Result<ImagePtr<'a, u8, C>, Error> {
    let mut width = 0;
    let mut height = 0;
    let mut channels = 0;

    let ptr = unsafe {
        stbi_load_from_memory(
            data.as_ref().as_ptr(),
            data.as_ref().len() as i32,
            &mut width,
            &mut height,
            &mut channels,
            C::channels() as i32,
        )
    };

    Ok(ImagePtr::new(
        width as usize,
        height as usize,
        ptr,
        Free::Default,
    ))
}

/// Decode an image with u16 components from memory
pub fn decode_u16<'a, Data: AsRef<[u8]>, C: Color>(
    data: Data,
) -> Result<ImagePtr<'a, u16, C>, Error> {
    let mut width = 0;
    let mut height = 0;
    let mut channels = 0;

    let ptr = unsafe {
        stbi_load_16_from_memory(
            data.as_ref().as_ptr(),
            data.as_ref().len() as i32,
            &mut width,
            &mut height,
            &mut channels,
            C::channels() as i32,
        )
    };

    Ok(ImagePtr::new(
        width as usize,
        height as usize,
        ptr,
        Free::Default,
    ))
}

/// Decode an image with f32 components from memory
pub fn decode_f32<'a, Data: AsRef<[u8]>, C: Color>(
    data: Data,
) -> Result<ImagePtr<'a, f32, C>, Error> {
    let mut width = 0;
    let mut height = 0;
    let mut channels = 0;

    let ptr = unsafe {
        stbi_loadf_from_memory(
            data.as_ref().as_ptr(),
            data.as_ref().len() as i32,
            &mut width,
            &mut height,
            &mut channels,
            C::channels() as i32,
        )
    };

    Ok(ImagePtr::new(
        width as usize,
        height as usize,
        ptr,
        Free::Default,
    ))
}

/// Decode an image from memory
pub fn decode<'a, Data: AsRef<[u8]>, T: Type, C: Color>(
    data: Data,
) -> Result<ImageBuf<T, C>, Error> {
    let x = decode_u8(data)?;
    let mut y = ImageBuf::new(x.width(), x.height());
    x.convert_type(&mut y);
    Ok(y)
}

/// Write png image to disk
pub fn write_png_u8<C: Color, I: Image<u8, C>, P: AsRef<Path>>(
    path: P,
    im: &I,
) -> Result<(), Error> {
    let f = path!(path);
    let filename = cstring!(f);

    let (w, h, c) = im.shape();
    let result = unsafe {
        stbi_write_png(
            filename.as_str().as_ptr() as *mut i8,
            w as i32,
            h as i32,
            c as i32,
            im.data().as_ptr() as *const std::ffi::c_void,
            (c * w) as i32,
        )
    };

    if result == 0 {
        return Err(Error::Message(format!("Unable to open file: {}", f)));
    }

    Ok(())
}

/// Write bmp image to disk
pub fn write_bmp_u8<C: Color, I: Image<u8, C>, P: AsRef<Path>>(
    path: P,
    im: &I,
) -> Result<(), Error> {
    let f = path!(path);
    let filename = cstring!(f);

    let (w, h, c) = im.shape();
    let result = unsafe {
        stbi_write_bmp(
            filename.as_str().as_ptr() as *mut i8,
            w as i32,
            h as i32,
            c as i32,
            im.data().as_ptr() as *const std::ffi::c_void,
        )
    };

    if result == 0 {
        return Err(Error::Message(format!("Unable to open file: {}", f)));
    }

    Ok(())
}

/// Write tga image to disk
pub fn write_tga_u8<C: Color, I: Image<u8, C>, P: AsRef<Path>>(
    path: P,
    im: &I,
) -> Result<(), Error> {
    let f = path!(path);
    let filename = cstring!(f);

    let (w, h, c) = im.shape();
    let result = unsafe {
        stbi_write_tga(
            filename.as_str().as_ptr() as *mut i8,
            w as i32,
            h as i32,
            c as i32,
            im.data().as_ptr() as *const std::ffi::c_void,
        )
    };

    if result == 0 {
        return Err(Error::Message(format!("Unable to open file: {}", f)));
    }

    Ok(())
}

/// Write jpg image to disk
pub fn write_jpg_u8<C: Color, I: Image<u8, C>, P: AsRef<Path>>(
    path: P,
    im: &I,
    quality: i32,
) -> Result<(), Error> {
    let f = path!(path);
    let filename = cstring!(f);

    let (w, h, c) = im.shape();
    let result = unsafe {
        stbi_write_jpg(
            filename.as_str().as_ptr() as *mut i8,
            w as i32,
            h as i32,
            c as i32,
            im.data().as_ptr() as *const std::ffi::c_void,
            quality,
        )
    };

    if result == 0 {
        return Err(Error::Message(format!("Unable to open file: {}", f)));
    }

    Ok(())
}

/// Write hdr image to disk
pub fn write_hdr_f32<C: Color, I: Image<f32, C>, P: AsRef<Path>>(
    path: P,
    im: &I,
) -> Result<(), Error> {
    let f = path!(path);
    let filename = cstring!(f);

    let (w, h, c) = im.shape();
    let result = unsafe {
        stbi_write_hdr(
            filename.as_str().as_ptr() as *mut i8,
            w as i32,
            h as i32,
            c as i32,
            im.data().as_ptr(),
        )
    };

    if result == 0 {
        return Err(Error::Message(format!("Unable to open file: {}", f)));
    }

    Ok(())
}

/// Write image to disk, the output type is determined by the extension of the output image path
pub fn write<P: AsRef<Path>, T: Type, C: Color, I: Image<T, C>>(
    path: P,
    image: &I,
) -> Result<(), Error> {
    let path = path.as_ref();

    match path.extension() {
        Some(s) => match s.to_str() {
            Some("jpg") | Some("jpeg") | Some("JPG") | Some("JPEG") => {
                let mut tmp: ImageBuf<u8, C> = ImageBuf::new(image.width(), image.height());
                image.convert_type(&mut tmp);
                write_jpg_u8(path, &tmp, 95)
            }
            Some("hdr") | Some("HDR") => {
                let mut tmp: ImageBuf<f32, C> = ImageBuf::new(image.width(), image.height());
                image.convert_type(&mut tmp);
                write_hdr_f32(path, &tmp)
            }
            Some("tga") | Some("TGA") => {
                let mut tmp: ImageBuf<u8, C> = ImageBuf::new(image.width(), image.height());
                image.convert_type(&mut tmp);
                write_tga_u8(path, &tmp)
            }
            Some("bmp") | Some("BMP") => {
                let mut tmp: ImageBuf<u8, C> = ImageBuf::new(image.width(), image.height());
                image.convert_type(&mut tmp);
                write_bmp_u8(path, &tmp)
            }
            None | Some("png") | Some("PNG") => {
                let mut tmp: ImageBuf<u8, C> = ImageBuf::new(image.width(), image.height());
                image.convert_type(&mut tmp);
                write_png_u8(path, &tmp)
            }
            Some(_) => Ok(magick::write(path, image)?),
        },
        None => Err(Error::Message(format!(
            "Unable to determine output format: {:?}",
            path,
        ))),
    }
}

/// Encode u8 image to png in memory
pub fn encode_png_u8<C: Color, I: Image<u8, C>>(image: &I) -> Result<Vec<u8>, Error> {
    let (w, h, c) = image.shape();
    let mut outlen = 0;
    let ptr = unsafe {
        stbi_write_png_to_mem(
            image.data().as_ptr() as *mut u8,
            (w * c) as i32,
            w as i32,
            h as i32,
            c as i32,
            &mut outlen,
        )
    };

    let mut dest = vec![0; outlen as usize];

    unsafe {
        std::ptr::copy(ptr, dest.as_mut_ptr(), outlen as usize);
    }

    unsafe { crate::image_ptr::free(ptr as *mut std::ffi::c_void) }

    Ok(dest)
}

/// Encode image to png in memory
pub fn encode_png<T: Type, C: Color, I: Image<T, C>>(image: &I) -> Result<Vec<u8>, Error> {
    let mut tmp = ImageBuf::new(image.width(), image.height());
    image.convert_type(&mut tmp);
    encode_png_u8(&tmp)
}
