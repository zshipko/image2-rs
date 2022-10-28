# image2 - image processing library

<a href="https://crates.io/crates/image2">
    <img src="https://img.shields.io/crates/v/image2.svg">
</a>

A Rust crate focused on generic image processing for a wide range of image formats and data types.

- Supported image data types: `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `f16`, `f32`, `f64`
- Supported color types: `gray`, `rgb`, `rgba`, `hsv`, `cmyk`, `xyz`
- Read and write images of any supported type/color
- Easy to add new colors
- Generic image processing across data types using `Pixel`
- Composable operations using `Filter` (with async support)

[OpenImageIO](https://github.com/OpenImageIO/oiio) is used to read/write images and supports:
  - `TIFF`
  - `JPEG`/`JFIF`
  - `OpenEXR`
  - `PNG`
  - `HDR`/`RGBE`
  - `ICO`
  - `BMP`
  - `Targa`
  - `JPEG-2000`
  - `RMan Zfile`
  - `FITS`
  - `DDS`
  - `Softimage PIC`
  - `PNM`
  - `DPX`
  - `Cineon`
  - `IFF`
  - `Field3D`
  - `Ptex`
  - `Photoshop PSD`
  - `Wavefront RLA`
  - `SGI`
  - `WebP`
  - `GIF`
  - A variety of RAW digital camera formats

`ImageMagick` can also be used in place of OpenImageIO.

This is not a pure Rust crate, if that's important to you then [image](https://github.com/image-rs/image) is probably a better fit.

## Features

- `oiio`
  * Enables I/O using OpenImageIO (default: enabled)
- `parallel`:
  * Enables parallel image iterators (default: enabled)
- `text`:
  * Enables loading fonts and drawing text on images (default: enabled)
- `window`:
  * Enables ability to draw images to a graphical window (default: disabled)
- `halide`:
  * [halide-runtime](https://github.com/zshipko/halide-runtime) interop (default: disabled)
- `serialize`:
  * Enables serde support for several data structures (default: disabled)
- `glfw-sys`:
  * Builds `glfw` with `glfw-sys` (default: disabled)

## External dependencies

- `libOpenImageIO` (optional)
  * `oiio` feature
  * Version >= 2.0
  * Debian-based distros: `apt install libopenimageio-dev`
  * macOS: `brew install openimageio`
- `ImageMagick` (optional)
  * Only required if `oiio` is not being used
  * Debian-based distros: `apt install imagemagick`
  * macOS: `brew install imagemagick`
- `libGLFW3` (optional)
  * `window` feature
  * Debian-based distros: `apt install libglfw3-dev`
  * macOS: `brew install glfw3`


