use crate::*;

use gl::types::*;

pub use glutin::{
    self,
    event::{
        ElementState, Event, KeyboardInput, ModifiersState, ScanCode, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{WindowBuilder, WindowId},
    ContextCurrentState as CurrentState, NotCurrent, PossiblyCurrent, WindowedContext as Context,
};

use glutin::platform::run_return::EventLoopExtRunReturn;

/// Window is used to display images
pub struct Window<T: Type, C: Color> {
    /// Window ID
    id: WindowId,

    /// OpenGL context
    context: Option<Context<NotCurrent>>,

    /// Window texture
    pub texture: Texture,

    /// Window image
    image: Image<T, C>,

    /// OpenGL framebuffer
    pub framebuffer: GLuint,

    /// Window's current size
    size: Size,

    /// Window dirty state, the window should be redrawn when set to true
    dirty: bool,

    /// Current mouse position
    position: Point,

    closed: bool,
    data: Option<Box<dyn std::any::Any>>,
}

/// `WindowSet` allows for multiple windows to run at once
pub struct WindowSet<T: Type, C: Color>(std::collections::BTreeMap<WindowId, Window<T, C>>);

impl<T: Type, C: Color> Default for WindowSet<T, C> {
    fn default() -> Self {
        WindowSet(std::collections::BTreeMap::new())
    }
}

impl<T: Type, C: Color> WindowSet<T, C> {
    /// Create new window set
    pub fn new() -> WindowSet<T, C> {
        Default::default()
    }

    /// Add an existing window
    pub fn add(&mut self, window: Window<T, C>) -> Result<WindowId, Error> {
        let id = window.id;
        self.0.insert(id, window);
        Ok(id)
    }

    /// Create a new window and add it
    pub fn create<X>(
        &mut self,
        event_loop: &EventLoop<X>,
        image: Image<T, C>,
        window_builder: WindowBuilder,
    ) -> Result<WindowId, Error>
    where
        Image<T, C>: ToTexture<T, C>,
    {
        let window = Window::new(event_loop, image, window_builder)?;
        self.add(window)
    }

    /// Get window by ID
    pub fn get(&self, window_id: &WindowId) -> Option<&Window<T, C>> {
        self.0.get(window_id)
    }

    /// Get mutable window by ID
    pub fn get_mut(&mut self, window_id: &WindowId) -> Option<&mut Window<T, C>> {
        self.0.get_mut(window_id)
    }

    /// Remove a window and return it
    pub fn remove(&mut self, window_id: &WindowId) -> Option<Window<T, C>> {
        self.0.remove(window_id)
    }

    /// Iterate over all windows
    pub fn iter(&self) -> impl Iterator<Item = (&WindowId, &Window<T, C>)> {
        self.0.iter()
    }

    /// Iterate over mutable windows
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&WindowId, &mut Window<T, C>)> {
        self.0.iter_mut()
    }

    /// Convert into an interator over windows
    pub fn iter_windows(self) -> impl Iterator<Item = Window<T, C>> {
        self.0.into_iter().map(|(_, v)| v)
    }

    /// Run the event loop until all windows are closed
    pub fn run<X, F: FnMut(&mut WindowSet<T, C>, Event<'_, X>) -> Option<ControlFlow>>(
        &mut self,
        event_loop: &mut EventLoop<X>,
        mut event_handler: F,
    ) {
        event_loop.run_return(move |event, _target, cf| {
            *cf = ControlFlow::Poll;

            match &event {
                Event::LoopDestroyed => {
                    *cf = ControlFlow::Exit;
                    return;
                }
                Event::WindowEvent { event, window_id } => match event {
                    WindowEvent::CloseRequested => {
                        if let Some(window) = self.get_mut(window_id) {
                            window.close();
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        if let Some(window) = self.get_mut(window_id) {
                            window.position = window
                                .fix_mouse_position((position.x as usize, position.y as usize));
                        }
                    }
                    _ => (),
                },
                Event::RedrawRequested(window_id) => {
                    if let Some(window) = self.get_mut(window_id) {
                        window.draw().unwrap();
                    }
                }
                _ => (),
            }

            if let Some(new_cf) = event_handler(self, event) {
                *cf = new_cf;
            }

            let mut open = 0;
            for (_, window) in self.0.iter_mut() {
                if window.closed {
                    continue;
                }

                open += 1;
                if window.dirty {
                    let _ = window.draw();
                }
            }

            if open == 0 {
                *cf = ControlFlow::Exit;
            }
        });
    }
}

impl<'a, T: Type, C: Color> Window<T, C> {
    /// Create a new window
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
        let id = context.window().id();

        Ok(Window {
            id,
            context: Some(context),
            image,
            texture,
            framebuffer,
            position: Point::default(),
            size: Size::new(size.width as usize, size.height as usize),
            dirty: true,
            closed: false,
            data: None,
        })
    }

    /// Set user data
    pub fn set_data<X: std::any::Any>(&mut self, data: X) {
        self.data = Some(Box::new(data))
    }

    /// Get user data
    pub fn data<X: std::any::Any>(&self) -> Option<&X> {
        self.data.as_deref().and_then(|x| x.downcast_ref())
    }

    /// Get mutable user data
    pub fn data_mut<X: std::any::Any>(&mut self) -> Option<&mut X> {
        self.data.as_deref_mut().and_then(|x| x.downcast_mut())
    }

    /// Execute a callback with a current OpenGL context
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
            return t;
        }

        Err(Error::GlutinContext(glutin::ContextError::ContextLost))
    }

    /// Get current mouse position
    pub fn mouse_position(&self) -> Point {
        self.position
    }

    /// Get mouse position  relative to image based on window mouse position
    pub fn fix_mouse_position(&self, pt: impl Into<Point>) -> Point {
        let pt = pt.into();
        let ratio = (self.size.width as f64 / self.image.meta.width() as f64)
            .min(self.size.height as f64 / self.image.meta.height() as f64);
        let display_width = (self.image.meta.width() as f64 * ratio) as usize;
        let display_height = (self.image.meta.height() as f64 * ratio) as usize;
        let x = self.size.width.saturating_sub(display_width) / 2;
        let y = self.size.height.saturating_sub(display_height) / 2;

        self.scale_mouse_position(pt, x, y, display_width, display_height, ratio)
    }

    fn scale_mouse_position(
        &self,
        pt: impl Into<Point>,
        x: usize,
        y: usize,
        display_width: usize,
        display_height: usize,
        ratio: f64,
    ) -> Point {
        let mut pt = pt.into();

        pt.x = pt.x.saturating_sub(x);
        pt.y = pt.y.saturating_sub(y);

        if pt.x >= display_width {
            pt.x = display_width.saturating_sub(1);
        }

        if pt.y >= display_height {
            pt.y = display_height.saturating_sub(1);
        }

        Point::new(
            (pt.x as f64 / ratio) as usize,
            (pt.y as f64 / ratio) as usize,
        )
    }

    /// Convert `Window` into `Image`
    pub fn into_image(self) -> Image<T, C> {
        self.image
    }

    /// Get image
    pub fn image(&self) -> &Image<T, C> {
        &self.image
    }

    /// Get mutable image
    pub fn image_mut(&mut self) -> &mut Image<T, C> {
        &mut self.image
    }

    /// Mark window as dirty - this tells the event handler to call `draw` on the next iteration
    pub fn mark_as_dirty(&mut self) {
        self.dirty = true;
    }

    /// Return true when the window is marked as dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Return true when the window is closed
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Show a window after being closed
    pub fn open(&mut self) {
        if let Some(ctx) = &self.context {
            ctx.window().set_visible(true)
        }
        self.closed = false
    }

    /// Close a window
    pub fn close(&mut self) {
        if let Some(ctx) = &self.context {
            ctx.window().set_visible(false)
        }
        self.closed = true
    }

    /// Update the texture with data from the window's image
    pub fn draw(&mut self) -> Result<(), Error> {
        let meta = self.image.meta();
        let image = self.image.data.as_ptr();
        let texture = self.texture;
        let framebuffer = self.framebuffer;
        let size = self.size;
        let size = self.with_current_context(|ctx| {
            let ratio = (size.width as f64 / meta.width() as f64)
                .min(size.height as f64 / meta.height() as f64);
            let display_width = (meta.width() as f64 * ratio) as usize;
            let display_height = (meta.height() as f64 * ratio) as usize;
            let x = size.width.saturating_sub(display_width) / 2;
            let y = size.height.saturating_sub(display_height) / 2;
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
                    x as i32 + display_width as i32,
                    y as i32 + display_height as i32,
                    gl::COLOR_BUFFER_BIT,
                    gl::NEAREST,
                );
                gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
            }
            ctx.swap_buffers()?;
            Ok(ctx.window().inner_size())
        })?;

        self.size = Size::new(size.width as usize, size.height as usize);
        self.dirty = false;
        Ok(())
    }
}

