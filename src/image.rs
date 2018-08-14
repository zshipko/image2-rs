use color::Color;
use ty::Type;
use pixel::{Pixel, PixelVec, PixelMut};
use image_ref::ImageRef;
use image_buf::ImageBuf;

use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Layout {
    Planar,
    Interleaved
}

impl Default for Layout {
    fn default() -> Layout {
        Layout::Interleaved
    }
}

#[macro_export]
macro_rules! image2_for_each_at {
    ($image:expr, $i:ident, $j:ident, $px:ident, $body:block) => {
        for $j in 0..$image.height() {
            for $i in 0..$image.width() {
                let $px = $image.at($i, $j);
                $body
            }
        }
    }
}

#[macro_export]
macro_rules! image2_for_each_get {
    ($image:expr, $i:ident, $j:ident, $px:ident, $body:block) => {
        let mut $px = $image.empty_pixel();
        for $j in 0..$image.height() {
            for $i in 0..$image.width() {
                $image.get_pixel($i, $j, &mut $px);
                $body
            }
        }
    }
}


#[inline]
pub fn index(layout: &Layout, width: usize, height: usize, channels: usize, x: usize, y: usize, c: usize) -> usize {
    match layout {
        Layout::Planar => width * height * c + width * y + x,
        Layout::Interleaved => width * channels * y + channels * x + c
    }
}

pub trait Image<T: Type, C: Color>: Sync + Send {
    fn shape(&self) -> (usize, usize, usize);
    fn layout(&self) -> &Layout;
    fn set_layout(&mut self, layout: Layout);

    fn data(&self) -> &[T];
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

    fn index(&self, x: usize, y: usize, c: usize) -> usize {
        let (width, height, channels) = self.shape();
        index(self.layout(), width, height, channels, x, y, c)
    }

    fn empty_pixel(&self) -> Vec<T> {
        vec![T::zero(); C::channels()]
    }

    fn at(&mut self, x: usize, y: usize) -> Vec<&mut T> {
        let mut px = Vec::with_capacity(C::channels());
        let (width, height, channels) = self.shape();
        let layout = self.layout().clone();
        let data = self.data_mut().as_mut_ptr();
        for i in 0..C::channels() {
            let index = index(&layout, width, height, channels, x, y, i);
            unsafe {
                px.push(&mut *data.offset(index as isize))
            }
        }
        px
    }

    fn get_pixel<'a, P: PixelMut<'a, T>>(&self, x: usize, y: usize, px: &mut P) {
        for i in 0..C::channels() {
            let index = self.index(x, y, i);
            px.as_mut()[i] = self.data()[index]
        }
    }

    fn set_pixel<'a, P: Pixel<'a, T>>(&mut self, x: usize, y: usize, px: &P) {
        for i in 0..C::channels() {
            let index = self.index(x, y, i);
            self.data_mut()[index] = px.as_ref()[i]
        }
    }

    fn get(&self, x: usize, y: usize, c: usize) -> f64 {
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

    fn set(&mut self, x: usize, y: usize, c: usize, f: f64) {
        let (width, height, channels) = self.shape();
        if x >= width || y >= height || c >= channels {
            return;
        }
        let index = self.index(x, y, c);
        match T::from(T::denormalize(f)) {
            Some(f) => self.data_mut()[index] = f,
            None => (),
        }
    }

    fn convert_type<U: Type, I: Image<U, C>>(&self, dest: &mut I) {
        let ddata = dest.data_mut();
        for (i, x) in self.data().iter().enumerate() {
            ddata[i] = x.convert();
        }
    }

    fn convert_layout(&mut self, layout: Layout) {
        if self.layout() == &layout {
            return
        }

        let mut buf: ImageBuf<T, C> = ImageBuf::new_with_layout(self.width(), self.height(), layout);
        buf.for_each(|(x, y), mut px| {
            for i in 0..C::channels() {
                let index = self.index(x, y, i);
                *px[i] = self.data()[index]
            }
        });
        self.data_mut().copy_from_slice(buf.data());
        self.set_layout(layout);
    }

    fn as_image_ref(&mut self) -> ImageRef<T, C> {
        ImageRef::new(self.width(), self.height(), self.layout().clone(), self.data_mut().as_mut())
    }

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
            },
            Layout::Planar => {
                image2_for_each_at!(self, x, y, px, {
                    f((x, y), px)
                });
            }
        }
    }

    fn mean_stddev(&self) -> (Vec<f64>, Vec<f64>) {
        let mut mean = PixelVec::empty();
        let mut variance = PixelVec::empty();

        image2_for_each_get!(self, i, j, px, {
            let v = px.to_pixel_vec();
            mean += v.clone();
            variance += &v * &v;
        });

        mean = mean.map(|x| x / (self.width() * self.height()) as f64);
        variance = variance.map(|x| (x / (self.width() * self.height()) as f64));
        variance -= &mean * &mean;

        (mean.to_vec::<C>(), variance.to_vec::<C>())
    }
}

