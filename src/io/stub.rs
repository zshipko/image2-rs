use std::path::Path;

use crate::*;

/// Read image from disk this implementation is a stub, to enable I/O use the `oiio` trait to use the
/// OpenImageIO backend, or `magick` to use the ImageMagick backend
pub fn read<P: AsRef<Path>, T: Type, C: Color>(_path: P) -> Result<Image<T, C>, crate::Error> {
    unimplemented!()
}

/// Write image to disk, this implementation is a stub, to enable I/O use the `oiio` trait to use the
/// OpenImageIO backend, or `magick` to use the ImageMagick backend
pub fn write<P: AsRef<Path>, T: Type, C: Color>(
    _path: P,
    _image: &Image<T, C>,
) -> Result<(), crate::Error> {
    unimplemented!()
}
