use crate::*;

/// Normalized image data
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Pixel<C: Color>(Box<[f64]>, std::marker::PhantomData<C>);

impl<C: Color> AsRef<[f64]> for Pixel<C> {
    fn as_ref(&self) -> &[f64] {
        self.0.as_ref()
    }
}

impl<C: Color> AsMut<[f64]> for Pixel<C> {
    fn as_mut(&mut self) -> &mut [f64] {
        self.0.as_mut()
    }
}

impl<C: Color> Default for Pixel<C> {
    fn default() -> Self {
        Pixel::new()
    }
}

impl<C: Color> Pixel<C> {
    /// Create an empty pixel
    pub fn new() -> Pixel<C> {
        let data = match C::CHANNELS {
            1 => Box::new([0.0; 1]) as Box<[f64]>,
            2 => Box::new([0.0; 2]),
            3 => Box::new([0.0; 3]),
            4 => Box::new([0.0; 4]),
            5 => Box::new([0.0; 5]),
            _ => vec![0.0; C::CHANNELS].into_boxed_slice(),
        };

        let mut px = Pixel(data, std::marker::PhantomData);
        px.with_alpha(1.0);
        px
    }

    /// Convert into a `Vec`
    pub fn into_vec(self) -> Vec<f64> {
        self.0.into_vec()
    }

    /// Copy and convert to a `Vec`
    pub fn to_vec(self) -> Vec<f64> {
        self.0.to_vec()
    }

    /// Fill a pixel with a single value
    pub fn fill<T: Type>(&mut self, x: T) -> &mut Self {
        self.0.iter_mut().for_each(|a| *a = x.to_norm());
        self
    }

    /// Pixel channel count
    pub fn len(&self) -> Channel {
        C::CHANNELS
    }

    /// Pixel has no channels
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true when the provided channel index matches the alpha channel index
    pub fn is_alpha(&self, index: Channel) -> bool {
        if let Some(alpha) = C::ALPHA {
            return alpha == index;
        }

        false
    }

    /// Get alpha value
    pub fn alpha(&self) -> Option<f64> {
        C::ALPHA.map(|x| self[x])
    }

    /// Set alpha value
    pub fn with_alpha(&mut self, value: f64) -> &mut Self {
        if let Some(alpha) = C::ALPHA {
            (*self)[alpha] = value
        }
        self
    }

    /// Convert pixel color type to an existing pixel
    pub fn convert_to<D: Color>(&self, dest: &mut Pixel<D>) {
        let mut tmp = Pixel::new();
        C::to_rgb(self, &mut tmp);
        D::from_rgb(&tmp, dest);
    }

    /// Convert pixel color type
    pub fn convert<D: Color>(&self) -> Pixel<D> {
        let mut dest = Pixel::new();
        self.convert_to(&mut dest);
        dest
    }

    /// Copy values from an existing slice
    #[inline]
    pub fn copy_from_slice<T: Type>(&mut self, data: &[T]) -> &mut Self {
        self.0.iter_mut().enumerate().for_each(|(i, x)| {
            *x = data[i].to_norm();
        });
        self
    }

    /// Copy values to an existing slice
    pub fn copy_to_slice<T: Type>(&self, data: &mut [T]) {
        self.0.iter().enumerate().for_each(|(i, x)| {
            data[i] = T::from_norm(*x);
        });
    }

    /// Create from slice
    pub fn from_slice<T: Type>(data: &[T]) -> Pixel<C> {
        let mut px = Pixel::new();
        px.copy_from_slice(data);
        px
    }

    /// Create from another pixel
    pub fn copy_from(&mut self, other: &Pixel<C>) -> &mut Self {
        self.map2(other, |_, b| b)
    }

    /// Blend alpha value
    pub fn blend_alpha(&mut self) -> &mut Self {
        if let Some(index) = C::ALPHA {
            let alpha = self[index];

            self.map(|x| x * alpha);
            (*self)[index] = 1.0;
        }

        self
    }

    /// Create a new pixel by applying `f` over an existing pixel
    pub fn map(&mut self, f: impl Fn(f64) -> f64) -> &mut Self {
        self.iter_mut().for_each(|x| *x = f(*x));
        self
    }

    /// Zip two pixels, apply `f` and return a new pixel with the results
    pub fn map2(&mut self, other: &Pixel<C>, f: impl Fn(f64, f64) -> f64) -> &mut Self {
        self.iter_mut()
            .zip(other.iter())
            .for_each(|(x, y)| *x = f(*x, *y));
        self
    }

    /// Apply `f` for each channel in a pixel
    pub fn for_each(&self, mut f: impl FnMut(f64)) {
        for i in 0..self.len() {
            f(self[i])
        }
    }

