use crate::*;

#[cfg(feature = "parallel")]
use rayon::{iter::ParallelIterator, prelude::*};

/// Image type
pub struct Image<T: Type, C: Color> {
    /// Metadata
    pub meta: Meta<T, C>,

    /// Pixel data
    pub data: Box<dyn ImageData<T>>,
}

impl<T: Type, C: Color> PartialEq for Image<T, C> {
    fn eq(&self, other: &Self) -> bool {
        self.meta == other.meta && self.data.as_ref().as_ref() == other.data.as_ref().as_ref()
    }
}

impl<T: Type, C: Color> Clone for Image<T, C> {
    fn clone(&self) -> Self {
        Image {
            meta: self.meta.clone(),
            data: Box::new(self.data.data().to_vec().into_boxed_slice()),
        }
    }
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
    /// Create a new image with the given size and data, returns `Err` if the provided `ImageData` isn't big enough
    /// for the specified dimensions
    pub fn new_with_data(
        size: impl Into<Size>,
        data: impl 'static + ImageData<T>,
    ) -> Result<Image<T, C>, Error> {
        let meta = Meta::new(size);
        if data.as_ref().len() < meta.num_values() {
            return Err(Error::InvalidDimensions(
                meta.width(),
                meta.height(),
                C::CHANNELS,
            ));
        }
        Ok(Image {
            meta,
            data: Box::new(data),
        })
    }

