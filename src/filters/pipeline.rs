use crate::*;

/// Used to determine if a filter can be executed and interleaved at the pixel level or if the
/// whole filter needs to be evaulated before moving to the next filter
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum Schedule {
    /// Allows pixel level composition
    Pixel,

    /// Only allows image level composition
    Image,
}

/// Pipelines are used to compose several filters
#[derive(Default)]
pub struct Pipeline<T: Type, C: Color, U: Type = T, D: Color = C> {
    pub(crate) filters: Vec<Box<dyn Filter<T, C, U, D>>>,
}

impl<T: Type, C: Color, U: Type, D: Color> Pipeline<T, C, U, D> {
    /// Create a new, empty pipeline
    pub fn new() -> Self {
        Pipeline {
            filters: Vec::new(),
        }
    }

    /// Add a filter to the pipeline
    pub fn push(&mut self, filter: impl 'static + Filter<T, C, U, D>) {
        self.filters.push(Box::new(filter));
    }

    /// Append a filter to a pipeline
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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn loop_inner<'a>(
        &self,
        input: &mut Input<'a, T, C>,
        output: &mut Image<U, D>,
        tmpconv: &mut Image<T, C>,
        j: usize,
        index: usize,
        image_schedule_filters: &[usize],
    ) {
        let current_filter = &self.filters[index];
        if current_filter.schedule() == Schedule::Image {
            let output_size = current_filter.output_size(input, output);
            if output_size != tmpconv.size() {
                *tmpconv = Image::new(output_size);
            }
        }
        output.iter_mut().for_each(|(pt, mut data)| {
            for f in self.filters[if j == 0 {
                0
            } else {
                image_schedule_filters[j - 1] + 1
            }..=index]
                .iter()
            {
                match f.schedule() {
                    Schedule::Pixel if j > 0 => {
                        let mut px = Pixel::new();
                        let input = input
                            .clone()
                            .with_pixel(pt, px.copy_from_data(&data.as_data()).convert());

                        f.compute_at(pt, &input, &mut data);
                    }
                    Schedule::Pixel => {
                        f.compute_at(pt, input, &mut data);
                    }
                    Schedule::Image => {
                        f.compute_at(pt, input, &mut data);
                    }
                }
            }
        });

        // Sketchy code in this block to allow re-use of `tmpconv`
        if index != self.filters.len() - 1 {
            output.convert_to(tmpconv);

            let tmp = tmpconv as *const _;
            input.images[0] = unsafe { &*tmp };
        }
    }

    /// Execute the pipeline
    pub fn execute(&self, input: &[&Image<T, C>], output: &mut Image<U, D>) {
        let mut input = Input::new(input);
        let image_schedule_filters = self.image_schedule_list();

        let mut tmpconv = Image::<T, C>::new(output.size());

        for (j, index) in image_schedule_filters.iter().enumerate() {
            self.loop_inner(
                &mut input,
                output,
                &mut tmpconv,
                j,
                *index,
                &image_schedule_filters,
            );
        }
    }

    /// Convert to `AsyncPipeline`
    pub fn to_async<'a>(
        &'a self,
        input: &'a [&'a Image<T, C>],
        output: &'a mut Image<U, D>,
    ) -> AsyncPipeline<'a, T, C, U, D> {
        let image_schedule_filters = self.image_schedule_list();
        let input = Input::new(input);
        let index = image_schedule_filters[0];
        let size = output.size();
        AsyncPipeline {
            pipeline: self,
            image_schedule_filters,
            j: 0,
            index,
            input,
            output,
            tmpconv: Image::<T, C>::new(size),
        }
    }
}

impl<T: Type, C: Color> Pipeline<T, C> {
    /// Execute the pipeline using the same input and output image
    pub fn execute_in_place(&self, output: &mut Image<T, C>) {
        let input = unsafe { &[&*(output as *const _)] };
        let mut input = Input::new(input);
        let image_schedule_filters = self.image_schedule_list();

        let mut tmpconv = Image::<T, C>::new(output.size());

        for (j, index) in image_schedule_filters.iter().enumerate() {
            self.loop_inner(
                &mut input,
                output,
                &mut tmpconv,
                j,
                *index,
                &image_schedule_filters,
            );
        }
    }
}
