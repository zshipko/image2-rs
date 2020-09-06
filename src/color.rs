#![allow(clippy::float_cmp)]

use crate::*;

pub trait Color: Unpin + PartialEq + Eq + Clone + Sync + Send {
    const NAME: &'static str;
    const CHANNELS: usize;
    const ALPHA: bool = false;

    fn to_rgb(_c: usize, _pixel: &Pixel<Self>) -> f64 {
        panic!("to_rgb not implemented");
    }

    fn from_rgb(c: usize, pixel: &Pixel<Rgb>) -> f64;

    fn convert<ToColor: Color>(c: usize, pixel: &Pixel<Self>) -> f64 {
        let mut rgb: Pixel<Rgb> = Pixel::new();
        rgb[0] = Self::to_rgb(0, pixel);
        rgb[1] = Self::to_rgb(1, pixel);
        rgb[2] = Self::to_rgb(2, pixel);
        ToColor::from_rgb(c, &rgb)
    }
}

#[macro_export]
macro_rules! color {
    ($t:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $t;

        unsafe impl Sync for $t {}
        unsafe impl Send for $t {}
    };
}

color!(Gray);
impl Color for Gray {
    const NAME: &'static str = "gray";
    const CHANNELS: usize = 1;

    fn to_rgb(_c: usize, pixel: &Pixel<Self>) -> f64 {
        pixel[0]
    }

    fn from_rgb(_c: usize, pixel: &Pixel<Rgb>) -> f64 {
        pixel[0] * 0.21 + pixel[1] * 0.72 + pixel[2] * 0.7
    }
}

color!(Rgb);
impl Color for Rgb {
    const NAME: &'static str = "rgb";
    const CHANNELS: usize = 3;

    fn to_rgb(c: usize, pixel: &Pixel<Self>) -> f64 {
        pixel[c]
    }

    fn from_rgb(c: usize, pixel: &Pixel<Rgb>) -> f64 {
        pixel[c]
    }
}

color!(Rgba);
impl Color for Rgba {
    const NAME: &'static str = "rgba";
    const CHANNELS: usize = 4;

    fn to_rgb(c: usize, pixel: &Pixel<Self>) -> f64 {
        pixel[c] * pixel[3]
    }

    fn from_rgb(c: usize, pixel: &Pixel<Rgb>) -> f64 {
        if c == 3 {
            return 1.0;
        }

        pixel[c]
    }
}

color!(Xyz);
impl Color for Xyz {
    const NAME: &'static str = "xyz";
    const CHANNELS: usize = 3;

    fn from_rgb(c: usize, rgb: &Pixel<Rgb>) -> f64 {
        match c {
            0 => rgb[0] * 0.576700 + rgb[1] * 0.185556 + rgb[2] * 0.188212,
            1 => rgb[0] * 0.297361 + rgb[1] * 0.627355 + rgb[2] * 0.0752847,
            2 => rgb[0] * 0.0270328 + rgb[1] * 0.0706879 + rgb[2] * 0.991248,
            _ => 0.0,
        }
    }
}

color!(Hsv);
impl Color for Hsv {
    const NAME: &'static str = "hsv";
    const CHANNELS: usize = 3;

    fn from_rgb(c: usize, rgb: &Pixel<Rgb>) -> f64 {
        let r = rgb[0];
        let g = rgb[1];
        let b = rgb[2];
        let cmax = r.max(g).max(b);
        let cmin = r.min(g).min(b);
        let delta = cmax - cmin;
        match c {
            0 => {
                if cmin == cmax {
                    0.0
                } else if cmax == r {
                    (60. * ((g - b) / delta) + 360.0) % 360.
                } else if cmax == g {
                    (60. * ((b - r) / delta) + 120.0) % 360.
                } else if cmax == b {
                    (60. * ((r - g) / delta) + 240.0) % 360.
                } else {
                    -1.0
                }
            }
            1 => {
                if cmax == 0.0 {
                    0.0
                } else {
                    (delta / cmax) * 100.0
                }
            }
            2 => cmax * 100.,
            _ => -1.0,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Convert<T: Color>(std::marker::PhantomData<T>);

impl<C: Color> Convert<C> {
    pub fn new() -> Convert<C> {
        Convert(std::marker::PhantomData)
    }
}

impl<T: Color> Filter for Convert<T> {
    fn compute_at(
        &self,
        x: usize,
        y: usize,
        c: usize,
        input: &[&Image<impl Type, impl Color>],
    ) -> f64 {
        Color::convert::<T>(c, &input[0].get_pixel(x, y))
    }
}
