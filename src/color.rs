pub trait Color: Send {
    fn name() -> &'static str;
    fn channels() -> usize;
    fn has_alpha() -> bool;
}

macro_rules! make_color {
    ($name:ident, $name_s:expr, $channels:expr, $alpha:expr) => {
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

make_color!(Gray, "gray", 1, false);
make_color!(Rgb, "rgb", 3, false);
make_color!(Rgba, "rgba", 4, true);
make_color!(Cmyk, "cmyk", 4, false);
make_color!(Yuv, "yuv", 3, false);
