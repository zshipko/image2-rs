#![allow(clippy::float_cmp)]

use crate::*;

/// `Channel` is an alias for `usize` used to identify channel values in function arguments
pub type Channel = usize;

/// `Color` trait is used to define color spaces
pub trait Color: Unpin + PartialEq + Eq + PartialOrd + Ord + Clone + Sync + Send {
    /// Color name
    const NAME: &'static str;

    /// Number of channels
    const CHANNELS: Channel;

    /// Index of alpha channel
    const ALPHA: Option<Channel> = None;

    /// Convert from Self -> Rgb
    fn to_rgb(src: &Pixel<Self>, dest: &mut Pixel<Rgb>);

    /// Convert from Rgb -> Self
    fn from_rgb(pixel: &Pixel<Rgb>, dest: &mut Pixel<Self>);

    /// Convert a single channel of a color to another color
    fn convert<ToColor: Color>(src: &Pixel<Self>, dest: &mut Pixel<ToColor>) {
        src.convert_to(dest);
    }
}

macro_rules! color {
    ($t:ident, $doc:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        #[doc = $doc]
        pub struct $t;

        unsafe impl Sync for $t {}
        unsafe impl Send for $t {}
    };
}

color!(Gray, "Single-channel grayscale");
impl Color for Gray {
    const NAME: &'static str = "gray";
    const CHANNELS: Channel = 1;

    fn to_rgb(src: &Pixel<Self>, pixel: &mut Pixel<Rgb>) {
        pixel.fill(src[0]);
    }

    fn from_rgb(src: &Pixel<Rgb>, mut dest: &mut Pixel<Self>) {
        dest[0] = src[0] * 0.21 + src[1] * 0.72 + src[2] * 0.7;
    }
}

color!(Rgb, "Three-channel red, green, blue");
impl Color for Rgb {
    const NAME: &'static str = "rgb";
    const CHANNELS: Channel = 3;

    fn to_rgb(rgb: &Pixel<Self>, pixel: &mut Pixel<Rgb>) {
        pixel.copy_from(rgb);
    }

    fn from_rgb(rgb: &Pixel<Rgb>, pixel: &mut Pixel<Self>) {
        pixel.copy_from(rgb);
    }
}

color!(Rgba, "Four-channel red, green, blue with alpha channel");
impl Color for Rgba {
    const NAME: &'static str = "rgba";
    const CHANNELS: Channel = 4;
    const ALPHA: Option<Channel> = Some(3);

    fn to_rgb(pixel: &Pixel<Self>, mut rgb: &mut Pixel<Rgb>) {
        rgb[0] = pixel[0] * pixel[3];
        rgb[1] = pixel[1] * pixel[3];
        rgb[2] = pixel[2] * pixel[3];
    }

    fn from_rgb(rgb: &Pixel<Rgb>, mut pixel: &mut Pixel<Self>) {
        pixel[0] = rgb[0];
        pixel[1] = rgb[1];
        pixel[2] = rgb[2];
        pixel[3] = 1.0;
    }
}

color!(Xyz, "Three-channel CIE-XYZ");
impl Color for Xyz {
    const NAME: &'static str = "xyz";
    const CHANNELS: Channel = 3;

    fn from_rgb(rgb: &Pixel<Rgb>, mut pixel: &mut Pixel<Self>) {
        let mut r = rgb[0];
        let mut g = rgb[1];
        let mut b = rgb[2];

        if r > 0.04045 {
            r = ((r + 0.055) / 1.055).powf(2.4)
        } else {
            r = r / 12.92
        }

        if g > 0.04045 {
            g = ((g + 0.055) / 1.055).powf(2.4);
        } else {
            g = g / 12.92
        }

        if b > 0.04045 {
            b = ((b + 0.055) / 1.055).powf(2.4)
        } else {
            b = b / 12.92
        }

        r *= 100.;
        g *= 100.;
        b *= 100.;

        pixel[0] = r * 0.4124 + g * 0.3576 + b * 0.1805;
        pixel[1] = r * 0.2126 + g * 0.7152 + b * 0.0722;
        pixel[2] = r * 0.0193 + g * 0.1192 + b * 0.9505;
    }

    fn to_rgb(px: &Pixel<Xyz>, mut rgb: &mut Pixel<Rgb>) {
        let x = px[0] / 100.;
        let y = px[1] / 100.;
        let z = px[2] / 100.;

        rgb[0] = {
            let var_r = x * 3.2406 + y * -1.5372 + z * -0.4986;
            if var_r > 0.0031308 {
                1.055 * (var_r.powf(1.0 / 2.4)) - 0.055
            } else {
                12.92 * var_r
            }
        };
        rgb[1] = {
            let var_g = x * -0.9689 + y * 1.8758 + z * 0.0415;
            if var_g > 0.0031308 {
                1.055 * (var_g.powf(1. / 2.4)) - 0.055
            } else {
                12.92 * var_g
            }
        };
        rgb[2] = {
            let var_b = x * 0.0557 + y * -0.2040 + z * 1.0570;
            if var_b > 0.0031308 {
                1.055 * (var_b.powf(1. / 2.4)) - 0.055
            } else {
                12.92 * var_b
            }
        };
    }
}

color!(Hsv, "Three-channel hue, saturation and value color");
impl Color for Hsv {
    const NAME: &'static str = "hsv";
    const CHANNELS: Channel = 3;

