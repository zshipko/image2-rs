use std::marker::PhantomData;

use crate::*;

use rayon::prelude::*;

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
    pub fn has_alpha(&self) -> bool {
        C::ALPHA
    }

    pub fn color_name(&self) -> &str {
        C::NAME
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
                    hash = hash | (1 << index)
                } else {
                    hash = hash & !(1 << index)
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
    pub fn at(&self, x: usize, y: usize, px: &mut [T]) -> bool {
        if !self.in_bounds(x, y) || px.len() < C::CHANNELS {
            return false;
        }

        px.copy_from_slice(self.get(x, y));
        true
    }

    /// Load data from and `Image` into an existing `Pixel` structure
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
    pub fn set_pixel(&mut self, x: usize, y: usize, px: &Pixel<C>) {
        let data = self.get_mut(x, y);
        px.copy_to_slice(data);
    }

    /// Open an image from disk
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Image<T, C>, Error> {
        let input = io::Input::open(path)?;
        input.read()
    }


    /// Save an image to disk
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        let output = io::Output::create(path)?;
        output.write(self)
    }

    /// Iterate over part of an image in parallel with mutable data access
    pub fn pixels_rect_mut<'a>(
        &'a mut self,
        x_: usize,
        y_: usize,
        width_: usize,
        height_: usize,
    ) -> impl 'a + rayon::iter::ParallelIterator<Item = ((usize, usize), &mut [T])> {
        let (width, _height, channels) = self.shape();
        self.data
            .as_mut_slice()
            .par_chunks_mut(channels)
            .enumerate()
            .filter_map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                if x >= x_ && x < x_ + width_ && y >= y_ && y < y_ + height_ {
                    return Some(((x, y), pixel));
                }

                None
            })
    }

    /// Iterate over part of an image in parallel
    pub fn pixels_rect<'a>(
        &'a self,
        x_: usize,
        y_: usize,
        width_: usize,
        height_: usize,
    ) -> impl 'a + rayon::iter::ParallelIterator<Item = ((usize, usize), &[T])> {
        let (width, _height, channels) = self.shape();
        self.data
            .as_slice()
            .par_chunks(channels)
            .enumerate()
            .filter_map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                if x >= x_ && x < x_ + width_ && y >= y_ && y < y_ + height_ {
                    return Some(((x, y), pixel));
                }

                None
            })
    }

    /// Get pixel iterator
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

    /// Get mutable pixel iterator
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

    /// Iterate over each pixel in parallel
    pub fn for_each<F: Sync + Send + Fn((usize, usize), &mut [T])>(&mut self, f: F) {
        self.pixels_mut().for_each(|((x, y), px)| f((x, y), px));
    }

    /// Iterate over each pixel of two images at once in parallel
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

    /// Iterate over each pixel without threads
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

    /// Iterate over each pixel without threads
    pub async fn each_pixel_mut<F: Sync + Send + FnMut((usize, usize), &mut [T])>(&mut self, mut f: F) {
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

    /// Apply a filter
    pub fn apply(
        &self,
        filter: impl Filter,
        mut dest: Image<impl Type, impl Color>,
    ) -> Image<impl Type, impl Color> {
        filter.eval(&mut dest, &[self]);
        dest
    }

    /// Convert to `ImageBuf`
    pub(crate) fn to_image_buf(&mut self) -> io::ImageBuf {
        io::ImageBuf::new_with_data(
            self.meta.width,
            self.meta.height,
            self.channels(),
            self.data.as_mut_slice(),
        )
    }

    /// Convert to `ImageBuf`
    pub(crate) fn to_const_image_buf(&self) -> io::ImageBuf {
        io::ImageBuf::const_new_with_data(
            self.meta.width,
            self.meta.height,
            self.channels(),
            self.data.as_slice(),
        )
    }

    /// Convert colorspace from `a` to `b` into an existing image
    pub fn convert_colorspace_to(
        &self,
        dest: &mut Image<T, C>,
        a: impl AsRef<str>,
        b: impl AsRef<str>,
    ) -> Result<(), Error> {
        let buf = self.to_const_image_buf();
        let ok = buf.convert_color(&mut dest.to_image_buf(), a.as_ref(), b.as_ref());
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
}
