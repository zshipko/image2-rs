use crate::*;

pub use convert::{Convert, ConvertColor};

/// Filters are used to manipulate images in a generic, composable manner
pub trait Filter<C: Color, D: Color = C>: Sized + Sync {
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, C>]) -> f64;

    /// Evaluate a filter on part of an image
    fn eval_partial<T: Type>(
        &self,
        start_x: usize,
        start_y: usize,
        width: usize,
        height: usize,
        output: &mut Image<T, D>,
        input: &[&Image<impl Type, C>],
    ) {
        let channels = output.channels();
        output.for_each_rect(start_x, start_y, width, height, |(x, y), pixel| {
            for c in 0..C::CHANNELS.min(channels) {
                pixel[c] = T::from_norm(self.compute_at(x, y, c, input));
            }
        });
    }

    /// Evaluate filter in parallel
    fn eval<T: Type>(&self, output: &mut Image<T, impl Color>, input: &[&Image<impl Type, C>]) {
        let channels = output.channels();
        output.for_each(|(x, y), pixel| {
            for c in 0..C::CHANNELS.min(channels) {
                pixel[c] = T::from_norm(self.compute_at(x, y, c, input));
            }
        });
    }

    fn join<
        'a,
        E: Color,
        Y: Color,
        A: 'a + Filter<C, D>,
        B: 'a + Filter<C, Y>,
        F: Fn(f64, f64) -> f64,
    >(
        &'a self,
        other: &'a B,
        f: F,
    ) -> Join<'a, C, D, Y, E, Self, B, F> {
        Join {
            a: self,
            b: other,
            f,
            _color: std::marker::PhantomData,
        }
    }

    fn and_then<E: Color, F: Fn(f64) -> f64>(&self, f: F) -> AndThen<C, D, E, Self, F> {
        AndThen {
            a: self,
            f,
            _color: std::marker::PhantomData,
        }
    }
}

/// Executes `a` then `b` and passes the results to `f`
pub struct Join<
    'a,
    C: Color,
    X: Color,
    Y: Color,
    E: Color,
    A: 'a + Filter<C, X>,
    B: 'a + Filter<C, Y>,
    F: Fn(f64, f64) -> f64,
> {
    a: &'a A,
    b: &'a B,
    f: F,
    _color: std::marker::PhantomData<(C, X, Y, E)>,
}

/// Executes `a` then `f(a)`
pub struct AndThen<'a, C: Color, D: Color, E: Color, A: 'a + Filter<C, D>, F: Fn(f64) -> f64> {
    a: &'a A,
    f: F,
    _color: std::marker::PhantomData<(C, D, E)>,
}

impl<'a, C: Color, D: Color, E: Color, A: Filter<C, D>, F: Sync + Fn(f64) -> f64> Filter<C, E>
    for AndThen<'a, C, D, E, A, F>
{
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, C>]) -> f64 {
        (self.f)(self.a.compute_at(x, y, c, input))
    }
}

impl<
        'a,
        C: Color,
        X: Color,
        Y: Color,
        E: Color,
        A: Filter<C, X>,
        B: Filter<C, Y>,
        F: Sync + Fn(f64, f64) -> f64,
    > Filter<C, E> for Join<'a, C, X, Y, E, A, B, F>
{
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, C>]) -> f64 {
        let f = &self.f;
        let a = self.a.compute_at(x, y, c, input);
        let b = self.b.compute_at(x, y, c, input);
        f(a, b)
    }
}

pub struct Invert;

impl<C: Color> Filter<C> for Invert {
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, C>]) -> f64 {
        let max = input[0].type_max();
        max - input[0].get_f(x, y, c)
    }
}

pub struct Blend;

impl<C: Color> Filter<C> for Blend {
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, C>]) -> f64 {
        (input[0].get_f(x, y, c) + input[1].get_f(x, y, c)) / 2.0
    }
}

pub struct Gamma(pub f64);

impl<C: Color> Filter<C> for Gamma {
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, C>]) -> f64 {
        input[0].get_f(x, y, c).powf(1.0 / self.0)
    }
}
