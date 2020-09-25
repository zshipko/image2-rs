use crate::*;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Pixel<C: Color>(Box<[f64]>, std::marker::PhantomData<C>);

impl<C: Color> AsRef<[f64]> for Pixel<C> {
    fn as_ref(&self) -> &[f64] {
        self.0.as_ref()
    }
}

impl<C: Color> Default for Pixel<C> {
    fn default() -> Self {
        Pixel::new()
    }
}

impl<C: Color> Pixel<C> {
    pub fn into_vec(self) -> Vec<f64> {
        self.0.to_vec()
    }

    pub fn new() -> Pixel<C> {
        Pixel(
            vec![0.0; C::CHANNELS].into_boxed_slice(),
            std::marker::PhantomData,
        )
        .with_alpha(1.0)
    }

    pub fn fill<T: Type>(mut self, x: T) -> Self {
        self.0.iter_mut().for_each(|a| *a = x.to_norm());
        self
    }

    pub fn len(&self) -> usize {
        C::CHANNELS
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_alpha(&self, index: usize) -> bool {
        if let Some(alpha) = C::ALPHA {
            return alpha == index;
        }

        false
    }

    pub fn with_alpha(mut self, value: f64) -> Self {
        if let Some(alpha) = C::ALPHA {
            self[alpha] = value
        }
        self
    }

    #[inline]
    pub fn copy_from_slice<T: Type>(&mut self, data: &[T]) -> &mut Self {
        for (i, px) in data.iter().enumerate() {
            if i >= C::CHANNELS {
                break;
            }
            (*self)[i] = px.to_norm();
        }
        self
    }

    pub fn copy_to_slice<T: Type>(&self, data: &mut [T]) {
        for i in 0..data.len() {
            if i >= C::CHANNELS {
                break;
            }
            data[i] = T::from_norm(self[i]);
        }
    }

    pub fn from_slice<T: Type>(data: &[T]) -> Pixel<C> {
        let mut px = Pixel::new();
        px.copy_from_slice(data);
        px
    }

    pub fn blend_alpha(mut self) -> Self {
        let index = self.len() - 1;
        let alpha = self[index];

        self.map_in_place(|x| x * alpha);
        self[index] = 1.0;
        self
    }

    pub fn map(mut self, f: impl Fn(f64) -> f64) -> Pixel<C> {
        for i in 0..self.len() {
            self[i] = f(self[i]);
        }
        self
    }

    pub fn map2(mut self, other: &Pixel<C>, f: impl Fn(f64, f64) -> f64) -> Pixel<C> {
        for i in 0..self.len() {
            self[i] = f(self[i], other[i])
        }
        self
    }

    pub fn map_in_place(&mut self, f: impl Fn(f64) -> f64) -> &mut Self {
        for i in 0..self.len() {
            (*self)[i] = f(self[i]);
        }
        self
    }

    pub fn map2_in_place(&mut self, other: &Pixel<C>, f: impl Fn(f64, f64) -> f64) -> &mut Self {
        for i in 0..self.len() {
            (*self)[i] = f(self[i], other[i]);
        }
        self
    }

    pub fn for_each(&self, mut f: impl FnMut(f64)) {
        for i in 0..self.len() {
            f(self[i])
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &f64> {
        let iter = self.0.iter();

        let alpha = C::ALPHA.unwrap_or(std::usize::MAX);
        iter.enumerate()
            .filter_map(move |(idx, item)| if idx != alpha { Some(item) } else { None })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f64> {
        let iter = self.0.iter_mut();
        let alpha = C::ALPHA.unwrap_or(std::usize::MAX);
        iter.enumerate()
            .filter_map(move |(idx, item)| if idx != alpha { Some(item) } else { None })
    }

    /// Gamma correction
    pub fn gamma(&mut self, value: f64) {
        self.map_in_place(|x| x.powf(value));
    }

    /// Convert to log RGB
    pub fn set_gamma_log(&mut self) {
        self.gamma(1. / 2.2)
    }

    /// Convert to linear RGB
    pub fn set_gamma_lin(&mut self) {
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
        self.0.to_vec().into_iter()
    }
}

impl<C: Color> std::ops::Index<usize> for Pixel<C> {
    type Output = f64;
    fn index(&self, index: usize) -> &f64 {
        &self.0[index]
    }
}

impl<'a, C: Color> std::ops::Index<usize> for &'a Pixel<C> {
    type Output = f64;
    fn index(&self, index: usize) -> &f64 {
        &self.0[index]
    }
}

impl<'a, C: Color> std::ops::Index<usize> for &'a mut Pixel<C> {
    type Output = f64;
    fn index(&self, index: usize) -> &f64 {
        &self.0[index]
    }
}

impl<C: Color> std::ops::IndexMut<usize> for Pixel<C> {
    fn index_mut(&mut self, index: usize) -> &mut f64 {
        &mut self.0[index]
    }
}

impl<'a, C: Color> std::ops::IndexMut<usize> for &'a mut Pixel<C> {
    fn index_mut(&mut self, index: usize) -> &mut f64 {
        &mut self.0[index]
    }
}

impl<T: Type, C: Color> std::ops::Add<T> for Pixel<C> {
    type Output = Pixel<C>;

    fn add(self, other: T) -> Pixel<C> {
        self.map(|x| x + other.to_norm())
    }
}

impl<C: Color> std::ops::Add<Pixel<C>> for Pixel<C> {
    type Output = Pixel<C>;

    fn add(self, other: Pixel<C>) -> Pixel<C> {
        self.map2(&other, |x, y| x + y)
    }
}

impl<T: Type, C: Color> std::ops::Sub<T> for Pixel<C> {
    type Output = Pixel<C>;

    fn sub(self, other: T) -> Pixel<C> {
        self.map(|x| x - other.to_norm())
    }
}

impl<C: Color> std::ops::Sub<Pixel<C>> for Pixel<C> {
    type Output = Pixel<C>;

    fn sub(self, other: Pixel<C>) -> Pixel<C> {
        self.map2(&other, |x, y| x - y)
    }
}

impl<T: Type, C: Color> std::ops::Mul<T> for Pixel<C> {
    type Output = Pixel<C>;

    fn mul(self, other: T) -> Pixel<C> {
        self.map(|x| x * other.to_norm())
    }
}

impl<C: Color> std::ops::Mul<Pixel<C>> for Pixel<C> {
    type Output = Pixel<C>;

    fn mul(self, other: Pixel<C>) -> Pixel<C> {
        self.map2(&other, |x, y| x * y)
    }
}

impl<T: Type, C: Color> std::ops::Div<T> for Pixel<C> {
    type Output = Pixel<C>;

    fn div(self, other: T) -> Pixel<C> {
        self.map(|x| x / other.to_norm())
    }
}

impl<C: Color> std::ops::Div<Pixel<C>> for Pixel<C> {
    type Output = Pixel<C>;

    fn div(self, other: Pixel<C>) -> Pixel<C> {
        self.map2(&other, |x, y| x / y)
    }
}

impl<T: Type, C: Color> std::ops::Rem<T> for Pixel<C> {
    type Output = Pixel<C>;

    fn rem(self, other: T) -> Pixel<C> {
        self.map(|x| x % other.to_norm())
    }
}

impl<C: Color> std::ops::Rem<Pixel<C>> for Pixel<C> {
    type Output = Pixel<C>;

    fn rem(self, other: Pixel<C>) -> Pixel<C> {
        self.map2(&other, |x, y| x % y)
    }
}

impl<T: Type, C: Color> std::ops::AddAssign<T> for Pixel<C> {
    fn add_assign(&mut self, other: T) {
        self.map_in_place(|x| x + other.to_norm());
    }
}

impl<C: Color> std::ops::AddAssign<Pixel<C>> for Pixel<C> {
    fn add_assign(&mut self, other: Pixel<C>) {
        self.map2_in_place(&other, |x, y| x + y);
    }
}

impl<T: Type, C: Color> std::ops::SubAssign<T> for Pixel<C> {
    fn sub_assign(&mut self, other: T) {
        self.map_in_place(|x| x - other.to_norm());
    }
}

impl<C: Color> std::ops::SubAssign<Pixel<C>> for Pixel<C> {
    fn sub_assign(&mut self, other: Pixel<C>) {
        self.map2_in_place(&other, |x, y| x - y);
    }
}

impl<T: Type, C: Color> std::ops::MulAssign<T> for Pixel<C> {
    fn mul_assign(&mut self, other: T) {
        self.map_in_place(|x| x * other.to_norm());
    }
}

impl<C: Color> std::ops::MulAssign<Pixel<C>> for Pixel<C> {
    fn mul_assign(&mut self, other: Pixel<C>) {
        self.map2_in_place(&other, |x, y| x * y);
    }
}

impl<T: Type, C: Color> std::ops::DivAssign<T> for Pixel<C> {
    fn div_assign(&mut self, other: T) {
        self.map_in_place(|x| x / other.to_norm());
    }
}

impl<C: Color> std::ops::DivAssign<Pixel<C>> for Pixel<C> {
    fn div_assign(&mut self, other: Pixel<C>) {
        self.map2_in_place(&other, |x, y| x / y);
    }
}

impl<T: Type, C: Color> std::ops::RemAssign<T> for Pixel<C> {
    fn rem_assign(&mut self, other: T) {
        self.map_in_place(|x| x % other.to_norm());
    }
}

impl<C: Color> std::ops::RemAssign<Pixel<C>> for Pixel<C> {
    fn rem_assign(&mut self, other: Pixel<C>) {
        self.map2_in_place(&other, |x, y| x % y);
    }
}
