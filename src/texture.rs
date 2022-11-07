use crate::*;
pub use glow;
use glow::*;

/// OpenGL texture for `Image` type
pub struct ImageTexture<T: Type, C: Color> {
    /// Framebuffer
    pub framebuffer: Framebuffer,

    /// Texture
    pub texture: Texture,
    _t: std::marker::PhantomData<(T, C)>,
}

impl<T: Type, C: Color> ImageTexture<T, C> {
    /// Create a new `ImageTexture` from framebuffer and texture
    pub fn new(framebuffer: Framebuffer, texture: Texture) -> Self {
        ImageTexture {
            framebuffer,
            texture,
            _t: std::marker::PhantomData,
        }
    }
}

/// ToTexture is defined for image types that can be converted to OpenGL textures
pub trait ToTexture<T: Type, C: Color> {
    /// OpenGL color
    const COLOR: u32;

    /// OpenGL type
    const KIND: u32;

    /// Get metadata
    fn get_meta(&self) -> &Meta<T, C>;

    /// Get data buffer
    fn get_data(&self) -> &[u8];

    /// Get internal color type
    fn internal(&self) -> Result<u32, Error> {
        let internal = match (Self::COLOR, Self::KIND) {
            (glow::RED, glow::BYTE) => glow::R8,
            (glow::RED, glow::SHORT) => glow::R16,
            (glow::RED, glow::UNSIGNED_BYTE) => glow::R8,
            (glow::RED, glow::UNSIGNED_SHORT) => glow::R16,
            (glow::RED, glow::FLOAT) => glow::R32F,
            (glow::RGB | glow::SRGB, glow::BYTE) => glow::RGB8,
            (glow::RGB | glow::SRGB, glow::SHORT) => glow::RGB16,
            (glow::RGB | glow::SRGB, glow::UNSIGNED_BYTE) => glow::RGB,
            (glow::RGB | glow::SRGB, glow::UNSIGNED_SHORT) => glow::RGB16,
            (glow::RGB | glow::SRGB, glow::FLOAT) => glow::RGB32F,
            (glow::RGBA | glow::SRGB_ALPHA, glow::BYTE) => glow::RGBA,
            (glow::RGBA | glow::SRGB_ALPHA, glow::SHORT) => glow::RGBA16,
            (glow::RGBA | glow::SRGB_ALPHA, glow::UNSIGNED_BYTE) => glow::RGBA,
            (glow::RGBA | glow::SRGB_ALPHA, glow::UNSIGNED_SHORT) => glow::RGBA16,
            (glow::RGBA | glow::SRGB_ALPHA, glow::FLOAT) => glow::RGBA32F,
            _ => return Err(Error::InvalidType),
        };
        Ok(internal)
    }

    /// Create `ImageTexture`
    fn create_image_texture(&self, gl: &glow::Context) -> Result<ImageTexture<T, C>, Error> {
        unsafe {
            let framebuffer = gl
                .create_framebuffer()
                .expect("Unable to create framebuffer");
            let texture = gl.create_texture().expect("Unable to create texture");
            let image_texture = ImageTexture::<T, C>::new(framebuffer, texture);

            // Texture
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );

            if Self::COLOR == glow::RED {
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_SWIZZLE_G, glow::RED as i32);
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_SWIZZLE_B, glow::RED as i32);
            }

            let meta = self.get_meta();
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                self.internal()? as i32,
                meta.width() as i32,
                meta.height() as i32,
                0,
                Self::COLOR,
                Self::KIND,
                Some(self.get_data()),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);

            // Framebuffer
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(image_texture.framebuffer));
            gl.framebuffer_texture_2d(
                glow::READ_FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(texture),
                0,
            );
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
            Ok(image_texture)
        }
    }

    /// Draw the texture on the framebuffer
    fn draw_image_texture(
        &self,
        gl: &glow::Context,
        image_texture: &ImageTexture<T, C>,
        display_size: Size,
        offset: Point,
    ) -> Result<(), Error> {
        let x = offset.x;
        let y = offset.y;
        let display_width = display_size.width;
        let display_height = display_size.height;

        unsafe {
            // Texture
            gl.bind_texture(glow::TEXTURE_2D, Some(image_texture.texture));
            let meta = self.get_meta();
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                self.internal()? as i32,
                meta.width() as i32,
                meta.height() as i32,
                0,
                Self::COLOR,
                Self::KIND,
                Some(self.get_data()),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);

            // Framebuffer
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(image_texture.framebuffer));
            gl.framebuffer_texture_2d(
                glow::READ_FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(image_texture.texture),
                0,
            );
            gl.blit_framebuffer(
                0,
                meta.height() as i32,
                meta.width() as i32,
                0,
                x as i32,
                y as i32,
                x as i32 + display_width as i32,
                y as i32 + display_height as i32,
                glow::COLOR_BUFFER_BIT,
                glow::NEAREST,
            );
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
        }
        Ok(())
    }
}

macro_rules! to_texture {
    ($t:ty, $c:ty, $kind:expr, $color:expr) => {
        impl ToTexture<$t, $c> for Image<$t, $c> {
            const COLOR: u32 = $color;
            const KIND: u32 = $kind;

            fn get_meta(&self) -> &Meta<$t, $c> {
                &self.meta
            }

            fn get_data(&self) -> &[u8] {
                self.buffer()
            }
        }
    };
}

to_texture!(f32, Rgb, glow::FLOAT, glow::RGB);
to_texture!(f32, Srgb, glow::FLOAT, glow::SRGB);
to_texture!(f32, Rgba, glow::FLOAT, glow::RGBA);
to_texture!(f32, Srgba, glow::FLOAT, glow::SRGB_ALPHA);
to_texture!(u16, Rgb, glow::UNSIGNED_SHORT, glow::RGB);
to_texture!(u16, Srgb, glow::UNSIGNED_SHORT, glow::SRGB);
to_texture!(u16, Rgba, glow::UNSIGNED_SHORT, glow::RGBA);
to_texture!(u16, Srgba, glow::UNSIGNED_SHORT, glow::SRGB_ALPHA);
to_texture!(i16, Rgb, glow::SHORT, glow::RGB);
to_texture!(i16, Srgb, glow::SHORT, glow::SRGB);
to_texture!(i16, Rgba, glow::SHORT, glow::RGBA);
to_texture!(i16, Srgba, glow::SHORT, glow::SRGB_ALPHA);
to_texture!(u8, Rgb, glow::UNSIGNED_BYTE, glow::RGB);
to_texture!(u8, Srgb, glow::UNSIGNED_BYTE, glow::SRGB);
to_texture!(u8, Rgba, glow::UNSIGNED_BYTE, glow::RGBA);
to_texture!(u8, Srgba, glow::UNSIGNED_BYTE, glow::SRGB_ALPHA);
