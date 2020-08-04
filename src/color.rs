pub trait Color {
    const NAME: &'static str;
    const CHANNELS: usize;
    const ALPHA: bool = false;
}

#[macro_export]
macro_rules! color {
    ($t:ident, $name:expr, $channels:expr $(, $alpha:expr)?) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $t;

        impl Color for $t {
            const NAME: &'static str = $name;
            const CHANNELS: usize = $channels;

            $(
            const ALPHA: bool = $alpha;
            )?
        }
    };
}

color!(Gray, "gray", 1);
color!(Rgb, "rgb", 3);
color!(Rgba, "rgba", 4, true);
