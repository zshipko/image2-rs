use std::marker::PhantomData;

use crate::*;

#[cfg(feature = "parallel")]
use rayon::{iter::ParallelIterator, prelude::*};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Region {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Region {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Region {
        Region {
            x,
            y,
            width,
            height,
        }
    }

    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

/// Image metadata
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub struct Meta<T: Type, C: Color> {
    pub width: usize,
    pub height: usize,
    _type: PhantomData<T>,
    _color: PhantomData<C>,
}

impl<T: Type, C: Color> Meta<T, C> {
    pub fn new(w: usize, h: usize) -> Meta<T, C> {
        Meta {
            width: w,
            height: h,
            _type: PhantomData,
            _color: PhantomData,
        }
    }

    pub fn has_alpha(&self) -> bool {
        C::ALPHA
    }

    pub fn color_name(&self) -> &str {
        C::NAME
    }

    pub fn type_name(&self) -> &str {
        T::type_name()
    }
}

/// Image type
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image<T: Type, C: Color> {
    /// Metadata
    pub meta: Meta<T, C>,

    /// Pixel data
    pub data: Vec<T>,
}

/// Hash is used for content-based hashing
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Hash(u128);

fn check_bit(number: u128, n: usize) -> bool {
    (number >> n) & 1 == 0
}

impl Hash {
    /// Compute difference between two hashes
    pub fn diff(&self, other: &Hash) -> u128 {
        let mut diff = 0;

        for i in 0..128 {
            if check_bit(self.0, i) != check_bit(other.0, i) {
                diff += 1;
            }
        }

        diff
    }
}

impl From<Hash> for String {
    fn from(hash: Hash) -> String {
        format!("{:016x}", hash.0)
    }
}

impl From<Hash> for u128 {
    fn from(hash: Hash) -> u128 {
        hash.0
    }
}

impl<T: Type, C: Color> std::ops::Index<(usize, usize)> for Image<T, C> {
    type Output = [T];

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        self.get(x, y)
    }
}

impl<T: Type, C: Color> std::ops::IndexMut<(usize, usize)> for Image<T, C> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        self.get_mut(x, y)
    }
}

impl<T: Type, C: Color> std::ops::Index<(usize, usize, usize)> for Image<T, C> {
    type Output = T;

    fn index(&self, (x, y, c): (usize, usize, usize)) -> &Self::Output {
        &self.get(x, y)[c]
    }
}

impl<T: Type, C: Color> std::ops::IndexMut<(usize, usize, usize)> for Image<T, C> {
    fn index_mut(&mut self, (x, y, c): (usize, usize, usize)) -> &mut Self::Output {
        &mut self.get_mut(x, y)[c]
    }
}

impl<T: Type, C: Color> Image<T, C> {
    /// Create a new image
    pub fn new(width: usize, height: usize) -> Image<T, C> {
        let data = vec![T::default(); width * height * C::CHANNELS];
        Image {
            meta: Meta {
                width,
                height,
                _type: PhantomData,
                _color: PhantomData,
            },
            data,
        }
    }

    /// Get image hash
    pub fn hash(&self) -> Hash {
        let mut small: Image<T, C> = Image::new(16, 8);
        crate::transform::resize(self, 16, 8).eval(&mut small, &[self]);
        let mut hash = 0u128;
        let mut index = 0;
        let mut px = Pixel::new();
        for j in 0..8 {
            for i in 0..16 {
                small.pixel_at(i, j, &mut px);
                let avg: f64 = px.iter().sum();
                let f = avg / C::CHANNELS as f64;
                if f > 0.5 {
                    hash |= 1 << index
                } else {
                    hash &= !(1 << index)
                }
                index += 1
            }
        }
        Hash(hash)
    }

    /// Create a new image with the same size, type and color
    pub fn new_like(&self) -> Image<T, C> {
        Image::new(self.meta.width, self.meta.height)
    }

    /// Create a new image with the same size and color as an existing image with the given type
    pub fn new_like_with_type<U: Type>(&self) -> Image<U, C> {
        Image::new(self.meta.width, self.meta.height)
    }

    /// Create a new image with the same size and type as an existing image with the given color
    pub fn new_like_with_color<D: Color>(&self) -> Image<T, D> {
        Image::new(self.meta.width, self.meta.height)
    }

    /// Create a new image with the same size as an existing image with the given type and color
    pub fn new_like_with_type_and_color<U: Type, D: Color>(&self) -> Image<U, D> {
        Image::new(self.meta.width, self.meta.height)
    }

    /// Maximum value for image type
    pub fn type_max(&self) -> f64 {
        T::MAX
    }

