use crate::*;

type EPoint<T> = euclid::Point2D<T, f64>;

/// Transform is used to perform pixel-level transformations on an image
pub type Transform = euclid::Transform2D<f64, f64, f64>;

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Transform {
    fn schedule(&self) -> Schedule {
        Schedule::Image
    }

    fn output_size(&self, input: &Input<T, C>, _dest: &mut Image<U, D>) -> Size {
        let rect = self.outer_transformed_rect(&euclid::Rect::new(
            euclid::Point2D::new(0., 0.),
            input.images()[0].size().to_f64(),
        ));
        rect.size.to_usize()
    }

    fn compute_at(&self, pt: Point, input: &Input<T, C>, px: &mut DataMut<U, D>) {
        let pt = EPoint::new(pt.x as f64, pt.y as f64);
        let dest = self.transform_point(pt);
        let px1 = input.get_pixel((dest.x.floor() as usize, dest.y.floor() as usize), None);
        let px2 = input.get_pixel((dest.x.ceil() as usize, dest.y.ceil() as usize), None);

        ((px1 + &px2) / 2.).copy_to_slice(px);
    }
}

#[cfg(test)]
mod test {
    use crate::{filter::*, Filter, Image, Rgb};

    #[test]
    fn test_rotate90() {
        let a = Image::<f32, Rgb>::open("images/A.exr").unwrap();
        let mut dest = Image::<f32, Rgb>::new((a.height(), a.width()));
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
        let mut dest = Image::<f32, Rgb>::new((a.height(), a.width()));
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
        assert!(dest0 == dest1);
    }
}
