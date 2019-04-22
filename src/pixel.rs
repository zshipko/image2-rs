use std::ops;

use crate::{Color, Type};

pub mod colorspace {
    pub use palette::*;
}

/// Pixel is used to access chunks of image data
pub trait Pixel<'a, T: Type, C: Color>: AsRef<[T]> {
    /// Create a new Vec<T> from existing pixel data
    fn to_vec(&self) -> Vec<T> {
        self.as_ref().iter().map(|x| x.clone()).collect()
    }

    /// Create a new Vec<f64> of normalized values from existing pixel data
    fn to_f(&self) -> Vec<f64> {
        self.as_ref().iter().map(|x| T::to_f(x)).collect()
    }

    /// Create a new PixelVec<T> from existing pixel data
    fn to_pixel_vec(&self) -> PixelVec<T> {
        PixelVec::from_pixel(self)
    }

    /// Create a new PixelVec<f64> of normalized values from existing pixel data
    fn to_pixel_vec_f(&self) -> PixelVec<f64> {
        PixelVec::from_pixel(self).to_f()
    }

    /// Returns true when every value is > 0
    fn is_true(&self) -> bool {
        self.as_ref().iter().all(|x| *x != T::zero())
    }

    /// Returns true when every value == 0
    fn is_false(&self) -> bool {
        self.as_ref().iter().all(|x| *x == T::zero())
    }

    /// Create a new PixelVec by executing `f` for each channel
    fn map<F: FnMut(&T) -> T>(&self, mut f: F) -> PixelVec<T> {
        let mut dest: PixelVec<T> = PixelVec::empty();
        let data = self.as_ref();
        for (i, item) in data.iter().enumerate() {
            (dest.0)[i] = f(item)
        }
        dest
    }

    fn iter(&self) -> std::slice::Iter<T> {
        self.as_ref().iter()
    }

    fn to_rgb(&self) -> colorspace::LinSrgb {
        let data = self.as_ref();
        palette::LinSrgb::new(
            data[0].to_f() as f32,
            data[1].to_f() as f32,
            data[2].to_f() as f32,
        )
    }

    fn from_rgb(px: colorspace::rgb::Rgb) -> PixelVec<f64> {
        PixelVec::new(px.red as f64, px.green as f64, px.blue as f64, 1.0)
    }

    fn to_rgba(&self) -> colorspace::LinSrgba {
        let data = self.as_ref();
        palette::LinSrgba::new(
            data[0].to_f() as f32,
            data[1].to_f() as f32,
            data[2].to_f() as f32,
            data[3].to_f() as f32,
        )
    }

    fn from_rgba(px: colorspace::rgb::Rgba) -> PixelVec<f64> {
        PixelVec::new(
            px.red as f64,
            px.green as f64,
            px.blue as f64,
            px.alpha as f64,
        )
    }

    fn to_luma(&self) -> colorspace::LinLuma {
        let data = self.as_ref();
        palette::luma::Luma::new(data[0].to_f() as f32)
    }

    fn from_luma(px: colorspace::luma::Luma) -> PixelVec<f64> {
        PixelVec::new_gray(px.luma as f64)
    }
}

/// PixelMut is used to access mutable chunks of image data
pub trait PixelMut<'a, T: Type, C: Color>: Pixel<'a, T, C> + AsMut<[T]> {
    /// Copy values from a normalized f64 pixel
    fn set_f<P: Pixel<'a, f64, C>>(&mut self, other: &P) {
        let a = self.as_mut().iter_mut();
        let b = other.as_ref().iter();
        a.zip(b).for_each(|(x, y)| *x = T::from_f(*y))
    }

    /// Copy values from another pixel
    fn set<P: Pixel<'a, T, C>>(&mut self, other: &P) {
        let a = self.as_mut().iter_mut();
        let b = other.as_ref().iter();
        a.zip(b).for_each(|(x, y)| *x = *y)
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.as_mut().iter_mut()
    }

    fn blend_alpha(&mut self) {
        if !C::has_alpha() {
            return;
        }

        let len = C::channels();

        let alpha = T::to_float(&self.as_ref()[len - 1]) / T::max_f();
        let data = self.as_mut();

        for i in 0..len - 1 {
            data[i] = T::from_float(T::to_float(&data[i]) * alpha);
        }

        data[len - 1] = T::max();
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

/// PixelVec is a 4-channel pixel backed by a static array
#[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct PixelVec<T: Type>([T; 4]);

impl<T: Type> PixelVec<T> {
    /// Create a new PixelVec, each channel set to 0
    pub fn empty() -> PixelVec<T> {
        PixelVec([T::zero(); 4])
    }

    /// Create a new PixelVec with the given values
    pub fn new(a: T, b: T, c: T, d: T) -> PixelVec<T> {
        PixelVec([a, b, c, d])
    }

    /// Create a new PixelBec with every channel set to the given value. The alpha channel is set
    /// to `T::max()`
    pub fn new_gray(a: T) -> PixelVec<T> {
        PixelVec([a, a, a, T::max()])
    }

    /// Create a new PixelVec from an existing Pixel
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

    /// Create a new PixelVec by mapping `f` over an existing PixelVec
    pub fn map<U: Type, F: Fn(&T) -> U>(&self, f: F) -> PixelVec<U> {
        let mut vec = PixelVec::empty();
        for i in 0..4 {
            vec.0[i] = f(&self.0[i])
        }
        vec
    }

    /// Convert from `PixelVec<T>` to `Vec<T>`
    pub fn to_vec<C: Color>(&self) -> Vec<T> {
        let mut vec = self.0.to_vec();
        vec.truncate(C::channels());
        vec
    }

    /// Convert from `PixelVec<T>` to `Vec<f64>` and normalize values
    pub fn to_vec_f<C: Color>(&self) -> Vec<f64> {
        let mut vec: Vec<f64> = self.0.to_vec().into_iter().map(|x| T::to_f(&x)).collect();
        vec.truncate(C::channels());
        vec
    }

    /// Convert from `PixelVec<T>` to `PixelVec<f64>`
    pub fn to_float(&self) -> PixelVec<f64> {
        self.map(|x| T::to_float(x))
    }

    /// Convert from `PixelVec<T>` to `PixelVec<f64>` and normalize values
    pub fn to_f(&self) -> PixelVec<f64> {
        self.map(|x| T::to_f(x))
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

impl<'a, T: Type, C: Color> Pixel<'a, T, C> for PixelVec<T> {}
impl<'a, T: Type, C: Color> PixelMut<'a, T, C> for PixelVec<T> {}

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