    /// Minimum value for image type
    pub fn type_min(&self) -> f64 {
        T::MIN
    }

    /// Returns the number of channels
    #[inline]
    pub fn channels(&self) -> usize {
        C::CHANNELS
    }

    /// Image width
    #[inline]
    pub fn width(&self) -> usize {
        self.meta.width
    }

    /// Image height
    #[inline]
    pub fn height(&self) -> usize {
        self.meta.height
    }

    /// Returns (width, height, channels)
    #[inline]
    pub fn shape(&self) -> (usize, usize, usize) {
        (self.meta.width, self.meta.height, self.channels())
    }

    pub fn buffer(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const u8,
                self.data.len() * std::mem::size_of::<T>(),
            )
        }
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_ptr() as *mut u8,
                self.data.len() * std::mem::size_of::<T>(),
            )
        }
    }

    /// Returns the size of a row
    #[inline]
    pub fn width_step(&self) -> usize {
        self.meta.width * self.channels()
    }

    /// Get the index of the specified pixel
    #[inline]
    pub fn index(&self, x: usize, y: usize) -> usize {
        self.width_step() * y + x * self.channels()
    }

    /// Get data at specified index
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> &[T] {
        let index = self.index(x, y);
        &self.data[index..index + self.channels()]
    }

    /// Get mutable data at specified index
    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut [T] {
        let index = self.index(x, y);
        let channels = self.channels();
        &mut self.data[index..index + channels]
    }

    /// Set data to specified location
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, data: impl AsRef<[T]>) {
        let image = self.get_mut(x, y);
        image.clone_from_slice(data.as_ref())
    }

    /// Returns true when (x, y) is in bounds for the given image
    #[inline]
    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.meta.width && y < self.meta.height
    }

    /// Get image data from an image, reusing an existing data buffer big enough for a single pixel
    #[inline]
    pub fn at(&self, x: usize, y: usize, px: &mut [T]) -> bool {
        if !self.in_bounds(x, y) || px.len() < C::CHANNELS {
            return false;
        }

        px.copy_from_slice(self.get(x, y));
        true
    }

    /// Get row
    #[inline]
    pub fn row(&self, y: usize) -> &[T] {
        let index = self.index(0, y);
        &self.data[index..index + self.channels() * self.width()]
    }

    /// Get mutable row
    #[inline]
    pub fn row_mut(&mut self, y: usize) -> &mut [T] {
        let index = self.index(0, y);
        let len = self.channels() * self.width();
        &mut self.data[index..index + len]
    }

    /// Load data from and `Image` into an existing `Pixel` structure
    #[inline]
    pub fn pixel_at(&self, x: usize, y: usize, px: &mut Pixel<C>) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }
        let data = self.get(x, y);
        px.copy_from_slice(data);
        true
    }

    /// Get a normalized pixel from an image
    pub fn get_pixel(&self, x: usize, y: usize) -> Pixel<C> {
        let mut px = Pixel::new();
        self.pixel_at(x, y, &mut px);
        px
    }

    /// Get a normalized float value
    pub fn get_f(&self, x: usize, y: usize, c: usize) -> f64 {
        if !self.in_bounds(x, y) || c >= C::CHANNELS {
            return 0.0;
        }

        let data = self.get(x, y);
        data[c].to_norm()
    }

    /// Set normalized float value
    pub fn set_f(&mut self, x: usize, y: usize, c: usize, f: f64) {
        if !self.in_bounds(x, y) || c >= C::CHANNELS {
            return;
        }
        let data = self.get_mut(x, y);
        data[c] = T::from_norm(f);
    }

    /// Set a normalized pixel to the specified location
    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, px: &Pixel<C>) {
        let data = self.get_mut(x, y);
        px.copy_to_slice(data);
    }

    /// Open an image from disk
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Image<T, C>, Error> {
        #[cfg(feature = "oiio")]
        {
            let input = io::Input::open(path)?;
            input.read()
        }

        #[cfg(not(feature = "oiio"))]
        {
            let x = io::magick::read(path)?;
            Ok(x)
        }
    }

    /// Save an image to disk
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        #[cfg(feature = "oiio")]
        {
            let output = io::Output::create(path)?;
            output.write(self)
        }

        #[cfg(not(feature = "oiio"))]
        {
            io::magick::write(path, self)?;
            Ok(())
        }
    }

    /// Iterate over part of an image with mutable data access
    #[cfg(feature = "parallel")]
    pub fn pixels_region_mut<'a>(
        &'a mut self,
        roi: Region,
    ) -> impl 'a + rayon::iter::ParallelIterator<Item = ((usize, usize), &mut [T])> {
        let (width, _height, channels) = self.shape();
        self.data
            .as_mut_slice()
            .par_chunks_mut(channels)
            .enumerate()
            .filter_map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                if roi.in_bounds(x, y) {
                    return Some(((x, y), pixel));
                }

                None
            })
    }

    /// Iterate over part of an image with mutable data access
    #[cfg(not(feature = "parallel"))]
    pub fn pixels_region_mut<'a>(
        &'a mut self,
        roi: Region,
    ) -> impl 'a + std::iter::Iterator<Item = ((usize, usize), &mut [T])> {
        let (width, _height, channels) = self.shape();
        self.data
            .as_mut_slice()
            .chunks_mut(channels)
            .enumerate()
            .filter_map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                if roi.in_bounds(x, y) {
                    return Some(((x, y), pixel));
                }

                None
            })
    }

    /// Iterate over part of an image
    #[cfg(feature = "parallel")]
    pub fn pixels_region<'a>(
        &'a self,
        roi: Region,
    ) -> impl 'a + rayon::iter::ParallelIterator<Item = ((usize, usize), &[T])> {
        let (width, _height, channels) = self.shape();
        self.data
            .as_slice()
            .par_chunks(channels)
            .enumerate()
            .filter_map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                if roi.in_bounds(x, y) {
                    return Some(((x, y), pixel));
                }

                None
            })
    }

    /// Iterate over part of an image
    #[cfg(not(feature = "parallel"))]
    pub fn pixels_region<'a>(
        &'a self,
        roi: Region,
    ) -> impl 'a + std::iter::Iterator<Item = ((usize, usize), &[T])> {
        let (width, _height, channels) = self.shape();
        self.data
            .as_slice()
            .chunks(channels)
            .enumerate()
            .filter_map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                if roi.in_bounds(x, y) {
                    return Some(((x, y), pixel));
                }

                None
            })
    }

    /// Get pixel iterator
    #[cfg(feature = "parallel")]
    pub fn pixels<'a>(
        &'a self,
    ) -> impl 'a + rayon::iter::ParallelIterator<Item = ((usize, usize), &[T])> {
        let (width, _height, channels) = self.shape();
        self.data
            .par_chunks(channels)
            .enumerate()
            .map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                ((x, y), pixel)
            })
    }

    /// Get pixel iterator
    #[cfg(not(feature = "parallel"))]
    pub fn pixels<'a>(&'a self) -> impl 'a + std::iter::Iterator<Item = ((usize, usize), &[T])> {
        let (width, _height, channels) = self.shape();
        self.data
            .chunks(channels)
            .enumerate()
            .map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                ((x, y), pixel)
            })
    }

    /// Get mutable pixel iterator
    #[cfg(feature = "parallel")]
    pub fn pixels_mut<'a>(
        &'a mut self,
    ) -> impl 'a + rayon::iter::ParallelIterator<Item = ((usize, usize), &mut [T])> {
        let (width, _height, channels) = self.shape();
        self.data
            .par_chunks_mut(channels)
            .enumerate()
            .map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                ((x, y), pixel)
            })
    }

    /// Get mutable pixel iterator
    #[cfg(not(feature = "parallel"))]
    pub fn pixels_mut<'a>(
        &'a mut self,
    ) -> impl 'a + std::iter::Iterator<Item = ((usize, usize), &mut [T])> {
        let (width, _height, channels) = self.shape();
        self.data
            .chunks_mut(channels)
            .enumerate()
            .map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                ((x, y), pixel)
            })
    }

    /// Iterate over each pixel applying `f` to every pixel
    pub fn for_each<F: Sync + Send + Fn((usize, usize), &mut [T])>(&mut self, f: F) {
        self.pixels_mut().for_each(|((x, y), px)| f((x, y), px));
    }

    /// Iterate over a region of pixels qpplying `f` to every pixel
    pub fn for_each_region<F: Sync + Send + Fn((usize, usize), &mut [T])>(
        &mut self,
        roi: Region,
        f: F,
    ) {
        self.pixels_region_mut(roi)
            .for_each(|((x, y), px)| f((x, y), px));
    }

    /// Iterate over each pixel of two images at once
    #[cfg(feature = "parallel")]
    pub fn for_each2<F: Sync + Send + Fn((usize, usize), &mut [T], &[T])>(
        &mut self,
        other: &Image<T, C>,
        f: F,
    ) {
        let (width, _height, channels) = self.shape();
        let b = other.data.as_slice().par_chunks(channels);
        self.data
            .as_mut_slice()
            .par_chunks_mut(channels)
            .zip(b)
            .enumerate()
            .for_each(|(n, (pixel, pixel1))| {
                let y = n / width;
                let x = n - (y * width);
                f((x, y), pixel, pixel1)
            });
    }

    /// Iterate over each pixel of two images at once
    #[cfg(not(feature = "parallel"))]
    pub fn for_each2<F: Sync + Send + Fn((usize, usize), &mut [T], &[T])>(
        &mut self,
        other: &Image<T, C>,
        f: F,
    ) {
        let (width, _height, channels) = self.shape();
        let b = other.data.as_slice().chunks(channels);
        self.data
            .as_mut_slice()
            .chunks_mut(channels)
            .zip(b)
            .enumerate()
            .for_each(|(n, (pixel, pixel1))| {
                let y = n / width;
                let x = n - (y * width);
                f((x, y), pixel, pixel1)
            });
    }

    /// Iterate over pixels, with a mutable closure
    pub fn each_pixel<F: Sync + Send + FnMut((usize, usize), &[T])>(&self, mut f: F) {
        let (width, _height, channels) = self.shape();

        self.data
            .as_slice()
            .chunks_exact(channels)
            .enumerate()
            .for_each(|(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                f((x, y), pixel)
            })
    }

    /// Iterate over mutable pixels, with a mutable closure
    pub fn each_pixel_mut<F: Sync + Send + FnMut((usize, usize), &mut [T])>(&mut self, mut f: F) {
        let (width, _height, channels) = self.shape();
        self.data
            .as_mut_slice()
            .chunks_exact_mut(channels)
            .enumerate()
            .for_each(|(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                f((x, y), pixel)
            });
    }

    /// Copy a region of an image to a new image
    pub fn crop(&self, roi: Region) -> Image<T, C> {
        let mut dest = Image::new(roi.width, roi.height);
        dest.copy_region(Region::new(0, 0, roi.width, roi.height), self, roi.x, roi.y);
        dest
    }

    /// Copy into a region from another image starting at the given offset
    pub fn copy_region(&mut self, roi: Region, other: &Image<T, C>, x_offs: usize, y_offs: usize) {
        self.for_each_region(roi, |(x, y), px| {
            px.copy_from_slice(other.get(x - roi.x + x_offs, y - roi.y + y_offs));
        });
    }

    /// Apply a filter
    pub fn apply(
        &mut self,
        input: &[&Image<impl Type, impl Color>],
        filter: impl Filter,
    ) -> &mut Self {
        filter.eval(self, input);
        self
    }

    /// Convert to `ImageBuf`
    #[cfg(feature = "oiio")]
    pub(crate) fn image_buf(&mut self) -> io::internal::ImageBuf {
        io::internal::ImageBuf::new_with_data(
            self.meta.width,
            self.meta.height,
            self.channels(),
            self.data.as_mut_slice(),
        )
    }

    /// Convert to `ImageBuf`
    #[cfg(feature = "oiio")]
    pub(crate) fn const_image_buf(&self) -> io::internal::ImageBuf {
        io::internal::ImageBuf::const_new_with_data(
            self.meta.width,
            self.meta.height,
            self.channels(),
            self.data.as_slice(),
        )
    }

    /// Convert colorspace from `a` to `b` into an existing image
    #[cfg(feature = "oiio")]
    pub fn convert_colorspace_to(
        &self,
        dest: &mut Image<T, C>,
        a: impl AsRef<str>,
        b: impl AsRef<str>,
    ) -> Result<(), Error> {
        let buf = self.const_image_buf();
        let ok = buf.convert_color(&mut dest.image_buf(), a.as_ref(), b.as_ref());
        if ok {
            Ok(())
        } else {
            Err(Error::FailedColorConversion(
                a.as_ref().into(),
                b.as_ref().into(),
            ))
        }
    }

    /// Convert colorspace from `a` to `b` into a new image
    #[cfg(feature = "oiio")]
    pub fn convert_colorspace(
        &self,
        a: impl AsRef<str>,
        b: impl AsRef<str>,
    ) -> Result<Image<T, C>, Error> {
        let mut dest = self.new_like_with_color();
        self.convert_colorspace_to(&mut dest, a, b)?;
        Ok(dest)
    }

    /// Get image histogram
    pub fn histogram(&self, bins: usize) -> Vec<Histogram> {
        let mut hist = vec![Histogram::new(bins); C::CHANNELS];

        self.each_pixel(|_, px| {
            for i in 0..C::CHANNELS {
                hist[i].add(px[i]);
            }
        });

        hist
    }

    /// Gamma correction
    pub fn gamma(&mut self, value: f64) {
        self.for_each(|_, px| {
            for x in px {
                *x = T::from_f64(T::to_f64(x).powf(1. / value))
            }
        })
    }
}
