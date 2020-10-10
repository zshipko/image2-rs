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
pub trait Filter<T: Type, C: Color, U: Type = T, D: Color = C>: Sync {
    /// Determines whether an image can accept pixels or full images from the previous filter in a
    /// pipeline
    fn schedule(&self) -> Schedule {
        Schedule::Pixel
    }

    /// Get filter output size
    fn output_size(&self, _input: &Input<T, C>, dest: &mut Image<U, D>) -> Size {
        dest.size()
    }

    /// Compute filter at the given point for the provided input
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
