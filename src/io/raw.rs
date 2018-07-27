#![cfg(feature = "raw")]

use rawloader;

use color::Rgb;
use error::Error;
use image::ImageBuf;

use std::path::Path;

/// RAW image type
pub struct Raw {
    /// A reference to the rawloader image
    pub image: rawloader::RawImage,
}

impl Raw {
    /// Decode a RAW image from a file
    pub fn decode<P: AsRef<Path>>(path: &P) -> Option<Raw> {
        let filename = match path.as_ref().to_str() {
            Some(f) => f,
            None => return None,
        };

        let raw_image = match rawloader::decode(filename) {
            Ok(r) => r,
            Err(_) => return None,
        };

        Some(Raw { image: raw_image })
    }

    /// Convert RAW image to RGB<f32>
    pub fn to_rgb(&self, w: usize, h: usize) -> Result<ImageBuf<f32, Rgb>, Error> {
        let decoded = self.image.to_rgb(w, h)?;
        Ok(ImageBuf::new_from(w, h, decoded.data))
    }

    /// Convert RAW image to linear RGB<f32>
    pub fn to_linear_rgb(&self, w: usize, h: usize) -> Result<ImageBuf<f32, Rgb>, Error> {
        let decoded = self.image.to_linear_rgb(w, h)?;
        Ok(ImageBuf::new_from(w, h, decoded.data))
    }

    /// Convert RAW image to SRGB<u8>
    pub fn to_srgb(&self, w: usize, h: usize) -> Result<ImageBuf<u8, Rgb>, Error> {
        let decoded = self.image.to_srgb(w, h)?;
        Ok(ImageBuf::new_from(w, h, decoded.data))
    }
}
