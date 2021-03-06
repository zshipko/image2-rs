use crate::*;

/// Filter input
#[derive(Clone)]
pub struct Input<'a, T: 'a + Type, C: 'a + Color> {
    /// Input images
    pub images: Vec<&'a Image<T, C>>,

    /// Input pixel
    pub pixel: Option<(Point, Pixel<C>)>,
}

impl<'a, T: 'a + Type, C: 'a + Color> Input<'a, T, C> {
    /// Create new `Input`
    pub fn new(images: &'a [&'a Image<T, C>]) -> Self {
        Input {
            images: images.to_vec(),
            pixel: None,
        }
    }

    /// Add chained pixel data
    pub fn with_pixel(mut self, point: Point, pixel: Pixel<C>) -> Self {
        self.pixel = Some((point, pixel));
        self
    }

    /// Remove chained pixel data
    pub fn without_pixel(mut self) -> Self {
        self.pixel = None;
        self
    }

    /// Returns optional pixel value
    pub fn pixel(&self) -> Option<&(Point, Pixel<C>)> {
        self.pixel.as_ref()
    }

    /// Get number of images
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Returns true when there are no inputs
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get input images
    pub fn images(&self) -> &[&Image<T, C>] {
        &self.images
    }

    /// Get input pixel at `pt` - if `pt` matches the stored pixel from a preview computation then
    /// that pixel will be returned instead of the actual input pixel. If `image_index` is not
    /// `None` then input from the image with that index will be used.
    pub fn get_pixel(&self, pt: impl Into<Point>, image_index: Option<usize>) -> Pixel<C> {
        let pt = pt.into();

        match (image_index, &self.pixel) {
            (None, Some((point, data))) if point.eq(&pt) => data.clone(),
            _ => self.images[image_index.unwrap_or_default()].get_pixel(pt),
        }
    }

    /// Get input float value - if `pt` matches the stored pixel from a preview computation then
    /// that pixel will be returned instead of the actual input pixel. If `image_index` is not
    /// `None` then input from the image with that index will be used.
    pub fn get_f(&self, pt: impl Into<Point>, c: Channel, image_index: Option<usize>) -> f64 {
        let pt = pt.into();

        match (image_index, &self.pixel) {
            (None, Some((point, data))) if point.eq(&pt) => data[c],
            _ => self.images[image_index.unwrap_or_default()].get_f(pt, c),
        }
    }

    /// Create a new pixel
    pub fn new_pixel(&self) -> Pixel<C> {
        Pixel::new()
    }
}
