use crate::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Pixel<C: Color>(Vec<f64>, std::marker::PhantomData<C>);

impl<C: Color> Pixel<C> {
    pub fn from_vec(v: Vec<f64>) -> Pixel<C> {
        Pixel(v, std::marker::PhantomData)
    }

    pub fn into_vec(self) -> Vec<f64> {
        self.0
    }

    pub fn new_len(len: usize) -> Pixel<C> {
        Self::from_vec(vec![0.0; len])
    }

    pub fn new() -> Pixel<C> {
        Pixel::new_len(C::CHANNELS)
    }

    pub fn fill<T: Type>(mut self, x: T) -> Pixel<C> {
        self.0.iter_mut().for_each(|dest| *dest = x.to_norm());
        self
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn copy_from_slice<T: Type>(&mut self, data: &[T]) {
        for i in 0..data.len() {
            if i >= C::CHANNELS {
                break;
            }
            self[i] = data[i].to_norm();
        }
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

    pub fn blend_alpha(mut self) -> Pixel<C> {
        let index = self.len() - 1;
        let alpha = self[index];

        self.map_in_place(|x| x * alpha);
        self[index] = 1.0;
        self
    }

    pub fn convert<D: Color>(&self, dest: &mut Pixel<D>, f: impl Fn(usize, f64) -> Option<f64>) {
        for (i, x) in self.0.iter().enumerate() {
            match f(i, *x) {
                Some(y) => dest[i] = y,
                None => break,
            }
        }
    }

    pub fn map(&self, f: impl Fn(f64) -> f64) -> Pixel<C> {
        Pixel::from_vec(self.0.iter().map(|x| f(*x)).collect())
    }

    pub fn map2(&self, other: Pixel<C>, f: impl Fn(f64, f64) -> f64) -> Pixel<C> {
        Pixel::from_vec(
            self.0
                .iter()
                .zip(other.0.iter())
                .map(|(x, y)| f(*x, *y))
                .collect(),
        )
    }

    pub fn map_in_place(&mut self, f: impl Fn(f64) -> f64) {
        self.0.iter_mut().for_each(|x| *x = f(*x))
    }

    pub fn map2_in_place(&mut self, other: Pixel<C>, f: impl Fn(f64, f64) -> f64) {
        self.0
            .iter_mut()
            .zip(other.0.iter())
            .for_each(|(x, y)| *x = f(*x, *y))
    }

    pub fn for_each(&self, mut f: impl FnMut(f64)) {
        self.0.iter().for_each(|x| f(*x))
    }

    pub fn iter(&self) -> std::slice::Iter<f64> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<f64> {
        self.0.iter_mut()
    }
}

impl<T: Type, C: Color> std::iter::FromIterator<T> for Pixel<C> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Pixel::from_vec(iter.into_iter().map(|x| x.to_norm()).collect())
    }
}

impl<C: Color> IntoIterator for Pixel<C> {
    type Item = f64;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<C: Color> std::ops::Index<usize> for Pixel<C> {
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

impl<T: Type, C: Color> std::ops::Add<T> for Pixel<C> {
    type Output = Pixel<C>;

    fn add(self, other: T) -> Pixel<C> {
        self.map(|x| x + other.to_norm())
    }
}

impl<C: Color> std::ops::Add<Pixel<C>> for Pixel<C> {
    type Output = Pixel<C>;

    fn add(self, other: Pixel<C>) -> Pixel<C> {
        self.map2(other, |x, y| x + y)
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
        self.map2(other, |x, y| x - y)
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
        self.map2(other, |x, y| x * y)
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
        self.map2(other, |x, y| x / y)
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
        self.map2(other, |x, y| x % y)
    }
}

impl<T: Type, C: Color> std::ops::AddAssign<T> for Pixel<C> {
    fn add_assign(&mut self, other: T) {
        self.map_in_place(|x| x + other.to_norm())
    }
}

impl<C: Color> std::ops::AddAssign<Pixel<C>> for Pixel<C> {
    fn add_assign(&mut self, other: Pixel<C>) {
        self.map2_in_place(other, |x, y| x + y)
    }
}

impl<T: Type, C: Color> std::ops::SubAssign<T> for Pixel<C> {
    fn sub_assign(&mut self, other: T) {
        self.map_in_place(|x| x - other.to_norm())
    }
}

impl<C: Color> std::ops::SubAssign<Pixel<C>> for Pixel<C> {
    fn sub_assign(&mut self, other: Pixel<C>) {
        self.map2_in_place(other, |x, y| x - y)
    }
}

impl<T: Type, C: Color> std::ops::MulAssign<T> for Pixel<C> {
    fn mul_assign(&mut self, other: T) {
        self.map_in_place(|x| x * other.to_norm())
    }
}

impl<C: Color> std::ops::MulAssign<Pixel<C>> for Pixel<C> {
    fn mul_assign(&mut self, other: Pixel<C>) {
        self.map2_in_place(other, |x, y| x * y)
    }
}

impl<T: Type, C: Color> std::ops::DivAssign<T> for Pixel<C> {
    fn div_assign(&mut self, other: T) {
        self.map_in_place(|x| x / other.to_norm())
    }
}

impl<C: Color> std::ops::DivAssign<Pixel<C>> for Pixel<C> {
    fn div_assign(&mut self, other: Pixel<C>) {
        self.map2_in_place(other, |x, y| x / y)
    }
}

impl<T: Type, C: Color> std::ops::RemAssign<T> for Pixel<C> {
    fn rem_assign(&mut self, other: T) {
        self.map_in_place(|x| x % other.to_norm())
    }
}

impl<C: Color> std::ops::RemAssign<Pixel<C>> for Pixel<C> {
    fn rem_assign(&mut self, other: Pixel<C>) {
        self.map2_in_place(other, |x, y| x % y)
    }
}
