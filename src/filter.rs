use crate::*;

use rayon::prelude::*;

pub use convert::{Convert, ConvertColor};

pub enum AsyncMode {
    Pixel,
    Row,
}

impl Default for AsyncMode {
    fn default() -> AsyncMode {
        AsyncMode::Row
    }
}

pub struct AsyncFilter<'a, F: Filter<C, D>, T: 'a + Type, U: 'a + Type, C: Color, D: Color = C> {
    pub filter: &'a F,
    pub output: &'a mut Image<T, D>,
    pub input: &'a [&'a Image<U, C>],
    x: usize,
    y: usize,
    mode: AsyncMode
}

impl<'a, F: Unpin + Filter<C, D>, T: 'a + Type, U: 'a  + Unpin + Type, C: Unpin + Color, D: Unpin + Color>
    AsyncFilter<'a, F, T, U, C, D>
{
    pub async fn eval(self) {
        self.await
    }
}

impl<'a, F: Unpin + Filter<C, D>, T: Type, U: Unpin + Type, C: Color, D: Unpin + Color>
    std::future::Future for AsyncFilter<'a, F, T, U, C, D>
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

        return std::task::Poll::Ready(());
    }
}

pub async fn eval_async<'a, F: Unpin + Filter<C, D>, T: Type, U: Type, C: Color, D: Color>(filter: &'a F, mode: AsyncMode, output: &'a mut Image<T, D>, input: &'a [&Image<U, C>]) {
    filter.to_async(mode, output, input).await
}

/// Filters are used to manipulate images in a generic, composable manner
pub trait Filter<C: Color, D: Color = C>: Sized + Sync {
    fn compute_at(&self, x: usize, y: usize, c: usize, input: &[&Image<impl Type, C>]) -> f64;

    fn to_async<'a, T: Type, U: Type>(
        &'a self,
        mode: AsyncMode,
        output: &'a mut Image<T, D>,
        input: &'a [&Image<U, C>],
    ) -> AsyncFilter<'a, Self, T, U, C, D> {
        AsyncFilter {
            mode,
            filter: self,
            input: input,
            output: output,
            x: 0,
            y: 0,
        }
    }


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
        output
            .pixels_rect_mut(start_x, start_y, width, height)
            .for_each(|((x, y), pixel)| {
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
        1.0 - input[0].get_f(x, y, c)
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

#[cfg(test)]
mod tests {
    use crate::*;
    use filter::*;

    #[test]
    fn test_async_filter() {
        let input = Image::<u16, Rgba>::open("images/A.exr").unwrap();
        let mut output = input.new_like();
        smol::run(eval_async(&Invert, AsyncMode::Row, &mut output, &[&input]));
        output.save("images/test-invert-async.jpg").unwrap();
    }
}
