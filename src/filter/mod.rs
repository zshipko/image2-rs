#![allow(missing_docs)]

use crate::*;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

mod input;

pub use input::Input;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Schedule {
    Pixel,
    Image,
    //Row,
    //Region,
}

pub struct Pipeline<T: Type, C: Color, U: Type = T, D: Color = C> {
    filters: Vec<Box<dyn Filter<T, C, U, D>>>,
}

impl<T: Type, C: Color, U: Type, D: Color> Pipeline<T, C, U, D> {
    pub fn new() -> Self {
        Pipeline {
            filters: Vec::new(),
        }
    }

    pub fn then(mut self, filter: impl 'static + Filter<T, C, U, D>) -> Self {
        self.filters.push(Box::new(filter));
        self
    }

    fn image_schedule_list(&self) -> Vec<usize> {
        let mut dest = Vec::new();
        for (i, f) in self.filters.iter().enumerate() {
            if f.schedule() == Schedule::Image {
                dest.push(i);
            }
        }
        dest.push(self.filters.len() - 1);
        dest
    }

    pub fn execute(&self, input: &[&Image<T, C>], output: &mut Image<U, D>) {
        let mut input = Input::new(input);
        let mut input_images = std::collections::VecDeque::from(input.images.to_vec());
        let image_schedule_filters = self.image_schedule_list();
        let mut tmp = if image_schedule_filters.len() == 1 {
            None
        } else {
            Some(Image::new(output.size()))
        };

        let mut tmpconv = if image_schedule_filters.len() == 1 {
            None
        } else {
            Some(Image::<T, C>::new(output.size()))
        };

        for (j, index) in image_schedule_filters.iter().enumerate() {
            tmp.as_mut()
                .unwrap_or(output)
                .iter_mut()
                .for_each(|(pt, mut data)| {
                    let mut px = Pixel::new();
                    for f in self.filters[if j == 0 {
                        0
                    } else {
                        image_schedule_filters[j - 1] + 1
                    }..=*index]
                        .iter()
                    {
                        match f.schedule() {
                            Schedule::Pixel if j > 0 => {
                                let input = input
                                    .clone()
                                    .with_pixel(pt, px.copy_from_data(&data.as_data()).convert());

                                f.compute_at(pt, &input, &mut data);
                            }
                            Schedule::Pixel => {
                                f.compute_at(pt, &input, &mut data);
                            }
                            Schedule::Image => {
                                f.compute_at(pt, &input, &mut data);
                            }
                        }
                    }
                });

            if let Some(tmp) = &tmp {
                {
                    let tmpconv_ = tmpconv.as_mut().unwrap();
                    tmp.convert_to(tmpconv_);

                    let _ = input_images.pop_front();
                    input_images.push_front(unsafe { std::mem::transmute(tmpconv_) });
                    input.images = unsafe { std::mem::transmute(input_images.make_contiguous()) };
                }
            }
        }

        if let Some(tmp) = tmp {
            output.data.copy_from_slice(&tmp.data)
        }
    }
}

/// Filters are used to manipulate images in a generic, composable manner
pub trait Filter<T: Type, C: Color, U: Type = T, D: Color = C>: Sync {
    fn schedule(&self) -> Schedule {
        Schedule::Pixel
    }

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

    /// Evaluate filter in parallel
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

pub fn combine<
    T: Type,
    C: Color,
    U: Type,
    D: Color,
    A: Filter<T, C, U, D>,
    B: Filter<T, C, U, D>,
    F: Fn(Point, Pixel<D>, Pixel<D>) -> Pixel<D>,
>(
    a: A,
    b: B,
    f: F,
) -> Combine<T, C, U, D, A, B, F> {
    Combine {
        a,
        b,
        f,
        _t: std::marker::PhantomData,
    }
}

/// Convert filter to `AsyncFilter`
pub fn to_async<'a, T: Type, C: Color, U: Type, D: Color, F: Filter<T, C, U, D>>(
    f: &'a F,
    mode: AsyncMode,
    input: Input<'a, T, C>,
    output: &'a mut Image<U, D>,
) -> AsyncFilter<'a, F, T, C, U, D> {
    AsyncFilter {
        mode,
        filter: f,
        input,
        output,
        x: 0,
        y: 0,
    }
}

