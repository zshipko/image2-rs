# image2

A Rust image processing crate focused on supporting a wide range of datatypes.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
imaged = "0.2"
```

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
    // Read an image using ImageMagick, `io::magick` is provided by default
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
    magick::write("inverted_grayscale.jpg", &output).unwrap();
}
```
