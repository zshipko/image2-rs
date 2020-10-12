use crate::*;

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

/// async-friendly `Pipeline`
pub struct AsyncPipeline<'a, T: 'a + Type, C: 'a + Color, U: 'a + Type, D: 'a + Color> {
    /// Underlying pipeline
    pub pipeline: &'a Pipeline<T, C, U, D>,

    /// Output image
    pub output: &'a mut Image<U, D>,

    /// Filter input
    pub input: Input<'a, T, C>,

    /// Intermediary image
    pub tmpconv: Image<T, C>,

    pub(crate) image_schedule_filters: Vec<usize>,
    pub(crate) j: usize,
    pub(crate) index: usize,
}

impl<'a, T: Type, C: Color, U: Unpin + Type, D: Unpin + Color> AsyncPipeline<'a, T, C, U, D> {
    /// Execute async pipeline
    pub async fn execute(self) {
        self.await
    }
}

impl<'a, T: Type, C: Color, U: Unpin + Type, D: Unpin + Color> std::future::Future
    for AsyncPipeline<'a, T, C, U, D>
{
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context,
    ) -> std::task::Poll<Self::Output> {
        let p = std::pin::Pin::get_mut(self);
        let pipeline = &p.pipeline;
        let j = p.j;
        let image_schedule_filters = &p.image_schedule_filters;
        let index = p.index;
        let input = &mut p.input;
        let output = &mut p.output;
        let tmpconv = &mut p.tmpconv;

        pipeline.loop_inner(input, output, tmpconv, j, index, &image_schedule_filters);

        if p.index != p.pipeline.filters.len() - 1 {
            p.j += 1;
            p.index = p.image_schedule_filters[p.j];

            ctx.waker().wake_by_ref();
            return std::task::Poll::Pending;
        }

        std::task::Poll::Ready(())
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
    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) mode: AsyncMode,
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
    /// Evaluate async filter
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

pub(crate) async fn eval_async<
    'a,
    F: Unpin + Filter<T, C, U, D>,
    T: Type,
    C: Color,
    U: Type,
    D: Color,
>(
    filter: &'a F,
    mode: AsyncMode,
    input: Input<'a, T, C>,
    output: &'a mut Image<U, D>,
) {
    filter.to_async(mode, input, output).await
}
