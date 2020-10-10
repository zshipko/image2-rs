use super::BaseType;
use crate::*;

use cpp::{cpp, cpp_class};

cpp! {{
    #include <OpenImageIO/imageio.h>
    #include <OpenImageIO/imagebuf.h>
    #include <OpenImageIO/imagebufalgo.h>
    using namespace OIIO;
}}

/// ImageOutput is used to write images to disk
pub struct ImageOutput {
    spec: ImageSpec,
    path: std::path::PathBuf,
    image_output: *mut u8,
    index: usize,
}

impl Drop for ImageOutput {
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

impl ImageOutput {
    /// Get reference to output ImageSpec
    pub fn spec(&self) -> &ImageSpec {
        &self.spec
    }

    /// Get mutable reference to output ImageSpec
    pub fn spec_mut(&mut self) -> &mut ImageSpec {
        &mut self.spec
    }

    /// Get the output path
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Create a new output file
    pub fn create(path: impl AsRef<std::path::Path>) -> Result<ImageOutput, Error> {
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

        Ok(ImageOutput {
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
        if !["gray", "rgb", "rgba"].contains(&C::NAME) {
            let image = image.convert::<T, Rgb>();
            return self.write(&image);
        }

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
        if !["gray", "rgb", "rgba"].contains(&C::NAME) {
            let image = image.convert::<T, Rgb>();
            return self.append(&image);
        }

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

/// ImageInput is used to load images from disk
pub struct ImageInput {
    path: std::path::PathBuf,
    spec: ImageSpec,
    subimage: usize,
    miplevel: usize,
    image_input: *mut u8,
}

impl Drop for ImageInput {
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

impl ImageInput {
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
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<ImageInput, Error> {
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

        Ok(ImageInput {
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
    ///
    /// Note: the `convert` method may be called if the requested color doesn't match
    pub fn read<T: Type, C: Color>(&self) -> Result<Image<T, C>, Error> {
        let nchannels = self.spec.nchannels();

        // `convert` is called if the channels don't match the image on disk or the color is not
        // Gray, Rgb, or Rgba
        if C::CHANNELS != nchannels || !["gray", "rgb", "rgba"].contains(&C::NAME) {
            if nchannels == 1 {
                let mut image = Image::<f32, Gray>::new((self.spec.width(), self.spec.height()));
                self.read_into(&mut image)?;
                Ok(image.convert())
            } else if nchannels == 4 {
                let mut image = Image::<f32, Rgba>::new((self.spec.width(), self.spec.height()));
                self.read_into(&mut image)?;
                Ok(image.convert())
            } else {
                let mut image = Image::<f32, Rgb>::new((self.spec.width(), self.spec.height()));
                self.read_into(&mut image)?;
                Ok(image.convert())
            }
        } else {
            let mut image = Image::new((self.spec.width(), self.spec.height()));
            self.read_into(&mut image)?;
            Ok(image)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// `Attr` is used to include metadata when reading and writing image files
pub enum Attr<'a> {
    /// Integer value
    Int(i32),

    /// Float value
    Float(f32),

    /// String value
    String(&'a str),
}

impl<'a> From<i32> for Attr<'a> {
    fn from(i: i32) -> Attr<'a> {
        Attr::Int(i)
    }
}

impl<'a> From<f32> for Attr<'a> {
    fn from(i: f32) -> Attr<'a> {
        Attr::Float(i)
    }
}

impl<'a> From<&'a str> for Attr<'a> {
    fn from(i: &'a str) -> Attr<'a> {
        Attr::String(i)
    }
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
    pub fn nchannels(&self) -> Channel {
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

    /// Get an attribute
    pub fn get_attr(&self, key: impl AsRef<str>) -> Option<Attr> {
        let key_str = std::ffi::CString::new(key.as_ref().as_bytes().to_vec()).unwrap();
        let key_ptr = key_str.as_ptr();
        let value = unsafe {
            cpp!([self as "const ImageSpec*", key_ptr as "const char*"] -> *const internal::ParamValue as "const ParamValue*" {
                ParamValue param;
                return self->find_attribute(key_ptr, param, TypeDesc::UNKNOWN, false);
            })
        };

        if value.is_null() {
            return None;
        }

        unsafe { internal::to_attr(&*value) }
    }

    /// Set an attribute
    pub fn set_attr<'a>(&mut self, key: impl AsRef<str>, value: impl Into<Attr<'a>>) {
        let key_str = std::ffi::CString::new(key.as_ref().as_bytes().to_vec()).unwrap();
        let key_ptr = key_str.as_ptr();

        match value.into() {
            Attr::Int(value) => unsafe {
                cpp!([self as "ImageSpec*", key_ptr as "const char*", value as "int32_t"] {
                    self->attribute(key_ptr, (int)value);
                });
            },
            Attr::Float(value) => unsafe {
                cpp!([self as "ImageSpec*", key_ptr as "const char*", value as "float"] {
                    self->attribute(key_ptr, (float)value);
                });
            },
            Attr::String(value) => {
                let value_str = std::ffi::CString::new(value.as_bytes().to_vec()).unwrap();
                let value_ptr = value_str.as_ptr();

                unsafe {
                    cpp!([self as "ImageSpec*", key_ptr as "const char*", value_ptr as "const char *"] {
                        self->attribute(key_ptr, value_ptr);
                    });
                }
            }
        }
    }

    /// Get the oiio:Colorspace tag value
    pub fn colorspace(&self) -> Option<&str> {
        match self.get_attr("oiio:ColorSpace") {
            Some(Attr::String(s)) => Some(s),
            _ => None,
        }
    }

    /// Return the number of subimages, if any
    pub fn subimages(&self) -> Option<i32> {
        match self.get_attr("oiio:ColorSpace") {
            Some(Attr::Int(i)) => Some(i),
            _ => None,
        }
    }

    /// Get a map with all attributes
    pub fn attrs(&self) -> std::collections::BTreeMap<&str, Attr> {
        let mut len = 0;
        let len_ptr = &mut len;
        let ptr = unsafe {
            cpp!([self as "const ImageSpec*", len_ptr as "size_t*"] -> *const internal::ParamValue as "const ParamValue*" {
                *len_ptr = self->extra_attribs.size();
                return self->extra_attribs.data();
            })
        };

        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };

        slice.iter().map(|x| {
            let mut len = 0;
            let len_ptr = &mut len;
            unsafe {
                let s = cpp!([x as "const ParamValue*", len_ptr as "size_t*"] -> *const u8 as "const char*" {
                    *len_ptr = x->name().size();
                   return x->name().c_str();
                });

                let slice = std::slice::from_raw_parts(s, len);
                (std::str::from_utf8_unchecked(slice), internal::to_attr(x).unwrap())
            }
        }).collect()
    }
}

pub(crate) mod internal {
    use super::*;

    pub fn to_attr(param: &ParamValue) -> Option<Attr<'_>> {
        let t = param.ty();

        match t {
            BaseType::Int32 => Some(Attr::Int(param.get_int())),
            BaseType::Float => Some(Attr::Float(param.get_float())),
            BaseType::String => Some(Attr::String(param.get_string())),
            _ => None,
        }
    }

    cpp_class!(
        /// ImageSpec wraps `OIIO::ParamValue`
        pub unsafe struct ParamValue as "ParamValue"
    );
    impl ParamValue {
        fn ty(&self) -> BaseType {
            let param = self as *const _;
            unsafe {
                cpp!([param as "const ParamValue*"] -> BaseType as "TypeDesc::BASETYPE" {
                    return (TypeDesc::BASETYPE)param->type().basetype;
                })
            }
        }

        fn get_int(&self) -> i32 {
            let param = self as *const _;
            unsafe {
                cpp!([param as "const ParamValue*"] -> i32 as "int32_t" {
                    return param->get_int();
                })
            }
        }

        fn get_float(&self) -> f32 {
            let param = self as *const _;
            unsafe {
                cpp!([param as "const ParamValue*"] -> f32 as "float" {
                    return param->get_float();
                })
            }
        }

        fn get_string(&self) -> &str {
            let param = self as *const _;
            let mut len = 0;
            let len_ptr = &mut len;
            let x = unsafe {
                cpp!([param as "const ParamValue*", len_ptr as "size_t*"] -> *const u8 as "const char*" {
                    auto s = param->get_ustring();
                    *len_ptr = s.size();
                    return s.c_str();
                })
            };

            unsafe {
                let x = std::slice::from_raw_parts(x, len);
                std::str::from_utf8_unchecked(x)
            }
        }
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
