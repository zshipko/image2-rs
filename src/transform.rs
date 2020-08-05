use crate::*;
use euclid;

pub type Point<T> = euclid::Point2D<T, T>;
pub struct Transform(pub euclid::Transform2D<f64, f64, f64>);

impl<C: Color> Filter<C> for Transform {
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, C>]) -> f64 {
        let pt = Point::new(x as f64, y as f64);
        let dest = self.0.transform_point(pt);
        (input[0].get_f(dest.x.floor() as usize, dest.y.floor() as usize, c)
            + input[0].get_f(dest.x.ceil() as usize, dest.y.ceil() as usize, c))
            / 2.
    }
}

#[inline]
pub fn rotate<C: Color>(
    dest: &mut Image<impl Type, C>,
    src: &Image<impl Type, C>,
    deg: f64,
    center: Point<f64>,
) {
    let filter = Transform(
        euclid::Transform2D::rotation(euclid::Angle::degrees(deg))
            .pre_translate(euclid::Vector2D::new(-center.x, -center.y))
            .then_translate(euclid::Vector2D::new(center.x, center.y)),
    );

    filter.eval(dest, &[src])
}

#[inline]
pub fn scale<T: Type, C: Color>(dest: &mut Image<T, C>, src: &Image<T, C>, x: f64, y: f64) {
    let filter = Transform(euclid::Transform2D::scale(1.0 / x, 1.0 / y));

    filter.eval(dest, &[src])
}

#[inline]
pub fn resize<T: Type, C: Color>(
    dest: &mut Image<T, C>,
    src: &Image<T, C>,
    mut x: usize,
    mut y: usize,
) {
    if x == 0 && y == 0 {
        x = dest.width();
        y = dest.height();
    } else if x == 0 {
        y = x * src.height() / src.width()
    } else if y == 0 {
        x = y * src.width() / src.height()
    }

    let filter = Transform(euclid::Transform2D::scale(
        src.width() as f64 / x as f64,
        src.height() as f64 / y as f64,
    ));

    filter.eval(dest, &[src])
}

pub fn rotate90<T: Type, C: Color>(dest: &mut Image<T, C>, src: &Image<T, C>) {
    let dwidth = src.width() as f64;
    let height = src.height() as f64;
    rotate(dest, src, 90., Point::new(dwidth / 2., height / 2.));
}

pub fn rotate180<T: Type, C: Color>(dest: &mut Image<T, C>, src: &Image<T, C>) {
    let dwidth = src.width() as f64;
    let height = src.height() as f64;
    rotate(dest, src, 180., Point::new(dwidth / 2., height / 2.));
}

pub fn rotate270<T: Type, C: Color>(dest: &mut Image<T, C>, src: &Image<T, C>) {
    let width = src.height() as f64;
    let dheight = dest.width() as f64;
    rotate(dest, src, 270., Point::new(width / 2., dheight / 2.));
}

#[cfg(test)]
mod test {
    use crate::{
        transform::{resize, rotate180, rotate90, scale},
        Image, Rgb,
    };

    #[test]
    fn test_rotate90() {
        let a = Image::<f32, Rgb>::open("images/A.exr").unwrap();
        let mut dest = Image::new(a.height(), a.width());
        rotate90(&mut dest, &a);
        assert!(dest.save("images/test-rotate90.jpg"))
    }

    #[test]
    fn test_rotate180() {
        let a = Image::<u8, Rgb>::open("images/A.exr").unwrap();
        let mut dest = Image::new(a.width(), a.height());
        rotate180(&mut dest, &a);
        assert!(dest.save("images/test-rotate180.jpg"))
    }

    #[test]
    fn test_scale() {
        let a = Image::<u8, Rgb>::open("images/A.exr").unwrap();
        let mut dest = Image::new(a.width() * 2, a.height() * 2);
        scale(&mut dest, &a, 2., 2.);
        assert!(dest.save("images/test-scale.jpg"))
    }

    #[test]
    fn test_scale_resize() {
        let a = Image::<u8, Rgb>::open("images/A.exr").unwrap();
        let mut dest0 = Image::new(a.width() * 2, a.height() * 2);
        let mut dest1 = Image::new(a.width() * 2, a.height() * 2);
        scale(&mut dest0, &a, 2., 2.);
        resize(&mut dest1, &a, a.width() * 2, a.height() * 2);
        assert_eq!(dest0, dest1);
    }
}
