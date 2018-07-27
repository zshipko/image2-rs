use color::Color;
use ty::Type;

#[cfg(feature = "simd")]
use std::simd;

pub trait Pixel<'a, T: Type, C: Color>: AsRef<[T]> {
    fn to_float(&self) -> Vec<f64> {
        self.as_ref().iter().map(|x| T::to_float(x)).collect()
    }

    #[cfg(feature = "simd")]
    fn to_vector(&self) -> simd::f64x4 {
        let data: Vec<f64> = self.as_ref().iter().map(|x| T::to_float(x)).collect();
        simd::f64x4::load_aligned(data.as_ref())
    }
}

pub trait PixelMut<'a, T: Type, C: Color>: Pixel<'a, T, C> + AsMut<[T]> {
    fn from_float<P: Pixel<'a, f64, C>>(&mut self, other: P) {
        let a = self.as_mut().iter_mut();
        let b = other.as_ref().iter();
        a.zip(b).for_each(|(x, y)| *x = T::from_float(*y))
    }

    #[cfg(feature = "simd")]
    fn from_vector(&mut self, other: &simd::f64x4) {
        let data = self.as_mut();
        for i in 0..C::channels() {
            data[i] = T::from_float(other.extract(i));
        }
    }
}

impl<'a, T: Type, C: Color> Pixel<'a, T, C> for &'a [T] {}

impl<'a, T: Type, C: Color> Pixel<'a, T, C> for &'a mut [T] {}

impl<'a, T: Type, C: Color> PixelMut<'a, T, C> for &'a mut [T] {}

impl<'a, T: Type, C: Color> Pixel<'a, T, C> for Vec<T> {}

impl<'a, T: Type, C: Color> PixelMut<'a, T, C> for Vec<T> {}
