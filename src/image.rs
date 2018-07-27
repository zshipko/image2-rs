use std::marker::PhantomData;

use color::Color;
use ty::Type;

use rayon::prelude::*;

#[inline]
fn index(width: usize, channels: usize, x: usize, y: usize) -> usize {
    width * channels * y + x * channels
}

pub trait Image<T: Type, C: Color> {
    fn new(width: usize, height: usize) -> Self;
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

    fn for_each<F: Sync + Fn((usize, usize), Vec<&mut T>)>(&mut self, f: F) {
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
}

pub struct ImageBuf<T: Type, C: Color> {
    width: usize,
    height: usize,
    data: Vec<T>,
    _color: PhantomData<C>,
}

impl<T: Type, C: Color> Image<T, C> for ImageBuf<T, C> {
    fn new(width: usize, height: usize) -> Self {
        ImageBuf {
            width,
            height,
            data: vec![T::zero(); width * height * C::channels()],
            _color: PhantomData,
        }
    }

    fn shape(&self) -> (usize, usize, usize) {
        (self.width, self.height, C::channels())
    }

    fn data(&self) -> &[T] {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut [T] {
        self.data.as_mut()
    }
}

impl<T: Type, C: Color> ImageBuf<T, C> {
    pub fn new_like(&self) -> Self {
        Self::new(self.width, self.height)
    }

    pub fn new_like_with_type<U: Type>(&self) -> ImageBuf<U, C> {
        ImageBuf::new(self.width, self.height)
    }

    pub fn new_like_with_color<D: Color>(&self) -> ImageBuf<T, C> {
        ImageBuf::new(self.width, self.height)
    }

    pub fn new_from(width: usize, height: usize, data: Vec<T>) -> Self {
        ImageBuf {
            width,
            height,
            data,
            _color: PhantomData,
        }
    }
}