    /// Get iterator over pixel data, ignoring alpha channel
    pub fn iter(&self) -> impl Iterator<Item = &f64> {
        let iter = self.0.iter();

        let alpha = C::ALPHA.unwrap_or(std::usize::MAX);
        iter.enumerate()
            .filter_map(move |(idx, item)| if idx != alpha { Some(item) } else { None })
    }

    /// Get a mutable iterator over the pixel data, ignoring alpha channel
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f64> {
        let iter = self.0.iter_mut();
        let alpha = C::ALPHA.unwrap_or(std::usize::MAX);
        iter.enumerate()
            .filter_map(move |(idx, item)| if idx != alpha { Some(item) } else { None })
    }

    /// Gamma correction
    pub fn gamma(&mut self, value: f64) -> &mut Self {
        self.map(|x| x.powf(value))
    }

    /// Convert to log RGB
    pub fn gamma_log(&mut self) -> &mut Self {
        self.gamma(1. / 2.2)
    }

    /// Convert to linear RGB
    pub fn gamma_lin(&mut self) -> &mut Self {
        self.gamma(2.2)
    }
}

impl<T: Type, C: Color> std::iter::FromIterator<T> for Pixel<C> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Pixel(
            iter.into_iter().map(|x| x.to_norm()).collect(),
            std::marker::PhantomData,
        )
    }
}

impl<C: Color> IntoIterator for Pixel<C> {
    type Item = f64;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_vec().into_iter()
    }
}

impl<C: Color> std::ops::Index<Channel> for Pixel<C> {
    type Output = f64;
    fn index(&self, index: Channel) -> &f64 {
        &self.0[index]
    }
}

impl<'a, C: Color> std::ops::Index<Channel> for &'a Pixel<C> {
    type Output = f64;
    fn index(&self, index: Channel) -> &f64 {
        &self.0[index]
    }
}

impl<'a, C: Color> std::ops::Index<Channel> for &'a mut Pixel<C> {
    type Output = f64;
    fn index(&self, index: Channel) -> &f64 {
        &self.0[index]
    }
}

impl<C: Color> std::ops::IndexMut<Channel> for Pixel<C> {
    fn index_mut(&mut self, index: Channel) -> &mut f64 {
        &mut self.0[index]
    }
}

impl<'a, C: Color> std::ops::IndexMut<Channel> for &'a mut Pixel<C> {
    fn index_mut(&mut self, index: Channel) -> &mut f64 {
        &mut self.0[index]
    }
}

impl<T: Type, C: Color> std::ops::Add<T> for Pixel<C> {
    type Output = Pixel<C>;

    fn add(mut self, other: T) -> Pixel<C> {
        self.map(|x| x + other.to_norm());
        self
    }
}

impl<'a, T: Type, C: Color> std::ops::Add<T> for &'a Pixel<C> {
    type Output = Pixel<C>;

    fn add(self, other: T) -> Pixel<C> {
        let mut dest = self.clone();
        dest.map(|x| x + other.to_norm());
        dest
    }
}

impl<C: Color> std::ops::Add<Pixel<C>> for Pixel<C> {
    type Output = Pixel<C>;

    fn add(mut self, other: Pixel<C>) -> Pixel<C> {
        self.map2(&other, |x, y| x + y);
        self
    }
}

impl<T: Type, C: Color> std::ops::Sub<T> for Pixel<C> {
    type Output = Pixel<C>;

    fn sub(mut self, other: T) -> Pixel<C> {
        self.map(|x| x - other.to_norm());
        self
    }
}

impl<'a, T: Type, C: Color> std::ops::Sub<T> for &'a Pixel<C> {
    type Output = Pixel<C>;

    fn sub(self, other: T) -> Pixel<C> {
        let mut dest = self.clone();
        dest.map(|x| x - other.to_norm());
        dest
    }
}

impl<C: Color> std::ops::Sub<Pixel<C>> for Pixel<C> {
    type Output = Pixel<C>;

    fn sub(mut self, other: Pixel<C>) -> Pixel<C> {
        self.map2(&other, |x, y| x - y);
        self
    }
}

impl<T: Type, C: Color> std::ops::Mul<T> for Pixel<C> {
    type Output = Pixel<C>;

    fn mul(mut self, other: T) -> Pixel<C> {
        self.map(|x| x * other.to_norm());
        self
    }
}

impl<'a, T: Type, C: Color> std::ops::Mul<T> for &'a Pixel<C> {
    type Output = Pixel<C>;

    fn mul(self, other: T) -> Pixel<C> {
        let mut dest = self.clone();
        dest.map(|x| x * other.to_norm());
        dest
    }
}

impl<C: Color> std::ops::Mul<Pixel<C>> for Pixel<C> {
    type Output = Pixel<C>;

    fn mul(mut self, other: Pixel<C>) -> Pixel<C> {
        self.map2(&other, |x, y| x * y).clone()
    }
}

impl<T: Type, C: Color> std::ops::Div<T> for Pixel<C> {
    type Output = Pixel<C>;

