use crate::color::{Bgr, Color, Gray, Rgb, Rgba};
use crate::filter::{Filter, RgbToBgr, RgbaToRgb, ToColor, ToGrayscale};
use crate::image_buf::ImageBuf;
use crate::image_ref::ImageRef;
use crate::pixel::{Pixel, PixelMut};
use crate::ty::Type;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Iterate over pixels using Image::at_mut
#[macro_export]
macro_rules! image2_for_each_mut {
    ($image:expr, $i:ident, $j:ident, $px:ident, $body:block) => {
        for $j in 0..$image.height() {
            for $i in 0..$image.width() {
                #[allow(unused_mut)]
                let mut $px = $image.at_mut($i, $j);
                $body
            }
        }
    };
}

/// Iterate over pixels using Image::get_pixel
#[macro_export]
macro_rules! image2_for_each {
    ($image:expr, $i:ident, $j:ident, $px:ident, $body:block) => {
        let mut $px = $image.empty_pixel();
        for $j in 0..$image.height() {
            for $i in 0..$image.width() {
                $image.get_pixel($i, $j, &mut $px);
                $body
            }
        }
    };
}

#[inline]
pub fn index(width: usize, channels: usize, x: usize, y: usize, c: usize) -> usize {
    width * channels * y + channels * x + c
}

/// The Image trait defines many methods for interaction with images in a generic manner
pub trait Image<T: Type, C: Color>: Sync + Send {
    /// Returns the width, height and channels of an image
    fn shape(&self) -> (usize, usize, usize);

    /// An immutable reference to the underlying image data
    fn data(&self) -> &[T];

    /// A mutable reference to the underlying image data
    fn data_mut(&mut self) -> &mut [T];

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

    /// Get the offset of the component at (x, y, c)
    fn index(&self, x: usize, y: usize, c: usize) -> usize {
        let (width, _height, channels) = self.shape();
        index(width, channels, x, y, c)
    }

    /// Create a new, empty pixel with each component set to 0
    fn empty_pixel(&self) -> Vec<T> {
        vec![T::zero(); C::channels()]
    }

    /// Get a vector of mutable references to each component at (x, y)
    fn at_mut(&mut self, x: usize, y: usize) -> &mut [T] {
        let (width, _height, channels) = self.shape();
        let index = index(width, channels, x, y, 0);
        &mut self.data_mut()[index..index + C::channels()]
    }

    /// Get a vector of immutable references to each component at (x, y)
    fn at(&self, x: usize, y: usize) -> &[T] {
        let (width, _height, channels) = self.shape();
        let index = index(width, channels, x, y, 0);
        &self.data()[index..index + C::channels()]
    }

    /// Load data from the pixel at (x, y) into px
    fn get_pixel<'a, P: PixelMut<'a, T, C>>(&self, x: usize, y: usize, px: &mut P) {
        for i in 0..C::channels() {
            let index = self.index(x, y, i);
            px.as_mut()[i] = self.data()[index]
        }
    }

    /// Set data at (x, y) from px
    fn set_pixel<'a, P: Pixel<'a, T, C>>(&mut self, x: usize, y: usize, px: &P) {
        for i in 0..C::channels() {
            let index = self.index(x, y, i);
            self.data_mut()[index] = px.as_ref()[i]
        }
    }

    /// Get a single component at (x, y, c) as a noramlized f64 value
    fn get_f(&self, x: usize, y: usize, c: usize) -> f64 {
        let (width, height, channels) = self.shape();
        if x >= width || y >= height || c >= channels {
            return 0.0;
        }
        let index = self.index(x, y, c);
        match self.data()[index].to_f64() {
            Some(f) => T::normalize(f),
            None => 0.0,
        }
    }

    /// Set the component at (x, y, c) using a normalized f64 value
    fn set_f(&mut self, x: usize, y: usize, c: usize, f: f64) {
        let (width, height, channels) = self.shape();
        if x >= width || y >= height || c >= channels {
            return;
        }
        let index = self.index(x, y, c);
        if let Some(f) = T::from(T::denormalize(f)) {
            self.data_mut()[index] = f
        }
    }

    /// Get a single component at (x, y, c)
    fn get(&self, x: usize, y: usize, c: usize) -> T {
        let (width, height, channels) = self.shape();
        if x >= width || y >= height || c >= channels {
            return T::zero();
        }
        let index = self.index(x, y, c);
        self.data()[index]
    }

