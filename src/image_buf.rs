use crate::color::Color;
use crate::image::Image;
use crate::ty::Type;

use std::marker::PhantomData;

/// Image implementation using `Vec<T>` to store data
#[cfg_attr(
    feature = "ser",
    derive(serde_derive::Serialize, serde_derive::Deserialize)
)]
#[derive(Debug, PartialEq, Clone)]
pub struct ImageBuf<T: Type, C: Color> {
    width: usize,
    height: usize,
    data: Vec<T>,
    _color: PhantomData<C>,
}

impl<T: Type, C: Color> Image<T, C> for ImageBuf<T, C> {
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
    /// Create a new ImageBuf with the given size
    pub fn new(width: usize, height: usize) -> Self {
        ImageBuf {
            width,
            height,
            data: vec![T::zero(); width * height * C::channels()],
            _color: PhantomData,
        }
    }

    /// Convert the ImageBuf back to the underlying data buffer
    pub fn inner(self) -> Vec<T> {
        self.data
    }

    /// Create a new image with the same type, shape and color as an existing image
    pub fn new_like(&self) -> Self {
        Self::new(self.width, self.height)
    }

    /// Create a new image with the given type and the same shape and color
    pub fn new_like_with_type<U: Type>(&self) -> ImageBuf<U, C> {
        ImageBuf::new(self.width, self.height)
    }

    /// Create a new image with the given color and the same shape and type
    pub fn new_like_with_color<D: Color>(&self) -> ImageBuf<T, D> {
        ImageBuf::new(self.width, self.height)
    }

    /// Create a new image from existing data
    ///
    /// Note: This function does not do bounds checking, so you need to ensure that `data` is the
    /// correct length to handle the specified width and height
    pub fn new_from(width: usize, height: usize, data: Vec<T>) -> Self {
        ImageBuf {
            width,
            height,
            data,
            _color: PhantomData,
        }
    }
}
