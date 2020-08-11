use crate::*;

use rayon::prelude::*;


/// Filters are used to manipulate images in a generic, composable manner
pub trait Filter: Sized + Sync {
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, impl Color>]) -> f64;

    /// Evaluate a filter on part of an image
    fn eval_partial<A: Type, B: Type, C: Color, D: Color>(
        &self,
        roi: Region,
        output: &mut Image<A, impl Color>,
        input: &[&Image<B, impl Color>],
    ) {
        let channels = output.channels();
        output
            .pixels_region_mut(roi)
            .for_each(|((x, y), pixel)| {
                for (c, px) in pixel.iter_mut().enumerate().take(channels) {
                    px.set_from_f64(self.compute_at(x, y, c, input));
                }
            });
    }

    /// Evaluate filter in parallel
    fn eval(&self, output: &mut Image<impl Type, impl Color>, input: &[&Image<impl Type, impl Color>]) {
        let channels = output.channels();
        output.for_each(|(x, y), pixel| {
            for (c, px) in pixel.iter_mut().enumerate().take(channels) {
                px.set_from_f64(self.compute_at(x, y, c, input));
            }
        });
    }

    fn join<
        'a,
        E: Color,
        Y: Color,
        A: 'a + Filter,
        B: 'a + Filter,
        F: Fn((usize, usize, usize), f64, f64) -> f64,
    >(
        &'a self,
        other: &'a B,
        f: F,
    ) -> Join<'a, Self, B, F> {
        Join {
            a: self,
            b: other,
            f,
        }
    }

    fn and_then<E: Color, F: Fn((usize, usize, usize), f64) -> f64>(&self, f: F) -> AndThen<Self, F> {
        AndThen {
            a: self,
            f,
        }
    }

    fn to_async<'a, T: Type, C: Color, U: Type, D: Color>(
        &'a self,
        mode: AsyncMode,
        output: &'a mut Image<U, D>,
        input: &'a [&Image<T, C>],
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

/// Executes `a` then `b` and passes the results to `f`
pub struct Join<
    'a,
    A: 'a + Filter,
    B: 'a + Filter,
    F: Fn((usize, usize, usize), f64, f64) -> f64,
> {
    a: &'a A,
    b: &'a B,
    f: F,
}

/// Executes `a` then `f(a)`
pub struct AndThen<'a, A: 'a + Filter, F: Fn((usize, usize, usize), f64) -> f64> {
    a: &'a A,
    f: F,
}

impl<'a, A: Filter, F: Sync + Fn((usize, usize, usize), f64) -> f64> Filter
    for AndThen<'a, A, F>
{
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, impl Color>]) -> f64 {
        (self.f)((x, y, c), self.a.compute_at(x, y, c, input))
    }
}

impl<
        'a,
        A: Filter,
        B: Filter,
        F: Sync + Fn((usize, usize, usize), f64, f64) -> f64,
    > Filter for Join<'a, A, B, F>
{
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, impl Color>]) -> f64 {
        (&self.f)((x, y, c), self.a.compute_at(x, y, c, input), self.b.compute_at(x, y, c, input))
    }
}

pub struct Invert;

impl Filter for Invert {
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, impl Color>]) -> f64 {
        1.0 - input[0].get_f(x, y, c)
    }
}

pub struct Blend;

impl Filter for Blend {
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, impl Color>]) -> f64 {
        (input[0].get_f(x, y, c) + input[1].get_f(x, y, c)) / 2.0
    }
}

pub struct Gamma(pub f64);

impl Filter for Gamma {
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, impl Color>]) -> f64 {
        input[0].get_f(x, y, c).powf(1.0 / self.0)
    }
}

pub enum AsyncMode {
    Pixel,
    Row,
}

impl Default for AsyncMode {
    fn default() -> AsyncMode {
        AsyncMode::Row
    }
}

pub struct AsyncFilter<'a, F: Filter, T: 'a + Type, C: Color, U: 'a + Type, D: Color = C> {
    pub filter: &'a F,
    pub output: &'a mut Image<U, D>,
    pub input: &'a [&'a Image<T, C>],
    x: usize,
    y: usize,
    mode: AsyncMode
}

impl<'a, F: Unpin + Filter, T: 'a + Type, C: Unpin + Color, U: 'a  + Unpin + Type,  D: Unpin + Color>
    AsyncFilter<'a, F, T, C, U, D>
{
    pub async fn eval(self) {
        self.await
    }
}

impl<'a, F: Unpin + Filter, T: Type, C: Color, U: Unpin + Type,  D: Unpin + Color>
    std::future::Future for AsyncFilter<'a, F, T, C, U, D>
{
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context,
    ) -> std::task::Poll<Self::Output> {
        let input = &self.input[0];
        let filter = std::pin::Pin::get_mut(self);


        match filter.mode {
            AsyncMode::Row => {
                for i in 0 .. input.width() {
                    for c in 0..C::CHANNELS {
                        let f = filter
                            .filter
                            .compute_at(i, filter.y, c, &filter.input);
                        filter.output.set_f(i, filter.y, c, f);
                    }
                }
                filter.y += 1;
            }
            AsyncMode::Pixel => {
                for c in 0..C::CHANNELS {
                    let f = filter
                        .filter
                        .compute_at(filter.x, filter.y, c, &filter.input);
                    filter.output.set_f(filter.x, filter.y, c, f);
                    filter.x += 1;
                    if filter.x >= input.width() {
                        filter.x = 0;
                        filter.y += 1;
                    }
                }
            }
        }


        if filter.y < input.height() {
            ctx.waker().wake_by_ref();
            return std::task::Poll::Pending;
        }

        std::task::Poll::Ready(())
    }
}

pub async fn eval_async<'a, F: Unpin + Filter, T: Type, U: Type, C: Color, D: Color>(filter: &'a F, mode: AsyncMode, output: &'a mut Image<T, D>, input: &'a [&Image<U, C>]) {
    filter.to_async(mode, output, input).await
}

