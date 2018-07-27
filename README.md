# image2

Another image processing library for Rust focused on generic, composable image operations.

- Straight-forward API
- Parallel pixel iteration using `Filter::eval_p`
- Built-in support for u8, u16, i32, u32, f32, i64, u64, f64 datatypes
- Generic image reader/writer based on `ImageMagick`, allowing for a wide range of formats to be read into an `ImageBuf` of any type and color.
- (Optional) Support for RAW images via [rawloader](https://crates.io/crates/rawloader) (build with the `raw` feature enabled)
- (Optional) Support for webcam capture via [rscam](https://github.com/loyd/rscam) (build with the `v4l` feature enabled)

## Installation

Add the following to your `Cargo.toml`:

    image2 = { git = "https://github.com/zshipko/image2-rs" }

### Crate features

- `raw`
    * RAW image support
- `v4l`
    * Webcam capture on Linux
