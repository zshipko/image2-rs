use std::io::Read;

use crate::*;

// Re-export `Font` type
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

/// Get size of text to be drawn
pub fn width<'a>(text: impl AsRef<str>, font: &Font<'a>, size: f32) -> usize {
    if text.as_ref().is_empty() {
        return 0;
    }

    let scale = rusttype::Scale::uniform(size);
    let layout = font.layout(text.as_ref(), scale, rusttype::point(0., 0.));
    let mut w = 0;

    for glyph in layout {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            w = bounding_box.max.x as usize;
        }
    }

    w
}

impl<T: Type, C: Color> Image<T, C> {
    /// Draw text on image
    pub fn draw_text<'a>(
        &mut self,
        text: impl AsRef<str>,
        font: &Font<'a>,
        size: f32,
        pos: impl Into<Point>,
        color: &Pixel<C>,
    ) {
        let pos = pos.into();
        let scale = rusttype::Scale::uniform(size);
        let layout = font.layout(
            text.as_ref(),
            scale,
            rusttype::point(pos.x as f32, pos.y as f32),
        );

        let mut data = vec![T::from_f64(0.0); C::CHANNELS];
        let mut tmp = Pixel::new();
        for glyph in layout {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    let pt = (
                        (x as isize + bounding_box.min.x as isize) as usize,
                        (y as isize + bounding_box.min.y as isize) as usize,
                    );
                    if self.at(pt, &mut data) {
                        tmp.copy_from_slice(&data);
                        let color = &tmp * (1.0 - v) + color * v;
                        self.set_pixel(pt, &color);
                    }
                });
            }
        }
    }
}
