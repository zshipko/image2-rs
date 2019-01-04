/// Stores colorspace information
pub trait Color: Sync + Send {
    /// The name of a colorspace, for example: "rgb"
    fn name() -> &'static str;

    /// The number of channels
    fn channels() -> usize;

    /// Determines if the last channel should be used as an alpha channel
    fn has_alpha() -> bool;
}

macro_rules! make_color {
    ($name:ident, $name_s:expr, $channels:expr, $alpha:expr) => {
        #[cfg_attr(
            feature = "ser",
            derive(serde_derive::Serialize, serde_derive::Deserialize)
        )]
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name;

        impl Color for $name {
            fn channels() -> usize {
                $channels
            }
            fn has_alpha() -> bool {
                $alpha
            }
            fn name() -> &'static str {
                $name_s
            }
        }
    };
}

/// Grayscale (1 channel, no alpha)
make_color!(Gray, "gray", 1, false);

/// RGB (3 channels, no alpha)
make_color!(Rgb, "rgb", 3, false);

/// BGR (3 channels, no alpha)
make_color!(Bgr, "bgr", 3, false);

/// RGB (packed, 1 channel, no alpha)
make_color!(RgbPacked, "rgb_packed", 1, false);

/// RGBA (4 channels)
make_color!(Rgba, "rgba", 4, true);

/// BGRA (4 channels)
make_color!(Bgra, "bgra", 4, true);

/// RGBA (packed, 1 channel, no alpha)
make_color!(RgbaPacked, "rgba_packed", 1, false);

/// CMYK (4 channels)
make_color!(Cmyk, "cmyk", 4, false);

/// YUV (3 channels)
make_color!(Yuv, "yuv", 3, false);
