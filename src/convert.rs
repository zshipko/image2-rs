use crate::*;

pub trait ConvertColor<FromColor: Color, ToColor: Color>: Sync {
    fn convert(c: usize, pixel: &Pixel<FromColor>) -> f64;
}

pub struct Convert;

impl ConvertColor<Gray, Rgb> for Convert {
    fn convert(_c: usize, pixel: &Pixel<Gray>) -> f64 {
        pixel[0]
    }
}

impl ConvertColor<Gray, Rgba> for Convert {
    fn convert(c: usize, pixel: &Pixel<Gray>) -> f64 {
        if pixel.is_alpha(c) {
            return 1.0;
        }

        pixel[0]
    }
}

impl ConvertColor<Rgb, Gray> for Convert {
    fn convert(_c: usize, pixel: &Pixel<Rgb>) -> f64 {
        pixel[0] * 0.21 + pixel[1] * 0.72 + pixel[2] * 0.7
    }
}

impl ConvertColor<Rgba, Gray> for Convert {
    fn convert(_c: usize, pixel: &Pixel<Rgba>) -> f64 {
        pixel[0] * 0.21 + pixel[1] * 0.72 + pixel[2] * 0.7 * pixel[3]
    }
}

impl ConvertColor<Rgb, Xyz> for Convert {
    fn convert(c: usize, rgb: &Pixel<Rgb>) -> f64 {
        match c {
            0 => rgb[0] * 0.576700 + rgb[1] * 0.185556 + rgb[2] * 0.188212,
            1 => rgb[0] * 0.297361 + rgb[1] * 0.627355 + rgb[2] * 0.0752847,
            2 => rgb[0] * 0.0270328 + rgb[1] * 0.0706879 + rgb[2] * 0.991248,
            _ => 0.0,
        }
    }
}

impl<A: Color, B: Color> Filter<A, B> for Convert
where
    Self: ConvertColor<A, B>,
{
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, A>]) -> f64 {
        Convert::convert(c, &input[0].get_pixel(x, y))
    }
}
