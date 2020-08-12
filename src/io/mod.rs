pub mod magick;

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

#[cfg(feature = "oiio")]
mod oiio;

#[cfg(feature = "oiio")]
pub use oiio::*;
