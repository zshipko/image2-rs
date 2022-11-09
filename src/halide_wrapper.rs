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
    pub fn to_halide_buffer(&mut self) -> Result<halide_runtime::Buffer, Error> {
        let kind = kind::<T>()?;

        Ok(halide_runtime::Buffer::new(
            self.width() as i32,
            self.height() as i32,
            self.channels() as i32,
            halide_runtime::Type::new(kind, T::bits() as u8),
            self.data.data_mut(),
        ))
    }

    /// Use the image as a const halide_buffer_t
    pub unsafe fn to_const_halide_buffer(&self) -> Result<halide_runtime::Buffer, Error> {
        let kind = kind::<T>()?;

        Ok(halide_runtime::Buffer::new_const(
            self.width() as i32,
            self.height() as i32,
            self.channels() as i32,
            halide_runtime::Type::new(kind, T::bits() as u8),
            self.data.data(),
        ))
    }
}
