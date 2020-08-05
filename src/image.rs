use std::marker::PhantomData;

use crate::*;

use rayon::prelude::*;

/// Image metadata
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image<T: Type, C: Color> {
    /// Metadata
    pub meta: Meta<T, C>,

    /// Pixel data
    pub data: Vec<T>,
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

    pub fn new_like(&self) -> Image<T, C> {
        Image::new(self.meta.width, self.meta.height)
    }

    pub fn new_like_with_type<U: Type>(&self) -> Image<U, C> {
        Image::new(self.meta.width, self.meta.height)
    }

    pub fn new_like_with_color<D: Color>(&self) -> Image<T, D> {
        Image::new(self.meta.width, self.meta.height)
    }

    pub fn type_max(&self) -> f64 {
        T::MAX
    }

    pub fn type_min(&self) -> f64 {
        T::MIN
    }

    /// Returns the number of channels
    #[inline]
    pub fn channels(&self) -> usize {
        C::CHANNELS
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.meta.width
    }

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

    #[inline]
    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.meta.width && y < self.meta.height
    }

    /// Get a normalized pixel from an image, reusing an existing `Pixel`
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

    /// Convert to `ImageBuf`
    pub fn to_image_buf(&mut self) -> ImageBuf {
        ImageBuf::new_with_data(
            self.meta.width,
            self.meta.height,
            self.channels(),
            self.data.as_mut_slice(),
        )
    }

    /// Open an image from disk
    pub fn open(path: impl AsRef<std::path::Path>) -> Option<Image<T, C>> {
        ImageBuf::read_to_image(path, 0, 0)
    }

    /// Open image, specifying a subimage, from disk
    pub fn open_subimage(
        path: impl AsRef<std::path::Path>,
        subimage: usize,
        miplevel: usize,
    ) -> Option<Image<T, C>> {
        ImageBuf::read_to_image(path, subimage, miplevel)
    }

    /// Save an image to disk
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> bool {
        ImageBuf::write_image(self, path)
    }

    /// Iterate over part of an image in parallel
    pub fn for_each_rect<F: Sync + Send + Fn((usize, usize), &mut [T])>(
        &mut self,
        x_: usize,
        width_: usize,
        y_: usize,
        height_: usize,
        f: F,
    ) {
        let (width, _height, channels) = self.shape();
        self.data
            .as_mut_slice()
            .par_chunks_mut(channels)
            .enumerate()
            .for_each(|(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                if x >= x_ && x < x_ + width_ && y >= y_ && y < y_ + height_ {
                    f((x, y), pixel)
                }
            });
    }

    /// Iterate over each pixel in parallel
    pub fn for_each<F: Sync + Send + Fn((usize, usize), &mut [T])>(&mut self, f: F) {
        let (width, _height, channels) = self.shape();
        self.data
            .as_mut_slice()
            .par_chunks_mut(channels)
            .enumerate()
            .for_each(|(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                f((x, y), pixel)
            });
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
    pub fn each_pixel<F: Sync + Send + FnMut((usize, usize), &mut [T])>(&mut self, mut f: F) {
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
}
