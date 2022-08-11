use std::io::Read;

use crate::*;

/// Defines size. location and contents for drawing text
pub struct Text<'a, C: Color> {
    text: String,
    color: Pixel<C>,
    width: Option<usize>,
    height: Option<usize>,
    size: f32,
    font: Font<'a>,
}

pub use rusttype::Font;

/// Load font from disk
pub fn load_font(path: impl AsRef<std::path::Path>) -> Result<Font<'static>, Error> {
    let mut font_file = std::fs::File::open(path)?;
    let mut data = Vec::new();
    font_file.read_to_end(&mut data)?;
    match Font::try_from_vec(data) {
        Some(x) => Ok(x),
        None => Err(Error::Message("Unable to load font".into())),
    }
}

/// Convert from slice to a `Font`
pub fn font(data: &[u8]) -> Result<Font<'_>, Error> {
    match Font::try_from_bytes(data) {
        Some(x) => Ok(x),
        None => Err(Error::Message("Unable to load font".into())),
    }
}

impl<'a, C: Color> Text<'a, C> {
    /// Create new `Text` from the given font, text and size (in pixels)
    pub fn new(font: Font<'a>, text: impl Into<String>, size: f32) -> Self {
        Text {
            text: text.into(),
            color: Pixel::new(),
            width: None,
            height: None,
            size,
            font,
        }
    }

    /// Set bounding width
    pub fn with_max_width(mut self, w: usize) -> Self {
        self.width = Some(w);
        self
    }

    /// Set bounding height
    pub fn with_max_height(mut self, h: usize) -> Self {
        self.height = Some(h);
        self
    }

    /// Set text color
    pub fn with_color(mut self, color: Pixel<Rgb>) -> Self {
        self.color = color.convert();
        self
    }

    /// Draw text on image
    pub fn draw<T: Type>(&self, image: &mut Image<T, C>, pos: impl Into<Point>) {
        let pos = pos.into();
        let scale = rusttype::Scale::uniform(self.size);
        let layout = self.font.layout(
            &self.text,
            scale,
            rusttype::point(pos.x as f32, pos.y as f32),
        );

        let mut data = vec![0.convert(); C::CHANNELS];
        let mut tmp = Pixel::new();
        for glyph in layout {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    let pt = (
                        (x as isize + bounding_box.min.x as isize) as usize,
                        (y as isize + bounding_box.min.y as isize) as usize,
                    );
                    if image.at(pt, &mut data) {
                        tmp.copy_from_slice(&data);
                        let color = &tmp * (1.0 - v) + &self.color * v;
                        image.set_pixel(pt, &color);
                    }
                });
            }
        }
    }
}
