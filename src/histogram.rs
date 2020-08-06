use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Histogram {
    pub bins: Box<[usize]>,
    step: f64,
}

impl Histogram {
    pub fn new(nbins: usize) -> Histogram {
        Histogram {
            bins: vec![0; nbins].into_boxed_slice(),
            step: 1.0 / nbins as f64,
        }
    }

    pub fn add<T: Type>(&mut self, value: T) {
        let mut x = (value.to_norm() / self.step).floor() as usize;
        if x == self.bins.len() {
            x -= 1
        }
        self.bins[x] += 1
    }

    pub fn bins<'a>(&'a self) -> impl 'a + Iterator<Item = (usize, usize)> {
        self.bins.iter().enumerate().map(|(a, b)| (a, *b))
    }

    /// Get the bin index of the minimum value. There may be other bins with the same value, which
    /// would not be reported by this function
    pub fn min(&self) -> usize {
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

    /// Get the bin index
    pub fn max(&self) -> usize {
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
