use crate::color::{Bgr, Color, Gray, Rgb, Rgba};
use crate::filter::{AlphaBlend, Filter, SwapChannel, ToColor, ToGrayscale};
use crate::image_buf::ImageBuf;
use crate::image_ptr::{Free, ImagePtr};
use crate::image_ref::ImageRef;
use crate::pixel::{Pixel, PixelMut};
use crate::ty::Type;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[inline]
pub fn index(width: usize, channels: usize, x: usize, y: usize, c: usize) -> usize {
    width * channels * y + channels * x + c
}

#[derive(Debug, Clone)]
pub struct Diff(std::collections::HashMap<(usize, usize, usize), f64>);

impl Diff {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn apply<T: Type, C: Color, I: Image<T, C>>(&self, image: &mut I) {
        self.0.iter().for_each(|((x, y, c), v)| {
            let f = image.get_f(*x, *y, *c);
            image.set_f(*x, *y, *c, f + v);
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Hash(u64);

fn check_bit(number: u64, n: usize) -> bool {
    (number >> n) & 1 == 0
}

impl Hash {
    pub fn diff(&self, other: &Hash) -> u64 {
        let mut diff = 0;

        for i in 0..64 {
            if check_bit(self.0, i) != check_bit(other.0, i) {
                diff += 1;
            }
        }

        diff
    }
}

impl From<Hash> for String {
    fn from(hash: Hash) -> String {
        format!("{:08x}", hash.0)
    }
}

impl From<Hash> for u64 {
    fn from(hash: Hash) -> u64 {
        hash.0
    }
}

fn free_slice<T: Type>(ptr: *mut T, size: usize) {
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, size) };
    std::mem::drop(slice)
}

/// The Image trait defines many methods for interaction with images in a generic manner
pub trait Image<T: Type, C: Color>: Sized + Sync + Send {
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

    /// Get the number of total elements in an image
    fn len(&self) -> usize {
        let (w, h, c) = self.shape();
        w * h * c
    }

