use crate::color::Color;
use crate::image::Image;
use crate::ty::Type;

use std::marker::PhantomData;

/// Image implementation backed by a mutable array reference
#[derive(Debug, PartialEq)]
pub struct ImageRef<'a, T: 'a + Type, C: Color> {
    width: usize,
    height: usize,
    data: &'a mut [T],
    _color: PhantomData<C>,
}

impl<'a, T: Type, C: Color> Image<T, C> for ImageRef<'a, T, C> {
    fn shape(&self) -> (usize, usize, usize) {
        (self.width, self.height, C::channels())
    }

    fn data(&self) -> &[T] {
        self.data
    }

    fn data_mut(&mut self) -> &mut [T] {
        self.data
    }
}

impl<'a, T: 'a + Type, C: Color> ImageRef<'a, T, C> {
    /// Create a new ImageRef with the given dimensions and data
    ///
    /// Note: this functions does not do any bounds checking regarding the specified size and
    /// length of the data
    pub fn new(width: usize, height: usize, data: &'a mut [T]) -> Self {
        ImageRef {
            width,
            height,
            data,
            _color: PhantomData,
        }
    }

    pub fn inner(self) -> &'a mut [T] {
        self.data
    }
}
