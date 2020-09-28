use crate::*;

#[cfg(feature = "parallel")]
use rayon::prelude::*;


/// Filters are used to manipulate images in a generic, composable manner
pub trait Filter: Sized + Sync {
    /// Compute value of filter at a single point and channel
    fn compute_at(&self, pt: Point, c: Channel, input: &[&Image<impl Type, impl Color>]) -> f64;

    /// Evaluate a filter on part of an image
    fn eval_partial<A: Type, B: Type, C: Color, D: Color>(
        &self,
        roi: Region,
        input: &[&Image<B, impl Color>],
        output: &mut Image<A, impl Color>,
    ) {
        let channels = output.channels();

        let iter =
            output.iter_region_mut(roi);

        iter.for_each(|(pt, pixel)| {
            for (c, px) in pixel.iter_mut().enumerate().take(channels) {
                px.set_from_norm(self.compute_at(pt, c, input));
            }
        });
    }

    /// Evaluate filter in parallel
    fn eval(&self, input: &[&Image<impl Type, impl Color>], output: &mut Image<impl Type, impl Color>) {
        let channels = output.channels();
        output.for_each(|pt, pixel| {
            for (c, px) in pixel.iter_mut().enumerate().take(channels) {
                px.set_from_norm(self.compute_at(pt, c, input));
            }
        });
    }

    /// Join two filters
    fn join<
        'a,
        E: Color,
        Y: Color,
        A: 'a + Filter,
        B: 'a + Filter,
        F: Fn((Point, usize), f64, f64) -> f64,
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

    /// Perform one filter then another using the result of the first
    fn and_then<E: Color, F: Fn((Point, usize), f64) -> f64>(&self, f: F) -> AndThen<Self, F> {
        AndThen {
            a: self,
            f,
        }
    }

    /// Convert filter to `AsyncFilter`
    fn to_async<'a, T: Type, C: Color, U: Type, D: Color>(
        &'a self,
        mode: AsyncMode,
        input: &'a [&Image<T, C>],
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

/// Executes `a` then `b` and passes the results to `f`
pub struct Join<
    'a,
    A: 'a + Filter,
    B: 'a + Filter,
    F: Fn((Point, usize), f64, f64) -> f64,
> {
    a: &'a A,
    b: &'a B,
    f: F,
}

/// Executes `a` then `f(a)`
pub struct AndThen<'a, A: 'a + Filter, F: Fn((Point, usize), f64) -> f64> {
    a: &'a A,
    f: F,
}

impl<'a, A: Filter, F: Sync + Fn((Point, usize), f64) -> f64> Filter
    for AndThen<'a, A, F>
{
    fn compute_at(&self, pt: Point, c: usize, input: &[&Image<impl Type, impl Color>]) -> f64 {
        (self.f)((pt, c), self.a.compute_at(pt, c, input))
    }
}

impl<
        'a,
        A: Filter,
        B: Filter,
        F: Sync + Fn((Point, usize), f64, f64) -> f64,
    > Filter for Join<'a, A, B, F>
{
    fn compute_at(&self, pt: Point, c: usize, input: &[&Image<impl Type, impl Color>]) -> f64 {
        (&self.f)((pt, c), self.a.compute_at(pt, c, input), self.b.compute_at(pt, c, input))
    }
}

/// Invert an image
pub struct Invert;

impl Filter for Invert {
    fn compute_at(&self, pt: Point, c: usize, input: &[&Image<impl Type, impl Color>]) -> f64 {
        if input[0].meta.is_alpha_channel(c) {
            return input[0].get_f(pt, c);
        }

        1.0 - input[0].get_f(pt, c)
    }
}

/// Blend two images
pub struct Blend;

impl Filter for Blend {
    fn compute_at(&self, pt: Point, c: usize, input: &[&Image<impl Type, impl Color>]) -> f64 {
        (input[0].get_f(pt, c) + input[1].get_f(pt, c)) / 2.0
    }
}

/// Convert to log gamma
pub struct GammaLog(pub f64);

impl Filter for GammaLog {
    fn compute_at(&self, pt: Point, c: usize, input: &[&Image<impl Type, impl Color>]) -> f64 {
        input[0].get_f(pt, c).powf(1.0 / self.0)
    }
}

/// Convert to linear gamma
pub struct GammaLin(pub f64);

impl Filter for GammaLin {
    fn compute_at(&self, pt: Point, c: usize, input: &[&Image<impl Type, impl Color>]) -> f64 {
        input[0].get_f(pt, c).powf(self.0)
    }
}

/// AsyncMode is used to schedule the type of iteration for an `AsyncFilter`
pub enum AsyncMode {
    /// Apply to one pixel at a time
    Pixel,

    /// Apply to a row at a time
    Row,
}

impl Default for AsyncMode {
    fn default() -> AsyncMode {
        AsyncMode::Row
    }
}

/// A `Filter` that can be executed using async
pub struct AsyncFilter<'a, F: Filter, T: 'a + Type, C: Color, U: 'a + Type, D: Color = C> {
    /// Regular filter
    pub filter: &'a F,

    /// Output image
    pub output: &'a mut Image<U, D>,

    /// Input images
    pub input: &'a [&'a Image<T, C>],
    x: usize,
    y: usize,
    mode: AsyncMode
}

impl<'a, F: Unpin + Filter, T: 'a + Type, C: Unpin + Color, U: 'a  + Unpin + Type,  D: Unpin + Color>
    AsyncFilter<'a, F, T, C, U, D>
{
    /// Evaluate the filter
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
                            .compute_at(Point::new(i, filter.y), c, &filter.input);
                        filter.output.set_f((i, filter.y), c, f);
                    }
                }
                filter.y += 1;
            }
            AsyncMode::Pixel => {
                for c in 0..C::CHANNELS {
                    let f = filter
                        .filter
                        .compute_at(Point::new(filter.x, filter.y), c, &filter.input);
                    filter.output.set_f((filter.x, filter.y), c, f);
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

/// Evaluate a `Filter` as an async filter
pub async fn eval_async<'a, F: Unpin + Filter, T: Type, U: Type, C: Color, D: Color>(filter: &'a F, mode: AsyncMode, input: &'a [&Image<U, C>], output: &'a mut Image<T, D>) {
    filter.to_async(mode, input, output).await
}

