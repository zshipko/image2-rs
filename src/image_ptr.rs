use std::marker::PhantomData;

use crate::{Color, Image, Type};

/// Image implementation backed by a raw pointer, typically used for storing C pointers allocated using
/// malloc.
#[derive(Debug, PartialEq)]
pub struct ImagePtr<'a, T: 'a + Type, C: Color> {
    width: usize,
    height: usize,
    data: &'a mut [T],
    _color: PhantomData<C>,
    free: fn(*mut T, usize),
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
    pub(crate) fn free(ptr: *mut std::ffi::c_void);
}

fn default_free<T>(ptr: *mut T, _: usize) {
    unsafe {
        free(ptr as *mut std::ffi::c_void);
    }
}

fn ignore_free<T>(_: *mut T, _: usize) {}

/// Determines how to free a pointer stored in an ImagePtr
pub enum Free<T> {
    /// Default uses the system defined `free` functions
    Default,
    /// Ignore does nothing
    Ignore,
    /// Function allows for a custom function to be specified
    Function(fn(*mut T, usize)),
}

impl<'a, T: 'a + Type, C: Color> ImagePtr<'a, T, C> {
    /// Create a new ImagePtr with the given `free` function used when the image is dropped, if
    /// no free function is provided then `free` from the C stdlib will be used
    pub fn new(width: usize, height: usize, data: *mut T, free: Free<T>) -> Self {
        let data = unsafe { std::slice::from_raw_parts_mut(data, width * height * C::channels()) };

        let free = match free {
            Free::Default => default_free,
            Free::Ignore => ignore_free,
            Free::Function(f) => f,
        };

        ImagePtr {
            width,
            height,
            data,
            free,
            _color: PhantomData,
        }
    }
}

impl<'a, T: Type, C: Color> Drop for ImagePtr<'a, T, C> {
    fn drop(&mut self) {
        let f = self.free;
        f(self.data.as_mut_ptr(), self.total_bytes())
    }
}
