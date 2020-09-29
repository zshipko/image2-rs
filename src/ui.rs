use std::path::Path;

pub use bevy;
use bevy::prelude::*;

use crate::{Color, Image, Region, Rgba, Type};
use anyhow::Result;
use bevy::asset::AssetLoader;
use bevy::math::Vec2;
use bevy::render::{prelude::Texture, texture::TextureFormat};

/// Loader for images that can be read by `image2`.
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

/// Create an Image widget
pub fn make_image(
    width: usize,
    height: usize,
    texture: Handle<Texture>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> ImageComponents {
    ImageComponents {
        style: Style {
            margin: Rect::all(Val::Auto),
            position_type: PositionType::Relative,
            align_content: AlignContent::Center,
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
    /// Convert image to bevy `Texture` and build Image widget
    pub fn texture(
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
    /// Clone image data to bevy `Texture` and build Image widget
    pub fn texture_clone(
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

    /// Update an existing texture with data from an image
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

fn convert(
    image: &Image<impl Type, impl Color>,
) -> (Option<TextureFormat>, Option<Image<impl Type, impl Color>>) {
    let fmt = match image.meta.color_name() {
        "rgba" => match image.meta.type_name() {
            "half" => TextureFormat::Rgba16Float,
            "float" => TextureFormat::Rgba32Float,
            "int8" => TextureFormat::Rgba8Sint,
            "uint8" => TextureFormat::Rgba8Uint,
            "int16" => TextureFormat::Rgba16Sint,
            "uint16" => TextureFormat::Rgba16Uint,
            "int32" => TextureFormat::Rgba32Sint,
            "uint32" => TextureFormat::Rgba32Uint,
            _ => return (None, Some(image.convert::<f32, Rgba>())),
        },
        "gray" => match image.meta.type_name() {
            "half" => TextureFormat::R16Float,
            "float" => TextureFormat::R32Float,
            "int8" => TextureFormat::R8Sint,
            "uint8" => TextureFormat::R8Uint,
            "int16" => TextureFormat::R16Sint,
            "uint16" => TextureFormat::R16Uint,
            "int32" => TextureFormat::R32Sint,
            "uint32" => TextureFormat::R32Uint,
            _ => return (None, Some(image.convert::<f32, Rgba>())),
        },
        _ => return (None, Some(image.convert::<f32, Rgba>())),
    };

    (Some(fmt), None)
}

fn into_texture<T: crate::Type, C: crate::Color>(image: Image<T, C>) -> Texture {
    let fmt = match convert(&image) {
        (_, Some(im)) => return into_texture(im),
        (Some(fmt), _) => fmt,
        _ => unreachable!(),
    };

    let size = Vec2::new(image.width() as f32, image.height() as f32);
    let buf = image.into_buffer();
    Texture::new(size, buf, fmt)
}

fn to_texture<T: crate::Type, C: crate::Color>(image: &Image<T, C>) -> Texture {
    let fmt = match convert(&image) {
        (_, Some(im)) => return into_texture(im),
        (Some(fmt), _) => fmt,
        _ => unreachable!(),
    };

    let size = Vec2::new(image.width() as f32, image.height() as f32);
    let buf = image.to_buffer();
    Texture::new(size, buf, fmt)
}

impl<T: Type, C: Color> From<Image<T, C>> for Texture {
    fn from(image: Image<T, C>) -> Texture {
        into_texture(image)
    }
}

impl<'a, T: Type, C: Color> From<&'a Image<T, C>> for Texture {
    fn from(image: &'a Image<T, C>) -> Texture {
        to_texture(image)
    }
}

/// Image winwdow
#[derive(Clone)]
pub struct ImageView<T: Type, C: Color> {
    /// Window title
    pub title: String,

    /// Underlying image
    pub image: Box<std::sync::Arc<Image<T, C>>>,

    /// Texture handle
    pub handle: Option<Handle<Texture>>,

    /// ImageComponents
    pub components: Option<ImageComponents>,

    /// Selection
    pub selection: Option<Region>,

    dirty: bool,
}

unsafe impl<T: Type, C: Color> Send for ImageView<T, C> {}
unsafe impl<T: Type, C: Color> Sync for ImageView<T, C> {}

impl<'a, T: 'a + Type, C: 'a + Color> ImageView<T, C> {
    /// Create new image view
    pub fn new(title: impl Into<String>, image: Image<T, C>) -> ImageView<T, C> {
        let arc = std::sync::Arc::new(image);

        Self::wrap_arc(title, arc)
    }

    /// Create new image view from an existing Arc<Image<T, C>>
    pub fn wrap_arc(
        title: impl Into<String>,
        image: std::sync::Arc<Image<T, C>>,
    ) -> ImageView<T, C> {
        ImageView {
            title: title.into(),
            image: Box::new(image),
            handle: None,
            components: None,
            selection: None,
            dirty: true,
        }
    }

    /// Mark image as dirty, triggering displayed image to be updated
    pub fn mark_as_dirty(&mut self) {
        self.dirty = true
    }

    /// Get mutable reference to underlying image
    pub fn image_mut(&mut self) -> &mut Image<T, C> {
        std::sync::Arc::make_mut(&mut self.image)
    }

    /// Get reference to underlying image
    pub fn image(&self) -> &Image<T, C> {
        self.image.as_ref()
    }
}

impl<'a, T: 'static + Type, C: 'static + Color> ImageView<T, C>
where
    &'a Image<T, C>: Into<Texture>,
{
    /// Redraw the image
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
            .add_startup_system(startup_window::<T, C>.system())
            .add_system(update_window::<T, C>.system())
            .add_resource(WindowDescriptor {
                title: self.title.clone(),
                vsync: true,
                ..Default::default()
            })
            .add_default_plugins();
    }
}

fn startup_window<T: 'static + Type, C: 'static + Color>(
    mut commands: Commands,
    assets: ResMut<Assets<Texture>>,
    materials: ResMut<Assets<ColorMaterial>>,
    mut window: ResMut<ImageView<T, C>>,
) {
    let (handle, image) = window.image().texture_clone(assets, materials);
    window.handle = Some(handle);
    window.components = Some(image.clone());
    commands.spawn(image);
}

fn update_window<T: 'static + Type, C: 'static + Color>(
    assets: ResMut<Assets<Texture>>,
    mut view: ResMut<ImageView<T, C>>,
) {
    view.draw(assets);
}

/// Initialize UI
pub fn init(mut commands: Commands) {
    commands.spawn(UiCameraComponents::default());
}

/// Show an image and exit with ESC is pressed
pub fn show<T: 'static + Type, C: 'static + Color>(title: &str, image: Image<T, C>) -> Image<T, C> {
    let image = std::sync::Arc::new(image);

    {
        App::build()
            .add_startup_system(init.system())
            .add_plugin(ImageView::<T, C>::wrap_arc(title, image.clone()))
            .add_system(bevy::input::system::exit_on_esc_system.system())
            .run();
    }

    match std::sync::Arc::try_unwrap(image) {
        Ok(x) => x,
        Err(_) => panic!("show: unable to get image handle out of Arc"),
    }
}

/// Show an image and exit with ESC is pressed with an additional startup system and system params
pub fn show_with_system<T: 'static + Type, C: 'static + Color>(
    title: &str,
    image: Image<T, C>,
    startup: impl 'static + System,
    system: impl 'static + System,
) -> Image<T, C> {
    let image = std::sync::Arc::new(image);

    {
        App::build()
            .add_startup_system(init.system())
            .add_startup_system(Box::new(startup))
            .add_plugin(ImageView::<T, C>::wrap_arc(title, image.clone()))
            .add_system(bevy::input::system::exit_on_esc_system.system())
            .add_system(Box::new(system))
            .run();
    }

    match std::sync::Arc::try_unwrap(image) {
        Ok(x) => x,
        Err(_) => panic!("show: unable to get image handle out of Arc"),
    }
}

/// Show an image and exit with ESC is pressed with an additional plugin param
pub fn show_with_plugin<T: 'static + Type, C: 'static + Color>(
    title: &str,
    image: Image<T, C>,
    plugin: impl 'static + Plugin,
) -> Image<T, C> {
    let image = std::sync::Arc::new(image);

    {
        App::build()
            .add_startup_system(init.system())
            .add_plugin(ImageView::<T, C>::wrap_arc(title, image.clone()))
            .add_system(bevy::input::system::exit_on_esc_system.system())
            .add_plugin(plugin)
            .run();
    }

    match std::sync::Arc::try_unwrap(image) {
        Ok(x) => x,
        Err(_) => panic!("show: unable to get image handle out of Arc"),
    }
}
