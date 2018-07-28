use ty::Type;
use color::Color;

use std::ops;

pub trait Pixel<'a, T: Type>: AsRef<[T]> {
    fn to_float(&self) -> Vec<f64> {
        self.as_ref().iter().map(|x| T::to_float(x)).collect()
    }

    fn to_pixel_vec(&self) -> PixelVec {
        PixelVec(self.to_float())
    }

    fn is_true(&self) -> bool {
        self.as_ref().iter().all(|x| *x != T::zero())
    }

    fn is_false(&self) -> bool {
        self.as_ref().iter().all(|x| *x == T::zero())
    }

    fn map<F: FnMut(&T) -> T>(&self, f: F) -> PixelVec {
        PixelVec(self.as_ref().iter().map(f).map(|x| T::to_float(&x)).collect())
    }

    fn map2<F: FnMut((&T, &T)) -> T>(&self, other: &Self, f: F) -> PixelVec {
        PixelVec(self.as_ref().iter().zip(other.as_ref()).map(f).map(|x| T::to_float(&x)).collect())
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

#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct PixelVec(Vec<f64>);

impl PixelVec {
    pub fn empty<C: Color>() -> PixelVec {
        PixelVec(vec![0.0; C::channels()])
    }

    pub fn to_vec(self) -> Vec<f64> {
        self.0
    }
}

impl AsRef<[f64]> for PixelVec {
    fn as_ref(&self) -> &[f64] {
        self.0.as_ref()
    }
}

impl AsMut<[f64]> for PixelVec {
    fn as_mut(&mut self) -> &mut [f64] {
        self.0.as_mut()
    }
}

impl<'a> Pixel<'a, f64> for PixelVec {}

macro_rules! pixelvec_op {
    ($name:ident, $fx:ident, $f:expr) => {
        impl ops::$name for PixelVec {
            type Output = PixelVec;

            fn $fx(self, other: Self) -> Self::Output {
                self.map2(&other, |(a, b)| $f(*a, *b))
            }
        }

        impl<'a> ops::$name for &'a PixelVec {
            type Output = PixelVec;

            fn $fx(self, other: Self) -> Self::Output {
                self.map2(&other, |(a, b)| $f(*a, *b))
            }
        }
    }
}

macro_rules! pixelvec_op_assign {
    ($name:ident, $fx:ident, $f:expr) => {
        impl ops::$name for PixelVec {
            fn $fx(&mut self, other: Self) {
                self.as_mut().iter_mut().zip(other.as_ref()).for_each(|(a, b)| *a = $f(*a, *b))
            }
        }

        impl<'a> ops::$name for &'a mut PixelVec {
            fn $fx(&mut self, other: Self) {
                self.as_mut().iter_mut().zip(other.as_ref()).for_each(|(a, b)| *a = $f(*a, *b))
            }
        }
    }
}

pixelvec_op!(Add, add, |a, b| a + b);
pixelvec_op_assign!(AddAssign, add_assign, |a, b| a + b);
pixelvec_op!(Sub, sub, |a, b| a - b);
pixelvec_op_assign!(SubAssign, sub_assign, |a, b| a - b);
pixelvec_op!(Mul, mul, |a, b| a * b);
pixelvec_op_assign!(MulAssign, mul_assign, |a, b| a * b);
pixelvec_op!(Div, div, |a, b| a / b);
pixelvec_op_assign!(DivAssign, div_assign, |a, b| a / b);
pixelvec_op!(Rem, rem, |a, b| a % b);
pixelvec_op_assign!(RemAssign, rem_assign, |a, b| a % b);
