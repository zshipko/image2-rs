use crate::*;

/// Type is used to represent supported image data types
pub trait Type:
    'static + Unpin + Default + Clone + Copy + Sync + Send + PartialEq + PartialOrd + std::fmt::Debug
{
    /// Min value
    const MIN: f64;

    /// Max value
    const MAX: f64;

    /// I/O base type
    const BASE: io::BaseType;

    /// Convert to f64
    fn to_f64(&self) -> f64;

    /// Convert from f64
    fn from_f64(f: f64) -> Self;

    /// Returns true when `T` is a floating point type
    fn is_float() -> bool {
        let x = Self::to_f64(&Self::from_f64(0.5));
        x > 0.0 && x < 1.0
    }

    /// Get the type name
    fn type_name() -> &'static str {
        use io::BaseType::*;
        match Self::BASE {
            Unknown => "unknown",
            None => "none",
            UInt8 => "uint8",
            Int8 => "int8",
            UInt16 => "uint16",
            Int16 => "int16",
            UInt32 => "uint32",
            Int32 => "int32",
            UInt64 => "uint64",
            Int64 => "int64",
            Half => "half",
            Float => "float",
            Double => "double",
            String => "string",
            Ptr => "ptr",
            Last => "",
        }
    }

    /// Set a value from an f64 value
    fn set_from_f64(&mut self, f: f64) {
        *self = Self::from_f64(f);
    }

    /// Set a value from normalized float
    fn set_from_norm(&mut self, f: f64) {
        *self = Self::from_norm(f);
    }

    /// Convert from `T` to normalized float
    fn to_norm(&self) -> f64 {
        Self::normalize(self.to_f64())
    }

    /// Convert to `T` from normalized float
    fn from_norm(f: f64) -> Self {
        Self::from_f64(Self::denormalize(f))
    }

    #[inline]
    /// Scale a value to fit between 0 and 1.0 based on the min/max values for `T`
    fn normalize(f: f64) -> f64 {
        (f - Self::MIN) / (Self::MAX - Self::MIN)
    }

    #[inline]
    /// Scale an f64 value to fit the range supported by `T`
    fn denormalize(f: f64) -> f64 {
        f * Self::MAX - Self::MIN
    }

    /// Ensure the given value is less than the max allowed and greater than or equal to the
    /// minimum value
    #[inline]
    fn clamp(f: f64) -> f64 {
        f.clamp(Self::MIN, Self::MAX)
    }

    /// Convert a value from one type to another
    #[inline]
    fn convert<X: Type>(&self) -> X {
        X::from_f64(X::denormalize(Self::normalize(Self::to_f64(self))))
    }

    /// Get the number of bits for a data type
    fn bits() -> usize {
        std::mem::size_of::<Self>() * 8
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
        *self
    }

    fn from_f64(f: f64) -> Self {
        f
    }
}
