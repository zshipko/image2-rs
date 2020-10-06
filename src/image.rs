use crate::*;

#[cfg(feature = "parallel")]
use rayon::{iter::ParallelIterator, prelude::*};

/// Image type
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image<T: Type, C: Color> {
    /// Metadata
    pub meta: Meta<T, C>,

    /// Pixel data
    pub data: Box<[T]>,
}

impl<X: Into<Point>, T: Type, C: Color> std::ops::Index<X> for Image<T, C> {
    type Output = [T];

    fn index(&self, pt: X) -> &Self::Output {
        let index = self.meta.index(pt);
        &self.data[index..index + self.channels()]
    }
}

impl<X: Into<Point>, T: Type, C: Color> std::ops::IndexMut<X> for Image<T, C> {
    fn index_mut(&mut self, pt: X) -> &mut Self::Output {
        let index = self.meta.index(pt);
        let channels = self.channels();
        &mut self.data[index..index + channels]
    }
}

impl<T: Type, C: Color> Image<T, C> {
    /// Create a new image with the given size and data
    pub unsafe fn new_with_data(size: impl Into<Size>, data: impl Into<Box<[T]>>) -> Image<T, C> {
        Image {
            meta: Meta::new(size),
            data: data.into(),
        }
    }

    /// Create a new image
    pub fn new(size: impl Into<Size>) -> Image<T, C> {
        let size = size.into();
        let data = vec![T::default(); size.width * size.height * C::CHANNELS];
        unsafe { Self::new_with_data(size, data) }
    }

    /// Create a new image with the same size, type and color
    pub fn new_like(&self) -> Image<T, C> {
        Image::new(self.size())
    }

    /// Create a new image with the same size and color as an existing image with the given type
    pub fn new_like_with_type<U: Type>(&self) -> Image<U, C> {
        Image::new(self.size())
    }

    /// Create a new image with the same size and type as an existing image with the given color
    pub fn new_like_with_color<D: Color>(&self) -> Image<T, D> {
        Image::new(self.size())
    }

    /// Create a new image with the same size as an existing image with the given type and color
    pub fn new_like_with_type_and_color<U: Type, D: Color>(&self) -> Image<U, D> {
        Image::new(self.size())
    }

    /// Returns the number of channels
    #[inline]
    pub fn channels(&self) -> Channel {
        C::CHANNELS
    }

    #[inline]
    /// Get image meta
    pub fn meta(&self) -> &Meta<T, C> {
        &self.meta
    }

    /// Image width
    #[inline]
    pub fn width(&self) -> usize {
        self.meta.size.width
    }

    /// Image height
    #[inline]
    pub fn height(&self) -> usize {
        self.meta.size.height
    }

    /// Returns (width, height, channels)
    #[inline]
    pub fn shape(&self) -> (usize, usize, Channel) {
        (self.meta.size.width, self.meta.size.height, self.channels())
    }

    /// Get image size
    #[inline]
    pub fn size(&self) -> Size {
        self.meta.size()
    }

    /// Convert image data into byte vec
    pub fn into_buffer(self) -> Vec<u8> {
        let mut from = self.data.into_vec();
        unsafe {
            let capacity = from.capacity() * std::mem::size_of::<T>();
            let len = from.len() * std::mem::size_of::<T>();
            let ptr = from.as_mut_ptr();
            std::mem::forget(from);
            Vec::from_raw_parts(ptr as *mut u8, len, capacity)
        }
    }

    /// Copy image data into new byte vec
    pub fn to_buffer(&self) -> Vec<u8> {
        let mut from = self.data.to_vec();
        unsafe {
            let capacity = from.capacity() * std::mem::size_of::<T>();
            let len = from.len() * std::mem::size_of::<T>();
            let ptr = from.as_mut_ptr();
            std::mem::forget(from);
            Vec::from_raw_parts(ptr as *mut u8, len, capacity)
        }
    }

