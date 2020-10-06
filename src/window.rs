#![allow(missing_docs)]

use crate::*;

use gl::types::*;

pub use glutin::{
    self,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextCurrentState as CurrentState, NotCurrent, PossiblyCurrent, WindowedContext as Context,
};

pub struct Window<T: Type, C: Color> {
    pub context: Option<Context<NotCurrent>>,
    pub texture: Texture,
    pub image: Image<T, C>,
    pub framebuffer: GLuint,
    pub size: Size,
}

impl<'a, T: Type, C: Color> Window<T, C> {
    pub fn new<X>(
        event_loop: &EventLoop<X>,
        image: Image<T, C>,
        window: WindowBuilder,
    ) -> Result<Window<T, C>, Error>
    where
        Image<T, C>: ToTexture<T, C>,
    {
        let mut framebuffer = 0;

        let context = glutin::ContextBuilder::new().build_windowed(window, event_loop)?;
        let context = match unsafe { context.make_current() } {
            Ok(ctx) => ctx,
            Err((_, e)) => return Err(e.into()),
        };

        gl::load_with(|ptr| context.context().get_proc_address(ptr) as *const _);

        let texture = image.to_texture()?;

        unsafe {
            gl::GenFramebuffers(1, &mut framebuffer);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer);
            gl::FramebufferTexture2D(
                gl::READ_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture.id,
                0,
            );
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
        }

        let context = match unsafe { context.make_not_current() } {
            Ok(ctx) => ctx,
            Err((_, e)) => return Err(e.into()),
        };

        let size = context.window().inner_size();

        Ok(Window {
            context: Some(context),
            image,
            texture,
            framebuffer,
            size: Size::new(size.width as usize, size.height as usize),
        })
    }

    pub fn with_current_context<X, F: FnOnce(&mut Context<PossiblyCurrent>) -> Result<X, Error>>(
        &mut self,
        f: F,
    ) -> Result<X, Error> {
        if let Some(ctx) = self.context.take() {
            let size = ctx.window().inner_size();
            self.size.width = size.width as usize;
            self.size.height = size.height as usize;

            let ctx = unsafe { ctx.make_current() };
            let mut ctx = match ctx {
                Ok(ctx) => ctx,
                Err((_, e)) => return Err(e.into()),
            };

            gl::load_with(|ptr| ctx.context().get_proc_address(ptr) as *const _);

            let t = f(&mut ctx);
            let ctx = match unsafe { ctx.make_not_current() } {
                Ok(x) => x,
                Err((_, e)) => return Err(e.into()),
            };
            self.context = Some(ctx);
            return Ok(t?);
        }

        Err(Error::GlutinContext(glutin::ContextError::ContextLost))
    }

    pub fn mouse_position(&self, pt: impl Into<Point>) -> Point {
        let pt = pt.into();
        let ratio = (self.size.width as f64 / self.image.meta.width() as f64)
            .min(self.size.height as f64 / self.image.meta.height() as f64);
        let display_width = (self.image.meta.width() as f64 * ratio) as usize;
        let display_height = (self.image.meta.height() as f64 * ratio) as usize;
        let x = self.size.width.saturating_sub(display_width);
        let y = self.size.height.saturating_sub(display_height);
        pt.map(|a, b| (a.saturating_sub(x), b.saturating_sub(y)))
    }

    pub fn draw(&mut self) -> Result<(), Error> {
        let meta = self.image.meta().clone();
        let image = self.image.data.as_ptr();
        let texture = self.texture.clone();
        let framebuffer = self.framebuffer;
        let size = self.size;
        let size = self.with_current_context(|ctx| {
            let ratio = (size.width as f64 / meta.width() as f64)
                .min(size.height as f64 / meta.height() as f64);
            let display_width = (meta.width() as f64 * ratio) as usize;
            let display_height = (meta.height() as f64 * ratio) as usize;
            let x = size.width.saturating_sub(display_width);
            let y = size.height.saturating_sub(display_height);
            unsafe {
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                gl::BindTexture(gl::TEXTURE_2D, texture.id);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    texture.internal as i32,
                    meta.width() as i32,
                    meta.height() as i32,
                    0,
                    texture.color,
                    texture.kind,
                    image as *const _,
                );
                gl::BindTexture(gl::TEXTURE_2D, 0);

                gl::BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer);
                gl::FramebufferTexture2D(
                    gl::READ_FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    texture.id,
                    0,
                );
                gl::BlitFramebuffer(
                    0,
                    meta.height() as i32,
                    meta.width() as i32,
                    0,
                    x as i32,
                    y as i32,
                    display_width as i32,
                    display_height as i32,
                    gl::COLOR_BUFFER_BIT,
                    gl::NEAREST,
                );
                gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
            }
            ctx.swap_buffers()?;
            Ok(ctx.window().inner_size())
        })?;

        self.size = Size::new(size.width as usize, size.height as usize);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Texture {
    pub id: GLuint,
    pub internal: GLuint,
    pub kind: GLuint,
    pub color: GLuint,
}

