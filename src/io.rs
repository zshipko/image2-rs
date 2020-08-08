use crate::*;

use cpp::{cpp, cpp_class};

cpp! {{
    #include <OpenImageIO/imageio.h>
    #include <OpenImageIO/imagebuf.h>
    #include <OpenImageIO/imagebufalgo.h>
    using namespace OIIO;
}}

/// `BaseType` is compatible with OpenImageIO's `TypeDesc::BASETYPE`
///
/// This enum is used to convert from `Type` into a representation that can be used with OIIO
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

cpp_class!(
    /// ImageSpec wraps `OIIO::ImageSpec`
    pub unsafe struct ImageSpec as "ImageSpec"
);
impl ImageSpec {
    fn empty() -> ImageSpec {
        unsafe {
            cpp!([] -> ImageSpec as "ImageSpec" {
                return ImageSpec();
            })
        }
    }

    /// Create new ImageSpec
    pub fn new(w: usize, h: usize, c: usize, t: BaseType) -> ImageSpec {
        unsafe {
            cpp!([w as "size_t", h as "size_t", c as "size_t", t as "TypeDesc::BASETYPE"] -> ImageSpec as "ImageSpec" {
                return ImageSpec(w, h, c, t);
            })
        }
    }

    /// Get width
    pub fn width(&self) -> usize {
        unsafe {
            cpp!([self as "const ImageSpec*"] -> usize as "size_t" {
                return (size_t)self->width;
            })
        }
    }

    /// Get height
    pub fn height(&self) -> usize {
        unsafe {
            cpp!([self as "const ImageSpec*"] -> usize as "size_t" {
                return (size_t)self->height;
            })
        }
    }

    /// Get number of channels
    pub fn nchannels(&self) -> usize {
        unsafe {
            cpp!([self as "const ImageSpec*"] -> usize as "size_t" {
                return (size_t)self->nchannels;
            })
        }
    }

    /// Get image format
    pub fn format(&self) -> BaseType {
        unsafe {
            cpp!([self as "const ImageSpec*"] -> BaseType as "TypeDesc::BASETYPE" {
                return (TypeDesc::BASETYPE)self->format.basetype;
            })
        }
    }
}

/// Output is used to write images to disk
pub struct Output {
    spec: ImageSpec,
    path: std::path::PathBuf,
    image_output: *mut u8,
    index: usize,
}

impl Drop for Output {
    fn drop(&mut self) {
        if self.image_output.is_null() {
            return;
        }

        let image_output = self.image_output;

        unsafe {
            cpp!([image_output as "ImageOutput*"] {
                image_output->close();
            })
        }

        self.image_output = std::ptr::null_mut();
    }
}

impl Output {
    /// Get reference to output ImageSpec
    pub fn spec(&self) -> &ImageSpec {
        &self.spec
    }

    /// Get mutable reference to output ImageSpec
    pub fn spec_mut(&mut self) -> &mut ImageSpec {
        &mut self.spec
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Create a new output file
    pub fn create(path: impl AsRef<std::path::Path>) -> Result<Output, Error> {
        let path = path.as_ref();
        let path_str = std::ffi::CString::new(path.to_string_lossy().as_bytes().to_vec()).unwrap();
        let filename = path_str.as_ptr();

        let image_output = unsafe {
            cpp!([filename as "const char *"] -> *mut u8 as "ImageOutput*" {
                std::unique_ptr<ImageOutput> out = ImageOutput::create (filename);
                if (! out)
                    return nullptr;
                return out.release();
            })
        };

        if image_output.is_null() {
            return Err(Error::UnableToWriteImage(
                path.to_string_lossy().to_string(),
            ));
        }

        Ok(Output {
            path: path.to_path_buf(),
            image_output,
            spec: ImageSpec::empty(),
            index: 0,
        })
    }

    /// Write an image to the file
    ///
    /// Note: `image` dimensions and type will take precendence over the ImageSpec
    pub fn write<T: Type, C: Color>(self, image: &Image<T, C>) -> Result<(), Error> {
        let base_type = T::BASE;
        let path: &std::path::Path = self.path.as_ref();
        let path_str = std::ffi::CString::new(path.to_string_lossy().as_bytes().to_vec()).unwrap();
        let filename = path_str.as_ptr();
        let pixels = image.data.as_ptr();
        let (width, height, channels) = image.shape();
        let out = self.image_output;
        let spec = &self.spec;
        unsafe {
            cpp!([out as "ImageOutput*", filename as "const char *", base_type as "TypeDesc::BASETYPE", spec as "const ImageSpec *", width as "size_t", height as "size_t", channels as "size_t", pixels as "const void*"] {
                ImageSpec outspec (*spec);
                outspec.width = width;
                outspec.height = height;
                outspec.nchannels = channels;
                outspec.format = TypeDesc(base_type);
                out->open (filename, outspec);
                out->write_image (base_type, pixels);
            })
        }
        Ok(())
    }

    /// Append an image to the file for formats with multi-image support
    ///
    /// Note: `image` dimensions and type will take precendence over the ImageSpec
    pub fn append<T: Type, C: Color>(&mut self, image: &Image<T, C>) -> Result<(), Error> {
        let base_type = T::BASE;
        let path: &std::path::Path = self.path.as_ref();
        let path_str = std::ffi::CString::new(path.to_string_lossy().as_bytes().to_vec()).unwrap();
        let filename = path_str.as_ptr();
        let pixels = image.data.as_ptr();
        let (width, height, channels) = image.shape();
        let out = self.image_output;
        let spec = &self.spec;
        let index = self.index;
        let ok = unsafe {
            cpp!([out as "ImageOutput*", index as "size_t", filename as "const char *", base_type as "TypeDesc::BASETYPE", spec as "ImageSpec *", width as "size_t", height as "size_t", channels as "size_t", pixels as "const void*"] -> bool as "bool" {
                if (!out->supports ("multiimage")){
                    return false;
                }

                ImageOutput::OpenMode mode = ImageOutput::Create;
                if (index == 0){
                    spec->width = width;
                    spec->height = height;
                    spec->nchannels = channels;
                    spec->format = TypeDesc(base_type);
                    out->open (filename, *spec);
                } else {
                    mode = ImageOutput::AppendSubimage;
                }

                out->write_image (base_type, pixels, mode);
                return true;
            })
        };
        if !ok {
            return Err(Error::MultipleImagesNotSupported(
                path.to_string_lossy().to_string(),
            ));
        }
        self.index += 1;
        Ok(())
    }
}

/// Input is used to load images from disk
pub struct Input {
    path: std::path::PathBuf,
    spec: ImageSpec,
    subimage: usize,
    miplevel: usize,
    image_input: *mut u8,
}

impl Drop for Input {
    fn drop(&mut self) {
        if self.image_input.is_null() {
            return;
        }

        let image_input = self.image_input;

        unsafe {
            cpp!([image_input as "std::unique_ptr<ImageInput>"] {
                image_input->close();
            })
        }

        self.image_input = std::ptr::null_mut();
    }
}

impl Input {
    /// Build input with subimage set to the provided value
    pub fn with_subimage(mut self, subimage: usize) -> Self {
        self.subimage = subimage;
        self
    }

