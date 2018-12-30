# image2

A Rust library focused on generic image processing for a wide range of datatypes. [stb_image](https://github.com/nothings/stb) is used as the default encoder/decoder and supportes the following formats:

- JPEG [RW]
- PNG [RW]
- TGA [RW]
- BMP [RW]
- PSD [R]
- GIF [R]
- HDR [RW]

Additional formats are provided by:

- [ImageMagick](https://imagemagick.org/script/formats.php)/[GraphicsMagick](http://www.graphicsmagick.org/formats.html)
- [FFmpeg](https://ffmpeg.org)
- [rawloader](https://crates.io/crates/rawloader)
- [rscam](https://github.com/loyd/rscam)

### Optional crate features

- `raw`
    * Enables support for [many RAW formats](https://github.com/pedrocr/rawloader#current-state)
- `v4l`
    * Enables support for webcam capture on Linux
- `ser`
    * Automatically derive serde traits for images and many other datatyes
- `parallel`
    * Uses rayon to iterate over pixels in parallel (enabled by default)

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
image2 = "0.5"
```
