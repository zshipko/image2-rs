use crate::*;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

mod r#async;
mod ext;
mod input;
mod pipeline;

pub use ext::*;
pub use input::Input;
pub use pipeline::*;
pub use r#async::*;

/// Filters are used to manipulate images in a generic, composable manner
pub trait Filter<T: Type, C: Color, U: Type = T, D: Color = C>: std::fmt::Debug + Sync {
    /// Determines whether a filter should be executed one pixel at a time, or a whole image at a time
    fn schedule(&self) -> Schedule {
        Schedule::Pixel
    }

    /// Get filter output size, this is typically the destination image size, however when used as
    /// part of a pipeline a single filter might have a different output size
    fn output_size(&self, _input: &Input<T, C>, dest: &mut Image<U, D>) -> Size {
        dest.size()
    }

    /// Compute filter at the given point for the provided input
    ///
    /// - `pt`: Current output point
    /// - `input`: Input images, input pixel from previous filters in chain
    /// - `dest`: Single pixel output buffer
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>);

    /// Evaluate a filter on part of an image
    fn eval_partial(&self, roi: Region, input: &[&Image<T, C>], output: &mut Image<U, D>) {
        let input = Input::new(input);

        let iter = output.iter_region_mut(roi);
        iter.for_each(|(pt, mut data)| {
            self.compute_at(pt, &input, &mut data);
        });
    }

    /// Evaluate filter on part of an image using the same image for input and output
    fn eval_partial_in_place(&self, roi: Region, output: &mut Image<U, D>) {
        let input = output as *mut _ as *const _;
        let input = unsafe { &[&*input] };

        let input = Input::new(input);

        output.iter_region_mut(roi).for_each(|(pt, mut data)| {
            self.compute_at(pt, &input, &mut data);
        });
    }

    /// Evaluate filter
    fn eval(&self, input: &[&Image<T, C>], output: &mut Image<U, D>) {
        let input = Input::new(input);

        output.for_each(|pt, mut data| {
            self.compute_at(pt, &input, &mut data);
        });
    }

    /// Evaluate filter using the same image for input and output
    fn eval_in_place(&self, output: &mut Image<U, D>) {
        let input = output as *mut _ as *const _;
        let input = unsafe { &[&*input] };

        let input = Input::new(input);

        output.for_each(|pt, mut data| {
            self.compute_at(pt, &input, &mut data);
        });
    }
}

/// Saturation
#[derive(Debug, Default)]
pub struct Saturation(f64);

impl Saturation {
    /// Create new saturation filter
    pub fn new(amt: f64) -> Self {
        Saturation(amt)
    }
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Saturation {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, data: &mut DataMut<U, D>) {
        let px = input.get_pixel(pt, None);
        let mut tmp: Pixel<Hsv> = px.convert();
        tmp[1] *= self.0;
        tmp.convert_to_data(data);
    }
}

/// Adjust image brightness
#[derive(Debug, Default)]
pub struct Brightness(f64);

impl Brightness {
    /// Create new brightness filter
    pub fn new(amt: f64) -> Self {
        Brightness(amt)
    }
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Brightness {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, data: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px *= self.0;
        px.convert_to_data(data);
    }
}

/// Adjust image contrast
#[derive(Debug, Default)]
pub struct Contrast(f64);

impl Contrast {
    /// Create new contrast filter
    pub fn new(amt: f64) -> Self {
        Contrast(amt)
    }
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Contrast {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, data: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px.map(|x| (self.0 * (x - 0.5)) + 0.5);
        px.convert_to_data(data);
    }
}

/// Crop an image
#[derive(Debug, Default)]
pub struct Crop(Region);

