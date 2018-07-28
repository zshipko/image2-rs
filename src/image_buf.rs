use ty::Type;
use color::Color;
use image::Image;

use std::marker::PhantomData;

#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone)]
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
