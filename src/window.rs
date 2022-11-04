use crate::*;

use gl::types::*;

use glfw::Context as GlfwContext;
pub use glfw::{Action, Key, Modifiers, MouseButton, WindowEvent as Event, WindowId};

/// Multiple windows
pub struct WindowSet<T: Type, C: Color> {
    /// GLFW handle
    pub glfw: std::cell::RefCell<glfw::Glfw>,

    /// Mapping from `WindowId` to `Window`
    pub windows: std::collections::BTreeMap<glfw::WindowId, Window<T, C>>,
}

unsafe impl<T: Type, C: Color> Send for WindowSet<T, C> {}
unsafe impl<T: Type, C: Color> Sync for WindowSet<T, C> {}

/// Window is used to display images
pub struct Window<T: Type, C: Color> {
    id: WindowId,

    /// GLFW Window
    inner: glfw::Window,

    /// Event stream
    events: std::sync::mpsc::Receiver<(f64, Event)>,

    image: Image<T, C>,

    /// Window texture
    pub texture: Texture,

    /// OpenGL framebuffer
    pub framebuffer: GLuint,

    /// Window's current size
    size: Size,

    /// Current mouse position
    position: Point,

    /// `true` when the window is closed
    closed: bool,

    /// User data
    data: Option<Box<dyn std::any::Any>>,

    dirty: bool,
}

impl<T: Type, C: Color> WindowSet<T, C> {
    /// Create a new context
    pub fn new() -> Result<Self, Error> {
        let glfw = std::cell::RefCell::new(glfw::init::<()>(glfw::FAIL_ON_ERRORS)?);
        Ok(WindowSet {
            glfw,
            windows: std::collections::BTreeMap::new(),
        })
    }

    /// Create a new context with error callback
    pub fn new_with_error_callback<E: 'static>(
        f: glfw::Callback<fn(glfw::Error, String, &E), E>,
    ) -> Result<Self, Error> {
        let glfw = std::cell::RefCell::new(glfw::init::<E>(Some(f))?);
        Ok(WindowSet {
            glfw,
            windows: std::collections::BTreeMap::new(),
        })
    }

    /// Access `Glfw` handle
    pub fn glfw_context(&self) -> std::cell::Ref<glfw::Glfw> {
        self.glfw.borrow()
    }

    /// Access mutable `Glfw` handle
    pub fn glfw_context_mut(&self) -> std::cell::RefMut<glfw::Glfw> {
        self.glfw.borrow_mut()
    }

    /// Add an existing window
    pub fn add(&mut self, window: Window<T, C>) -> Result<WindowId, Error> {
        let id = window.id;
        self.windows.insert(id, window);
        Ok(id)
    }

    /// Create a new window and add it
    pub fn create(&mut self, title: impl AsRef<str>, image: Image<T, C>) -> Result<WindowId, Error>
    where
        Image<T, C>: ToTexture<T, C>,
    {
        let window = Window::new(self, image, title)?;
        self.add(window)
    }

    /// Get window by ID
    pub fn get(&self, window_id: &WindowId) -> Option<&Window<T, C>> {
        self.windows.get(window_id)
    }

    /// Get mutable window by ID
    pub fn get_mut(&mut self, window_id: &WindowId) -> Option<&mut Window<T, C>> {
        self.windows.get_mut(window_id)
    }

    /// Remove a window and return it
    pub fn remove(&mut self, window_id: &WindowId) -> Option<Window<T, C>> {
        self.windows.remove(window_id)
    }

    /// Iterate over all windows
    pub fn iter(&self) -> impl Iterator<Item = (&WindowId, &Window<T, C>)> {
        self.windows.iter().filter(|(_, x)| !x.is_closed())
    }

    /// Iterate over mutable windows
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&WindowId, &mut Window<T, C>)> {
        self.windows.iter_mut().filter(|(_, x)| !x.is_closed())
    }

    /// Convert into an interator over windows
    pub fn iter_windows(&self) -> impl Iterator<Item = &Window<T, C>> {
        self.windows.values().filter(|x| !x.is_closed())
    }

    /// Convert into an interator over mutable windows
    pub fn iter_windows_mut(&mut self) -> impl Iterator<Item = &mut Window<T, C>> {
        self.windows.values_mut().filter(|x| !x.is_closed())
    }

    /// Convert into an iterator over images
    pub fn into_images(self) -> impl Iterator<Item = Image<T, C>> {
        self.windows.into_values().map(|x| x.into_image())
    }

    /// Returns false when there are no more open windows
    pub fn step<F: FnMut(&mut Window<T, C>, Option<Event>) -> Result<(), Error>>(
        &mut self,
        mut event_handler: F,
    ) -> Result<bool, Error> {
        let mut count = 0;

        for window in self.iter_windows_mut() {
            count += 1;
            window.handle_events(&mut event_handler)?;
        }

        Ok(count > 0)
    }

    /// Poll for new events with the given timeout
    pub fn wait_events(&self, timeout: f64) {
        self.glfw_context_mut().wait_events_timeout(timeout);
    }

    /// Run the event loop until all windows are closed
    pub fn run<F: FnMut(&mut Window<T, C>, Option<Event>) -> Result<(), Error>>(
        &mut self,
        mut event_handler: F,
    ) -> Result<(), Error> {
        while self.step(&mut event_handler)? {
            self.wait_events(0.1)
        }
        Ok(())
    }
}