    /// Create a new image
    pub fn new(size: impl Into<Size>) -> Image<T, C> {
        let size = size.into();
        let data = vec![T::default(); size.width * size.height * C::CHANNELS];
        Image {
            meta: Meta::new(size),
            data: Box::new(data.into_boxed_slice()),
        }
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

    #[cfg(feature = "mmap")]
    /// New memory mapped image - if `meta` is None then it is assumed the image already exists on disk
    /// otherwise it will be created
    pub fn new_mmap(
        filename: impl AsRef<std::path::Path>,
        meta: Option<Meta<T, C>>,
    ) -> Result<Image<T, C>, Error> {
        match meta {
            Some(meta) => Mmap::create_image(filename, &meta),
            None => Mmap::load_image(filename),
        }
    }

    #[cfg(feature = "mmap")]
    /// Map an existing image to disk, this consumes the original and returns the memory mapped
    /// image
    pub fn mmap(mut self, filename: impl AsRef<std::path::Path>) -> Result<Image<T, C>, Error> {
        let mut data = Mmap::create(filename, &self.meta)?;
        data.data_mut().copy_from_slice(self.data.data());
        self.data = Box::new(data);
        Ok(self)
    }

    /// Returns the number of channels
    #[inline]
    pub fn channels(&self) -> Channel {
        C::CHANNELS
    }

    #[inline]
    /// Get image meta
    pub fn meta(&self) -> Meta<T, C> {
        self.meta.clone()
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

    /// Update the colorspace associated with an image without performing any conversion
    pub fn with_color<D: Color>(self) -> Image<T, D> {
        assert!(C::CHANNELS == D::CHANNELS);
        Image {
            meta: Meta::new(self.meta.size),
            data: self.data,
        }
    }

    /// Get image data as bytes
    pub fn buffer(&self) -> &[u8] {
        self.data.buffer()
    }

    /// Get image data as mutable bytes
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        self.data.buffer_mut()
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
        let pt = pt.into();
        pt.x < self.width() && pt.y < self.height()
    }

    /// Get image data from an image, reusing an existing data buffer big enough for a single pixel
    #[inline]
    pub fn at(&self, pt: impl Into<Point>, mut px: impl AsMut<[T]>) -> bool {
        let pt = pt.into();
        let px = px.as_mut();
        if !self.in_bounds(pt) || px.len() < C::CHANNELS {
            return false;
        }

        px.copy_from_slice(self.get(pt).as_ref());
        true
    }

    /// Load data from and `Image` into an existing `Pixel` structure
    #[inline]
    pub fn pixel_at(&self, pt: impl Into<Point>, px: &mut Pixel<C>) -> bool {
        let pt = pt.into();
        if !self.in_bounds(pt) {
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
        if !self.in_bounds(pt) || c >= C::CHANNELS {
            return 0.0;
        }

        let data = self.get(pt);
        data[c].to_norm()
    }

    /// Set normalized float value
    pub fn set_f(&mut self, pt: impl Into<Point>, c: Channel, f: f64) {
        let pt = pt.into();
        if !self.in_bounds(pt) || c >= C::CHANNELS {
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
    #[cfg(not(feature = "parallel"))]
    pub fn rows(&self) -> impl Iterator<Item = (usize, &[T])> {
        self.data.data().chunks(self.meta.width_step()).enumerate()
    }

    /// Iterate over mutable image rows
    #[cfg(not(feature = "parallel"))]
    pub fn rows_mut(&mut self) -> impl Iterator<Item = (usize, &mut [T])> {
        self.data
            .data_mut()
            .chunks_mut(self.meta.width_step())
            .enumerate()
    }

    /// Iterate over image rows
    #[cfg(feature = "parallel")]
    pub fn rows(&self) -> impl ParallelIterator<Item = (usize, &[T])> {
        self.data
            .data()
            .par_chunks(self.meta.width_step())
            .enumerate()
    }

    /// Iterate over mutable image rows
    #[cfg(feature = "parallel")]
    pub fn rows_mut(&mut self) -> impl ParallelIterator<Item = (usize, &mut [T])> {
        self.data
            .data_mut()
            .par_chunks_mut(self.meta.width_step())
            .enumerate()
    }

    /// Iterate over image rows
    #[cfg(not(feature = "parallel"))]
    pub fn row_range(&self, y: usize, height: usize) -> impl Iterator<Item = (usize, &[T])> {
        self.data
            .data()
            .chunks(self.meta.width_step())
            .skip(y)
            .take(height)
            .enumerate()
            .map(move |(i, d)| (i + y, d))
    }

    /// Iterate over mutable image rows
    #[cfg(not(feature = "parallel"))]
    pub fn row_range_mut(
        &mut self,
        y: usize,
        height: usize,
    ) -> impl Iterator<Item = (usize, &mut [T])> {
        self.data
            .data_mut()
            .chunks_mut(self.meta.width_step())
            .skip(y)
            .take(height)
            .enumerate()
            .map(move |(i, d)| (i + y, d))
    }

    /// Iterate over image rows
    #[cfg(feature = "parallel")]
    pub fn row_range(
        &self,
        y: usize,
        height: usize,
    ) -> impl ParallelIterator<Item = (usize, &[T])> {
        self.data
            .data()
            .par_chunks(self.meta.width_step())
            .skip(y)
            .take(height)
            .enumerate()
            .map(move |(i, d)| (i + y, d))
    }

    /// Iterate over mutable image rows
    #[cfg(feature = "parallel")]
    pub fn row_range_mut(
        &mut self,
        y: usize,
        height: usize,
    ) -> impl ParallelIterator<Item = (usize, &mut [T])> {
        self.data
            .data_mut()
            .par_chunks_mut(self.meta.width_step())
            .skip(y)
            .take(height)
            .enumerate()
            .map(move |(i, d)| (i + y, d))
    }

    /// Read an image from disk
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Image<T, C>, Error> {
        io::read(path)
    }

    /// Write an image to disk
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        io::write(path, self)
    }

    /// Iterate over part of an image with mutable data access
    #[cfg(feature = "parallel")]
    pub fn iter_region_mut(
        &mut self,
        roi: Region,
    ) -> impl rayon::iter::ParallelIterator<Item = (Point, DataMut<T, C>)> {
        self.row_range_mut(roi.origin.y, roi.height())
            .flat_map(move |(y, row)| {
                row.par_chunks_mut(C::CHANNELS)
                    .skip(roi.origin.x)
                    .take(roi.width())
                    .map(DataMut::new)
                    .enumerate()
                    .map(move |(x, d)| (Point::new(x, y), d))
            })
    }

    /// Iterate over part of an image with mutable data access
    #[cfg(not(feature = "parallel"))]
    pub fn iter_region_mut(
        &mut self,
        roi: Region,
    ) -> impl std::iter::Iterator<Item = (Point, DataMut<T, C>)> {
        self.row_range_mut(roi.origin.y, roi.height())
            .flat_map(move |(y, row)| {
                row.chunks_mut(C::CHANNELS)
                    .skip(roi.origin.x)
                    .take(roi.width())
                    .map(DataMut::new)
                    .enumerate()
                    .map(move |(x, d)| (Point::new(x, y), d))
            })
    }

    /// Iterate over part of an image
    #[cfg(feature = "parallel")]
    pub fn iter_region(
        &self,
        roi: Region,
    ) -> impl rayon::iter::ParallelIterator<Item = (Point, Data<T, C>)> {
        self.row_range(roi.origin.y, roi.height())
            .flat_map(move |(y, row)| {
                row.par_chunks(C::CHANNELS)
                    .skip(roi.origin.x)
                    .take(roi.width())
                    .map(Data::new)
                    .enumerate()
                    .map(move |(x, d)| (Point::new(x, y), d))
            })
    }

    /// Iterate over part of an image
    #[cfg(not(feature = "parallel"))]
    pub fn iter_region(&self, roi: Region) -> impl std::iter::Iterator<Item = (Point, Data<T, C>)> {
        self.row_range(roi.origin.y, roi.height())
            .flat_map(move |(y, row)| {
                row.chunks(C::CHANNELS)
                    .skip(roi.origin.x)
                    .take(roi.width())
                    .map(Data::new)
                    .enumerate()
                    .map(move |(x, d)| (Point::new(x, y), d))
            })
    }

    /// Get pixel iterator
    #[cfg(feature = "parallel")]
    pub fn iter(&self) -> impl rayon::iter::ParallelIterator<Item = (Point, Data<T, C>)> {
        self.rows().flat_map(move |(y, row)| {
            row.par_chunks(C::CHANNELS)
                .map(Data::new)
                .enumerate()
                .map(move |(x, d)| (Point::new(x, y), d))
        })
    }

    /// Get pixel iterator
    #[cfg(not(feature = "parallel"))]
    pub fn iter(&self) -> impl std::iter::Iterator<Item = (Point, Data<T, C>)> {
        self.rows().flat_map(move |(y, row)| {
            row.chunks(C::CHANNELS)
                .map(Data::new)
                .enumerate()
                .map(move |(x, d)| (Point::new(x, y), d))
        })
    }

    /// Get mutable pixel iterator
    #[cfg(feature = "parallel")]
    pub fn iter_mut(
        &mut self,
    ) -> impl rayon::iter::ParallelIterator<Item = (Point, DataMut<T, C>)> {
        self.rows_mut().flat_map(move |(y, row)| {
            row.par_chunks_mut(C::CHANNELS)
                .map(DataMut::new)
                .enumerate()
                .map(move |(x, d)| (Point::new(x, y), d))
        })
    }

    /// Get mutable data iterator
    #[cfg(not(feature = "parallel"))]
    pub fn iter_mut(&mut self) -> impl std::iter::Iterator<Item = (Point, DataMut<T, C>)> {
        self.rows_mut().flat_map(move |(y, row)| {
            row.chunks_mut(C::CHANNELS)
                .map(DataMut::new)
                .enumerate()
                .map(move |(x, d)| (Point::new(x, y), d))
        })
    }

    /// Iterate over each pixel applying `f` to every pixel
    pub fn for_each<F: Sync + Send + Fn(Point, DataMut<T, C>)>(&mut self, f: F) {
        self.rows_mut().for_each(|(y, row)| {
            row.chunks_mut(C::CHANNELS)
                .map(DataMut::new)
                .enumerate()
                .for_each(|(x, px)| f(Point::new(x, y), px))
        })
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
        let meta = self.meta();
        let b = other.data.data().par_chunks(C::CHANNELS);
        self.data
            .data_mut()
            .par_chunks_mut(C::CHANNELS)
            .zip(b)
            .enumerate()
            .for_each(|(n, (pixel, pixel1))| {
                let pt = meta.convert_index_to_point(n * C::CHANNELS);
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
        let meta = self.meta();
        let b = other.data.data().chunks(C::CHANNELS);
        self.data
            .data_mut()
            .chunks_mut(C::CHANNELS)
            .zip(b)
            .enumerate()
            .for_each(|(n, (pixel, pixel1))| {
                let pt = meta.convert_index_to_point(n * C::CHANNELS);
                f(pt, DataMut::new(pixel), Data::new(pixel1))
            });
    }

    /// Iterate over pixels, with a mutable closure
    pub fn each_pixel<F: Sync + Send + FnMut(Point, &Pixel<C>)>(&self, mut f: F) {
        let meta = self.meta();
        let mut pixel = Pixel::new();

        self.data
            .data()
            .chunks_exact(C::CHANNELS)
            .enumerate()
            .for_each(|(n, px)| {
                let pt = meta.convert_index_to_point(n * C::CHANNELS);
                pixel.copy_from_slice(px);
                f(pt, &pixel)
            })
    }

    /// Iterate over pixels in region, with a mutable closure
    pub fn each_pixel_region<F: Sync + Send + FnMut(Point, &Pixel<C>)>(
        &self,
        region: Region,
        mut f: F,
    ) {
        let meta = self.meta();
        let mut pixel = Pixel::new();

        self.data
            .data()
            .chunks_exact(C::CHANNELS)
            .enumerate()
            .map(|(n, px)| {
                let pt = meta.convert_index_to_point(n * C::CHANNELS);
                (pt, px)
            })
            .filter(|(pt, _px)| region.contains(*pt))
            .for_each(|(pt, px)| {
                pixel.copy_from_slice(px);
                f(pt, &pixel);
            })
    }

    /// Iterate over mutable pixels, with a mutable closure
    pub fn each_pixel_mut<F: Sync + Send + FnMut(Point, &mut Pixel<C>)>(&mut self, mut f: F) {
        let meta = self.meta();
        let mut pixel = Pixel::new();

        self.data
            .data_mut()
            .chunks_exact_mut(C::CHANNELS)
            .enumerate()
            .for_each(|(n, px)| {
                let pt = meta.convert_index_to_point(n * C::CHANNELS);
                pixel.copy_from_slice(&px);
                f(pt, &mut pixel);
                pixel.copy_to_slice(px);
            });
    }

    /// Iterate over mutable pixels in region, with a mutable closure
    pub fn each_pixel_region_mut<F: Sync + Send + FnMut(Point, &mut Pixel<C>)>(
        &mut self,
        region: Region,
        mut f: F,
    ) {
        let meta = self.meta();
        let mut pixel = Pixel::new();

        self.data
            .data_mut()
            .chunks_exact_mut(C::CHANNELS)
            .enumerate()
            .map(|(n, px)| {
                let pt = meta.convert_index_to_point(n * C::CHANNELS);
                (pt, px)
            })
            .filter(|(pt, _px)| region.contains(*pt))
            .for_each(|(pt, px)| {
                pixel.copy_from_slice(&px);
                f(pt, &mut pixel);

                pixel.copy_to_slice(px);
            })
    }

    /// Copy a region of an image to a new image
    pub fn crop(&self, roi: Region) -> Image<T, C> {
        let mut dest = Image::new(roi.size);
        dest.apply(filter::crop(roi), &[self]);
        dest
    }

    /// Copy into a region from another image starting at the given offset
    pub fn copy_from_region(&mut self, offs: impl Into<Point>, other: &Image<T, C>, roi: Region) {
        let offs = offs.into();
        self.for_each_region(roi, |pt, mut px| {
            px.copy_from_slice(
                other.get((pt.x - roi.origin.x + offs.x, pt.y - roi.origin.y + offs.y)),
            );
        });
    }

    /// Apply a filter using an Image as output
    pub fn apply<U: Type, D: Color>(
        &mut self,
        filter: impl Filter<U, D, T, C>,
        input: &[&Image<U, D>],
    ) -> &mut Self {
        filter.eval(input, self);
        self
    }

    /// Apply an async filter using an Image as output
    pub async fn apply_async<'a, U: Type, D: Color>(
        &mut self,
        mode: AsyncMode,
        filter: impl Filter<U, D, T, C> + Unpin,
        input: &[&Image<U, D>],
    ) -> &mut Self {
        filters::eval_async(&filter, mode, Input::new(input), self).await;
        self
    }

    /// Run a filter using the same Image as input and output
    pub fn run_in_place(&mut self, filter: impl Filter<T, C>) -> &mut Self {
        filter.eval_in_place(self);
        self
    }

    /// Run a filter using an Image as input
    pub fn run<U: Type, D: Color>(
        &self,
        filter: impl Filter<T, C, U, D>,
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
    pub async fn run_async<'a, U: 'a + Type, D: 'a + Color>(
        &self,
        mode: AsyncMode,
        filter: impl Filter<T, C, U, D> + Unpin,
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
        self.run(filter::convert(), None)
    }

    /// Convert image type/color
    pub fn convert_to<U: Type, D: Color>(&self, dest: &mut Image<U, D>) {
        dest.apply(filter::convert(), &[self]);
    }

    /// Convert to `ImageBuf`
    #[cfg(feature = "oiio")]
    pub(crate) fn image_buf(&mut self) -> io::oiio::internal::ImageBuf {
        io::oiio::internal::ImageBuf::new_with_data(
            self.width(),
            self.height(),
            self.channels(),
            self.data.data_mut(),
        )
    }

    /// Convert to `ImageBuf`
    #[cfg(feature = "oiio")]
    pub(crate) fn const_image_buf(&self) -> io::oiio::internal::ImageBuf {
        io::oiio::internal::ImageBuf::const_new_with_data(
            self.width(),
            self.height(),
            self.channels(),
            self.data.data(),
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
    pub fn gamma(&mut self, value: f64) {
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
        self.run(filter::resize(self.size(), size), Some(Meta::new(size)))
    }

    /// Scale an image
    pub fn scale(&self, width: f64, height: f64) -> Image<T, C> {
        self.run(
            filter::scale(width, height),
            Some(Meta::new((
                (self.width() as f64 * width) as usize,
                (self.height() as f64 * height) as usize,
            ))),
        )
    }

    /// Image data
    pub fn data(&self) -> &[T] {
        self.data.data()
    }

    /// Mutable image data
    pub fn data_mut(&mut self) -> &mut [T] {
        self.data.data_mut()
    }
}
