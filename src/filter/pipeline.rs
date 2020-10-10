use crate::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Schedule {
    Pixel,
    Image,
}

pub struct Pipeline<T: Type, C: Color, U: Type = T, D: Color = C> {
    pub(crate) filters: Vec<Box<dyn Filter<T, C, U, D>>>,
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

    pub(crate) fn loop_inner<'a>(
        &self,
        input: &mut Input<'a, T, C>,
        output: &mut Image<U, D>,
        tmpconv: &mut Image<T, C>,
        j: usize,
        index: usize,
        image_schedule_filters: &[usize],
        input_images: &mut Vec<&'a Image<T, C>>,
    ) {
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

        // Sketchy code in this block to allow re-use of some resources
        // - tmpconv
        // - input_images
        if index != self.filters.len() - 1 {
            let tmp = tmpconv as *const _;
            output.convert_to(tmpconv);
            input_images[0] = unsafe { &*tmp };
            input.images = unsafe { &*(input_images.as_slice() as *const _) };
        }
    }

    pub fn execute(&self, input: &[&Image<T, C>], output: &mut Image<U, D>) {
        let mut input = Input::new(input);
        let mut input_images = input.images.to_vec();
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
                &mut input_images,
            );
        }
    }

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
            input_images: input.images.to_vec(),
            input,
            output,
            tmpconv: Image::<T, C>::new(size),
        }
    }
}
