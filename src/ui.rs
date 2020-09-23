use crate::{Image, Rgba};
use anyhow::Result;
use bevy_asset::AssetLoader;
use bevy_math::Vec2;
use bevy_render::{prelude::Texture, texture::TextureFormat};
use std::path::Path;

/// Loader for images that can be read by the `image` crate.
///
/// Reads only PNG images for now.
#[derive(Clone, Default)]
pub struct ImageLoader;

impl AssetLoader<Texture> for ImageLoader {
    fn from_bytes(&self, asset_path: &Path, _bytes: Vec<u8>) -> Result<Texture> {
        let mut image = Image::<f32, Rgba>::open(asset_path)?;

        let size = Vec2::new(image.width() as f32, image.height() as f32);
        let len = image.data.len() * std::mem::size_of::<f32>();
        let cap = image.data.capacity() * std::mem::size_of::<f32>();
        let data = image.data.as_mut_ptr();
        std::mem::forget(image.data);
        let data = unsafe { Vec::from_raw_parts(data as *mut u8, len, cap) };

        let format: TextureFormat = TextureFormat::Rgba32Float;
        Ok(Texture::new(size, data, format))
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "exr", "tiff", "tif", "hdr"];
        EXTENSIONS
    }
}