    /// Set a single component at (x, y, c)
    fn set(&mut self, x: usize, y: usize, c: usize, t: T) {
        let (width, height, channels) = self.shape();
        if x >= width || y >= height || c >= channels {
            return;
        }
        let index = self.index(x, y, c);
        self.data_mut()[index] = t;
    }

    /// Convert from type T to type U
    fn convert_type<U: Type, I: Image<U, C>>(&self, dest: &mut I) {
        let ddata = dest.data_mut();
        for (i, x) in self.data().iter().enumerate() {
            ddata[i] = x.convert();
        }
    }

    /// Convert Image to ImageRef
    fn as_image_ref(&mut self) -> ImageRef<T, C> {
        ImageRef::new(self.width(), self.height(), self.data_mut())
    }

    /// Iterate over each pixel
    #[cfg(feature = "parallel")]
    fn for_each<F: Sync + Send + Fn((usize, usize), &mut [T])>(&mut self, f: F) {
        let (width, _height, channels) = self.shape();
        self.data_mut()
            .par_chunks_mut(channels)
            .enumerate()
            .for_each(|(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                f((x, y), pixel)
            });
    }

    /// Iterate over each pixel
    #[cfg(not(feature = "parallel"))]
    fn for_each<F: Sync + Send + Fn((usize, usize), &mut [T])>(&mut self, f: F) {
        let (width, _height, channels) = self.shape();
        self.data_mut()
            .chunks_mut(channels)
            .enumerate()
            .for_each(|(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                f((x, y), pixel)
            });
    }

    /// Create a new image from the region specified by (x, y, width, height)
    fn crop(&self, x: usize, y: usize, width: usize, height: usize) -> ImageBuf<T, C> {
        let mut dest = ImageBuf::new(width, height);

        dest.for_each(|(i, j), px| {
            let src = self.at(x + i, y + j);
            for c in 0..C::channels() {
                px[c] = src[c]
            }
        });

        dest
    }

    fn copy(&self) -> ImageBuf<T, C> {
        let (width, height, _) = self.shape();
        let mut dest = ImageBuf::new(width, height);

        dest.for_each(|(i, j), px| {
            let src = self.at(i, j);
            for c in 0..C::channels() {
                px[c] = src[c]
            }
        });

        dest
    }
}

/// Provides a way to convert between image types
pub trait Convert<FromType: Type, FromColor: Color, ToType: Type, ToColor: Color> {
    fn convert(&self, to: &mut impl Image<ToType, ToColor>);
}

impl<T: Type, U: Type, I: Image<T, Rgb>> Convert<T, Rgb, U, Rgba> for I {
    fn convert(&self, to: &mut impl Image<U, Rgba>) {
        ToColor.eval(to, &[self]);
    }
}

impl<T: Type, U: Type, I: Image<T, Rgba>> Convert<T, Rgba, U, Rgb> for I {
    fn convert(&self, to: &mut impl Image<U, Rgb>) {
        RgbaToRgb.eval(to, &[self]);
    }
}

impl<T: Type, U: Type, I: Image<T, Rgb>> Convert<T, Rgb, U, Gray> for I {
    fn convert(&self, to: &mut impl Image<U, Gray>) {
        ToGrayscale.eval(to, &[self]);
    }
}

impl<T: Type, U: Type, I: Image<T, Rgba>> Convert<T, Rgba, U, Gray> for I {
    fn convert(&self, to: &mut impl Image<U, Gray>) {
        ToGrayscale.eval(to, &[self]);
    }
}

impl<T: Type, U: Type, I: Image<T, Gray>> Convert<T, Gray, U, Rgb> for I {
    fn convert(&self, to: &mut impl Image<U, Rgb>) {
        ToColor.eval(to, &[self]);
    }
}

impl<T: Type, U: Type, I: Image<T, Gray>> Convert<T, Gray, U, Rgba> for I {
    fn convert(&self, to: &mut impl Image<U, Rgba>) {
        ToColor.eval(to, &[self]);
    }
}

impl<T: Type, U: Type, I: Image<T, Rgb>> Convert<T, Rgb, U, Bgr> for I {
    fn convert(&self, to: &mut impl Image<U, Bgr>) {
        RgbToBgr.eval(to, &[self]);
    }
}

impl<T: Type, U: Type, I: Image<T, Bgr>> Convert<T, Bgr, U, Rgb> for I {
    fn convert(&self, to: &mut impl Image<U, Rgb>) {
        RgbToBgr.eval(to, &[self]);
    }
}
