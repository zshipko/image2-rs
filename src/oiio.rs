use crate::*;

use cpp::{cpp, cpp_class};

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

cpp_class!(pub unsafe struct ImageBuf as "ImageBuf");
impl ImageBuf {
    pub fn empty() -> Self {
        unsafe { cpp!([] -> ImageBuf as "ImageBuf" { return ImageBuf(); }) }
    }

    pub fn new(width: usize, height: usize, channels: usize, base_type: BaseType) -> Self {
        unsafe {
            cpp!([width as "size_t", height as "size_t", channels as "size_t", base_type as "TypeDesc::BASETYPE"] -> ImageBuf as "ImageBuf" {
                return ImageBuf(ImageSpec(width, height, channels, base_type));
            })
        }
    }

    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Self {
        let path = path.as_ref();
        let c_str = std::ffi::CString::new(path.to_string_lossy().as_bytes().to_vec()).unwrap();
        let filename = c_str.as_ptr();
        unsafe {
            cpp!([filename as "const char *"] -> ImageBuf as "ImageBuf" {
                return ImageBuf(filename);
            })
        }
    }

    pub fn open_subimage<P: AsRef<std::path::Path>>(
        path: P,
        subimage: Option<usize>,
        miplevel: Option<usize>,
    ) -> Self {
        let subimage = subimage.unwrap_or(0);
        let miplevel = miplevel.unwrap_or(0);
        let path = path.as_ref();
        let c_str = std::ffi::CString::new(path.to_string_lossy().as_bytes().to_vec()).unwrap();
        let filename = c_str.as_ptr();
        unsafe {
            cpp!([filename as "const char *", subimage as "size_t", miplevel as "size_t"] -> ImageBuf as "ImageBuf" {
                return ImageBuf(filename, subimage, miplevel);
            })
        }
    }

    pub fn read<T: Type>(&mut self, index: usize, miplevel: usize, force: bool) -> bool {
        let base_type = T::BASE;
        unsafe {
            cpp!([self as "ImageBuf*", index as "size_t", miplevel as "size_t", force as "bool", base_type as "TypeDesc::BASETYPE"] -> bool as "bool" {
                return self->read(index, miplevel, force, base_type);
            })
        }
    }

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

    pub fn width(&self) -> usize {
        unsafe {
            cpp!([self as "const ImageBuf*"] -> usize as "size_t" {
                return (size_t)self->spec().width;
            })
        }
    }

    pub fn height(&self) -> usize {
        unsafe {
            cpp!([self as "const ImageBuf*"] -> usize as "size_t" {
                return (size_t)self->spec().height;
            })
        }
    }

    pub fn nchannels(&self) -> usize {
        unsafe {
            cpp!([self as "const ImageBuf*"] -> usize as "size_t" {
                return (size_t)self->spec().nchannels;
            })
        }
    }

    pub fn base_type(&self) -> BaseType {
        unsafe {
            cpp!([self as "const ImageBuf*"] -> BaseType as "TypeDesc::BASETYPE" {
                return (TypeDesc::BASETYPE)self->spec().format.basetype;
            })
        }
    }

    pub fn pixels(&mut self) -> *mut u8 {
        unsafe {
            cpp!([self as "ImageBuf*"] -> *mut u8 as "void *" {
                self->make_writable();
                return self->localpixels();
            })
        }
    }

    pub fn write(&mut self, path: impl AsRef<std::path::Path>) -> bool {
        let path = path.as_ref();
        let c_str = std::ffi::CString::new(path.to_string_lossy().as_bytes().to_vec()).unwrap();
        let filename = c_str.as_ptr();
        unsafe {
            cpp!([self as "const ImageBuf*", filename as "const char *"] -> bool as "bool" {
                return self->write(filename);
            })
        }
    }

    pub fn write_with_format(
        &mut self,
        path: impl AsRef<std::path::Path>,
        format: impl AsRef<str>,
    ) -> bool {
        let fmt = format.as_ref();
        let fmt_str = std::ffi::CString::new(fmt.as_bytes().to_vec()).unwrap();
        let format = fmt_str.as_ptr();
        let path = path.as_ref();
        let path_str = std::ffi::CString::new(path.to_string_lossy().as_bytes().to_vec()).unwrap();
        let filename = path_str.as_ptr();
        unsafe {
            cpp!([self as "const ImageBuf*", filename as "const char *",  format as "const char *"] -> bool as "bool" {
                return self->write(filename, TypeDesc::UNKNOWN, format);
            })
        }
    }

    pub fn write_with_type<T: Type>(&mut self, path: impl AsRef<std::path::Path>) -> bool {
        let base_type = T::BASE;
        let path = path.as_ref();
        let path_str = std::ffi::CString::new(path.to_string_lossy().as_bytes().to_vec()).unwrap();
        let filename = path_str.as_ptr();
        unsafe {
            cpp!([self as "const ImageBuf*", filename as "const char *", base_type as "TypeDesc::BASETYPE"] -> bool as "bool" {
                return self->write(filename, base_type);
            })
        }
    }

    pub fn write_with_format_and_type<T: Type>(
        &mut self,
        path: impl AsRef<std::path::Path>,
        format: impl AsRef<str>,
    ) -> bool {
        let base_type = T::BASE;
        let fmt = format.as_ref();
        let fmt_str = std::ffi::CString::new(fmt.as_bytes().to_vec()).unwrap();
        let format = fmt_str.as_ptr();
        let path = path.as_ref();
        let path_str = std::ffi::CString::new(path.to_string_lossy().as_bytes().to_vec()).unwrap();
        let filename = path_str.as_ptr();
        unsafe {
            cpp!([self as "const ImageBuf*", filename as "const char *", base_type as "TypeDesc::BASETYPE", format as "const char *"] -> bool as "bool" {
                return self->write(filename, base_type, format);
            })
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
    ) -> Option<Image<T, C>> {
        let image = ImageBuf::open_subimage(path.as_ref(), Some(index), Some(miplevel));
        let width = image.width();
        let height = image.height();
        let channels = C::CHANNELS;
        if image.nchannels() < channels || width == 0 || height == 0 {
            return None;
        }
        let path = path.as_ref();
        let path_str = std::ffi::CString::new(path.to_string_lossy().as_bytes().to_vec()).unwrap();
        let filename = path_str.as_ptr();

        let base_type = T::BASE;

        let mut image = Image::new(width, height);
        let data = image.data.as_mut_ptr();

        let res = unsafe {
            cpp!([filename as "const char *", index as "size_t", miplevel as "size_t", channels as "size_t", base_type as "TypeDesc::BASETYPE", data as "void *"] ->  bool as "bool" {
                auto input = ImageInput::open(filename);
                if (!input) {
                    return false;
                }

                bool res = input->read_image(index, miplevel, 0, channels, base_type, data);
                input->close();
                return res;
            })
        };

        if !res {
            return None;
        }

        Some(image)
    }
}
