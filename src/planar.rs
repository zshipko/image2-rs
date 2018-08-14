use std::marker::PhantomData;

use ::{Type, Color, ImageBuf, ImageRef, Image, Pixel};

#[derive(Debug, Clone, PartialEq)]
pub struct Planar<T: Type, C: Color> {
    pub width: usize,
    pub height: usize,
    pub data: Vec<T>,
    _color: PhantomData<C>,
}

impl<T: Type, C: Color> Planar<T, C> {
    pub fn new(width: usize, height: usize) -> Self {
        Planar {
            width,
            height,
            data: vec![T::zero(); width * height * C::channels()],
            _color: PhantomData
        }
    }

    pub fn new_like(&self) -> Self {
        Self::new(self.width, self.height)
    }

    pub fn new_like_with_color<D: Color>(&self) -> Planar<T, D> {
        Planar::new(self.width, self.height)
    }

    pub fn new_like_with_type<U: Type>(&self) -> Planar<U, C> {
        Planar::new(self.width, self.height)
    }

    fn index(&self, x: usize, y: usize, c: usize) -> usize {
        c * self.width * self.height + y * self.width + x
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Vec<T> {
        let mut dest = Vec::with_capacity(C::channels());
        for i in 0..C::channels() {
            let index = self.index(x, y, i);
            dest.push(self.data[index])
        }
        dest
    }

    pub fn set_pixel<'a, P: Pixel<'a, T>>(&mut self, x: usize, y: usize, px: P) {
        for i in 0..C::channels() {
            let index = self.index(x, y, i);
            self.data[index] = px.as_ref()[i]
        }
    }

    pub fn convert_type<U: Type, I: Image<U, C>>(&self, dest: &mut I) {
        let ddata = dest.data_mut();
        for (i, x) in self.data.iter().enumerate() {
            ddata[i] = x.convert();
        }
    }
}

impl<T: Type, C: Color> From<ImageBuf<T, C>> for Planar<T, C> {
    fn from(buf: ImageBuf<T, C>) -> Planar<T, C> {
        let (width, height, channels) = buf.shape();
        let mut planar: Planar<T, C> = Planar::new(width, height);
        image2_for_each!(buf, x, y, px, {
            for i in 0..channels {
                let index = planar.index(x, y, i);
                planar.data[index] = px[i]
            }
        });
        planar
    }
}

impl<'a, T: Type, C: Color> From<ImageRef<'a, T, C>> for Planar<T, C> {
    fn from(buf: ImageRef<T, C>) -> Planar<T, C> {
        let (width, height, channels) = buf.shape();
        let mut planar: Planar<T, C> = Planar::new(width, height);
        image2_for_each!(buf, x, y, px, {
            for i in 0..channels {
                let index = planar.index(x, y, i);
                planar.data[index] = px[i]
            }
        });
        planar
    }
}

impl<T: Type, C: Color> From<Planar<T, C>> for ImageBuf<T, C> {
    fn from(planar: Planar<T, C>) -> ImageBuf<T, C> {
        let mut buf = ImageBuf::new(planar.width, planar.height);
        buf.for_each(|(x, y), mut px| {
            for i in 0..C::channels() {
                let index = planar.index(x, y, i);
                *px[i] = planar.data[index]
            }
        });
        buf
    }
}
