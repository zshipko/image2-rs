pub trait Color {
    fn channels() -> usize;
    fn has_alpha() -> bool;
}

macro_rules! make_color {
    ($name:ident, $channels:expr, $alpha:expr) => {
        pub struct $name;

        impl Color for $name {
            fn channels() -> usize { $channels }
            fn has_alpha() -> bool {  $alpha }
        }
    }
}

make_color!(Gray, 1, false);
make_color!(Rgb, 3, false);
make_color!(Rgba, 4, true);

