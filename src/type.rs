use crate::*;

pub trait Type: Unpin + Default + Clone + Copy + Sync + Send + PartialEq + PartialOrd {
    const MIN: f64;
    const MAX: f64;
    const BASE: io::BaseType;

    fn to_f64(&self) -> f64;

    fn from_f64(f: f64) -> Self;

    fn name() -> &'static str {
        io::internal::type_name(Self::BASE)
    }

    fn set_from_f64(&mut self, f: f64) {
        *self = Self::from_f64(f);
    }

    fn to_norm(&self) -> f64 {
        Self::normalize(self.to_f64())
    }

    fn from_norm(f: f64) -> Self {
        Self::from_f64(Self::denormalize(f))
    }

    #[inline]
    fn normalize(f: f64) -> f64 {
        (f - Self::MIN) / (Self::MAX - Self::MIN)
    }

    #[inline]
    fn denormalize(f: f64) -> f64 {
        f * Self::MAX - Self::MIN
    }

    #[inline]
    fn clamp(f: f64) -> f64 {
        if f > Self::MAX {
            Self::MAX
        } else if f < Self::MIN {
            Self::MIN
        } else {
            f
        }
    }

    #[inline]
    fn convert<X: Type>(&self) -> X {
        X::from_f64(X::denormalize(Self::normalize(Self::to_f64(self))))
    }
}

impl Type for u8 {
    const MIN: f64 = 0.0;
    const MAX: f64 = u8::MAX as f64;
    const BASE: io::BaseType = io::BaseType::UInt8;

    fn to_f64(&self) -> f64 {
        *self as f64
    }

    fn from_f64(f: f64) -> Self {
        f as Self
    }
}

impl Type for i8 {
    const MIN: f64 = i8::MIN as f64;
    const MAX: f64 = i8::MAX as f64;
    const BASE: io::BaseType = io::BaseType::Int8;

    fn to_f64(&self) -> f64 {
        *self as f64
    }

    fn from_f64(f: f64) -> Self {
        f as Self
    }
}

impl Type for u16 {
    const MIN: f64 = 0.0;
    const MAX: f64 = u16::MAX as f64;
    const BASE: io::BaseType = io::BaseType::UInt16;

    fn to_f64(&self) -> f64 {
        *self as f64
    }

    fn from_f64(f: f64) -> Self {
        f as Self
    }
}

impl Type for i16 {
    const MIN: f64 = i16::MIN as f64;
    const MAX: f64 = i16::MAX as f64;
    const BASE: io::BaseType = io::BaseType::Int16;

    fn to_f64(&self) -> f64 {
        *self as f64
    }

    fn from_f64(f: f64) -> Self {
        f as Self
    }
}

impl Type for u32 {
    const MIN: f64 = 0.0;
    const MAX: f64 = u32::MAX as f64;
    const BASE: io::BaseType = io::BaseType::UInt32;

    fn to_f64(&self) -> f64 {
        *self as f64
    }

    fn from_f64(f: f64) -> Self {
        f as Self
    }
}

impl Type for i32 {
    const MIN: f64 = i32::MIN as f64;
    const MAX: f64 = i32::MAX as f64;
    const BASE: io::BaseType = io::BaseType::Int32;

    fn to_f64(&self) -> f64 {
        *self as f64
    }

    fn from_f64(f: f64) -> Self {
        f as Self
    }
}

impl Type for u64 {
    const MIN: f64 = 0.0;
    const MAX: f64 = u64::MAX as f64;
    const BASE: io::BaseType = io::BaseType::UInt64;

    fn to_f64(&self) -> f64 {
        *self as f64
    }

    fn from_f64(f: f64) -> Self {
        f as Self
    }
}

impl Type for i64 {
    const MIN: f64 = i64::MIN as f64;
    const MAX: f64 = i64::MAX as f64;
    const BASE: io::BaseType = io::BaseType::Int64;

    fn to_f64(&self) -> f64 {
        *self as f64
    }

    fn from_f64(f: f64) -> Self {
        f as Self
    }
}

impl Type for f16 {
    const MIN: f64 = 0.0;
    const MAX: f64 = 1.0;
    const BASE: io::BaseType = io::BaseType::Half;

    fn to_f64(&self) -> f64 {
        f16::to_f64(*self)
    }

    fn from_f64(f: f64) -> Self {
        f16::from_f64(f)
    }
}

impl Type for f32 {
    const MIN: f64 = 0.0;
    const MAX: f64 = 1.0;
    const BASE: io::BaseType = io::BaseType::Float;

    fn to_f64(&self) -> f64 {
        *self as f64
    }

    fn from_f64(f: f64) -> Self {
        f as Self
    }
}

impl Type for f64 {
    const MIN: f64 = 0.0;
    const MAX: f64 = 1.0;
    const BASE: io::BaseType = io::BaseType::Double;

    fn to_f64(&self) -> f64 {
        *self as f64
    }

    fn from_f64(f: f64) -> Self {
        f as Self
    }
}
