use crate::{Color, Gray, Rgb, Rgba, Type};

use std::ops;

pub trait Pixel<'a, T: Type, C: Color>: AsRef<[T]> {
    fn to_float(&self) -> Vec<f64> {
        self.as_ref().iter().map(|x| T::to_float(x)).collect()
    }

    fn to_pixel_vec(&self) -> PixelVec<T> {
        PixelVec::from_pixel(self)
    }

    fn is_true(&self) -> bool {
        self.as_ref().iter().all(|x| *x != T::zero())
    }

    fn is_false(&self) -> bool {
        self.as_ref().iter().all(|x| *x == T::zero())
    }

    fn map<F: FnMut(&T) -> T>(&self, mut f: F) -> PixelVec<T> {
        let mut dest: PixelVec<T> = PixelVec::empty();
        let data = self.as_ref();
        for (i, item) in data.iter().enumerate() {
            (dest.0)[i] = f(item)
        }
        dest
    }
}

pub trait PixelMut<'a, T: Type, C: Color>: Pixel<'a, T, C> + AsMut<[T]> {
    fn set_from_float<P: Pixel<'a, f64, C>>(&mut self, other: &P) {
        let a = self.as_mut().iter_mut();
        let b = other.as_ref().iter();
        a.zip(b).for_each(|(x, y)| *x = T::from_float(*y))
    }

    fn set_from<P: Pixel<'a, T, C>>(&mut self, other: &P) {
        let a = self.as_mut().iter_mut();
        let b = other.as_ref().iter();
        a.zip(b).for_each(|(x, y)| *x = *y)
    }
}

impl<'a, T: Type, C: Color> Pixel<'a, T, C> for &'a [T] {}
impl<'a, T: Type, C: Color> Pixel<'a, T, C> for &'a mut [T] {}
impl<'a, T: Type, C: Color> PixelMut<'a, T, C> for &'a mut [T] {}
impl<'a, T: Type, C: Color> Pixel<'a, T, C> for Vec<T> {}
impl<'a, T: Type, C: Color> PixelMut<'a, T, C> for Vec<T> {}
impl<'a, T: Type, C: Color> Pixel<'a, T, C> for &'a Vec<T> {}
impl<'a, T: Type, C: Color> Pixel<'a, T, C> for &'a mut Vec<T> {}
impl<'a, T: Type, C: Color> PixelMut<'a, T, C> for &'a mut Vec<T> {}

#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct PixelVec<T: Type>([T; 4]);

impl<T: Type> PixelVec<T> {
    pub fn empty() -> PixelVec<T> {
        PixelVec([T::zero(); 4])
    }

    pub fn new(a: T, b: T, c: T, d: T) -> PixelVec<T> {
        PixelVec([a, b, c, d])
    }

    pub fn from_pixel<P: AsRef<[T]>>(pixel: P) -> PixelVec<T> {
        let data: &[T] = pixel.as_ref();
        let len = data.len();

        if len == 0 {
            PixelVec::empty()
        } else if len == 1 {
            let d0 = data[0];
            PixelVec::new(d0, d0, d0, T::max())
        } else if len == 2 {
            let d0 = data[0];
            let d1 = data[1];
            PixelVec::new(d0, d1, T::min(), T::max())
        } else if len == 3 {
            let d0 = data[0];
            let d1 = data[1];
            let d2 = data[2];
            PixelVec::new(d0, d1, d2, T::max())
        } else {
            let d0 = data[0];
            let d1 = data[1];
            let d2 = data[2];
            let d3 = data[3];
            PixelVec::new(d0, d1, d2, d3)
        }
    }

    pub fn map<U: Type, F: Fn(&T) -> U>(&self, f: F) -> PixelVec<U> {
        let mut vec = PixelVec::empty();
        for i in 0..4 {
            vec.0[i] = f(&self.0[i])
        }
        vec
    }

    pub fn to_vec<C: Color>(&self) -> Vec<T> {
        let mut vec = self.0.to_vec();
        vec.truncate(C::channels());
        vec
    }

    pub fn to_vec_f<C: Color>(&self) -> Vec<f64> {
        let mut vec: Vec<f64> = self.0.to_vec().iter().map(|x| T::to_float(x)).collect();
        vec.truncate(C::channels());
        vec
    }

    pub fn f(&self) -> PixelVec<f64> {
        self.map(|x| T::to_float(x))
    }
}

impl<T: Type> AsRef<[T]> for PixelVec<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T: Type> AsMut<[T]> for PixelVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<'a, T: Type> Pixel<'a, T, Gray> for PixelVec<T> {}
impl<'a, T: Type> Pixel<'a, T, Rgb> for PixelVec<T> {}
impl<'a, T: Type> Pixel<'a, T, Rgba> for PixelVec<T> {}
impl<'a, T: Type> PixelMut<'a, T, Gray> for PixelVec<T> {}
impl<'a, T: Type> PixelMut<'a, T, Rgb> for PixelVec<T> {}
impl<'a, T: Type> PixelMut<'a, T, Rgba> for PixelVec<T> {}

macro_rules! pixelvec_op {
    ($name:ident, $fx:ident, $f:expr) => {
        impl<T: Type> ops::$name for PixelVec<T> {
            type Output = PixelVec<T>;

            fn $fx(mut self, other: Self) -> Self::Output {
                for i in 0..4 {
                    self.0[i] = $f(self.0[i], other.0[i]);
                }
                self
            }
        }

        impl<'a, T: Type> ops::$name for &'a PixelVec<T> {
            type Output = PixelVec<T>;

            fn $fx(self, other: Self) -> Self::Output {
                let mut dest = PixelVec::empty();
                for i in 0..4 {
                    dest.0[i] = $f(self.0[i], other.0[i]);
                }
                self.clone()
            }
        }
    };
}

macro_rules! pixelvec_op_assign {
    ($name:ident, $fx:ident, $f:expr) => {
        impl<T: Type> ops::$name for PixelVec<T> {
            fn $fx(&mut self, other: Self) {
                for i in 0..4 {
                    self.0[i] = $f(self.0[i], other.0[i]);
                }
            }
        }

        impl<'a, T: Type> ops::$name for &'a mut PixelVec<T> {
            fn $fx(&mut self, other: Self) {
                for i in 0..4 {
                    self.0[i] = $f(self.0[i], other.0[i]);
                }
            }
        }
    };
}

pixelvec_op!(Add, add, |a, b| a + b);
pixelvec_op_assign!(AddAssign, add_assign, |a: T, b: T| a + b);
pixelvec_op!(Sub, sub, |a, b| a - b);
pixelvec_op_assign!(SubAssign, sub_assign, |a, b| a - b);
pixelvec_op!(Mul, mul, |a, b| a * b);
pixelvec_op_assign!(MulAssign, mul_assign, |a, b| a * b);
pixelvec_op!(Div, div, |a, b| a / b);
pixelvec_op_assign!(DivAssign, div_assign, |a, b| a / b);
pixelvec_op!(Rem, rem, |a, b| a % b);
pixelvec_op_assign!(RemAssign, rem_assign, |a, b| a % b);
