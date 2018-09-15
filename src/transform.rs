use euclid;
use {Color, Filter, Image, Type};

pub type Point<T> = euclid::Point2D<T>;
pub struct Transform(pub euclid::Transform2D<f64>);

impl Filter for Transform {
    fn compute_at<T: Type, C: Color, I: Image<T, C>>(
        &self,
        x: usize,
        y: usize,
        c: usize,
        input: &[&I],
    ) -> f64 {
        let pt = Point::new(x as f64, y as f64);
        let dest = self.0.transform_point(&pt);
        (input[0].get_f(dest.x.floor() as usize, dest.y.floor() as usize, c)
            + input[0].get_f(dest.x.ceil() as usize, dest.y.ceil() as usize, c))
            / 2.
    }
}

#[inline]
pub fn rotate<T: Type, C: Color, I: Image<T, C>>(
    dest: &mut I,
    src: &I,
    deg: f64,
    center: Point<f64>,
) {
    let filter = Transform(
        euclid::Transform2D::create_rotation(euclid::Angle::degrees(deg))
            .pre_translate(euclid::TypedVector2D::new(-center.x, -center.y))
            .post_translate(euclid::TypedVector2D::new(center.x, center.y)),
    );

    filter.eval(dest, &[src])
}

#[inline]
pub fn scale<T: Type, C: Color, I: Image<T, C>>(dest: &mut I, src: &I, x: f64, y: f64) {
    let filter = Transform(euclid::Transform2D::create_scale(1.0 / x, 1.0 / y));

    filter.eval(dest, &[src])
}

#[inline]
pub fn resize<T: Type, C: Color, I: Image<T, C>>(dest: &mut I, src: &I, x: usize, y: usize) {
    let filter = Transform(euclid::Transform2D::create_scale(
        src.width() as f64 / x as f64,
        src.height() as f64 / y as f64,
    ));

    filter.eval(dest, &[src])
}

pub fn rotate90<T: Type, C: Color, I: Image<T, C>>(dest: &mut I, src: &I) {
    let dwidth = dest.width() as f64;
    let height = src.height() as f64;
    rotate(dest, src, 90., Point::new(dwidth / 2., height / 2.));
}

pub fn rotate180<T: Type, C: Color, I: Image<T, C>>(dest: &mut I, src: &I) {
    let dwidth = src.width() as f64;
    let height = src.height() as f64;
    rotate(dest, src, 180., Point::new(dwidth / 2., height / 2.));
}

pub fn rotate270<T: Type, C: Color, I: Image<T, C>>(dest: &mut I, src: &I) {
    let width = src.height() as f64;
    let dheight = dest.width() as f64;
    rotate(dest, src, 270., Point::new(width / 2., dheight / 2.));
}

#[cfg(test)]
mod test {
    use {
        io::magick,
        transform::{resize, rotate180, rotate90, scale},
        Image, ImageBuf, Rgb,
    };

    #[test]
    fn test_rotate90() {
        let a: ImageBuf<u8, Rgb> = magick::read("test/test.jpg").unwrap();
        let mut dest = ImageBuf::new(a.height(), a.width());
        rotate90(&mut dest, &a);
        magick::write("test/test-rotate90.jpg", &dest).unwrap();
    }

    #[test]
    fn test_rotate180() {
        let a: ImageBuf<u8, Rgb> = magick::read("test/test.jpg").unwrap();
        let mut dest = ImageBuf::new(a.width(), a.height());
        rotate180(&mut dest, &a);
        magick::write("test/test-rotate180.jpg", &dest).unwrap();
    }

    #[test]
    fn test_scale() {
        let a: ImageBuf<u8, Rgb> = magick::read("test/test.jpg").unwrap();
        let mut dest = ImageBuf::new(a.width() * 2, a.height() * 2);
        scale(&mut dest, &a, 2., 2.);
        magick::write("test/test-scale.jpg", &dest).unwrap();
    }

    #[test]
    fn test_scale_resize() {
        let a: ImageBuf<u8, Rgb> = magick::read("test/test.jpg").unwrap();
        let mut dest0 = ImageBuf::new(a.width() * 2, a.height() * 2);
        let mut dest1 = ImageBuf::new(a.width() * 2, a.height() * 2);
        scale(&mut dest0, &a, 2., 2.);
        resize(&mut dest1, &a, a.width() * 2, a.height() * 2);
        assert_eq!(dest0, dest1);
    }
}
