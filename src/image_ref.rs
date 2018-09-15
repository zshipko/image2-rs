use color::Color;
use image::{Image, Layout};
use ty::Type;

use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub struct ImageRef<'a, T: 'a + Type, C: Color> {
    width: usize,
    height: usize,
    data: &'a mut [T],
    layout: Layout,
    _color: PhantomData<C>,
}

impl<'a, T: Type, C: Color> Image<T, C> for ImageRef<'a, T, C> {
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

impl<'a, T: 'a + Type, C: Color> ImageRef<'a, T, C> {
    pub fn new(width: usize, height: usize, layout: Layout, data: &'a mut [T]) -> Self {
        ImageRef {
            width,
            height,
            data,
            layout,
            _color: PhantomData,
        }
    }

    pub fn inner(self) -> &'a mut [T] {
        self.data
    }
}