/// Wraps OpenGL textures
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Texture {
    /// OpenGL texture id
    pub id: GLuint,

    /// OpenGL data type
    pub internal: GLuint,

    /// OpenGL type
    pub kind: GLuint,

    /// OpenGL color
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

/// ToTexture is defined for image types that can be converted to OpenGL textures
pub trait ToTexture<T: Type, C: Color> {
    /// OpenGL color
    const COLOR: GLuint;

    /// OpenGL type
    const KIND: GLuint;

    /// Get metadata
    fn get_meta(&self) -> &Meta<T, C>;

    /// Get data buffer
    fn get_data(&self) -> &[T];

    /// Convert to texture
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
        Ok(Texture::new(texture_id, internal, Self::KIND, Self::COLOR))
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
to_texture!(i16, Rgb, gl::SHORT, gl::RGB);
to_texture!(i16, Rgba, gl::SHORT, gl::RGBA);
to_texture!(u8, Rgb, gl::UNSIGNED_BYTE, gl::RGB);
to_texture!(u8, Rgba, gl::UNSIGNED_BYTE, gl::RGBA);

/// Show an image and exit when ESC is pressed
pub fn show<
    T: Type,
    C: Color,
    F: FnMut(&mut WindowSet<T, C>, Event<'_, ()>) -> Option<ControlFlow>,
>(
    title: impl AsRef<str>,
    image: Image<T, C>,
    mut f: F,
) -> Result<Image<T, C>, Error>
where
    Image<T, C>: ToTexture<T, C>,
{
    let mut event_loop = EventLoop::new();
    let mut windows = WindowSet::new();
    let id = windows.create(
        &event_loop,
        image,
        WindowBuilder::new().with_title(title.as_ref()),
    )?;

    windows.run(&mut event_loop, move |windows, event| {
        if let Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } = &event
        {
            if input.scancode == 0x01 {
                return Some(ControlFlow::Exit);
            }
        }
        f(windows, event)
    });

    if let Some(window) = windows.remove(&id) {
        return Ok(window.into_image());
    }

    Err(Error::Message("Cannot find window".into()))
}

/// Show multiple images and exit when ESC is pressed
pub fn show_all<
    T: Type,
    C: Color,
    F: FnMut(&mut WindowSet<T, C>, Event<'_, ()>) -> Option<ControlFlow>,
>(
    images: impl IntoIterator<Item = (impl Into<String>, Image<T, C>)>,
    mut f: F,
) -> Result<Vec<Image<T, C>>, Error>
where
    Image<T, C>: ToTexture<T, C>,
{
    let mut event_loop = EventLoop::new();
    let mut windows = WindowSet::new();

    for (title, image) in images.into_iter() {
        windows.create(
            &event_loop,
            image,
            WindowBuilder::new().with_title(title.into()),
        )?;
    }

    windows.run(&mut event_loop, move |windows, event| {
        if let Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } = &event
        {
            if input.scancode == 0x01 {
                return Some(ControlFlow::Exit);
            }
        }
        f(windows, event)
    });

    Ok(windows.iter_windows().map(|w| w.into_image()).collect())
}
