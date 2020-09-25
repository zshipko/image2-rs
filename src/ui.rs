use crate::{Color, Image, Rgba, Type};
use anyhow::Result;
use bevy::prelude::*;
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
        static EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "exr", "tiff", "tif", "hdr", "dng"];
        EXTENSIONS
    }
}

impl<T: Type, C: Color> Image<T, C>
where
    Image<T, C>: Into<Texture>,
{
    pub fn show(
        self,
        mut assets: ResMut<Assets<Texture>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) -> (Handle<Texture>, ImageComponents) {
        let texture: Texture = self.into();
        let texture = assets.add(texture);
        (
            texture,
            ImageComponents {
                material: materials.add(texture.into()),
                ..Default::default()
            },
        )
    }
}

impl<'a, T: 'a + Type, C: 'a + Color> Image<T, C>
where
    &'a Image<T, C>: Into<Texture>,
{
    pub fn show_clone(
        &'a self,
        mut assets: ResMut<Assets<Texture>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) -> (Handle<Texture>, ImageComponents) {
        let texture: Texture = self.into();
        let texture = assets.add(texture);
        (
            texture,
            ImageComponents {
                material: materials.add(texture.into()),
                ..Default::default()
            },
        )
    }

    pub fn update_texture(&'a self, texture: &mut Texture) {
        unsafe {
            std::ptr::copy(
                self.data.as_ptr() as *const u8,
                texture.data.as_mut_ptr(),
                self.data.len() * std::mem::size_of::<T>(),
            )
        }
    }
}

fn transmute_to_bytes_vec<T>(mut from: Vec<T>) -> Vec<u8> {
    unsafe {
        let capacity = from.capacity() * std::mem::size_of::<T>();
        let len = from.len() * std::mem::size_of::<T>();
        let ptr = from.as_mut_ptr();
        std::mem::forget(from);
        Vec::from_raw_parts(ptr as *mut u8, len, capacity)
    }
}

fn into_texture<T: crate::Type, C: crate::Color>(
    image: Image<T, C>,
    fmt: TextureFormat,
) -> Texture {
    let size = Vec2::new(image.width() as f32, image.height() as f32);
    Texture::new(size, transmute_to_bytes_vec(image.data.into_vec()), fmt)
}

fn to_texture<T: crate::Type, C: crate::Color>(image: &Image<T, C>, fmt: TextureFormat) -> Texture {
    let size = Vec2::new(image.width() as f32, image.height() as f32);
    Texture::new(size, transmute_to_bytes_vec(image.data.to_vec()), fmt)
}

impl From<Image<f32, Rgba>> for Texture {
    fn from(image: Image<f32, Rgba>) -> Texture {
        into_texture(image, TextureFormat::Rgba32Float)
    }
}

impl<'a> From<&'a Image<f32, Rgba>> for Texture {
    fn from(image: &'a Image<f32, Rgba>) -> Texture {
        to_texture(image, TextureFormat::Rgba32Float)
    }
}

impl From<Image<u8, Rgba>> for Texture {
    fn from(image: Image<u8, Rgba>) -> Texture {
        let size = Vec2::new(image.width() as f32, image.height() as f32);
        let format: TextureFormat = TextureFormat::Rgba8Uint;
        Texture::new(size, image.data.into_vec(), format)
    }
}

impl<'a> From<&'a Image<u8, Rgba>> for Texture {
    fn from(image: &'a Image<u8, Rgba>) -> Texture {
        let size = Vec2::new(image.width() as f32, image.height() as f32);
        let format: TextureFormat = TextureFormat::Rgba8Uint;
        Texture::new(size, image.data.to_vec(), format)
    }
}

impl From<Image<crate::f16, Rgba>> for Texture {
    fn from(image: Image<crate::f16, Rgba>) -> Texture {
        into_texture(image, TextureFormat::Rgba16Float)
    }
}

impl<'a> From<&'a Image<crate::f16, Rgba>> for Texture {
    fn from(image: &'a Image<crate::f16, Rgba>) -> Texture {
        to_texture(image, TextureFormat::Rgba16Float)
    }
}

impl From<Image<i16, Rgba>> for Texture {
    fn from(image: Image<i16, Rgba>) -> Texture {
        into_texture(image, TextureFormat::Rgba16Sint)
    }
}

impl<'a> From<&'a Image<i16, Rgba>> for Texture {
    fn from(image: &'a Image<i16, Rgba>) -> Texture {
        to_texture(image, TextureFormat::Rgba16Sint)
    }
}

impl From<Image<u16, Rgba>> for Texture {
    fn from(image: Image<u16, Rgba>) -> Texture {
        into_texture(image, TextureFormat::Rgba16Uint)
    }
}

impl<'a> From<&'a Image<u16, Rgba>> for Texture {
    fn from(image: &'a Image<u16, Rgba>) -> Texture {
        to_texture(image, TextureFormat::Rgba16Uint)
    }
}

impl From<Image<u32, Rgba>> for Texture {
    fn from(image: Image<u32, Rgba>) -> Texture {
        into_texture(image, TextureFormat::Rgba32Uint)
    }
}

impl<'a> From<&'a Image<u32, Rgba>> for Texture {
    fn from(image: &'a Image<u32, Rgba>) -> Texture {
        to_texture(image, TextureFormat::Rgba32Uint)
    }
}

impl From<Image<i32, Rgba>> for Texture {
    fn from(image: Image<i32, Rgba>) -> Texture {
        into_texture(image, TextureFormat::Rgba32Sint)
    }
}

impl<'a> From<&'a Image<i32, Rgba>> for Texture {
    fn from(image: &'a Image<i32, Rgba>) -> Texture {
        to_texture(image, TextureFormat::Rgba32Sint)
    }
}

#[derive(Clone)]
pub struct ImageView<T: Type, C: crate::Color> {
    image: std::sync::Arc<Image<T, C>>,
    handle: Option<Handle<Texture>>,
    timer: Timer,
}

impl<T: Type, C: crate::Color> ImageView<T, C> {
    pub fn new(image: Image<T, C>) -> ImageView<T, C> {
        ImageView {
            image: std::sync::Arc::new(image),
            handle: None,
            timer: Timer::from_seconds(1.0, true),
        }
    }
}

impl<T: 'static + Type, C: 'static + crate::Color> Plugin for ImageView<T, C> {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(self.clone())
            .add_startup_system(init.system())
            .add_startup_system(startup_window.system())
            .add_system(update_window.system());
    }
}

fn startup_window(
    mut commands: Commands,
    assets: ResMut<Assets<Texture>>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut window: ResMut<ImageView<f32, Rgba>>,
) {
    let (handle, image) = window.image.show_clone(assets, materials);
    window.handle = Some(handle);
    commands.spawn(image);
}

fn update_window(
    time: Res<Time>,
    mut assets: ResMut<Assets<Texture>>,
    mut window: ResMut<ImageView<f32, Rgba>>,
) {
    window.timer.tick(time.delta_seconds);

    if window.timer.finished {
        if let Some(handle) = &window.handle {
            if let Some(texture) = assets.get_mut(&handle) {
                window.image.update_texture(texture);
            }
        }
    }
}

pub fn init(mut commands: Commands) {
    commands.spawn(UiCameraComponents::default());
}
