use crate::*;

use cpp::cpp;

cpp! {{
    #include <OpenImageIO/imageio.h>
    #include <OpenImageIO/imagebuf.h>
    using namespace OIIO;
}}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum BaseType {
    Unknown,
    None,
    UInt8,
    Int8,
    UInt16,
    Int16,
    UInt32,
    Int32,
    UInt64,
    Int64,
    Half,
    Float,
    Double,
    String,
    Ptr,
    Last,
}

pub(crate) fn write_image<T: Type, C: Color>(
    image: &Image<T, C>,
    path: impl AsRef<std::path::Path>,
) -> bool {
    let base_type = T::BASE;
    let path = path.as_ref();
    let path_str = std::ffi::CString::new(path.to_string_lossy().as_bytes().to_vec()).unwrap();
    let filename = path_str.as_ptr();
    let pixels = image.data.as_ptr();
    let (width, height, channels) = image.shape();
    unsafe {
        cpp!([filename as "const char *", base_type as "TypeDesc::BASETYPE", width as "size_t", height as "size_t", channels as "size_t", pixels as "const void*"] -> bool as "bool" {
            std::unique_ptr<ImageOutput> out = ImageOutput::create (filename);
            if (! out)
                return false;
            ImageSpec spec (width, height, channels, base_type);
            out->open (filename, spec);
            out->write_image (base_type, pixels);
            out->close ();
            return true;
        })
    }
}

pub(crate) fn read_to_image<T: Type, C: Color>(
    path: impl AsRef<std::path::Path>,
    index: usize,
    miplevel: usize,
) -> Result<Image<T, C>, Error> {
    let mut width = 0;
    let mut height = 0;
    let mut channels = 0;

    let w = &mut width;
    let h = &mut height;
    let c = &mut channels;

    let path = path.as_ref();
    let path_str = std::ffi::CString::new(path.to_string_lossy().as_bytes().to_vec()).unwrap();
    let filename = path_str.as_ptr();

    let input = unsafe {
        cpp!([filename as "const char *", w as "size_t*", h as "size_t*", c as "size_t*"] ->  *mut u8 as "std::unique_ptr<ImageInput>" {
            auto input = ImageInput::open(filename);
            if (!input) {
                return nullptr;
            }

            auto spec = input->spec();

            *w = (size_t) spec.width;
            *h = (size_t) spec.height;
            *c = (size_t) spec.nchannels;

            return input;
        })
    };

    if input.is_null() {
        return Err(Error::UnableToOpenImage);
    }

    let base_type = T::BASE;

    let mut image = Image::new(width, height);
    let data = image.data.as_mut_ptr();

    if channels < C::CHANNELS || width == 0 || height == 0 {
        unsafe {
            cpp!([input as "std::unique_ptr<ImageInput>"] {
                input->close();
            })
        }
        return Err(Error::InvalidDimensions);
    }

    let channels = C::CHANNELS;

    let res = unsafe {
        cpp!([input as "std::unique_ptr<ImageInput>", index as "size_t", miplevel as "size_t", channels as "size_t", base_type as "TypeDesc::BASETYPE", data as "void *"] ->  bool as "bool" {
            bool res = input->read_image(index, miplevel, 0, channels, base_type, data);
            input->close();
            return res;
        })
    };

    if !res {
        return Err(Error::CannotReadImage);
    }

    Ok(image)
}