impl<T: Type, C: Color> Window<T, C> {
    /// Create a new window
    pub fn new(
        context: &WindowSet<T, C>,
        image: Image<T, C>,
        title: impl AsRef<str>,
    ) -> Result<Window<T, C>, Error>
    where
        Image<T, C>: ToTexture<T, C>,
    {
        let (mut inner, events) = match context.glfw.borrow_mut().create_window(
            image.width() as u32,
            image.height() as u32,
            title.as_ref(),
            glfw::WindowMode::Windowed,
        ) {
            Some(x) => x,
            None => return Err(Error::Message("Unable to open window".into())),
        };
        inner.set_all_polling(true);
        inner.make_current();

        gl::load_with(|ptr| context.glfw.borrow().get_proc_address_raw(ptr));

        let mut framebuffer = 0;
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

        let (width, height) = inner.get_size();
        let id = inner.window_id();
        let size = Size::new(width as usize, height as usize);

        let mut window = Window {
            id,
            inner,
            events,
            position: Point::default(),
            size,
            closed: false,
            data: None,
            texture,
            framebuffer,
            image,
            dirty: false,
        };

        window.draw()?;
        Ok(window)
    }

    /// Get window ID
    pub fn id(&self) -> WindowId {
        self.id
    }

    /// Mark window as dirty, this will trigger a draw on the next iteration
    pub fn mark_as_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if window is dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Get pending events for a window
    pub fn events(&mut self) -> Result<Vec<Event>, Error> {
        let mut events = vec![];
        for (_, event) in glfw::flush_messages(&self.events) {
            let event = match event {
                Event::CursorPos(x, y) => {
                    let pt = self.fix_mouse_position((x as usize, y as usize));
                    self.position = pt;
                    Event::CursorPos(pt.x as f64, pt.y as f64)
                }
                Event::Size(w, h) => {
                    self.size = Size::new(w as usize, h as usize);
                    Event::Size(w, h)
                }
                Event::Close => {
                    self.close();
                    break;
                }
                event => event,
            };

            events.push(event);
        }
        Ok(events)
    }

    /// Handle events using `event_handler`
    pub fn handle_events<F: FnMut(&mut Window<T, C>, Option<Event>) -> Result<(), Error>>(
        &mut self,
        mut event_handler: F,
    ) -> Result<(), Error> {
        let events = self.events()?;

        if events.is_empty() {
            event_handler(self, None)?;
        } else {
            for event in events {
                event_handler(self, Some(event))?;
            }
        }

        if self.is_dirty() {
            self.draw()?;
        }

        Ok(())
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
        self.mark_as_dirty();
        &mut self.image
    }

    /// Return true when the window is closed
    pub fn is_closed(&self) -> bool {
        self.closed || self.inner.should_close()
    }

    /// Close a window
    pub fn close(&mut self) {
        self.inner.set_should_close(true);
        self.inner.hide();
        self.closed = true;
    }

    /// Update the texture with data from the window's image
    pub fn draw(&mut self) -> Result<(), Error> {
        self.inner.make_current();
        let meta = self.image.meta();
        let image = self.image.data.as_ptr();
        let texture = self.texture;
        let framebuffer = self.framebuffer;
        let size = self.size;
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

        self.inner.swap_buffers();
        self.dirty = false;
        Ok(())
    }
}

/// Wraps OpenGL textures
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Default)]
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
pub fn show<T: Type, C: Color, F: FnMut(&mut Window<T, C>, Option<Event>) -> Result<(), Error>>(
    title: impl AsRef<str>,
    image: Image<T, C>,
    mut f: F,
) -> Result<Image<T, C>, Error>
where
    Image<T, C>: ToTexture<T, C>,
{
    let mut windows = WindowSet::new()?;
    let id = windows.create(title, image)?;

    windows.run(|window, event| {
        if let Some(Event::Key(k, _, action, _)) = event {
            if k == Key::Escape && action == Action::Press {
                window.close();
            }
        }
        f(window, event)
    })?;

    if let Some(window) = windows.remove(&id) {
        return Ok(window.into_image());
    }

    Err(Error::Message("Cannot find window".into()))
}

/// Show multiple images and exit when ESC is pressed
pub fn show_all<
    T: Type,
    C: Color,
    F: FnMut(&mut Window<T, C>, Option<Event>) -> Result<(), Error>,
>(
    images: impl IntoIterator<Item = (impl Into<String>, Image<T, C>)>,
    mut f: F,
) -> Result<Vec<Image<T, C>>, Error>
where
    Image<T, C>: ToTexture<T, C>,
{
    let mut windows = WindowSet::new()?;

    for (title, image) in images.into_iter() {
        windows.create(title.into(), image)?;
    }

    windows.run(|window, event| {
        if let Some(Event::Key(k, _, action, _)) = event {
            if k == Key::Escape && action == Action::Press {
                window.close();
            }
        }
        f(window, event)
    })?;

    Ok(windows.into_images().collect())
}