impl Crop {
    /// Create new crop filter
    pub fn new(r: Region) -> Self {
        Crop(r)
    }
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Crop {
    fn output_size(&self, _input: &Input<T, C>, _dest: &mut Image<U, D>) -> Size {
        self.0.size
    }

    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        if pt.x > self.0.origin.x + self.0.size.width || pt.y > self.0.origin.y + self.0.size.height
        {
            return;
        }

        let x = pt.x + self.0.origin.x;
        let y = pt.y + self.0.origin.y;
        let px = input.get_pixel((x, y), None);
        px.copy_to_slice(dest);
    }
}

/// Invert an image
#[derive(Debug, Default)]
pub struct Invert;

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Invert {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px.map(|x| 1.0 - x);
        px.copy_to_slice(dest);
    }
}

impl Invert {
    /// Invert image colors
    fn new() -> Self {
        Default::default()
    }
}

/// Blend two images
#[derive(Debug, Default)]
pub struct Blend;

impl Blend {
    /// Blend two images
    fn new() -> Self {
        Default::default()
    }
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Blend {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        let a = input.get_pixel(pt, None);
        let b = input.get_pixel(pt, Some(1));
        ((a + &b) / 2.).copy_to_slice(dest);
    }
}

/// Convert to log gamma
#[derive(Debug)]
pub struct GammaLog(f64);

impl GammaLog {
    /// Create new log gamma filter
    pub fn new(amt: f64) -> Self {
        GammaLog(amt)
    }
}

impl Default for GammaLog {
    fn default() -> GammaLog {
        GammaLog(2.2)
    }
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for GammaLog {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px.map(|x| x.powf(1.0 / self.0));
        px.copy_to_slice(dest);
    }
}

/// Convert to linear gamma
#[derive(Debug)]
pub struct GammaLin(f64);

impl GammaLin {
    /// Create new linear gamma filter
    pub fn new(amt: f64) -> Self {
        GammaLin(amt)
    }
}

impl Default for GammaLin {
    fn default() -> GammaLin {
        GammaLin(2.2)
    }
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for GammaLin {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px.map(|x| x.powf(self.0));
        px.copy_to_slice(dest);
    }
}

/// Conditional filter
#[derive(Debug)]
pub struct If<
    F: Fn(Point, &Input<T, C>) -> bool,
    G: Filter<T, C, U, D>,
    H: Filter<T, C, U, D>,
    T: Type,
    C: Color,
    U: Type,
    D: Color,
> {
    cond: F,
    then: G,
    else_: H,
    _t: std::marker::PhantomData<(T, C, U, D)>,
}

impl<
        F: Fn(Point, &Input<T, C>) -> bool,
        G: Filter<T, C, U, D>,
        H: Filter<T, C, U, D>,
        T: Type,
        C: Color,
        U: Type,
        D: Color,
    > If<F, G, H, T, C, U, D>
{
    /// Create new conditional filter
    pub fn new(cond: F, then: G, else_: H) -> Self {
        If {
            cond,
            then,
            else_,
            _t: std::marker::PhantomData,
        }
    }
}

impl<
        F: Sync + Fn(Point, &Input<T, C>) -> bool,
        G: Filter<T, C, U, D>,
        H: Filter<T, C, U, D>,
        T: Type,
        C: Color,
        U: Type,
        D: Color,
    > Filter<T, C, U, D> for If<F, G, H, T, C, U, D>
{
    fn schedule(&self) -> Schedule {
        if self.then.schedule() == Schedule::Image || self.else_.schedule() == Schedule::Image {
            return Schedule::Image;
        }

        Schedule::Pixel
    }

    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        if (self.cond)(pt, input) {
            self.then.compute_at(pt, input, dest)
        } else {
            self.else_.compute_at(pt, input, dest)
        }
    }
}

/// Filter that does nothing
#[derive(Debug, Default)]
pub struct Noop;

impl Noop {
    /// Create new no-op filter
    fn new() -> Self {
        Default::default()
    }
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Noop {
    fn compute_at(&self, _pt: Point, _input: &Input<T, C>, _dest: &mut DataMut<U, D>) {}
}
