# image2

Another image processing library for Rust focused on generic, composable image operations.

## Installation

Add the following to your `Cargo.toml`:

    image2 = { git = "https://github.com/zshipko/image2-rs" }

### Crate features

- `raw`
    * RAW image support via [rawloader](https://crates.io/crates/rawloader)
- `v4l`
    * Webcam capture on Linux via [rscam](https://github.com/loyd/rscam)

## Examples

```rust
use image2::{
    ImageBuf,
    Rgb, Gray,
    Type,
    io::magick,
    Filter,
    filter::ToGrayscale
};

fn main() {
    // Read an image using ImageMagick
    let image: ImageBuf<f64, Rgb> = magick::read("../test/test.jpg").unwrap();

    // Setup a filter
    let filter = ToGrayscale.and_then(|f| {
        f64::max_f() - f
    });

    // Create an output image
    let mut output: ImageBuf<f64, Gray> = ImageBuf::new_like_with_color::<Gray>(&image);

    // Execute the filter in parallel
    filter.eval_p(&mut output, &[&image]);

    // Save the image using ImageMagick
    magick::write("interted_grayscale.jpg", &output).unwrap();
}
```
