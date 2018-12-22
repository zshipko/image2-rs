use std::marker::PhantomData;

use crate::{Color, Image, Type};

#[derive(Debug, PartialEq)]
pub struct ImagePtr<'a, T: 'a + Type, C: Color> {
    width: usize,
    height: usize,
    data: &'a mut [T],
    _color: PhantomData<C>,
    free: fn(*mut T),
}

impl<'a, T: Type, C: Color> Image<T, C> for ImagePtr<'a, T, C> {
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

extern "C" {
    fn free(ptr: *mut std::ffi::c_void);
}

fn default_free<T>(ptr: *mut T) {
    unsafe {
        free(ptr as *mut std::ffi::c_void);
    }
}

impl<'a, T: 'a + Type, C: Color> ImagePtr<'a, T, C> {
    pub fn new(width: usize, height: usize, data: *mut T, free: Option<fn(*mut T)>) -> Self {
        let data = unsafe { std::slice::from_raw_parts_mut(data, width * height * C::channels()) };

        ImagePtr {
            width,
            height,
            data,
            _color: PhantomData,
            free: free.unwrap_or(default_free),
        }
    }

    pub fn inner(self) -> &'a mut [T] {
        self.data
    }
}
