use std::marker::PhantomData;

use crate::*;

use rayon::prelude::*;

/// Image metadata
pub struct Meta<T: Type, C: Color> {
    pub width: usize,
    pub height: usize,
    _type: PhantomData<T>,
    _color: PhantomData<C>,
}

/// Image type
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

    /// Returns the number of channels
    #[inline]
    pub fn channels(&self) -> usize {
        C::CHANNELS
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

    /// Conver to `ImageBuf`
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
