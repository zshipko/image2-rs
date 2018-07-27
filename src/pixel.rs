use ty::Type;
use std::simd;

pub trait Pixel<'a, T: Type>: AsRef<[T]> {
    fn to_float(&self) -> Vec<f64> {
        self.as_ref().iter().map(|x| T::to_float(x)).collect()
    }

    fn to_pixel_vec(&self) -> PixelVec {
        let data: Vec<f64> = self.as_ref().iter().map(|x| T::to_float(x)).collect();
        match data.len() {
            0 => PixelVec::empty(),
            1 => PixelVec::new(data[0], 0.0, 0.0, 0.0),
            2 => PixelVec::new(data[0], data[1], 0.0, 0.0),
            3 => PixelVec::new(data[0], data[1], data[2], 0.0),
            4 => PixelVec{data: simd::f64x4::load_unaligned(data.as_ref())},
            _ => PixelVec{data: simd::f64x4::load_unaligned(&data[0..4])},
        }
    }
}

pub trait PixelMut<'a, T: Type>: Pixel<'a, T> + AsMut<[T]> {
    fn from_float<P: Pixel<'a, f64>>(&mut self, other: P) {
        let a = self.as_mut().iter_mut();
        let b = other.as_ref().iter();
        a.zip(b).for_each(|(x, y)| *x = T::from_float(*y))
    }

    fn from_pixel_vec(&mut self, other: &PixelVec) {
        let data = self.as_mut();
        data.iter_mut().enumerate().for_each(|(i, x)| *x = T::from_float(other.data.extract(i)));
    }
}

pub struct PixelVec {
    pub data: simd::f64x4
}

impl PixelVec {
    pub fn empty() -> Self {
        PixelVec{data: simd::f64x4::splat(0.0)}
    }

    pub fn new<T: Type>(a: T, b: T, c: T, d: T) -> Self {
        PixelVec{data:simd::f64x4::new(a.convert(), b.convert(), c.convert(), d.convert())}
    }

    pub fn to_vec<T: Type>(&self, n: usize) -> Vec<T> {
        let mut dest = vec![T::zero(); n];
        dest.from_pixel_vec(self);
        dest
    }

    pub fn get(&self, i: usize) -> f64 {
        self.data.extract(i)
    }

    pub fn set(&mut self, i: usize, x: f64) {
        self.data = self.data.replace(i, x)
    }
}

impl<'a, T: Type> Pixel<'a, T> for &'a [T] {}
impl<'a, T: Type> Pixel<'a, T> for &'a mut [T] {}
impl<'a, T: Type> PixelMut<'a, T> for &'a mut [T] {}
impl<'a, T: Type> Pixel<'a, T> for Vec<T> {}
impl<'a, T: Type> PixelMut<'a, T> for Vec<T> {}
