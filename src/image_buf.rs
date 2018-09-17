use color::Color;
use image::{Image, Layout};
use ty::Type;

use std::marker::PhantomData;

/// Image implementation that uses a Vec for image data
#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub struct ImageBuf<T: Type, C: Color> {
    width: usize,
    height: usize,
    data: Vec<T>,
    layout: Layout,
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

    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn set_layout(&mut self, layout: Layout) {
        self.layout = layout
    }
}

impl<T: Type, C: Color> ImageBuf<T, C> {
    /// Create a new ImageBuf with the given size and layout
    pub fn new_with_layout(width: usize, height: usize, layout: Layout) -> Self {
        ImageBuf {
            width,
            height,
            data: vec![T::zero(); width * height * C::channels()],
            layout: layout,
            _color: PhantomData,
        }
    }

    /// Create a new ImageBuf with the default layout
    pub fn new(width: usize, height: usize) -> Self {
        Self::new_with_layout(width, height, Layout::default())
    }

    /// Convert the ImageBuf back to the underlying data buffer
    pub fn inner(self) -> Vec<T> {
        self.data
    }

    /// Create a new image with the same type, shape and layout as an existing image
    pub fn new_like(&self) -> Self {
        Self::new_with_layout(self.width, self.height, self.layout)
    }

    pub fn new_like_with_type<U: Type>(&self) -> ImageBuf<U, C> {
        ImageBuf::new_with_layout(self.width, self.height, self.layout)
    }

    pub fn new_like_with_color<D: Color>(&self) -> ImageBuf<T, D> {
        ImageBuf::new_with_layout(self.width, self.height, self.layout)
    }

    pub fn new_from(width: usize, height: usize, layout: Layout, data: Vec<T>) -> Self {
        ImageBuf {
            width,
            height,
            data,
            layout,
            _color: PhantomData,
        }
    }
}
