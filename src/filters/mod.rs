use crate::*;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

mod r#async;
mod ext;
mod input;
mod pipeline;

/// Image processing filters
pub mod filter;

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
