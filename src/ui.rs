pub use bevy;

use std::path::Path;

use crate::{Color, Image, Rgba, Type};
use anyhow::Result;
use bevy::prelude::*;
use bevy_asset::AssetLoader;
use bevy_math::Vec2;
use bevy_render::{prelude::Texture, texture::TextureFormat};

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

pub fn make_image(
    width: usize,
    height: usize,
    texture: Handle<Texture>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> ImageComponents {
    ImageComponents {
        style: Style {
            size: Size {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..Default::default()
            },
            position_type: PositionType::Relative,
            margin: Rect::all(Val::Auto),
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            aspect_ratio: Some(width as f32 / height as f32),
            ..Default::default()
        },
        material: materials.add(texture.into()),
        ..Default::default()
    }
}

impl<T: Type, C: Color> Image<T, C>
where
    Image<T, C>: Into<Texture>,
{
    pub fn show(
        self,
        mut assets: ResMut<Assets<Texture>>,
        materials: ResMut<Assets<ColorMaterial>>,
    ) -> (Handle<Texture>, ImageComponents) {
        let width = self.width();
        let height = self.height();
        let texture: Texture = self.into();
        let texture = assets.add(texture);
        (texture, make_image(width, height, texture, materials))
    }
}

impl<'a, T: 'a + Type, C: 'a + Color> Image<T, C>
where
    &'a Image<T, C>: Into<Texture>,
{
    pub fn show_clone(
        &'a self,
        mut assets: ResMut<Assets<Texture>>,
        materials: ResMut<Assets<ColorMaterial>>,
    ) -> (Handle<Texture>, ImageComponents) {
        let texture: Texture = self.into();
        let texture = assets.add(texture);
        (
            texture,
            make_image(self.width(), self.height(), texture, materials),
        )
    }

    pub fn update_texture(&'a self, texture: &mut Texture) {
        if texture.data.len()
            == self.width() * self.height() * self.channels() * std::mem::size_of::<T>()
        {
            unsafe {
                std::ptr::copy(
                    self.data.as_ptr() as *const u8,
                    texture.data.as_mut_ptr(),
                    self.data.len() * std::mem::size_of::<T>(),
                )
            }
        }
    }
}

fn into_texture<T: crate::Type, C: crate::Color>(
    image: Image<T, C>,
    fmt: TextureFormat,
) -> Texture {
    let size = Vec2::new(image.width() as f32, image.height() as f32);
    Texture::new(size, image.into_buffer(), fmt)
}

fn to_texture<T: crate::Type, C: crate::Color>(image: &Image<T, C>, fmt: TextureFormat) -> Texture {
    let size = Vec2::new(image.width() as f32, image.height() as f32);
    Texture::new(size, image.to_buffer(), fmt)
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
    pub image: Box<std::sync::Arc<Image<T, C>>>,
    pub handle: Option<Handle<Texture>>,
    pub components: Option<ImageComponents>,
    dirty: bool,
}

impl<'a, T: 'a + Type, C: 'a + crate::Color> ImageView<T, C>
where
    &'a Image<T, C>: Into<Texture>,
{
    pub fn new(image: Image<T, C>) -> ImageView<T, C> {
        ImageView {
            image: Box::new(std::sync::Arc::new(image)),
            handle: None,
            components: None,
            dirty: true,
        }
    }

    pub fn mark_as_dirty(&mut self) {
        self.dirty = true
    }

    pub fn image_mut(&mut self) -> &mut Image<T, C> {
        std::sync::Arc::make_mut(&mut self.image)
    }

    pub fn image(&self) -> &Image<T, C> {
        self.image.as_ref()
    }

    pub fn draw(&'a mut self, mut assets: ResMut<Assets<Texture>>) {
        if let Some(handle) = &self.handle {
            if self.dirty {
                if let Some(texture) = assets.get_mut(&handle) {
                    self.dirty = false;
                    self.image().update_texture(texture);
                }
            }
        }
    }
}

impl<T: 'static + Type, C: 'static + crate::Color> Plugin for ImageView<T, C> {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(self.clone())
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
    let (handle, image) = window.image().show_clone(assets, materials);
    window.handle = Some(handle);
    window.components = Some(image.clone());
    commands.spawn(image);
}

fn update_window(assets: ResMut<Assets<Texture>>, mut window: ResMut<ImageView<f32, Rgba>>) {
    window.draw(assets)
}

pub fn init(mut commands: Commands) {
    commands.spawn(UiCameraComponents::default());
}
