use crate::*;

/// Combine two filters using the function `F`
pub struct Combine<
    T: Type,
    C: Color,
    U: Type,
    D: Color,
    A: Filter<T, C, U, D>,
    B: Filter<T, C, U, D>,
    F: Fn(Point, Pixel<D>, Pixel<D>) -> Pixel<D>,
> {
    a: A,
    b: B,
    f: F,
    _t: std::marker::PhantomData<(T, C, U, D)>,
}

impl<
        T: Type,
        C: Color,
        U: Type,
        D: Color,
        A: Filter<T, C, U, D>,
        B: Filter<T, C, U, D>,
        F: Sync + Fn(Point, Pixel<D>, Pixel<D>) -> Pixel<D>,
    > Filter<T, C, U, D> for Combine<T, C, U, D, A, B, F>
{
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        self.a.compute_at(pt, input, dest);
        let a: Pixel<D> = Pixel::from_data(&dest.as_data());

        self.b.compute_at(pt, input, dest);
        let b: Pixel<D> = Pixel::from_data(&dest.as_data());

        (self.f)(pt, a, b).copy_to_slice(dest);
    }
}

impl<T: Type, C: Color, U: Type, D: Color, F: Filter<T, C, U, D>> FilterExt<T, C, U, D> for F {}

/// Filter extension methods
pub trait FilterExt<T: Type, C: Color, U: Type, D: Color>: Sized + Filter<T, C, U, D> {
    /// Combine two filters using a function
    fn combine<B: Filter<T, C, U, D>, F: Fn(Point, Pixel<D>, Pixel<D>) -> Pixel<D>>(
        self,
        b: B,
        f: F,
    ) -> Combine<T, C, U, D, Self, B, F> {
        Combine {
            a: self,
            b,
            f,
            _t: std::marker::PhantomData,
        }
    }

    /// Convert filter to `AsyncFilter`
    fn to_async<'a>(
        &'a self,
        mode: filter::AsyncMode,
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
}
