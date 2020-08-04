pub use half::f16;

mod color;
mod image;
mod pixel;
mod r#type;

pub mod oiio;

pub use color::{Color, Gray, Rgb, Rgba};
pub use image::{Image, Meta};
pub use oiio::ImageBuf;
pub use pixel::Pixel;
pub use r#type::Type;

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        let im = Image::<f32, Rgb>::open("images/A.exr").unwrap();
        assert!(im.save("images/out.png"));
    }
}
