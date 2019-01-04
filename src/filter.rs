use crate::color::Color;
use crate::image::Image;
use crate::ty::Type;

/// Executes `a` then `b` and passes the results to `f`
pub struct Join<'a, A: 'a + Filter, B: Filter, F: Fn(f64, f64) -> f64> {
    a: &'a A,
    b: B,
    f: F,
}

/// Executes `a` then `f(a)`
pub struct AndThen<'a, A: 'a + Filter, F: Fn(f64) -> f64> {
    a: &'a A,
    f: F,
}

impl<'a, A: Filter, F: Sync + Fn(f64) -> f64> Filter for AndThen<'a, A, F> {
    fn compute_at<T: Type, C: Color, I: Image<T, C>>(
        &self,
        x: usize,
        y: usize,
        c: usize,
        input: &[&I],
    ) -> f64 {
        let f = &self.f;
        f(self.a.compute_at(x, y, c, input))
    }
}

impl<'a, A: Filter, B: Filter, F: Sync + Fn(f64, f64) -> f64> Filter for Join<'a, A, B, F> {
    fn compute_at<T: Type, C: Color, I: Image<T, C>>(
        &self,
        x: usize,
        y: usize,
        c: usize,
        input: &[&I],
    ) -> f64 {
        let f = &self.f;
        f(
            self.a.compute_at(x, y, c, input),
            self.b.compute_at(x, y, c, input),
        )
    }
}

/// Filters are used to manipulate images in a generic, composable manner
pub trait Filter: Sized + Sync {
    fn compute_at<T: Type, C: Color, I: Image<T, C>>(
        &self,
        x: usize,
        y: usize,
        c: usize,
        input: &[&I],
    ) -> f64;

    fn eval_s<T: Type, C: Color, U: Type, D: Color, I: Image<T, C>, J: Image<U, D>>(
        &self,
        output: &mut I,
        input: &[&J],
    ) {
        let (width, height, channels) = output.shape();
        for y in 0..height {
            for x in 0..width {
                for c in 0..channels {
                    output.set_f(x, y, c, T::clamp(self.compute_at(x, y, c, input)));
                }
            }
        }
    }

    fn eval<
        T: Send + Type,
        C: Color,
        U: Type,
        D: Color,
        I: Sync + Send + Image<T, C>,
        J: Sync + Image<U, D>,
    >(
        &self,
        output: &mut I,
        input: &[&J],
    ) {
        output.for_each(|(x, y), pixel| {
            for c in 0..C::channels() {
                pixel[c] = T::from_float(T::denormalize(T::clamp(self.compute_at(x, y, c, input))));
            }
        });
    }

    fn join<A: Filter, F: Fn(f64, f64) -> f64>(&self, other: A, f: F) -> Join<Self, A, F> {
        Join {
            a: self,
            b: other,
            f,
        }
    }

    fn and_then<F: Fn(f64) -> f64>(&self, f: F) -> AndThen<Self, F> {
        AndThen { a: self, f }
    }
}

/// filter is used to simplify the process of defining `compute_at` to create new filters
#[macro_export]
macro_rules! filter {
    ($name:ident, $input:ident, $f:expr) => {
        impl $crate::Filter for $name {
            fn compute_at<T: crate::Type, C: crate::Color, I: crate::Image<T, C>>(
                &self,
                x: usize,
                y: usize,
                c: usize,
                $input: &[&I],
            ) -> f64 {
                $f(self, x, y, c)
            }
        }
    };
    (.$name:ident, $input:ident, $f:expr) => {
        pub struct $name;

        filter!($name, $input, $f);
    };
}

/// Invert subtracts the current component value from the maxiumum
filter!(.Invert, input, |_, x, y, c| T::max_f()
    - input[0].get_f(x, y, c));

/// Blend takes the average of two images
filter!(.Blend, input, |_, x, y, c| (input[0].get_f(x, y, c)
    + input[1].get_f(x, y, c))
    / 2.0);

/// ToGrayscale converts an Rgb or Rgba image to Gray
filter!(.ToGrayscale, input, |_, x, y, _c| {
    let a: &I = input[0];
    let v = a.get_f(x, y, 0) * 0.21 + a.get_f(x, y, 1) * 0.72 + a.get_f(x, y, 2) * 0.07;
    if C::has_alpha() {
        return v * a.get_f(x, y, C::channels() - 1);
    }
    v
});

/// ToColor converts a Gray image to Rgb or Rgba
filter!(.ToColor, input, |_, x, y, c| {
    if c == 4 {
        return T::max_f();
    }

    input[0].get_f(x, y, c % C::channels())
});

/// Returns a new pixel with premultiplied alpha values
filter!(.AlphaBlend, input, |_, x, y, c| {
    let a = input[0];

    if c == a.channels() - 1 {
        return 1.0;
    }

    a.get_f(x, y, c) * a.get_f(x, y, a.channels() - 1)
});

pub struct SwapChannel(pub usize, pub usize);
/// Swaps a value from one channel to another
filter!(
    SwapChannel,
    input,
    |this: &SwapChannel, x, y, c| if c == this.0 {
        input[0].get_f(x, y, this.1)
    } else if c == this.1 {
        input[0].get_f(x, y, this.0)
    } else {
        input[0].get_f(x, y, c)
    }
);
