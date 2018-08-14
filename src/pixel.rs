use ty::Type;
use color::Color;

use std::ops;

pub trait Pixel<'a, T: Type>: AsRef<[T]> {
    fn to_float(&self) -> Vec<f64> {
        self.as_ref().iter().map(|x| T::to_float(x)).collect()
    }

    fn to_pixel_vec(&self) -> PixelVec {
        PixelVec::from_pixel(self)
    }

    fn is_true(&self) -> bool {
        self.as_ref().iter().all(|x| *x != T::zero())
    }

    fn is_false(&self) -> bool {
        self.as_ref().iter().all(|x| *x == T::zero())
    }

    fn map<F: FnMut(&T) -> T>(&self, mut f: F) -> PixelVec {
        let len = self.as_ref().len();
        let mut dest: PixelVec = PixelVec::empty();
        let data = self.as_ref();
        for i in 0..len {
            (dest.0)[i] = T::to_float(&f(&data[i]))
        }
        dest
    }
}

pub trait PixelMut<'a, T: Type>: Pixel<'a, T> + AsMut<[T]> {
    fn set_from_float<P: Pixel<'a, f64>>(&mut self, other: &P) {
        let a = self.as_mut().iter_mut();
        let b = other.as_ref().iter();
        a.zip(b).for_each(|(x, y)| *x = T::from_float(*y))
    }

    fn set_from_pixel_vec(&mut self, other: &PixelVec) {
        self.set_from_float(other)
    }

    fn set_from<P: Pixel<'a, T>>(&mut self, other: &P) {
        let a = self.as_mut().iter_mut();
        let b = other.as_ref().iter();
        a.zip(b).for_each(|(x, y)| *x = *y)
    }
}

impl<'a, T: Type> Pixel<'a, T> for &'a [T] {}
impl<'a, T: Type> Pixel<'a, T> for &'a mut [T] {}
impl<'a, T: Type> PixelMut<'a, T> for &'a mut [T] {}
impl<'a, T: Type> Pixel<'a, T> for Vec<T> {}
impl<'a, T: Type> PixelMut<'a, T> for Vec<T> {}
impl<'a, T: Type> Pixel<'a, T> for &'a Vec<T> {}
impl<'a, T: Type> Pixel<'a, T> for &'a mut Vec<T> {}
impl<'a, T: Type> PixelMut<'a, T> for &'a mut Vec<T> {}

#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct PixelVec([f64; 4]);

impl PixelVec {
    pub fn empty() -> PixelVec {
        PixelVec([0.0, 0.0, 0.0, 0.0])
    }

    pub fn new(a: f64, b: f64, c: f64, d: f64) -> PixelVec {
        PixelVec([a, b, c, d])
    }

    pub fn from_pixel<'a, T: Type, P: AsRef<[T]>>(pixel: P) -> PixelVec {
        let data: &[T] = pixel.as_ref();
        let len = data.len();

        if len == 0 {
            PixelVec::empty()
        } else if len == 1 {
            let d0 = T::to_float(&data[0]);
            PixelVec::new(d0, d0, d0, 0.0)
        } else if len == 2 {
            let d0 = T::to_float(&data[0]);
            let d1 = T::to_float(&data[1]);
            PixelVec::new(d0, d1, 0.0, 0.0)
        } else if len == 3 {
            let d0 = T::to_float(&data[0]);
            let d1 = T::to_float(&data[1]);
            let d2 = T::to_float(&data[2]);
            PixelVec::new(d0, d1, d2, 0.0)
        } else{
            let d0 = T::to_float(&data[0]);
            let d1 = T::to_float(&data[1]);
            let d2 = T::to_float(&data[2]);
            let d3 = T::to_float(&data[3]);
            PixelVec::new(d0, d1, d2, d3)
        }
    }

    pub fn to_vec<C: Color>(&self) -> Vec<f64> {
        let mut vec = self.0.to_vec();
        vec.truncate(C::channels());
        vec
    }
}

impl AsRef<[f64]> for PixelVec {
    fn as_ref(&self) -> &[f64] {
        &self.0
    }
}

impl AsMut<[f64]> for PixelVec {
    fn as_mut(&mut self) -> &mut [f64] {
        &mut self.0
    }
}

impl<'a> Pixel<'a, f64> for PixelVec {}

macro_rules! pixelvec_op {
    ($name:ident, $fx:ident, $f:expr) => {
        impl ops::$name for PixelVec {
            type Output = PixelVec;

            fn $fx(mut self, other: Self) -> Self::Output {
                for i in 0..4 {
                    self.0[i] = $f(self.0[i], other.0[i]);
                }
                self
            }
        }

        impl<'a> ops::$name for &'a PixelVec {
            type Output = PixelVec;

            fn $fx(self, other: Self) -> Self::Output {
                let mut dest = PixelVec::empty();
                for i in 0..4 {
                    dest.0[i] = $f(self.0[i], other.0[i]);
                }
                self.clone()
            }
        }
    }
}

macro_rules! pixelvec_op_assign {
    ($name:ident, $fx:ident, $f:expr) => {
        impl ops::$name for PixelVec {
            fn $fx(&mut self, other: Self) {
                for i in 0..4 {
                    self.0[i] = $f(self.0[i], other.0[i]);
                }
            }
        }

        impl<'a> ops::$name for &'a mut PixelVec {
            fn $fx(&mut self, other: Self) {
                for i in 0..4 {
                    self.0[i] = $f(self.0[i], other.0[i]);
                }
            }
        }
    }
}

pixelvec_op!(Add, add, |a, b| a + b);
pixelvec_op_assign!(AddAssign, add_assign, |a: f64, b: f64| a + b);
pixelvec_op!(Sub, sub, |a, b| a - b);
pixelvec_op_assign!(SubAssign, sub_assign, |a, b| a - b);
pixelvec_op!(Mul, mul, |a, b| a * b);
pixelvec_op_assign!(MulAssign, mul_assign, |a, b| a * b);
pixelvec_op!(Div, div, |a, b| a / b);
pixelvec_op_assign!(DivAssign, div_assign, |a, b| a / b);
pixelvec_op!(Rem, rem, |a, b| a % b);
pixelvec_op_assign!(RemAssign, rem_assign, |a, b| a % b);
