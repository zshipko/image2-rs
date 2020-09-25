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
        let image = Image::<u8, Rgba>::open(asset_path)?;
        Ok(image.into())
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "exr", "tiff", "tif", "hdr"];
        EXTENSIONS
    }
}

fn into_texture<T: crate::Type, C: crate::Color>(
    mut image: Image<T, C>,
    fmt: TextureFormat,
) -> Texture {
    let size = Vec2::new(image.width() as f32, image.height() as f32);
    let len = image.data.len() * std::mem::size_of::<T>();
    let cap = image.data.capacity() * std::mem::size_of::<T>();
    let data = image.data.as_mut_ptr();
    std::mem::forget(image.data);
    let data = unsafe { Vec::from_raw_parts(data as *mut u8, len, cap) };
    Texture::new(size, data, fmt)
}

impl From<Image<f32, Rgba>> for Texture {
    fn from(image: Image<f32, Rgba>) -> Texture {
        into_texture(image, TextureFormat::Rgba32Float)
    }
}

impl From<Image<u8, Rgba>> for Texture {
    fn from(image: Image<u8, Rgba>) -> Texture {
        let size = Vec2::new(image.width() as f32, image.height() as f32);
        let format: TextureFormat = TextureFormat::Rgba8Uint;
        Texture::new(size, image.data, format)
    }
}

impl From<Image<crate::f16, Rgba>> for Texture {
    fn from(image: Image<crate::f16, Rgba>) -> Texture {
        into_texture(image, TextureFormat::Rgba16Float)
    }
}

impl From<Image<i16, Rgba>> for Texture {
    fn from(image: Image<i16, Rgba>) -> Texture {
        into_texture(image, TextureFormat::Rgba16Sint)
    }
}

impl From<Image<u16, Rgba>> for Texture {
    fn from(image: Image<u16, Rgba>) -> Texture {
        into_texture(image, TextureFormat::Rgba16Uint)
    }
}

impl From<Image<u32, Rgba>> for Texture {
    fn from(image: Image<u32, Rgba>) -> Texture {
        into_texture(image, TextureFormat::Rgba32Uint)
    }
}

impl From<Image<i32, Rgba>> for Texture {
    fn from(image: Image<i32, Rgba>) -> Texture {
        into_texture(image, TextureFormat::Rgba32Sint)
    }
}