    /// Get image data as bytes
    pub fn buffer(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const u8,
                self.data.len() * std::mem::size_of::<T>(),
            )
        }
    }

    /// Get image data as mutable bytes
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_ptr() as *mut u8,
                self.data.len() * std::mem::size_of::<T>(),
            )
        }
    }

    /// Get data at specified index
    #[inline]
    pub fn get(&self, pt: impl Into<Point>) -> Data<T, C> {
        let index = self.meta.index(pt);
        Data::new(&self.data[index..index + self.channels()])
    }

    /// Get mutable data at specified index
    #[inline]
    pub fn get_mut(&mut self, pt: impl Into<Point>) -> DataMut<T, C> {
        let index = self.meta.index(pt);
        let channels = self.channels();
        DataMut::new(&mut self.data[index..index + channels])
    }

    /// Set data to specified location
    #[inline]
    pub fn set(&mut self, pt: impl Into<Point>, data: impl AsRef<[T]>) {
        let mut image = self.get_mut(pt);
        image.as_mut().clone_from_slice(data.as_ref())
    }

    /// Returns true when (x, y) is in bounds for the given image
    #[inline]
    pub fn in_bounds(&self, pt: impl Into<Point>) -> bool {
        self.size().in_bounds(pt)
    }

    /// Get image data from an image, reusing an existing data buffer big enough for a single pixel
    #[inline]
    pub fn at(&self, pt: impl Into<Point>, mut px: impl AsMut<[T]>) -> bool {
        let pt = pt.into();
        let px = px.as_mut();
        if !self.in_bounds(&pt) || px.len() < C::CHANNELS {
            return false;
        }

        px.copy_from_slice(self.get(pt).as_ref());
        true
    }

    /// Load data from and `Image` into an existing `Pixel` structure
    #[inline]
    pub fn pixel_at(&self, pt: impl Into<Point>, px: &mut Pixel<C>) -> bool {
        let pt = pt.into();
        if !self.in_bounds(&pt) {
            return false;
        }
        let data = self.get(pt);
        px.copy_from_slice(data.as_ref());
        true
    }

    /// Get an empty pixel for the image color type
    #[inline]
    pub fn new_pixel(&self) -> Pixel<C> {
        Pixel::new()
    }

    /// Get a normalized pixel from an image
    #[inline]
    pub fn get_pixel(&self, pt: impl Into<Point>) -> Pixel<C> {
        let mut px = Pixel::new();
        self.pixel_at(pt, &mut px);
        px
    }

    /// Set a normalized pixel to the specified location
    #[inline]
    pub fn set_pixel(&mut self, pt: impl Into<Point>, px: &Pixel<C>) {
        let data = self.get_mut(pt);
        px.copy_to_slice(data);
    }

    /// Get a normalized float value
    pub fn get_f(&self, pt: impl Into<Point>, c: Channel) -> f64 {
        let pt = pt.into();
        if !self.in_bounds(&pt) || c >= C::CHANNELS {
            return 0.0;
        }

        let data = self.get(pt);
        data[c].to_norm()
    }

    /// Set normalized float value
    pub fn set_f(&mut self, pt: impl Into<Point>, c: Channel, f: f64) {
        let pt = pt.into();
        if !self.in_bounds(&pt) || c >= C::CHANNELS {
            return;
        }
        let mut data = self.get_mut(pt);
        data[c] = T::from_norm(f);
    }

    /// Get row
    #[inline]
    pub fn row(&self, y: usize) -> Data<T, C> {
        let index = self.meta.index((0, y));
        Data::new(&self.data[index..index + self.channels() * self.width()])
    }

    /// Get mutable row
    #[inline]
    pub fn row_mut(&mut self, y: usize) -> DataMut<T, C> {
        let index = self.meta.index((0, y));
        let len = self.channels() * self.width();
        DataMut::new(&mut self.data[index..index + len])
    }

    /// Iterate over image rows
    pub fn rows(&self) -> impl Iterator<Item = Data<T, C>> {
        self.data.chunks(self.meta.width_step()).map(Data::new)
    }

    /// Iterate over mutable image rows
    pub fn rows_mut(&mut self) -> impl Iterator<Item = DataMut<T, C>> {
        self.data
            .chunks_mut(self.meta.width_step())
            .map(DataMut::new)
    }

    /// Open an image from disk
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Image<T, C>, Error> {
        #[cfg(feature = "oiio")]
        {
            let input = io::Input::open(path)?;
            input.read()
        }

        #[cfg(not(feature = "oiio"))]
        {
            let x = io::magick::read(path)?;
            Ok(x)
        }
    }

    /// Save an image to disk
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        #[cfg(feature = "oiio")]
        {
            let output = io::Output::create(path)?;
            output.write(self)
        }

        #[cfg(not(feature = "oiio"))]
        {
            io::magick::write(path, self)?;
            Ok(())
        }
    }

    /// Iterate over part of an image with mutable data access
    #[cfg(feature = "parallel")]
    pub fn iter_region_mut<'a>(
        &'a mut self,
        roi: Region,
    ) -> impl 'a + rayon::iter::ParallelIterator<Item = (Point, DataMut<T, C>)> {
        let (width, _height, channels) = self.shape();
        self.data
            .par_chunks_mut(channels)
            .map(DataMut::new)
            .enumerate()
            .filter_map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                let pt = Point::new(x, y);
                if roi.in_bounds(&pt) {
                    return Some((pt, pixel));
                }

                None
            })
    }

    /// Iterate over part of an image with mutable data access
    #[cfg(not(feature = "parallel"))]
    pub fn iter_region_mut<'a>(
        &'a mut self,
        roi: Region,
    ) -> impl 'a + std::iter::Iterator<Item = (Point, DataMut<T, C>)> {
        let (width, _height, channels) = self.shape();
        self.data
            .chunks_mut(channels)
            .map(DataMut::new)
            .enumerate()
            .filter_map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                let pt = Point::new(x, y);
                if roi.in_bounds(&pt) {
                    return Some((pt, pixel));
                }

                None
            })
    }

    /// Iterate over part of an image
    #[cfg(feature = "parallel")]
    pub fn iter_region<'a>(
        &'a self,
        roi: Region,
    ) -> impl 'a + rayon::iter::ParallelIterator<Item = (Point, Data<T, C>)> {
        let (width, _height, channels) = self.shape();
        self.data
            .par_chunks(channels)
            .map(Data::new)
            .enumerate()
            .filter_map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                let pt = Point::new(x, y);
                if roi.in_bounds(&pt) {
                    return Some((pt, pixel));
                }

                None
            })
    }

    /// Iterate over part of an image
    #[cfg(not(feature = "parallel"))]
    pub fn iter_region<'a>(
        &'a self,
        roi: Region,
    ) -> impl 'a + std::iter::Iterator<Item = (Point, Data<T, C>)> {
        let (width, _height, channels) = self.shape();
        self.data
            .chunks(channels)
            .map(Data::new)
            .enumerate()
            .filter_map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                let pt = Point::new(x, y);
                if roi.in_bounds(&pt) {
                    return Some((pt, pixel));
                }

                None
            })
    }

    /// Get pixel iterator
    #[cfg(feature = "parallel")]
    pub fn iter<'a>(
        &'a self,
    ) -> impl 'a + rayon::iter::ParallelIterator<Item = (Point, Data<T, C>)> {
        let (width, _height, channels) = self.shape();
        self.data
            .par_chunks(channels)
            .map(Data::new)
            .enumerate()
            .map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                let pt = Point::new(x, y);
                (pt, pixel)
            })
    }

    /// Get pixel iterator
    #[cfg(not(feature = "parallel"))]
    pub fn iter<'a>(&'a self) -> impl 'a + std::iter::Iterator<Item = (Point, Data<T, C>)> {
        let (width, _height, channels) = self.shape();
        self.data
            .chunks(channels)
            .map(Data::new)
            .enumerate()
            .map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                let pt = Point::new(x, y);
                (pt, pixel)
            })
    }

    /// Get mutable pixel iterator
    #[cfg(feature = "parallel")]
    pub fn iter_mut<'a>(
        &'a mut self,
    ) -> impl 'a + rayon::iter::ParallelIterator<Item = (Point, DataMut<T, C>)> {
        let (width, _height, channels) = self.shape();
        self.data
            .par_chunks_mut(channels)
            .map(DataMut::new)
            .enumerate()
            .map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                let pt = Point::new(x, y);
                (pt, pixel)
            })
    }

    /// Get mutable data iterator
    #[cfg(not(feature = "parallel"))]
    pub fn iter_mut<'a>(
        &'a mut self,
    ) -> impl 'a + std::iter::Iterator<Item = (Point, DataMut<T, C>)> {
        let (width, _height, channels) = self.shape();
        self.data
            .chunks_mut(channels)
            .map(DataMut::new)
            .enumerate()
            .map(move |(n, pixel)| {
                let y = n / width;
                let x = n - (y * width);
                let pt = Point::new(x, y);
                (pt, pixel)
            })
    }

    /// Iterate over each pixel applying `f` to every pixel
    pub fn for_each<F: Sync + Send + Fn(Point, DataMut<T, C>)>(&mut self, f: F) {
        self.iter_mut().for_each(|(pt, px)| f(pt, px))
    }

    /// Iterate over a region of pixels qpplying `f` to every pixel
    pub fn for_each_region<F: Sync + Send + Fn(Point, DataMut<T, C>)>(
        &mut self,
        roi: Region,
        f: F,
    ) {
        self.iter_region_mut(roi).for_each(|(pt, px)| f(pt, px))
    }

    /// Iterate over each pixel of two images at once
    #[cfg(feature = "parallel")]
    pub fn for_each2<F: Sync + Send + Fn(Point, DataMut<T, C>, Data<T, C>)>(
        &mut self,
        other: &Image<T, C>,
        f: F,
    ) {
        let (width, _height, channels) = self.shape();
        let b = other.data.par_chunks(channels);
        self.data
            .par_chunks_mut(channels)
            .zip(b)
            .enumerate()
            .for_each(|(n, (pixel, pixel1))| {
                let y = n / width;
                let x = n - (y * width);
                let pt = Point::new(x, y);
                f(pt, DataMut::new(pixel), Data::new(pixel1))
            });
    }

    /// Iterate over each pixel of two images at once
    #[cfg(not(feature = "parallel"))]
    pub fn for_each2<F: Sync + Send + Fn(Point, DataMut<T, C>, Data<T, C>)>(
        &mut self,
        other: &Image<T, C>,
        f: F,
    ) {
        let (width, _height, channels) = self.shape();
        let b = other.data.chunks(channels);
        self.data
            .chunks_mut(channels)
            .zip(b)
            .enumerate()
            .for_each(|(n, (pixel, pixel1))| {
                let y = n / width;
                let x = n - (y * width);
                let pt = Point::new(x, y);
                f(pt, DataMut::new(pixel), Data::new(pixel1))
            });
    }

    /// Iterate over pixels, with a mutable closure
    pub fn each_pixel<F: Sync + Send + FnMut(Point, &Pixel<C>)>(&self, mut f: F) {
        let (width, _height, channels) = self.shape();
        let mut pixel = Pixel::new();

        self.data
            .chunks_exact(channels)
            .enumerate()
            .for_each(|(n, px)| {
                let y = n / width;
                let x = n - (y * width);
                pixel.copy_from_slice(px);
                f(Point::new(x, y), &pixel)
            })
    }

    /// Iterate over pixels in region, with a mutable closure
    pub fn each_pixel_region<F: Sync + Send + FnMut(Point, &Pixel<C>)>(
        &self,
        region: Region,
        mut f: F,
    ) {
        let (width, _height, channels) = self.shape();
        let mut pixel = Pixel::new();

        self.data
            .chunks_exact(channels)
            .enumerate()
            .map(|(n, px)| {
                let y = n / width;
                let x = n - (y * width);
                (Point::new(x, y), px)
            })
            .filter(|(pt, _px)| region.in_bounds(pt))
            .for_each(|(pt, px)| {
                pixel.copy_from_slice(px);
                f(pt, &pixel);
            })
    }

    /// Iterate over mutable pixels, with a mutable closure
    pub fn each_pixel_mut<F: Sync + Send + FnMut(Point, &mut Pixel<C>)>(&mut self, mut f: F) {
        let (width, _height, channels) = self.shape();
        let mut pixel = Pixel::new();

        self.data
            .chunks_exact_mut(channels)
            .enumerate()
            .for_each(|(n, px)| {
                let y = n / width;
                let x = n - (y * width);
                pixel.copy_from_slice(&px);
                f(Point::new(x, y), &mut pixel);
                pixel.copy_to_slice(px);
            });
    }

    /// Iterate over mutable pixels in region, with a mutable closure
    pub fn each_pixel_region_mut<F: Sync + Send + FnMut(Point, &mut Pixel<C>)>(
        &mut self,
        region: Region,
        mut f: F,
    ) {
        let (width, _height, channels) = self.shape();
        let mut pixel = Pixel::new();

        self.data
            .chunks_exact_mut(channels)
            .enumerate()
            .map(|(n, px)| {
                let y = n / width;
                let x = n - (y * width);
                (Point::new(x, y), px)
            })
            .filter(|(pt, _px)| region.in_bounds(pt))
            .for_each(|(pt, px)| {
                pixel.copy_from_slice(&px);
                f(pt, &mut pixel);

                pixel.copy_to_slice(px);
            })
    }

    /// Copy a region of an image to a new image
    pub fn crop(&self, roi: Region) -> Image<T, C> {
        let mut dest = Image::new(roi);
        dest.copy_from_region(
            roi,
            self,
            Region::new(Point::new(0, 0), Size::new(roi.size.width, roi.size.height)),
        );
        dest
    }

    /// Copy into a region from another image starting at the given offset
    pub fn copy_from_region(&mut self, offs: impl Into<Point>, other: &Image<T, C>, roi: Region) {
        let offs = offs.into();
        self.for_each_region(roi, |pt, mut px| {
            px.copy_from_slice(
                other.get((pt.x - roi.point.x + offs.x, pt.y - roi.point.y + offs.y)),
            );
        });
    }

    /// Apply a filter using an Image as output
    pub fn apply(
        &mut self,
        filter: impl Filter,
        input: &[&Image<impl Type, impl Color>],
    ) -> &mut Self {
        filter.eval(input, self);
        self
    }

    /// Apply an async filter using an Image as output
    pub async fn apply_async(
        &mut self,
        mode: filter::AsyncMode,
        filter: impl Filter + Unpin,
        input: &[&Image<impl Type, impl Color>],
    ) -> &mut Self {
        filter::eval_async(&filter, mode, input, self).await;
        self
    }

    /// Run a filter using the same Image as input and output
    pub fn run_in_place(&mut self, filter: impl Filter) -> &mut Self {
        filter.eval_in_place(self);
        self
    }

    /// Run a filter using an Image as input
    pub fn run<U: Type, D: Color>(
        &self,
        filter: impl Filter,
        output: Option<Meta<U, D>>,
    ) -> Image<U, D> {
        let size = if let Some(o) = output {
            o.size
        } else {
            self.size()
        };
        let mut dest = Image::new(size);
        dest.apply(filter, &[self]);
        dest
    }

    /// Run an async filter using an Image as input
    pub async fn run_async<U: Type, D: Color>(
        &self,
        mode: filter::AsyncMode,
        filter: impl Filter + Unpin,
        output: Option<Meta<U, D>>,
    ) -> Image<U, D> {
        let size = if let Some(o) = output {
            o.size
        } else {
            self.size()
        };
        let mut dest = Image::new(size);
        dest.apply_async(mode, filter, &[self]).await;
        dest
    }

    /// Convert image type/color
    pub fn convert<U: Type, D: Color>(&self) -> Image<U, D> {
        self.run(Convert::<D>::new(), None)
    }

    /// Convert to `ImageBuf`
    #[cfg(feature = "oiio")]
    pub(crate) fn image_buf(&mut self) -> io::internal::ImageBuf {
        io::internal::ImageBuf::new_with_data(
            self.width(),
            self.height(),
            self.channels(),
            &mut self.data,
        )
    }

    /// Convert to `ImageBuf`
    #[cfg(feature = "oiio")]
    pub(crate) fn const_image_buf(&self) -> io::internal::ImageBuf {
        io::internal::ImageBuf::const_new_with_data(
            self.width(),
            self.height(),
            self.channels(),
            &self.data,
        )
    }

    /// Convert colorspace from `a` to `b` into an existing image
    #[cfg(feature = "oiio")]
    pub fn convert_colorspace_to(
        &self,
        dest: &mut Image<T, C>,
        a: impl AsRef<str>,
        b: impl AsRef<str>,
    ) -> Result<(), Error> {
        let buf = self.const_image_buf();
        let ok = buf.convert_color(&mut dest.image_buf(), a.as_ref(), b.as_ref());
        if ok {
            Ok(())
        } else {
            Err(Error::FailedColorConversion(
                a.as_ref().into(),
                b.as_ref().into(),
            ))
        }
    }

    /// Convert colorspace from `a` to `b` into a new image
    #[cfg(feature = "oiio")]
    pub fn convert_colorspace(
        &self,
        a: impl AsRef<str>,
        b: impl AsRef<str>,
    ) -> Result<Image<T, C>, Error> {
        let mut dest = self.new_like_with_color();
        self.convert_colorspace_to(&mut dest, a, b)?;
        Ok(dest)
    }

    /// Get image histogram
    pub fn histogram(&self, bins: usize) -> Vec<Histogram> {
        let mut hist = vec![Histogram::new(bins); C::CHANNELS];

        self.each_pixel(|_, px| {
            for i in 0..C::CHANNELS {
                hist[i].add_value(px[i]);
            }
        });

        hist
    }

    /// Gamma correction
    fn gamma(&mut self, value: f64) {
        self.for_each(|_, px| {
            for x in px {
                *x = T::from_f64(T::to_f64(x).powf(value))
            }
        })
    }

    /// Convert to log RGB
    pub fn set_gamma_log(&mut self) {
        self.gamma(1. / 2.2)
    }

    /// Convert to linear RGB
    pub fn set_gamma_lin(&mut self) {
        self.gamma(2.2)
    }

    /// Resize an image
    pub fn resize(&self, size: impl Into<Size>) -> Image<T, C> {
        let size = size.into();
        self.run(transform::resize(self.size(), size), Some(Meta::new(size)))
    }

    /// Scale an image
    pub fn scale(&self, width: f64, height: f64) -> Image<T, C> {
        self.run(
            transform::scale(width, height),
            Some(Meta::new((
                (self.width() as f64 * width) as usize,
                (self.height() as f64 * height) as usize,
            ))),
        )
    }
}
