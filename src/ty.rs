use num::{NumCast, Zero, ToPrimitive, FromPrimitive};
use std::ops::*;

pub trait Type:  NumCast +
                FromPrimitive +
                Zero +
                Clone +
                Copy +
                Add<Output=Self> +
                Sub<Output=Self> +
                Mul<Output=Self> +
                Div<Output=Self>
{
    fn min() -> f64;
    fn max() -> f64;

    #[inline]
    fn normalize(f: f64) -> f64 {
        (f - Self::min()) / (Self::max() - Self::min())
    }

    #[inline]
    fn denormalize(f: f64) -> f64 {
        f * Self::max() - Self::min()
    }

    #[inline]
    fn to_float(x: &Self) -> f64 {
        match ToPrimitive::to_f64(x) {
            Some(f) => f,
            None => 0.0
        }
    }

    #[inline]
    fn from_float(x: f64) -> Self {
        match FromPrimitive::from_f64(x) {
            Some(p) => p,
            None => Self::zero(),
        }
    }

    #[inline]
    fn clamp(f: f64) -> f64 {
        if f > Self::max() {
            Self::max()
        } else if f < Self::min() {
            Self::min()
        } else {
            f
        }
    }

    #[inline]
    fn convert<X: Type>(&self) -> X {
        X::from_float(X::denormalize(Self::normalize(Self::to_float(self))))
    }
}

macro_rules! make_type {
    ($t:ty, $min:expr, $max:expr) => {
        impl Type for $t {
            fn min() -> f64 {
                match ToPrimitive::to_f64(&$min) {
                    Some(f) => f,
                    None => panic!("Invalid type minimum"),
                }
            }
            fn max() -> f64 {
                match ToPrimitive::to_f64(&$max) {
                    Some(f) => f,
                    None => panic!("Invalid type maximum"),
                }
            }
        }
    };

    ($t:ty) => {
        make_type!($t, <$t>::min_value(), <$t>::max_value());
    };
}

make_type!(u8);
make_type!(u16);
make_type!(i32);
make_type!(u32);
make_type!(f32, 0, 1);
make_type!(i64);
make_type!(u64);
make_type!(f64, 0, 1);
