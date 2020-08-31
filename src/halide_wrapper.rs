#![cfg(feature = "halide")]
use crate::*;

fn kind<T: Type>() -> Result<halide_runtime::Kind, Error> {
    use io::BaseType::*;
    let kind = match T::BASE {
        UInt8 | UInt16 | UInt32 | UInt64 => halide_runtime::Kind::UInt,
        Int8 | Int16 | Int32 | Int64 => halide_runtime::Kind::Int,
        Half | Float | Double => halide_runtime::Kind::Float,
        _ => return Err(Error::InvalidType),
    };
    Ok(kind)
}

impl<T: Type, C: Color> crate::Image<T, C> {
    /// Use the image as a mutable halide_buffer_t
    pub fn as_mut_halide_buffer(&mut self) -> Result<halide_runtime::Buffer, Error> {
        let kind = kind::<T>()?;

        Ok(halide_runtime::Buffer::new(
            self.width() as i32,
            self.height() as i32,
            self.channels() as i32,
            halide_runtime::Type::new(kind, T::bits() as u8),
            &mut self.data,
        ))
    }

    /// Use the image as a halide_buffer_t
    ///
    /// NOTE: This buffer should only be used immutably, it is not safe to
    /// use the resulting Buffer as an output argument. Use `as_mut_halide_buffer`
    /// if you will be mutating the contents
    pub fn as_halide_buffer(&self) -> Result<halide_runtime::Buffer, Error> {
        let kind = kind::<T>()?;

        Ok(halide_runtime::Buffer::new(
            self.width() as i32,
            self.height() as i32,
            self.channels() as i32,
            halide_runtime::Type::new(kind, T::bits() as u8),
            unsafe { &mut *(self.data.as_slice() as *const [T] as *mut [T]) },
        ))
    }
}