    /// Build input with incremented subimage
    pub fn incr_subimage(mut self) -> Self {
        self.subimage += 1;
        self
    }

    /// Build  input with miplevel set to the provided value
    pub fn with_miplevel(mut self, miplevel: usize) -> Self {
        self.miplevel = miplevel;
        self
    }

    /// Get input image spec
    pub fn spec(&self) -> &ImageSpec {
        &self.spec
    }

    /// Get input path
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Open image for reading
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Input, Error> {
        let mut spec = ImageSpec::empty();
        let tmp = &mut spec;

        let path = path.as_ref();
        let path_str = std::ffi::CString::new(path.to_string_lossy().as_bytes().to_vec()).unwrap();
        let filename = path_str.as_ptr();

        let input = unsafe {
            cpp!([filename as "const char *", tmp as "ImageSpec*"] ->  *mut u8 as "std::unique_ptr<ImageInput>" {
                auto input = ImageInput::open(filename);
                if (!input) {
                    return nullptr;
                }

                *tmp = input->spec();

                return input;
            })
        };

        if input.is_null() {
            return Err(Error::UnableToOpenImage(path.to_string_lossy().to_string()));
        }

        Ok(Input {
            spec,
            image_input: input,
            subimage: 0,
            miplevel: 0,
            path: path.to_path_buf(),
        })
    }

    /// Read into existing Image
    pub fn read_into<T: Type, C: Color>(&self, image: &mut Image<T, C>) -> Result<(), Error> {
        let data = image.data.as_mut_ptr();

        let channels = C::CHANNELS;

        let input = self.image_input;
        let index = self.subimage;
        let miplevel = self.miplevel;
        let spec = &self.spec;
        let fmt = T::BASE;

        if spec.nchannels() < C::CHANNELS
            || spec.width() != image.width()
            || spec.height() != image.height()
        {
            return Err(Error::InvalidDimensions(
                spec.width(),
                spec.height(),
                spec.nchannels(),
            ));
        }

        let res = unsafe {
            cpp!([input as "std::unique_ptr<ImageInput>", index as "size_t", miplevel as "size_t", channels as "size_t", fmt as "TypeDesc::BASETYPE", data as "void *"] ->  bool as "bool" {
                return input->read_image(index, miplevel, 0, channels, fmt, data);
            })
        };

        if !res {
            return Err(Error::CannotReadImage(
                self.path.to_string_lossy().to_string(),
            ));
        }

        Ok(())
    }

    /// Read to new image
    pub fn read<T: Type, C: Color>(&self) -> Result<Image<T, C>, Error> {
        let mut image = Image::new(self.spec.width(), self.spec.height());
        self.read_into(&mut image)?;
        Ok(image)
    }
}

pub(crate) mod internal {
    use crate::*;
    use cpp::{cpp, cpp_class};
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

            let to_space_str =
                std::ffi::CString::new(to_space.as_ref().as_bytes().to_vec()).unwrap();
            let to_space = to_space_str.as_ptr();

            unsafe {
                cpp!([dest as "ImageBuf*", self as "const ImageBuf*", from_space as "const char *", to_space as "const char *"] -> bool as "bool" {
                    return ImageBufAlgo::colorconvert(*dest, *self, from_space, to_space);
                })
            }
        }
    }
}
