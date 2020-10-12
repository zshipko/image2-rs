use crate::*;

impl<T: Type, C: Color, U: Type, D: Color, F: Filter<T, C, U, D>> FilterExt<T, C, U, D> for F {}

/// Filter extension methods
pub trait FilterExt<T: Type, C: Color, U: Type, D: Color>: Sized + Filter<T, C, U, D> {
    /// Convert filter to `AsyncFilter`
    fn to_async<'a>(
        &'a self,
        mode: AsyncMode,
        input: Input<'a, T, C>,
        output: &'a mut Image<U, D>,
    ) -> AsyncFilter<'a, Self, T, C, U, D> {
        AsyncFilter {
            mode,
            filter: self,
            input,
            output,
            x: 0,
            y: 0,
        }
    }

    /// Create a new pipeline
    fn then(self, other: impl 'static + Filter<T, C, U, D>) -> Pipeline<T, C, U, D>
    where
        Self: 'static,
    {
        Pipeline::new().then(self).then(other)
    }
}
