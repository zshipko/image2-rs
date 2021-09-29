#[cfg(not(feature = "oiio"))]
/// ImageMagick/GraphicsMagick based I/O
///
/// Note: This is enabled when OpenImageIO is disabled, to use OpenImageIO make sure the `oiio` feature is enabled at
/// compile time
///
/// See [oiio.rs](https://github.com/zshipko/image2-rs/blob/master/src/io/oiio.rs) for more
/// information about the OpenImageIO bindings
pub mod magick;

/// `BaseType` is compatible with OpenImageIO's `TypeDesc::BASETYPE`
///
/// This enum is used to convert from `Type` into a representation that can be used with OIIO
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum BaseType {
    /// Unknown image type
    Unknown,

    /// No image type
    None,

    /// u8
    UInt8,

    /// i8
    Int8,

    /// u16
    UInt16,

    /// i16
    Int16,

    /// u32
    UInt32,

    /// i32
    Int32,

    /// u64
    UInt64,

    /// i64
    Int64,

    /// f16
    Half,

    /// f32
    Float,

    /// f64
    Double,

    /// String
    String,

    /// Pointer
    Ptr,

    /// End of type enum
    Last,
}

#[cfg(all(feature = "oiio", not(feature = "docs-rs")))]
/// OpenImageIO bindings
pub mod oiio;

#[cfg(all(feature = "oiio", not(feature = "docs-rs")))]
pub use oiio::{read, write};

#[cfg(not(feature = "oiio"))]
pub use magick::{read, write};
