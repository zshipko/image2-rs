use crate::*;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use std::marker::PhantomData;

/// Image metadata
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meta<T: Type, C: Color> {
    /// Image size
    pub size: Size,
    _type: PhantomData<T>,
    _color: PhantomData<C>,
}

impl<T: Type, C: Color> Meta<T, C> {
    /// Create a new `Meta`
    pub fn new(size: impl Into<Size>) -> Meta<T, C> {
        Meta {
            size: size.into(),
            _type: PhantomData,
            _color: PhantomData,
        }
    }

    /// Returns the size of a row
    #[inline]
    pub fn width_step(&self) -> usize {
        self.size.width * C::CHANNELS
    }

    /// Number of pixels
    #[inline]
    pub fn num_pixels(&self) -> usize {
        self.size.width * self.size.height
    }

    /// Number of items
    #[inline]
    pub fn num_values(&self) -> usize {
        self.size.width * self.size.height * C::CHANNELS
    }

    /// Number of bytes
    #[inline]
    pub fn num_bytes(&self) -> usize {
        self.size.width * self.size.height * C::CHANNELS * std::mem::size_of::<T>()
    }

    /// Returns true when the configured color has an alpha channel
    #[inline]
    pub fn has_alpha(&self) -> bool {
        C::ALPHA.is_some()
    }

    /// Returns true when the provided index matches the alpha channel index
    #[inline]
    pub fn is_alpha_channel(&self, c: Channel) -> bool {
        C::ALPHA == Some(c)
    }

    /// Get color name
    #[inline]
    pub fn color_name(&self) -> &str {
        C::NAME
    }

    /// Get type name
    #[inline]
    pub fn type_name(&self) -> &str {
        T::type_name()
    }

    /// Image size
    #[inline]
    pub fn size(&self) -> Size {
        self.size
    }

    /// Image width
    #[inline]
    pub fn width(&self) -> usize {
        self.size.width
    }

    /// Image height
    #[inline]
    pub fn height(&self) -> usize {
        self.size.height
    }

    /// Maximum value for image type
    #[inline]
    pub fn type_max(&self) -> f64 {
        T::MAX
    }

    /// Minimum value for image type
    #[inline]
    pub fn type_min(&self) -> f64 {
        T::MIN
    }

    /// Get the index of the specified pixel
    #[inline]
    pub fn index(&self, pt: impl Into<Point>) -> usize {
        let pt = pt.into();
        self.width_step() * pt.y + pt.x * C::CHANNELS
    }

    /// Get an empty pixel for the image color type
    #[inline]
    pub fn new_pixel(&self) -> Pixel<C> {
        Pixel::new()
    }

    /// Convert from index to Point
    pub fn convert_index_to_point(&self, n: usize) -> Point {
        let width = self.size.width;
        let y = n / width / C::CHANNELS;
        let x = (n - (y * width * C::CHANNELS)) / C::CHANNELS;
        Point::new(x, y)
    }

    /// Get pixel iterator
    #[cfg(not(feature = "parallel"))]
    pub fn iter(&self) -> impl '_ + std::iter::Iterator<Item = Point> {
        (0..self.num_pixels())
            .into_iter()
            .map(move |n| self.convert_index_to_point(n))
    }

    /// Get pixel iterator
    #[cfg(feature = "parallel")]
    pub fn iter(&self) -> impl '_ + rayon::iter::ParallelIterator<Item = Point> {
        (0..self.num_pixels())
            .into_par_iter()
            .map(move |n| self.convert_index_to_point(n))
    }
}