impl Texture {
    fn new(id: GLuint, internal: GLuint, kind: GLuint, color: GLuint) -> Self {
        Texture {
            id,
            internal,
            kind,
            color,
        }
    }
}

pub trait ToTexture<T: Type, C: Color> {
    const COLOR: GLuint;
    const KIND: GLuint;

    fn get_meta(&self) -> &Meta<T, C>;

    fn get_data(&self) -> &[T];

    fn to_texture(&self) -> Result<Texture, Error> {
        let mut texture_id: GLuint = 0;

        unsafe {
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
        }

        let internal = match (Self::COLOR, Self::KIND) {
            (gl::RED, gl::BYTE) => gl::R8,
            (gl::RED, gl::SHORT) => gl::R16,
            (gl::RED, gl::UNSIGNED_BYTE) => gl::R8,
            (gl::RED, gl::UNSIGNED_SHORT) => gl::R16,
            (gl::RED, gl::FLOAT) => gl::R32F,
            (gl::RG, gl::BYTE) => gl::RG8,
            (gl::RG, gl::SHORT) => gl::RG16,
            (gl::RG, gl::UNSIGNED_BYTE) => gl::RG8,
            (gl::RG, gl::UNSIGNED_SHORT) => gl::RG16,
            (gl::RG, gl::FLOAT) => gl::RG32F,
            (gl::RGB, gl::BYTE) => gl::RGB8,
            (gl::RGB, gl::SHORT) => gl::RGB16,
            (gl::RGB, gl::UNSIGNED_BYTE) => gl::RGB,
            (gl::RGB, gl::UNSIGNED_SHORT) => gl::RGB16,
            (gl::RGB, gl::FLOAT) => gl::RGB32F,
            (gl::RGBA, gl::BYTE) => gl::RGBA,
            (gl::RGBA, gl::SHORT) => gl::RGBA16,
            (gl::RGBA, gl::UNSIGNED_BYTE) => gl::RGBA,
            (gl::RGBA, gl::UNSIGNED_SHORT) => gl::RGBA16,
            (gl::RGBA, gl::FLOAT) => gl::RGBA32F,
            _ => return Err(Error::InvalidType),
        };

        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal as i32,
                self.get_meta().width() as i32,
                self.get_meta().height() as i32,
                0,
                Self::COLOR,
                Self::KIND,
                self.get_data().as_ptr() as *const _,
            );
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        return Ok(Texture::new(texture_id, internal, Self::KIND, Self::COLOR));
    }
}

macro_rules! to_texture {
    ($t:ty, $c:ty, $kind:expr, $color:expr) => {
        impl ToTexture<$t, $c> for Image<$t, $c> {
            const COLOR: GLuint = $color;
            const KIND: GLuint = $kind;

            fn get_meta(&self) -> &Meta<$t, $c> {
                &self.meta
            }

            fn get_data(&self) -> &[$t] {
                &self.data
            }
        }
    };
}

to_texture!(f32, Rgb, gl::FLOAT, gl::RGB);
to_texture!(f32, Rgba, gl::FLOAT, gl::RGBA);
to_texture!(u16, Rgb, gl::UNSIGNED_SHORT, gl::RGB);
to_texture!(u16, Rgba, gl::UNSIGNED_SHORT, gl::RGBA);
to_texture!(u8, Rgb, gl::UNSIGNED_BYTE, gl::RGB);
to_texture!(u8, Rgba, gl::UNSIGNED_BYTE, gl::RGBA);