    fn from_rgb(rgb: &Pixel<Rgb>, mut pixel: &mut Pixel<Self>) {
        let r = rgb[0];
        let g = rgb[1];
        let b = rgb[2];
        let cmax = r.max(g).max(b);
        let cmin = r.min(g).min(b);
        let delta = cmax - cmin;
        let del_r = (((cmax - r) / 6.) + (delta / 2.)) / delta;
        let del_g = (((cmax - g) / 6.) + (delta / 2.)) / delta;
        let del_b = (((cmax - b) / 6.) + (delta / 2.)) / delta;
        pixel[0] = {
            let x = if cmin == cmax {
                0.0
            } else if cmax == r {
                del_b - del_g
            } else if cmax == g {
                (1. / 3.) + del_r - del_b
            } else if cmax == b {
                (2. / 3.) + del_g - del_r
            } else {
                -1.0
            };

            if x < 0. {
                x + 1.
            } else if x > 1. {
                x - 1.
            } else {
                x
            }
        };
        pixel[1] = {
            if cmax == 0.0 {
                0.0
            } else {
                delta / cmax
            }
        };
        pixel[2] = cmax;
    }

    fn to_rgb(px: &Pixel<Hsv>, mut rgb: &mut Pixel<Rgb>) {
        if px[1] == 0. {
            rgb.fill(px[2]);
            return;
        }

        let (h, s, v) = (px[0], px[1], px[2]);
        let mut var_h = h * 6.;
        if var_h == 6. {
            var_h = 0.0;
        }
        let var_i = var_h.floor();
        let var_1 = v * (1. - s);
        let var_2 = v * (1. - s * (var_h - var_i));
        let var_3 = v * (1. - s * (1. - (var_h - var_i)));

        if var_i == 0. {
            rgb[0] = v;
            rgb[1] = var_3;
            rgb[2] = var_1;
        } else if var_i == 1. {
            rgb[0] = var_2;
            rgb[1] = v;
            rgb[2] = var_1;
        } else if var_i == 2. {
            rgb[0] = var_1;
            rgb[1] = v;
            rgb[2] = var_3;
        } else if var_i == 3. {
            rgb[0] = var_1;
            rgb[1] = var_2;
            rgb[2] = v;
        } else if var_i == 4. {
            rgb[0] = var_3;
            rgb[1] = var_1;
            rgb[2] = v;
        } else {
            rgb[0] = v;
            rgb[1] = var_1;
            rgb[2] = var_2;
        }
    }
}

color!(
    Yuv,
    "Three-channel, luma, blue projection and red projection"
);
impl Color for Yuv {
    const NAME: &'static str = "yuv";
    const CHANNELS: Channel = 3;

    fn from_rgb(rgb: &Pixel<Rgb>, mut pixel: &mut Pixel<Self>) {
        let r = rgb[0];
        let g = rgb[1];
        let b = rgb[2];

        pixel[0] = 0.299 * r + 0.587 * g + 0.114 * b;
        pixel[1] = -0.147 * r + 0.289 + g + 0.436 * b;
        pixel[2] = 0.615 * r + 0.515 * g + 0.1 * b;
    }

    fn to_rgb(px: &Pixel<Self>, mut rgb: &mut Pixel<Rgb>) {
        let y = px[0];
        let u = px[1];
        let v = px[2];
        rgb[0] = y + 1.14 * v;
        rgb[1] = y - 0.395 * u - 0.581 * v;
        rgb[2] = y + 2.032 * u;
    }
}

color!(Cmyk, "Four-channel, cyan, magenta, yellow and black");
impl Color for Cmyk {
    const NAME: &'static str = "cmyk";
    const CHANNELS: Channel = 4;

    fn from_rgb(rgb: &Pixel<Rgb>, mut pixel: &mut Pixel<Self>) {
        let r = rgb[0];
        let g = rgb[1];
        let b = rgb[2];
        let c = 1.0 - r;
        let m = 1.0 - g;
        let y = 1.0 - b;
        let mut k = 1.0;

        if c < k {
            k = c
        }

        if m < k {
            k = m
        }

        if y < k {
            k = y
        }

        if k == 1. {
            pixel[0] = 0.;
            pixel[1] = 0.;
            pixel[2] = 0.;
            pixel[3] = k;
        } else {
            pixel[0] = (c - k) / (1. - k);
            pixel[1] = (m - k) / (1. - k);
            pixel[2] = (y - k) / (1. - k);
            pixel[3] = k;
        }
    }

    fn to_rgb(cmyk: &Pixel<Cmyk>, mut rgb: &mut Pixel<Rgb>) {
        let mut c = cmyk[0];
        let mut m = cmyk[1];
        let mut y = cmyk[2];
        let k = cmyk[3];

        c = c * (1.0 - k) + k;
        m = m * (1.0 - k) + k;
        y = y * (1.0 - k) + k;

        rgb[0] = 1.0 - c;
        rgb[1] = 1.0 - m;
        rgb[2] = 1.0 - y;
    }
}

/// Convert between colors
#[derive(Clone, Copy, Default)]
pub struct Convert<T: Color>(std::marker::PhantomData<T>);

impl<C: Color> Convert<C> {
    /// Create new color conversion context
    pub fn new() -> Convert<C> {
        Convert(std::marker::PhantomData)
    }
}

impl<T: Color> Filter for Convert<T> {
    fn compute_at(
        &self,
        pt: Point,
        input: &[&Image<impl Type, impl Color>],
        dest: &mut DataMut<impl Type, impl Color>,
    ) {
        input[0].get_pixel(pt).convert_to_data(dest);
    }
}
