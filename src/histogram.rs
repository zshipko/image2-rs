use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Histogram {
    pub bins: Box<[usize]>,
}

impl std::ops::Index<usize> for Histogram {
    type Output = usize;

    fn index(&self, i: usize) -> &usize {
        &self.bins[i]
    }
}

impl std::ops::IndexMut<usize> for Histogram {
    fn index_mut(&mut self, i: usize) -> &mut usize {
        &mut self.bins[i]
    }
}

impl Histogram {
    pub fn new(nbins: usize) -> Histogram {
        Histogram {
            bins: vec![0; nbins].into_boxed_slice(),
        }
    }

    pub fn add<T: Type>(&mut self, value: T) {
        let x = value.to_norm() * (self.bins.len() - 1) as f64;
        self.bins[x as usize] += 1
    }

    pub fn bins<'a>(&'a self) -> impl 'a + Iterator<Item = (usize, usize)> {
        self.bins.iter().enumerate().map(|(a, b)| (a, *b))
    }

    pub fn len(&self) -> usize {
        self.bins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the bin index of the minimum value. There may be other bins with the same value, which
    /// would not be reported by this function
    pub fn min_index(&self) -> usize {
        let mut min = usize::MAX;
        let mut index = 0;
        for (i, n) in self.bins.iter().enumerate() {
            if *n < min {
                min = *n;
                index = i;
            }
        }

        index
    }

    /// Get the bin index of the maximum value. There may be other bins with the same value, which
    /// would not be reported by this function
    pub fn max_index(&self) -> usize {
        let mut max = 0;
        let mut index = 0;
        for (i, n) in self.bins.iter().enumerate() {
            if *n > max {
                max = *n;
                index = i;
            }
        }
        index
    }

    /// Count the number of bins with the given value
    pub fn count(&self, v: usize) -> usize {
        self.bins.iter().map(|bin| (*bin == v) as usize).sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_histogram_basic() {
        let image = Image::<f32, Rgb>::new(100, 100);
        let hist = image.histogram(255);

        for h in hist {
            assert!(h.bins[0] == 100 * 100);
            assert!(h.min() == 1);
            assert!(h.max() == 0);
        }
    }
}
