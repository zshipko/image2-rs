use crate::*;

/// Wraps image data slices, tagging them with a Color type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Data<'a, T: 'a + Type, C: 'a + Color>(&'a [T], std::marker::PhantomData<C>);

/// Wraps mutable image data slices, tagging them with a Color type
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct DataMut<'a, T: 'a + Type, C: 'a + Color>(&'a mut [T], std::marker::PhantomData<C>);

impl<'a, T: Type, C: Color> Data<'a, T, C> {
    #[inline]
    pub(crate) fn new(data: &'a [T]) -> Self {
        Data(data, std::marker::PhantomData)
    }

    /// Number of elements
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true when the inner slice is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of pixels available
    #[inline]
    pub fn num_pixels(&self) -> usize {
        self.len() / C::CHANNELS
    }

    /// Get the number of channels
    #[inline]
    pub fn channels(&self) -> usize {
        C::CHANNELS
    }

    /// Get information about data
    pub fn meta(&self) -> Meta<T, C> {
        Meta::new((self.num_pixels(), 1))
    }

    /// Convert to pixel
    pub fn to_pixel(&self) -> Pixel<C> {
        Pixel::from_slice(self)
    }

    /// Get inner slice
    pub fn as_slice(&self) -> &[T] {
        self.0
    }
}

impl<'a, T: Type, C: Color> DataMut<'a, T, C> {
    #[inline]
    pub(crate) fn new(data: &'a mut [T]) -> Self {
        DataMut(data, std::marker::PhantomData)
    }

    /// Number of elements
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true when the inner slice is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of pixels availible
    #[inline]
    pub fn num_pixels(&self) -> usize {
        self.len() / C::CHANNELS
    }

    /// Get the number of channels
    #[inline]
    pub fn channels(&self) -> usize {
        C::CHANNELS
    }

    /// Copy values from slice
    #[inline]
    pub fn copy_from_slice(&mut self, slice: impl AsRef<[T]>) {
        self.0.copy_from_slice(slice.as_ref())
    }

    /// Get information about data
    pub fn meta(&self) -> Meta<T, C> {
        Meta::new((self.num_pixels(), 1))
    }

    /// Convert to pixel
    pub fn to_pixel(&self) -> Pixel<C> {
        Pixel::from_slice(self)
    }

    /// Get inner slice
    pub fn as_slice(&self) -> &[T] {
        self.0
    }

    /// Get mutable inner slice
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        self.0
    }

    /// Downcast to `Data` type
    pub fn as_data(&'a self) -> Data<'a, T, C> {
        Data::new(self.0)
    }
}

impl<'a, T: Type, C: Color> AsRef<[T]> for Data<'a, T, C> {
    fn as_ref(&self) -> &[T] {
        self.0
    }
}

impl<'a, T: Type, C: Color> AsRef<[T]> for DataMut<'a, T, C> {
    fn as_ref(&self) -> &[T] {
        self.0
    }
}

impl<'a, T: Type, C: Color> AsMut<[T]> for DataMut<'a, T, C> {
    fn as_mut(&mut self) -> &mut [T] {
        self.0
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
        self.0.iter()
    }
}

impl<'a, T: 'a + Type, C: 'a + Color> IntoIterator for DataMut<'a, T, C> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> std::slice::IterMut<'a, T> {
        self.0.iter_mut()
    }
}
