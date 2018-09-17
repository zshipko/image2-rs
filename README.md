# image2

A Rust image processing crate focused on supporting a wide range of datatypes.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
imaged = "0.3"
```

### Optional crate features

- `raw`
    * RAW image support via [rawloader](https://crates.io/crates/rawloader)
- `v4l`
    * Webcam capture on Linux via [rscam](https://github.com/loyd/rscam)
- `serce`
    * Serde support for many datatypes in `image2`
