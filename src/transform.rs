use crate::*;

pub type Point<T> = euclid::Point2D<T, T>;
pub struct Transform(pub euclid::Transform2D<f64, f64, f64>);

impl Filter for Transform {
    fn compute_at(
        &self,
        x: usize,
        y: usize,
        c: usize,
        input: &[&Image<impl Type, impl Color>],
    ) -> f64 {
        let pt = Point::new(x as f64, y as f64);
        let dest = self.0.transform_point(pt);
        (input[0].get_f(dest.x.floor() as usize, dest.y.floor() as usize, c)
            + input[0].get_f(dest.x.ceil() as usize, dest.y.ceil() as usize, c))
            / 2.
    }
}

#[inline]
pub fn rotate(deg: f64, center: Point<f64>) -> Transform {
    Transform(
        euclid::Transform2D::rotation(euclid::Angle::degrees(-deg))
            .pre_translate(euclid::Vector2D::new(-center.x, -center.y))
            .then_translate(euclid::Vector2D::new(center.x, center.y)),
    )
}

#[inline]
pub fn scale(x: f64, y: f64) -> Transform {
    Transform(euclid::Transform2D::scale(1.0 / x, 1.0 / y))
}

#[inline]
pub fn resize(src: &Image<impl Type, impl Color>, x: usize, y: usize) -> Transform {
    Transform(euclid::Transform2D::scale(
        src.width() as f64 / x as f64,
        src.height() as f64 / y as f64,
    ))
}

pub fn rotate90(
    dest: &Image<impl Type, impl Color>,
    src: &Image<impl Type, impl Color>,
) -> Transform {
    let dwidth = dest.width() as f64;
    let height = src.height() as f64;
    rotate(90., Point::new(dwidth / 2., height / 2.))
}

pub fn rotate180(src: &Image<impl Type, impl Color>) -> Transform {
    let dwidth = src.width() as f64;
    let height = src.height() as f64;
    rotate(180., Point::new(dwidth / 2., height / 2.))
}

pub fn rotate270(
    dest: &Image<impl Type, impl Color>,
    src: &Image<impl Type, impl Color>,
) -> Transform {
    let width = dest.height() as f64;
    let dheight = src.width() as f64;
    rotate(270., Point::new(width / 2., dheight / 2.))
}

#[cfg(test)]
mod test {
    use crate::{
        transform::{resize, rotate180, rotate270, rotate90, scale},
        Filter, Image, Rgb,
    };

    #[test]
    fn test_rotate90() {
        let a = Image::<f32, Rgb>::open("images/A.exr").unwrap();
        let mut dest: Image<f32, Rgb> = Image::new(a.height(), a.width());
        rotate90(&dest, &a).eval(&mut dest, &[&a]);
        assert!(dest.save("images/test-rotate90.jpg").is_ok())
    }

    #[test]
    fn test_rotate180() {
        let a = Image::<f32, Rgb>::open("images/A.exr").unwrap();
        let mut dest = a.new_like();
        rotate180(&a).eval(&mut dest, &[&a]);
        assert!(dest.save("images/test-rotate180.jpg").is_ok())
    }

    #[test]
    fn test_rotate270() {
        let a = Image::<f32, Rgb>::open("images/A.exr").unwrap();
        let mut dest: Image<f32, Rgb> = Image::new(a.height(), a.width());
        rotate270(&dest, &a).eval(&mut dest, &[&a]);
        assert!(dest.save("images/test-rotate270.jpg").is_ok())
    }

    #[test]
    fn test_scale() {
        let a = Image::<u8, Rgb>::open("images/A.exr").unwrap();
        let mut dest: Image<f32, Rgb> = Image::new(a.width() * 2, a.height() * 2);
        scale(2., 2.).eval(&mut dest, &[&a]);
        assert!(dest.save("images/test-scale.jpg").is_ok())
    }

    #[test]
    fn test_scale_resize() {
        let a = Image::<u8, Rgb>::open("images/A.exr").unwrap();
        let mut dest0: Image<u16, Rgb> = Image::new(a.width() * 2, a.height() * 2);
        let mut dest1: Image<u16, Rgb> = Image::new(a.width() * 2, a.height() * 2);
        scale(2., 2.).eval(&mut dest0, &[&a]);
        resize(&a, a.width() * 2, a.height() * 2).eval(&mut dest1, &[&a]);
        assert_eq!(dest0, dest1);
    }
}