    fn div(mut self, other: T) -> Pixel<C> {
        self.map(|x| x / other.to_norm());
        self
    }
}

impl<'a, T: Type, C: Color> std::ops::Div<T> for &'a Pixel<C> {
    type Output = Pixel<C>;

    fn div(self, other: T) -> Pixel<C> {
        let mut dest = self.clone();
        dest.map(|x| x / other.to_norm());
        dest
    }
}

impl<C: Color> std::ops::Div<Pixel<C>> for Pixel<C> {
    type Output = Pixel<C>;

    fn div(mut self, other: Pixel<C>) -> Pixel<C> {
        self.map2(&other, |x, y| x / y);
        self
    }
}

impl<T: Type, C: Color> std::ops::Rem<T> for Pixel<C> {
    type Output = Pixel<C>;

    fn rem(mut self, other: T) -> Pixel<C> {
        self.map(|x| x % other.to_norm());
        self
    }
}

impl<'a, T: Type, C: Color> std::ops::Rem<T> for &'a Pixel<C> {
    type Output = Pixel<C>;

    fn rem(self, other: T) -> Pixel<C> {
        let mut dest = self.clone();
        dest.map(|x| x % other.to_norm());
        dest
    }
}

impl<C: Color> std::ops::Rem<Pixel<C>> for Pixel<C> {
    type Output = Pixel<C>;

    fn rem(mut self, other: Pixel<C>) -> Pixel<C> {
        self.map2(&other, |x, y| x % y);
        self
    }
}

impl<T: Type, C: Color> std::ops::AddAssign<T> for Pixel<C> {
    fn add_assign(&mut self, other: T) {
        self.map(|x| x + other.to_norm());
    }
}

impl<C: Color> std::ops::AddAssign<Pixel<C>> for Pixel<C> {
    fn add_assign(&mut self, other: Pixel<C>) {
        self.map2(&other, |x, y| x + y);
    }
}

impl<'a, C: Color> std::ops::AddAssign<&'a Pixel<C>> for Pixel<C> {
    fn add_assign(&mut self, other: &'a Pixel<C>) {
        self.map2(&other, |x, y| x + y);
    }
}

impl<T: Type, C: Color> std::ops::SubAssign<T> for Pixel<C> {
    fn sub_assign(&mut self, other: T) {
        self.map(|x| x - other.to_norm());
    }
}

impl<C: Color> std::ops::SubAssign<Pixel<C>> for Pixel<C> {
    fn sub_assign(&mut self, other: Pixel<C>) {
        self.map2(&other, |x, y| x - y);
    }
}

impl<'a, C: Color> std::ops::SubAssign<&'a Pixel<C>> for Pixel<C> {
    fn sub_assign(&mut self, other: &'a Pixel<C>) {
        self.map2(&other, |x, y| x - y);
    }
}

impl<T: Type, C: Color> std::ops::MulAssign<T> for Pixel<C> {
    fn mul_assign(&mut self, other: T) {
        self.map(|x| x * other.to_norm());
    }
}

impl<C: Color> std::ops::MulAssign<Pixel<C>> for Pixel<C> {
    fn mul_assign(&mut self, other: Pixel<C>) {
        self.map2(&other, |x, y| x * y);
    }
}

impl<'a, C: Color> std::ops::MulAssign<&'a Pixel<C>> for Pixel<C> {
    fn mul_assign(&mut self, other: &'a Pixel<C>) {
        self.map2(&other, |x, y| x * y);
    }
}

impl<T: Type, C: Color> std::ops::DivAssign<T> for Pixel<C> {
    fn div_assign(&mut self, other: T) {
        self.map(|x| x / other.to_norm());
    }
}

impl<C: Color> std::ops::DivAssign<Pixel<C>> for Pixel<C> {
    fn div_assign(&mut self, other: Pixel<C>) {
        self.map2(&other, |x, y| x / y);
    }
}

impl<'a, C: Color> std::ops::DivAssign<&'a Pixel<C>> for Pixel<C> {
    fn div_assign(&mut self, other: &'a Pixel<C>) {
        self.map2(&other, |x, y| x / y);
    }
}

impl<T: Type, C: Color> std::ops::RemAssign<T> for Pixel<C> {
    fn rem_assign(&mut self, other: T) {
        self.map(|x| x % other.to_norm());
    }
}

impl<C: Color> std::ops::RemAssign<Pixel<C>> for Pixel<C> {
    fn rem_assign(&mut self, other: Pixel<C>) {
        self.map2(&other, |x, y| x % y);
    }
}

impl<'a, C: Color> std::ops::RemAssign<&'a Pixel<C>> for Pixel<C> {
    fn rem_assign(&mut self, other: &'a Pixel<C>) {
        self.map2(&other, |x, y| x % y);
    }
}
