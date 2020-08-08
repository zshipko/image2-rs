use crate::*;

use cpp::{cpp, cpp_class};

cpp! {{
    #include <OpenImageIO/imageio.h>
    #include <OpenImageIO/imagebuf.h>
    #include <OpenImageIO/imagebufalgo.h>
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

cpp_class!(pub unsafe struct ImageBuf as "ImageBuf");
impl ImageBuf {
    pub fn new_with_data<T: Type>(
        width: usize,
        height: usize,
        channels: usize,
        data: &mut [T],
    ) -> Self {
        let base_type = T::BASE;
        let data = data.as_mut_ptr();
        unsafe {
            cpp!([width as "size_t", height as "size_t", channels as "size_t", base_type as "TypeDesc::BASETYPE", data as "void *"] -> ImageBuf as "ImageBuf" {
                return ImageBuf(ImageSpec(width, height, channels, base_type), data);
            })
        }
    }

    pub fn const_new_with_data<T: Type>(
        width: usize,
        height: usize,
        channels: usize,
        data: &[T],
    ) -> Self {
        let base_type = T::BASE;
        let data = data.as_ptr();
        unsafe {
            cpp!([width as "size_t", height as "size_t", channels as "size_t", base_type as "TypeDesc::BASETYPE", data as "void *"] -> ImageBuf as "ImageBuf" {
                return ImageBuf(ImageSpec(width, height, channels, base_type), data);
            })
        }
    }

    pub fn convert_color(
        &self,
        dest: &mut ImageBuf,
        from_space: impl AsRef<str>,
        to_space: impl AsRef<str>,
    ) -> bool {
        let from_space_str =
            std::ffi::CString::new(from_space.as_ref().as_bytes().to_vec()).unwrap();
        let from_space = from_space_str.as_ptr();

        let to_space_str = std::ffi::CString::new(to_space.as_ref().as_bytes().to_vec()).unwrap();
        let to_space = to_space_str.as_ptr();

        unsafe {
            cpp!([dest as "ImageBuf*", self as "const ImageBuf*", from_space as "const char *", to_space as "const char *"] -> bool as "bool" {
                return ImageBufAlgo::colorconvert(*dest, *self, from_space, to_space);
            })
        }
    }
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
        return Err(Error::UnableToOpenImage(path.to_string_lossy().to_string()));
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
        return Err(Error::InvalidDimensions(width, height, channels));
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
        return Err(Error::CannotReadImage(path.to_string_lossy().to_string()));
    }

    Ok(image)
}
