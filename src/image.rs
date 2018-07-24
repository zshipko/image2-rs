use std::marker::PhantomData;

use ty::Type;
use color::Color;

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

    fn at(&mut self, x: usize, y: usize) -> &mut [T] {
        let (width, _, channels) = self.shape();
        let index = index (width, channels, x, y);
        &mut self.data_mut()[index..index + channels]
    }

    fn get(&self, x: usize, y: usize, c: usize) -> f64 {
        let (width, height, channels) = self.shape();
        if x >= width || y >= height || c >= channels {
            return 0.0
        }
        let index = index (width, channels, x, y);
        match self.data()[index + c].to_f64() {
            Some(f) => T::normalize(f),
            None => 0.0
        }
    }

    fn set(&mut self, x: usize, y: usize, c: usize, f: f64) {
        let (width, height, channels) = self.shape();
        if x >= width || y >= height || c >= channels {
            return
        }
        match T::from(T::denormalize(f)) {
            Some(f) => self.at(x, y)[c] = f,
            None => ()
        }
    }

    fn convert_type<U: Type, I: Image<U, C>>(&self, dest: &mut I) {
        let ddata = dest.data_mut();
        for (i, x) in self.data().iter().enumerate() {
            ddata[i] = x.convert();
        }
    }
}

pub struct ImageBuffer<T: Type, C: Color> {
    width: usize,
    height: usize,
    data: Vec<T>,
    _color: PhantomData<C>
}

impl <T: Type, C: Color> Image<T, C> for ImageBuffer<T, C> {
    fn new(width: usize, height: usize) -> Self {
        ImageBuffer {
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

impl<T: Type, C: Color> ImageBuffer<T, C> {
    pub fn new_like(&self) -> Self {
        Self::new(self.width, self.height)
    }

    pub fn new_like_with_type<U: Type>(&self) -> ImageBuffer<U, C> {
        ImageBuffer::new(self.width, self.height)
    }

    pub fn new_like_with_color<D: Color>(&self) -> ImageBuffer<T, C> {
        ImageBuffer::new(self.width, self.height)
    }

    pub fn new_from(width: usize, height: usize, data: Vec<T>) -> Self {
        ImageBuffer {
            width,
            height,
            data,
            _color: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use ::{Image,ImageBuffer};
    use ::color::{Gray, Rgb};
    use ::filter::{Filter, Invert, ToGrayscale};
    use ::io::magick;
    use ::kernel::Kernel;
    use test::Bencher;

    #[test]
    fn test_image_buffer_new() {
        let mut image: ImageBuffer<u8, Rgb> = Image::new(1000, 1000);
        let mut dest = image.new_like();
        image.set(3, 15, 0, 1.);
        assert_eq!(image.get(3, 15, 0), 1.);
        Invert.eval(&mut dest, &[&image]);
    }

    #[bench]
    fn test_magick_read(b: &mut Bencher) {
        let image: ImageBuffer<f32, Rgb> = magick::DEFAULT.read("/home/zach/Downloads/zach.jpg").unwrap();
        let mut dest = image.new_like();
        b.iter (|| ToGrayscale.eval(&mut dest, &[&image]));
        magick::DEFAULT.write("test0.jpg", dest).unwrap();
    }

    #[bench]
    fn bench_invert(b: &mut Bencher) {
        let image: ImageBuffer<f32, Rgb> = magick::DEFAULT.read("/home/zach/Downloads/zach.jpg").unwrap();
        let mut dest = image.new_like();
        b.iter(|| Invert.eval(&mut dest, &[&image]));
        magick::DEFAULT.write("test1.jpg", dest).unwrap();
    }

    #[bench]
    fn bench_kernel(b: &mut Bencher) {
        let image: ImageBuffer<f32, Gray> = magick::DEFAULT.read("/home/zach/Downloads/zach.jpg").unwrap();
        let mut dest = image.new_like();
        let k = Kernel::from([
            [-1.0, -1.0, -1.0],
            [-1.0, 8.0, -1.0],
            [-1.0, -1.0, -1.0],
        ]);

        b.iter(|| k.eval(&mut dest, &[&image]));
        magick::DEFAULT.write("test2.jpg", dest).unwrap();
    }
}
