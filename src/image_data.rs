use crate::*;

/// ImageData is used to hold pixel data for images
pub trait ImageData<T: Type>: Sync + Send + AsRef<[T]> + AsMut<[T]>
where
    T: Copy,
{
    /// Flush image data to disk, this is a no-op except when using memory-mapped data
    fn flush(&self) -> Result<(), Error> {
        Ok(())
    }

    /// Get slice
    fn data(&self) -> &[T] {
        self.as_ref()
    }

    /// Get mutable slice
    fn data_mut(&mut self) -> &mut [T] {
        self.as_mut()
    }

    /// Get pointer
    fn as_ptr(&self) -> *const T {
        self.as_ref().as_ptr()
    }

    /// Get mutable pointer
    fn as_mut_ptr(&mut self) -> *mut T {
        self.as_mut().as_mut_ptr()
    }

    /// Get byte slice
    fn buffer(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().as_ptr() as *const u8,
                self.as_ref().len() * std::mem::size_of::<T>(),
            )
        }
    }

    /// Get mutable byte slice
    fn buffer_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut().as_ptr() as *mut u8,
                self.as_mut().len() * std::mem::size_of::<T>(),
            )
        }
    }
}

impl<T: Type> std::ops::Index<usize> for dyn ImageData<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_ref()[index]
    }
}

impl<T: Type> std::ops::IndexMut<usize> for dyn ImageData<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_mut()[index]
    }
}

impl<T: Type> std::ops::Index<std::ops::Range<usize>> for dyn ImageData<T> {
    type Output = [T];

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        &self.as_ref()[index]
    }
}

impl<T: Type> std::ops::IndexMut<std::ops::Range<usize>> for dyn ImageData<T> {
    fn index_mut(&mut self, index: std::ops::Range<usize>) -> &mut Self::Output {
        &mut self.as_mut()[index]
    }
}

#[cfg(feature = "mmap")]
pub mod mmap {
    use super::*;
    use memmap2::MmapOptions;
    use std::io::{Read, Write};

    /// Memory-mapped image data
    pub struct Mmap<T: Type> {
        inner: memmap2::MmapMut,
        _t: std::marker::PhantomData<T>,
    }

    impl<T: Type> Mmap<T> {
        fn header_len() -> u64 {
            4 + std::mem::size_of::<u64>() as u64
                + std::mem::size_of::<u64>() as u64
                + std::mem::size_of::<u64>() as u64
                + std::mem::size_of::<u16>() as u64
        }

        /// Create new `Mmap` on disk
        pub fn create<C: Color>(
            filename: impl AsRef<std::path::Path>,
            meta: &Meta<T, C>,
        ) -> Result<Self, Error> {
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(filename)?;

            file.set_len(Self::header_len() + meta.num_bytes() as u64)?;

            file.write_all(b"img2")?;
            file.write_all(&(std::mem::size_of::<T>() as u64).to_le_bytes())?;
            file.write_all(&(meta.width() as u64).to_le_bytes())?;
            file.write_all(&(meta.height() as u64).to_le_bytes())?;
            file.write_all(&(C::CHANNELS as u16).to_le_bytes())?;

            let inner = unsafe {
                MmapOptions::new()
                    .offset(Self::header_len())
                    .map_mut(&file)?
            };

            let data = Self {
                inner,
                _t: std::marker::PhantomData,
            };

            Ok(data)
        }

        /// Create new image on disk
        pub(crate) fn create_image<C: Color>(
            filename: impl AsRef<std::path::Path>,
            meta: &Meta<T, C>,
        ) -> Result<Image<T, C>, Error> {
            let data = Self::create(filename, meta)?;
            unsafe { Ok(Image::new_with_data(meta.size(), data)) }
        }

        /// Load `Mmap` from disk
        pub(crate) fn load<C: Color>(
            filename: impl AsRef<std::path::Path>,
        ) -> Result<(Mmap<T>, Meta<T, C>), Error> {
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(filename)?;

            let mut hdr = [0u8; 4];
            file.read_exact(&mut hdr)?;

            if hdr.as_slice() != b"img2" {
                return Err(Error::Message("invalid mmap header".to_string()));
            }

            let mut size = [0u8; 8];
            file.read_exact(&mut size)?;

            if size != (std::mem::size_of::<T>() as u64).to_le_bytes() {
                return Err(Error::InvalidType);
            }

            let mut width = [0u8; 8];
            file.read_exact(&mut width)?;

            let mut height = [0u8; 8];
            file.read_exact(&mut height)?;

            let mut channels = [0u8; 2];
            file.read_exact(&mut channels)?;

            if u16::from_le_bytes(channels) as usize != C::CHANNELS {
                return Err(Error::InvalidType);
            }

            let inner = unsafe {
                MmapOptions::new()
                    .offset(Self::header_len())
                    .map_mut(&file)?
            };

            let width = u64::from_le_bytes(width) as usize;
            let height = u64::from_le_bytes(height) as usize;

            let data = Self {
                inner,
                _t: std::marker::PhantomData,
            };
            Ok((data, Meta::new((width, height))))
        }

        /// Load image from disk
        pub(crate) fn load_image<C: Color>(
            filename: impl AsRef<std::path::Path>,
        ) -> Result<Image<T, C>, Error> {
            let (data, meta) = Self::load::<C>(filename)?;
            unsafe { Ok(Image::new_with_data(meta.size(), data)) }
        }
    }

    impl<T: Type> AsRef<[T]> for Mmap<T> {
        fn as_ref(&self) -> &[T] {
            unsafe {
                std::slice::from_raw_parts(
                    self.inner.as_ptr() as *const _,
                    self.inner.len() / std::mem::size_of::<T>(),
                )
            }
        }
    }

    impl<T: Type> AsMut<[T]> for Mmap<T> {
        fn as_mut(&mut self) -> &mut [T] {
            unsafe {
                std::slice::from_raw_parts_mut(
                    self.inner.as_ptr() as *mut _,
                    self.inner.len() / std::mem::size_of::<T>(),
                )
            }
        }
    }

    impl<T: Type> ImageData<T> for Mmap<T> {
        fn flush(&self) -> Result<(), Error> {
            self.inner.flush()?;
            Ok(())
        }
    }

    impl<T: Type> Drop for Mmap<T> {
        fn drop(&mut self) {
            let _ = self.flush();
        }
    }
}

impl<T: Type> ImageData<T> for [T] {}
impl<T: Type> ImageData<T> for Vec<T> {}
impl<T: Type> ImageData<T> for Box<[T]> {}
