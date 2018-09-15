#![cfg(feature = "raw")]

use rawloader;

use color::Rgb;
use image::{Image, Layout};
use image_buf::ImageBuf;
use ty::Type;

use std::path::Path;

/// RAW image type
pub struct Raw {
    /// A reference to the rawloader image
    pub image: rawloader::RawImage,
}

impl Raw {
    /// Read a RAW image from a file
    pub fn read<P: AsRef<Path>>(path: &P) -> Option<Raw> {
        let filename = match path.as_ref().to_str() {
            Some(f) => f,
            None => return None,
        };

        let raw_image = match rawloader::decode_file(filename) {
            Ok(r) => r,
            Err(_) => return None,
        };

        Some(Raw { image: raw_image })
    }

    pub fn as_image<T: Type>(self) -> Option<ImageBuf<T, Rgb>> {
        if self.image.cpp == 1 {
            return None;
        }

        match self.image.data {
            rawloader::RawImageData::Integer(data) => {
                let im = ImageBuf::new_from(
                    self.image.width,
                    self.image.height,
                    Layout::Interleaved,
                    data,
                );
                let mut dest = ImageBuf::new(self.image.width, self.image.height);
                im.convert_type(&mut dest);
                Some(dest)
            }
            rawloader::RawImageData::Float(data) => {
                let im = ImageBuf::new_from(
                    self.image.width,
                    self.image.height,
                    Layout::Interleaved,
                    data,
                );
                let mut dest = ImageBuf::new(self.image.width, self.image.height);
                im.convert_type(&mut dest);
                Some(dest)
            }
        }
    }
}
