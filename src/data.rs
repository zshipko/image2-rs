#![allow(missing_docs)]

use crate::*;

/// Wraps slices, tagging them with a Color type
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Data<'a, T: 'a + Type, C: 'a + Color>(&'a [T], std::marker::PhantomData<C>);

/// Wraps mutable slices, tagging them with a Color type
#[derive(Debug, PartialEq, PartialOrd)]
pub struct DataMut<'a, T: 'a + Type, C: 'a + Color>(&'a mut [T], std::marker::PhantomData<C>);

impl<'a, T: Type, C: Color> Data<'a, T, C> {
    #[inline]
    pub(crate) fn new(data: &'a [T]) -> Self {
        Data(data, std::marker::PhantomData)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn num_pixels(&self) -> usize {
        self.len() / C::CHANNELS
    }

    pub fn channels(&self) -> usize {
        C::CHANNELS
    }
}

impl<'a, T: Type, C: Color> DataMut<'a, T, C> {
    #[inline]
    pub(crate) fn new(data: &'a mut [T]) -> Self {
        DataMut(data, std::marker::PhantomData)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn num_pixels(&self) -> usize {
        self.len() / C::CHANNELS
    }

    pub fn copy_from_slice(&mut self, slice: impl AsRef<[T]>) {
        self.0.copy_from_slice(slice.as_ref())
    }

    pub fn channels(&self) -> usize {
        C::CHANNELS
    }
}

impl<'a, T: Type, C: Color> AsRef<[T]> for Data<'a, T, C> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<'a, T: Type, C: Color> AsRef<[T]> for DataMut<'a, T, C> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<'a, T: Type, C: Color> AsMut<[T]> for DataMut<'a, T, C> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<'a, T: Type, C: Color> std::ops::Index<usize> for Data<'a, T, C> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        &self.0[i]
    }
}

impl<'a, T: Type, C: Color> std::ops::Index<usize> for DataMut<'a, T, C> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        &self.0[i]
    }
}

impl<'a, T: Type, C: Color> std::ops::IndexMut<usize> for DataMut<'a, T, C> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        &mut self.0[i]
    }
}

impl<'a, T: 'a + Type, C: 'a + Color> IntoIterator for Data<'a, T, C> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> std::slice::Iter<'a, T> {
        self.0.into_iter()
    }
}

impl<'a, T: 'a + Type, C: 'a + Color> IntoIterator for DataMut<'a, T, C> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> std::slice::IterMut<'a, T> {
        self.0.into_iter()
    }
}
