use crate::*;

/// Convert between colors
#[derive(Clone, Copy, Default, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Convert<T: Color>(std::marker::PhantomData<T>);

/// Create new color conversion filter
pub fn convert<T: Type, C: Color, U: Type, D: Color>() -> impl Filter<T, C, U, D> {
    Convert(std::marker::PhantomData)
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Convert<D> {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        input.get_pixel(pt, None).convert_to_data(dest);
    }
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Saturation(pub f64);

/// Adjust saturation
pub fn saturation<T: Type, C: Color, U: Type, D: Color>(amt: f64) -> impl Filter<T, C, U, D> {
    Saturation(amt)
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Saturation {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, data: &mut DataMut<U, D>) {
        let px = input.get_pixel(pt, None);
        let mut tmp: Pixel<Hsv> = px.convert();
        tmp[1] *= self.0;
        tmp.convert_to_data(data);
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Brightness(f64);

/// Adjust image brightness
pub fn brightness<T: Type, C: Color, U: Type, D: Color>(amt: f64) -> impl Filter<T, C, U, D> {
    Brightness(amt)
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Brightness {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, data: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px *= self.0;
        px.convert_to_data(data);
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Exposure(f64);

/// Adjust image exposure, the argument is the number of stops to increase or decrease exposure by
pub fn exposure<T: Type, C: Color, U: Type, D: Color>(stops: f64) -> impl Filter<T, C, U, D> {
    Exposure(stops)
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Exposure {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, data: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px *= 2f64.powf(self.0);
        px.convert_to_data(data);
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Contrast(pub f64);

/// Adjust image contrast
pub fn contrast<T: Type, C: Color, U: Type, D: Color>(amt: f64) -> impl Filter<T, C, U, D> {
    Contrast(amt)
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Contrast {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, data: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px.map(|x| (self.0 * (x - 0.5)) + 0.5);
        px.convert_to_data(data);
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Crop(Region);

/// Crop an image
pub fn crop<T: Type, C: Color, U: Type, D: Color>(r: Region) -> impl Filter<T, C, U, D> {
    Crop(r)
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

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Invert;

/// Invert an image
pub fn invert<T: Type, C: Color, U: Type, D: Color>() -> impl Filter<T, C, U, D> {
    Invert
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Invert {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px.map(|x| 1.0 - x);
        px.copy_to_slice(dest);
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Blend;

/// Average two images
pub fn blend<T: Type, C: Color, U: Type, D: Color>() -> impl Filter<T, C, U, D> {
    Blend
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Blend {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        let a = input.get_pixel(pt, None);
        let b = input.get_pixel(pt, Some(1));
        ((a + &b) / 2.).copy_to_slice(dest);
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct GammaLog(f64);

/// Convert to log gamma
pub fn gamma_log<T: Type, C: Color, U: Type, D: Color>(
    gamma: Option<f64>,
) -> impl Filter<T, C, U, D> {
    GammaLog(gamma.unwrap_or(2.2))
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for GammaLog {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px.map(|x| x.powf(1.0 / self.0));
        px.copy_to_slice(dest);
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct GammaLin(f64);

/// Convert to linear gamma
pub fn gamma_lin<T: Type, C: Color, U: Type, D: Color>(
    gamma: Option<f64>,
) -> impl Filter<T, C, U, D> {
    GammaLin(gamma.unwrap_or(2.2))
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for GammaLin {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        let mut px = input.get_pixel(pt, None);
        px.map(|x| x.powf(self.0));
        px.copy_to_slice(dest);
    }
}

/// Conditional filter
struct If<
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

/// Create new conditional filter
pub fn if_then_else<
    F: Sync + Fn(Point, &Input<T, C>) -> bool,
    G: Filter<T, C, U, D>,
    H: Filter<T, C, U, D>,
    T: Type,
    C: Color,
    U: Type,
    D: Color,
>(
    cond: F,
    then: G,
    else_: H,
) -> impl Filter<T, C, U, D> {
    If {
        cond,
        then,
        else_,
        _t: std::marker::PhantomData,
    }
}

impl<
        F: Fn(Point, &Input<T, C>) -> bool,
        G: Filter<T, C, U, D>,
        H: Filter<T, C, U, D>,
        T: Type,
        C: Color,
        U: Type,
        D: Color,
    > std::fmt::Debug for If<F, G, H, T, C, U, D>
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("If")
            .field("cond", &"Function")
            .field("then", &self.then)
            .field("else", &self.else_)
            .finish()
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

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Clamp;

/// Clamp pixel values
pub fn clamp<T: Type, C: Color, U: Type, D: Color>() -> impl Filter<T, C, U, D> {
    Clamp
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Clamp {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        input.get_pixel(pt, None).clamped().copy_to_slice(dest)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Normalize {
    min: f64,
    max: f64,
    new_min: f64,
    new_max: f64,
}

/// Normalize image data
pub fn normalize<T: Type, C: Color, U: Type, D: Color>(
    min: f64,
    max: f64,
    new_min: f64,
    new_max: f64,
) -> impl Filter<T, C, U, D> {
    Normalize {
        min,
        max,
        new_min,
        new_max,
    }
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Normalize {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        input
            .get_pixel(pt, None)
            .map(|x| {
                (x - self.min) * ((self.new_max - self.new_min) / (self.max - self.min))
                    + self.new_min
            })
            .copy_to_slice(dest)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Noop;

/// Filter that does nothing
pub fn noop<T: Type, C: Color, U: Type, D: Color>() -> impl Filter<T, C, U, D> {
    Noop
}

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Noop {
    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        input.get_pixel(pt, None).copy_to_slice(dest)
    }
}

#[inline]
/// Build rotation `Transform` using the specified degrees and center point
pub fn rotate<T: Type, C: Color, U: Type, D: Color>(
    deg: f64,
    center: Point,
) -> impl Filter<T, C, U, D> {
    let center = center.to_tuple();
    Transform::rotation(euclid::Angle::degrees(-deg))
        .pre_translate(euclid::Vector2D::new(
            -(center.0 as f64),
            -(center.1 as f64),
        ))
        .then_translate(euclid::Vector2D::new(center.0 as f64, center.1 as f64))
}

#[inline]
/// Build scale `Transform`
pub fn scale<T: Type, C: Color, U: Type, D: Color>(x: f64, y: f64) -> impl Filter<T, C, U, D> {
    Transform::scale(1.0 / x, 1.0 / y)
}

#[inline]
/// Build resize transform
pub fn resize<T: Type, C: Color, U: Type, D: Color>(
    from: Size,
    to: Size,
) -> impl Filter<T, C, U, D> {
    Transform::scale(
        from.width as f64 / to.width as f64,
        from.height as f64 / to.height as f64,
    )
}

/// 90 degree rotation
pub fn rotate90<T: Type, C: Color, U: Type, D: Color>(
    from: Size,
    to: Size,
) -> impl Filter<T, C, U, D> {
    let dwidth = to.width as f64;
    let height = from.height as f64;
    rotate(
        90.,
        Point::new((dwidth / 2.) as usize, (height / 2.) as usize),
    )
}

/// 180 degree rotation
pub fn rotate180<T: Type, C: Color, U: Type, D: Color>(src: Size) -> impl Filter<T, C, U, D> {
    let dwidth = src.width as f64;
    let height = src.height as f64;
    rotate(
        180.,
        Point::new((dwidth / 2.) as usize, (height / 2.) as usize),
    )
}

/// 270 degree rotation
pub fn rotate270<T: Type, C: Color, U: Type, D: Color>(
    from: Size,
    to: Size,
) -> impl Filter<T, C, U, D> {
    let width = to.height as f64;
    let dheight = from.width as f64;
    rotate(
        270.,
        Point::new((width / 2.) as usize, (dheight / 2.) as usize),
    )
}