/// Saturation
pub struct Saturation(pub f64);

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Saturation {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, data: &mut DataMut<U, D>) {
        let px = input.get_pixel(pt, None);
        let mut tmp: Pixel<Hsv> = px.convert();
        tmp[1] *= self.0;
        tmp.convert_to_data(data);
    }
}

/// Adjust image brightness
pub struct Brightness(pub f64);

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Brightness {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, data: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px *= self.0;
        px.convert_to_data(data);
    }
}

/// Adjust image contrast
pub struct Contrast(pub f64);

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Contrast {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, data: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px.map(|x| (self.0 * (x - 0.5)) + 0.5);
        px.convert_to_data(data);
    }
}

/// Crop an image
pub struct Crop(pub Region);

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Crop {
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
pub struct Invert;

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Invert {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px.map(|x| 1.0 - x);
        px.copy_to_slice(dest);
    }
}

/// Blend two images
pub struct Blend;

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Blend {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        let a = input.get_pixel(pt, None);
        let b = input.get_pixel(pt, Some(1));
        ((a + &b) / 2.).copy_to_slice(dest);
    }
}

/// Convert to log gamma
pub struct GammaLog(pub f64);

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
pub struct GammaLin(pub f64);

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
pub struct AsyncFilter<
    'a,
    F: Filter<T, C, U, D>,
    T: 'a + Type,
    C: Color,
    U: 'a + Type,
    D: Color = C,
> {
    /// Regular filter
    pub filter: &'a F,

    /// Output image
    pub output: &'a mut Image<U, D>,

    /// Input images
    pub input: Input<'a, T, C>,
    x: usize,
    y: usize,
    mode: AsyncMode,
}

impl<
        'a,
        F: Unpin + Filter<T, C, U, D>,
        T: 'a + Type,
        C: Unpin + Color,
        U: 'a + Unpin + Type,
        D: Unpin + Color,
    > AsyncFilter<'a, F, T, C, U, D>
{
    /// Evaluate the filter
    pub async fn eval(self) {
        self.await
    }
}

impl<'a, F: Unpin + Filter<T, C, U, D>, T: Type, C: Color, U: Unpin + Type, D: Unpin + Color>
    std::future::Future for AsyncFilter<'a, F, T, C, U, D>
{
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context,
    ) -> std::task::Poll<Self::Output> {
        let filter = std::pin::Pin::get_mut(self);
        let width = filter.output.width();
        let height = filter.output.height();

        match filter.mode {
            AsyncMode::Row => {
                for i in 0..width {
                    let mut data = filter.output.get_mut((i, filter.y));
                    filter
                        .filter
                        .compute_at(Point::new(i, filter.y), &filter.input, &mut data);
                }
                filter.y += 1;
            }
            AsyncMode::Pixel => {
                let mut data = filter.output.get_mut((filter.x, filter.y));
                filter
                    .filter
                    .compute_at(Point::new(filter.x, filter.y), &&filter.input, &mut data);
                filter.x += 1;
                if filter.x >= width {
                    filter.x = 0;
                    filter.y += 1;
                }
            }
        }

        if filter.y < height {
            ctx.waker().wake_by_ref();
            return std::task::Poll::Pending;
        }

        std::task::Poll::Ready(())
    }
}

/// Evaluate a `Filter` as an async filter
pub async fn eval_async<'a, F: Unpin + Filter<T, C, U, D>, T: Type, C: Color, U: Type, D: Color>(
    filter: &'a F,
    mode: AsyncMode,
    input: Input<'a, T, C>,
    output: &'a mut Image<U, D>,
) {
    to_async(filter, mode, input, output).await
}
