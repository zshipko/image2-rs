use color::{Color, Gray, Rgb, Rgba};
use filter::{Filter, RgbaToRgb, ToColor, ToGrayscale};
use image_buf::ImageBuf;
use image_ref::ImageRef;
use pixel::{Pixel, PixelMut};
use ty::Type;

use rayon::prelude::*;

#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub enum Layout {
    Planar,
    Interleaved,
}

impl Default for Layout {
    fn default() -> Layout {
        Layout::Interleaved
    }
}

/// Iterate over pixels using Image::at_mut
#[macro_export]
macro_rules! image2_for_each_mut {
    ($image:expr, $i:ident, $j:ident, $px:ident, $body:block) => {
        for $j in 0..$image.height() {
            for $i in 0..$image.width() {
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
pub fn index(
    layout: Layout,
    width: usize,
    height: usize,
    channels: usize,
    x: usize,
    y: usize,
    c: usize,
) -> usize {
    match layout {
        Layout::Planar => width * height * c + width * y + x,
        Layout::Interleaved => width * channels * y + channels * x + c,
    }
}

/// The Image trait defines many methods for interaction with images in a generic manner
pub trait Image<T: Type, C: Color>: Sync + Send {
    /// Returns the width, height and channels of an image
    fn shape(&self) -> (usize, usize, usize);

    /// Determines the layout of image data
    fn layout(&self) -> Layout;

    /// Change layout without changing any image data
    fn set_layout(&mut self, layout: Layout);

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
        let (width, height, channels) = self.shape();
        index(self.layout(), width, height, channels, x, y, c)
    }

    /// Create a new, empty pixel with each component set to 0
    fn empty_pixel(&self) -> Vec<T> {
        vec![T::zero(); C::channels()]
    }

    /// Get a vector of mutable references to each component at (x, y)
    fn at_mut(&mut self, x: usize, y: usize) -> Vec<&mut T> {
        let mut px = Vec::with_capacity(C::channels());
        let (width, height, channels) = self.shape();
        let layout = self.layout();
        let data = self.data_mut().as_mut_ptr();
        for i in 0..C::channels() {
            let index = index(layout, width, height, channels, x, y, i);
            unsafe { px.push(&mut *data.add(index)) }
        }
        px
    }

    /// Get a vector of immutable references to each component at (x, y)
    fn at(&self, x: usize, y: usize) -> Vec<&T> {
        let mut px = Vec::with_capacity(C::channels());
        let (width, height, channels) = self.shape();
        let layout = self.layout();
        let data = self.data().as_ptr();
        for i in 0..C::channels() {
            let index = index(layout, width, height, channels, x, y, i);
            unsafe { px.push(&*data.add(index)) }
        }
        px
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

    /// Convert image layout type
    fn convert_layout(&mut self, layout: Layout) {
        if self.layout() == layout {
            return;
        }

        let mut buf: ImageBuf<T, C> =
            ImageBuf::new_with_layout(self.width(), self.height(), layout);
        buf.for_each(|(x, y), mut px| {
            for i in 0..C::channels() {
                let index = self.index(x, y, i);
                *px[i] = self.data()[index]
            }
        });
        self.data_mut().copy_from_slice(buf.data());
        self.set_layout(layout);
    }

    /// Convert Image to ImageRef
    fn as_image_ref(&mut self) -> ImageRef<T, C> {
        ImageRef::new(self.width(), self.height(), self.layout(), self.data_mut())
    }

    /// Iterate over each pixel in parallel
    fn for_each<F: Sync + Send + Fn((usize, usize), Vec<&mut T>)>(&mut self, f: F) {
        match self.layout() {
            Layout::Interleaved => {
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
            Layout::Planar => {
                image2_for_each_mut!(self, x, y, px, { f((x, y), px) });
            }
        }
    }

    /// Create a new image from the region specified by (x, y, width, height)
    fn crop(&self, x: usize, y: usize, width: usize, height: usize) -> ImageBuf<T, C> {
        let mut dest = ImageBuf::new(width, height);

        dest.for_each(|(i, j), mut px| {
            let src = self.at(x + i, y + j);
            for c in 0..C::channels() {
                *px[c] = *src[c]
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
