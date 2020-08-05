use crate::*;

/// Executes `a` then `b` and passes the results to `f`
pub struct Join<
    'a,
    C: Color,
    X: Color,
    Y: Color,
    E: Color,
    A: 'a + Filter<C, X>,
    B: 'a + Filter<C, Y>,
    F: Fn(Pixel<X>, Pixel<Y>) -> Pixel<E>,
> {
    a: &'a A,
    b: &'a B,
    f: F,
    _color: std::marker::PhantomData<C>,
    _color2: std::marker::PhantomData<X>,
    _color3: std::marker::PhantomData<Y>,
}

/// Executes `a` then `f(a)`
pub struct AndThen<
    'a,
    C: Color,
    D: Color,
    E: Color,
    A: 'a + Filter<C, D>,
    F: Fn(Pixel<D>) -> Pixel<E>,
> {
    a: &'a A,
    f: F,
    _color: std::marker::PhantomData<C>,
    _color2: std::marker::PhantomData<D>,
}

impl<'a, C: Color, D: Color, E: Color, A: Filter<C, D>, F: Sync + Fn(Pixel<D>) -> Pixel<E>>
    Filter<C, E> for AndThen<'a, C, D, E, A, F>
{
    fn compute_at(&self, x: usize, y: usize, input: &[&Image<impl Type, C>]) -> Pixel<E> {
        let f = &self.f;
        f(self.a.compute_at(x, y, input))
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
        F: Sync + Fn(Pixel<X>, Pixel<Y>) -> Pixel<E>,
    > Filter<C, E> for Join<'a, C, X, Y, E, A, B, F>
{
    fn compute_at(&self, x: usize, y: usize, input: &[&Image<impl Type, C>]) -> Pixel<E> {
        let f = &self.f;
        let a = self.a.compute_at(x, y, input);
        let b = self.b.compute_at(x, y, input);
        f(a, b)
    }
}

/// Filters are used to manipulate images in a generic, composable manner
pub trait Filter<C: Color, D: Color = C>: Sized + Sync {
    fn compute_at(&self, x: usize, y: usize, input: &[&Image<impl Type, C>]) -> Pixel<D>;

    /// Evaluate a filter on part of an image
    fn eval_partial(
        &self,
        start_x: usize,
        start_y: usize,
        width: usize,
        height: usize,
        output: &mut Image<impl Type, D>,
        input: &[&Image<impl Type, C>],
    ) {
        for y in start_y..start_y + height {
            for x in start_x..start_x + width {
                output.set_pixel(x, y, &self.compute_at(x, y, input));
            }
        }
    }

    /// Evaluate filter in parallel
    fn eval(&self, output: &mut Image<impl Type, impl Color>, input: &[&Image<impl Type, C>]) {
        output.for_each(|(x, y), pixel| {
            let px = self.compute_at(x, y, input);
            px.copy_to_slice(pixel);
        });
    }

    fn join<
        'a,
        E: Color,
        Y: Color,
        A: 'a + Filter<C, D>,
        B: 'a + Filter<C, Y>,
        F: Fn(Pixel<D>, Pixel<Y>) -> Pixel<E>,
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
            _color2: std::marker::PhantomData,
            _color3: std::marker::PhantomData,
        }
    }

    fn and_then<E: Color, F: Fn(Pixel<D>) -> Pixel<E>>(&self, f: F) -> AndThen<C, D, E, Self, F> {
        AndThen {
            a: self,
            f,
            _color: std::marker::PhantomData,
            _color2: std::marker::PhantomData,
        }
    }
}

pub struct Invert;

impl<C: Color> Filter<C> for Invert {
    fn compute_at(&self, x: usize, y: usize, input: &[&Image<impl Type, C>]) -> Pixel<C> {
        let max = input[0].type_max();
        input[0].get_pixel(x, y).map(|x| max - x)
    }
}

pub struct Blend;

impl<C: Color> Filter<C> for Blend {
    fn compute_at(&self, x: usize, y: usize, input: &[&Image<impl Type, C>]) -> Pixel<C> {
        (input[0].get_pixel(x, y) + input[2].get_pixel(x, y)) / 2.0
    }
}

pub struct Grayscale;

impl Filter<Rgb, Gray> for Grayscale {
    fn compute_at(&self, x: usize, y: usize, input: &[&Image<impl Type, Rgb>]) -> Pixel<Gray> {
        let mut dest = Pixel::new();
        let px = input[0].get_pixel(x, y);
        dest[0] = px[0] * 0.21 + px[1] * 0.72 + px[2] * 0.7;
        dest
    }
}

impl Filter<Rgba, Gray> for Grayscale {
    fn compute_at(&self, x: usize, y: usize, input: &[&Image<impl Type, Rgba>]) -> Pixel<Gray> {
        let mut dest = Pixel::new();
        let px = input[0].get_pixel(x, y);
        dest[0] = px[0] * 0.21 + px[1] * 0.72 + px[2] * 0.7 * px[3];
        dest
    }
}

pub struct Gamma(pub f64);

impl<C: Color> Filter<C> for Gamma {
    fn compute_at(&self, x: usize, y: usize, input: &[&Image<impl Type, C>]) -> Pixel<C> {
        input[0].get_pixel(x, y).map(|x| x.powf(1.0 / self.0))
    }
}

pub struct Convert<C: Color, D: Color, F: Fn(Pixel<C>) -> Pixel<D>> {
    f: F,
    _from: std::marker::PhantomData<C>,
    _to: std::marker::PhantomData<D>,
}

impl<C: Color, D: Color, F: Sync + Fn(Pixel<C>) -> Pixel<D>> Filter<C, D> for Convert<C, D, F> {
    fn compute_at(&self, x: usize, y: usize, input: &[&Image<impl Type, C>]) -> Pixel<D> {
        let px = input[0].get_pixel(x, y);
        (self.f)(px)
    }
}
