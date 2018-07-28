use color::Color;
use ty::Type;
use pixel::{Pixel, PixelVec};

use rayon::prelude::*;

#[inline]
fn index(width: usize, channels: usize, x: usize, y: usize) -> usize {
    width * channels * y + x * channels
}

/// `image2_for_each` allows you to iterate over each pixel in an image using a nested for-loop,
/// this is provided for convenience to avoid having to write this for-loop by hand over and
/// over again
#[macro_export]
macro_rules! image2_for_each {
    ($image:expr, $i:ident, $j:ident, $px:ident, $body:block) => {
        for $j in 0..$image.height() {
            for $i in 0..$image.width() {
                let $px = $image.at($i, $j);
                $body
            }
        }
    }
}

/// The `Image` trait is the core trait for `image2`
pub trait Image<T: Type, C: Color>: Sync + Send {
    fn shape(&self) -> (usize, usize, usize);

    fn width(&self) -> usize {
        let (width, _, _) = self.shape();
        width
    }

    fn height(&self) -> usize {
        let (_, height, _) = self.shape();
        height
    }

    fn channels(&self) -> usize {
        let (_, _, channels) = self.shape();
        channels
    }

    fn data(&self) -> &[T];
    fn data_mut(&mut self) -> &mut [T];

    fn at(&self, x: usize, y: usize) -> &[T] {
        let (width, _, channels) = self.shape();
        let index = index(width, channels, x, y);
        &self.data()[index..index + channels]
    }

    fn at_mut(&mut self, x: usize, y: usize) -> &mut [T] {
        let (width, _, channels) = self.shape();
        let index = index(width, channels, x, y);
        &mut self.data_mut()[index..index + channels]
    }

    fn get(&self, x: usize, y: usize, c: usize) -> f64 {
        let (width, height, channels) = self.shape();
        if x >= width || y >= height || c >= channels {
            return 0.0;
        }
        let index = index(width, channels, x, y);
        match self.data()[index + c].to_f64() {
            Some(f) => T::normalize(f),
            None => 0.0,
        }
    }

    fn set(&mut self, x: usize, y: usize, c: usize, f: f64) {
        let (width, height, channels) = self.shape();
        if x >= width || y >= height || c >= channels {
            return;
        }
        match T::from(T::denormalize(f)) {
            Some(f) => self.at_mut(x, y)[c] = f,
            None => (),
        }
    }

    fn convert_type<U: Type, I: Image<U, C>>(&self, dest: &mut I) {
        let ddata = dest.data_mut();
        for (i, x) in self.data().iter().enumerate() {
            ddata[i] = x.convert();
        }
    }

    fn for_each<F: Sync + Send + Fn((usize, usize), Vec<&mut T>)>(&mut self, f: F) {
        let (width, _height, channels) = self.shape();

        self.data_mut()
            .par_iter_mut()
            .chunks(channels)
            .enumerate()
            .for_each(|(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                f((x, y), pixel)
            });
    }

    fn mean_stddev(&self) -> (Vec<f64>, Vec<f64>) {
        let mut mean = PixelVec::empty::<C>();
        let mut variance = PixelVec::empty::<C>();

        image2_for_each!(self, i, j, px, {
            let v = px.to_pixel_vec();
            mean += v.clone();
            variance += &v * &v;
        });

        mean = mean.map(|x| x / (self.width() * self.height()) as f64);
        variance = variance.map(|x| (x / (self.width() * self.height()) as f64));
        variance -= &mean * &mean;

        (mean.to_vec(), variance.to_vec())
    }
}