    /// Get the total number of bytes needed to store the image data
    fn total_bytes(&self) -> usize {
        self.len() * std::mem::size_of::<T>()
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

    /// Create a new, empty pixel with each component set to 0
    fn empty_pixel_f(&self) -> Vec<f64> {
        vec![0.0; C::channels()]
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
        let data = self.data();
        let px = px.as_mut();
        let index = self.index(x, y, 0);
        for i in 0..C::channels() {
            px[i] = data[index + i]
        }
    }

    /// Load data from the pixel at (x, y) into px and convert to normalized f64
    fn get_pixel_f<'a, P: PixelMut<'a, f64, C>>(&self, x: usize, y: usize, px: &mut P) {
        let data = self.data();
        let index = self.index(x, y, 0);
        let px = px.as_mut();
        for i in 0..C::channels() {
            px[i] = T::to_f(&data[index + i]);
        }
    }

    /// Set data at (x, y) to px
    fn set_pixel<'a, P: Pixel<'a, T, C>>(&mut self, x: usize, y: usize, px: &P) {
        let index = self.index(x, y, 0);
        let data = self.data_mut();
        let px = px.as_ref();
        for i in 0..C::channels() {
            data[index + i] = px[i]
        }
    }

    /// Set data at (x, y) to px after denormalizing
    fn set_pixel_f<'a, P: Pixel<'a, f64, C>>(&mut self, x: usize, y: usize, px: &P) {
        let index = self.index(x, y, 0);
        let data = self.data_mut();
        let px = px.as_ref();
        for i in 0..C::channels() {
            data[index + i] = T::from_f(px[i]);
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
        self.data_mut()[index] = T::from_f(f);
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

    /// Consume and convert Image to ImagePtr
    fn to_image_ptr<'a>(mut self) -> ImagePtr<'a, T, C> {
        let ptr = self.data_mut().as_mut_ptr();
        std::mem::forget(ptr);
        ImagePtr::new(self.width(), self.height(), ptr, Free::Function(free_slice))
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
            .chunks_exact_mut(channels)
            .enumerate()
            .for_each(|(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                f((x, y), pixel)
            });
    }

    /// Iterate over each pixel
    #[cfg(feature = "parallel")]
    fn for_each2<F: Sync + Send + Fn((usize, usize), &mut [T], &[T]), I: Image<T, C>>(
        &mut self,
        other: &I,
        f: F,
    ) {
        let (width, _height, channels) = self.shape();
        let b = other.data().par_chunks(channels);
        self.data_mut()
            .par_chunks_mut(channels)
            .zip(b)
            .enumerate()
            .for_each(|(n, (pixel, pixel1))| {
                let y = n / width;
                let x = n - (y * width);
                f((x, y), pixel, pixel1)
            });
    }

    /// Iterate over each pixel
    #[cfg(not(feature = "parallel"))]
    fn for_each2<F: Sync + Send + Fn((usize, usize), &mut [T], &[T]), I: Image<T, C>>(
        &mut self,
        other: &I,
        f: F,
    ) {
        let (width, _height, channels) = self.shape();
        let b = other.data().chunks(channels);
        self.data_mut()
            .chunks_mut(channels)
            .zip(b)
            .enumerate()
            .for_each(|(n, (pixel, pixel1))| {
                let y = n / width;
                let x = n - (y * width);
                f((x, y), pixel, pixel1)
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

    fn clone(&self) -> ImageBuf<T, C> {
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

    #[cfg(feature = "parallel")]
    fn multiply<'a, P: Pixel<'a, f64, C>>(&mut self, px: P) {
        let data = self.data_mut();
        let px = px.to_vec();
        data.par_chunks_mut(C::channels()).for_each(|x| {
            for (n, i) in x.into_iter().enumerate() {
                *i = T::from_float(T::clamp(px[n] * T::to_float(i)));
            }
        });
    }

    #[cfg(feature = "parallel")]
    fn add<'a, P: Pixel<'a, f64, C>>(&mut self, px: P) {
        let data = self.data_mut();
        let px = px.to_vec();
        data.par_chunks_mut(C::channels()).for_each(|x| {
            for (n, i) in x.into_iter().enumerate() {
                *i = T::from_float(T::clamp(px[n] + T::to_float(i)));
            }
        });
    }

    #[cfg(not(feature = "parallel"))]
    fn multiply<'a, P: Pixel<'a, f64, C>>(&mut self, px: P) {
        let data = self.data_mut();
        let px = px.to_vec();
        data.chunks_mut(C::channels()).for_each(|x| {
            for (n, i) in x.into_iter().enumerate() {
                *i = T::from_f(px[n] * T::to_f(i));
            }
        });
    }

    #[cfg(not(feature = "parallel"))]
    fn add<'a, P: Pixel<'a, f64, C>>(&mut self, px: P) {
        let data = self.data_mut();
        let px = px.to_vec();
        data.chunks_mut(C::channels()).for_each(|x| {
            for (n, i) in x.into_iter().enumerate() {
                *i = T::from_f(px[n] + T::to_f(i));
            }
        });
    }

    fn hash(&self) -> Hash {
        let mut small = ImageBuf::new(8, 8);
        crate::transform::resize(&mut small, self, 8, 8);
        let mut hash = 0u64;
        let mut index = 0;
        let mut px = self.empty_pixel();
        for j in 0..8 {
            for i in 0..8 {
                small.get_pixel(i, j, &mut px);
                let avg: T = Pixel::<T, C>::iter(&px).map(|x| *x).sum();
                let f = T::to_f(&avg) / C::channels() as f64;
                if f > 0.5 {
                    hash = hash | (1 << index)
                } else {
                    hash = hash & !(1 << index)
                }
                index += 1
            }
        }
        Hash(hash)
    }

    fn diff<I: Image<T, C>>(&self, other: &I) -> Diff {
        let mut map = std::collections::HashMap::new();

        for j in 0..self.height() {
            for i in 0..self.width() {
                let a = self.at(i, j);
                let b = other.at(i, j);
                for c in 0..C::channels() {
                    let a = T::normalize(T::to_float(&a[c]));
                    let b = T::normalize(T::to_float(&b[c]));
                    if a != b {
                        map.insert((i, j, c), a - b);
                    }
                }
            }
        }

        Diff(map)
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
        AlphaBlend.eval(to, &[self]);
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
        SwapChannel(0, 2).eval(to, &[self]);
    }
}

impl<T: Type, U: Type, I: Image<T, Bgr>> Convert<T, Bgr, U, Rgb> for I {
    fn convert(&self, to: &mut impl Image<U, Rgb>) {
        SwapChannel(0, 2).eval(to, &[self]);
    }
}
