use crate::*;

type EPoint<T> = euclid::Point2D<T, T>;
pub struct Transform(pub euclid::Transform2D<f64, f64, f64>);

impl Filter for Transform {
    fn compute_at(&self, pt: Point, c: usize, input: &[&Image<impl Type, impl Color>]) -> f64 {
        let pt = EPoint::new(pt.x as f64, pt.y as f64);
        let dest = self.0.transform_point(pt);
        (input[0].get_f((dest.x.floor() as usize, dest.y.floor() as usize), c)
            + input[0].get_f((dest.x.ceil() as usize, dest.y.ceil() as usize), c))
            / 2.
    }
}

#[inline]
pub fn rotate(deg: f64, center: (f64, f64)) -> Transform {
    Transform(
        euclid::Transform2D::rotation(euclid::Angle::degrees(-deg))
            .pre_translate(euclid::Vector2D::new(-center.0, -center.1))
            .then_translate(euclid::Vector2D::new(center.0, center.1)),
    )
}

#[inline]
pub fn scale(x: f64, y: f64) -> Transform {
    Transform(euclid::Transform2D::scale(1.0 / x, 1.0 / y))
}

#[inline]
pub fn resize(from: Size, to: Size) -> Transform {
    Transform(euclid::Transform2D::scale(
        from.width as f64 / to.width as f64,
        from.height as f64 / to.height as f64,
    ))
}

pub fn rotate90(from: Size, to: Size) -> Transform {
    let dwidth = to.width as f64;
    let height = from.height as f64;
    rotate(90., (dwidth / 2., height / 2.))
}

pub fn rotate180(src: Size) -> Transform {
    let dwidth = src.width as f64;
    let height = src.height as f64;
    rotate(180., (dwidth / 2., height / 2.))
}

pub fn rotate270(from: Size, to: Size) -> Transform {
    let width = to.height as f64;
    let dheight = from.width as f64;
    rotate(270., (width / 2., dheight / 2.))
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
        let mut dest: Image<f32, Rgb> = a.new_like();
        rotate90(a.size(), dest.size()).eval(&[&a], &mut dest);
        assert!(dest.save("images/test-rotate90.jpg").is_ok())
    }

    #[test]
    fn test_rotate180() {
        let a = Image::<f32, Rgb>::open("images/A.exr").unwrap();
        let mut dest = a.new_like();
        rotate180(a.size()).eval(&[&a], &mut dest);
        assert!(dest.save("images/test-rotate180.jpg").is_ok())
    }

    #[test]
    fn test_rotate270() {
        let a = Image::<f32, Rgb>::open("images/A.exr").unwrap();
        let mut dest: Image<f32, Rgb> = a.new_like();
        rotate270(a.size(), dest.size()).eval(&[&a], &mut dest);
        assert!(dest.save("images/test-rotate270.jpg").is_ok())
    }

    #[test]
    fn test_scale() {
        let a = Image::<u8, Rgb>::open("images/A.exr").unwrap();
        let mut dest: Image<f32, Rgb> = Image::new(a.size() * 2);
        scale(2., 2.).eval(&[&a], &mut dest);
        assert!(dest.save("images/test-scale.jpg").is_ok())
    }

    #[test]
    fn test_scale_resize() {
        let a = Image::<u8, Rgb>::open("images/A.exr").unwrap();
        let mut dest0: Image<u16, Rgb> = Image::new(a.size() * 2);
        let mut dest1: Image<u16, Rgb> = Image::new(a.size() * 2);
        scale(2., 2.).eval(&[&a], &mut dest0);
        resize(a.size(), a.size() * 2).eval(&[&a], &mut dest1);
        assert_eq!(dest0, dest1);
    }
}
