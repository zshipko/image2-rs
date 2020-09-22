use crate::{Convert, Filter, Image, Rgb, Rgba};
use anyhow::Result;
use bevy_asset::AssetLoader;
use bevy_math::Vec2;
use bevy_render::{prelude::Texture, texture::TextureFormat};
use std::path::Path;

/// Loader for images that can be read by the `image` crate.
///
/// Reads only PNG images for now.
#[derive(Clone, Default)]
pub struct ImageTextureLoader;

impl AssetLoader<Texture> for ImageTextureLoader {
    fn from_bytes(&self, asset_path: &Path, _bytes: Vec<u8>) -> Result<Texture> {
        let image = Image::<f32, Rgba>::open(asset_path)?;

        let data: Vec<u8> = image.buffer().into_iter().copied().collect();
        let format: TextureFormat = TextureFormat::Rgba32Float;

        Ok(Texture::new(
            Vec2::new(image.width() as f32, image.height() as f32),
            data,
            format,
        ))
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["png"];
        EXTENSIONS
    }
}
