use std::f64;
use std::ops;

use crate::*;

/// 2-dimensional convolution kernel
#[derive(Debug, Clone, PartialEq)]
pub struct Kernel {
    rows: usize,
    cols: usize,
    data: Vec<Vec<f64>>,
}

impl From<Vec<Vec<f64>>> for Kernel {
    fn from(data: Vec<Vec<f64>>) -> Kernel {
        let rows = data.len();
        let cols = data[0].len();
        Kernel { data, rows, cols }
    }
}

impl<'a> From<&'a [&'a [f64]]> for Kernel {
    fn from(data: &'a [&'a [f64]]) -> Kernel {
        let rows = data.len();
        let cols = data[0].len();
        let mut v = Vec::new();
        for d in data {
            v.push(Vec::from(*d))
        }
        Kernel {
            data: v,
            rows,
            cols,
        }
    }
}

macro_rules! kernel_from {
    ($n:expr) => {
        impl From<[[f64; $n]; $n]> for Kernel {
            fn from(data: [[f64; $n]; $n]) -> Kernel {
               let data = data.iter().map(|d| d.to_vec()).collect();
               Kernel {
                   data,
                   rows: $n,
                   cols: $n,
               }
           }
       }
   };
   ($($n:expr,)*) => {
       $(
           kernel_from!($n);
       )*
   }
}

kernel_from!(
    2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
    28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
);

impl<T: Type, C: Color, U: Type, D: Color> Filter<T, C, U, D> for Kernel {
    fn schedule(&self) -> Schedule {
        Schedule::Image
    }

    fn compute_at(&self, pt: Point, input: &Input<T, C>, dest: &mut DataMut<U, D>) {
        let r2 = (self.rows / 2) as isize;
        let c2 = (self.cols / 2) as isize;
        let mut f = input.new_pixel();
        let mut x: f64;
        for ky in -r2..=r2 {
            let kr = &self.data[(ky + r2) as usize];
            let pty = (pt.y as isize + ky) as usize;
            for kx in -c2..=c2 {
                let krc = kr[(kx + c2) as usize];
                for c in 0..f.len() {
                    x = input.get_f(((pt.x as isize + kx) as usize, pty), c, Some(0));
                    f[c] += x * krc;
                }
            }
        }
        f.copy_to_slice(dest);
    }
}

impl Kernel {
    /// Create a new kernel with the given number of rows and columns
    pub fn new(rows: usize, cols: usize) -> Kernel {
        let data = vec![vec![0.0; cols]; rows];
        Kernel { data, rows, cols }
    }

    /// Create a new, square kernel
    pub fn square(x: usize) -> Kernel {
        Self::new(x, x)
    }

    /// Ensures the sum of the kernel is <= 1
    pub fn normalize(&mut self) {
        let sum: f64 = self.data.iter().map(|x| -> f64 { x.iter().sum() }).sum();
        if sum == 0.0 {
            return;
        }

        for j in 0..self.rows {
            for i in 0..self.cols {
                self.data[j][i] /= sum
            }
        }
    }

    /// Create a new kernel and fill it by executing `f` with each possible (row, col) pair
    pub fn create<F: Fn(usize, usize) -> f64>(rows: usize, cols: usize, f: F) -> Kernel {
        let mut k = Self::new(rows, cols);
        for j in 0..rows {
            let d = &mut k.data[j];
            for (i, item) in d.iter_mut().enumerate() {
                *item = f(i, j);
            }
        }
        k
    }

    /// Generate gaussian blur kernel
    pub fn gaussian(n: usize, std: f64) -> Kernel {
        assert!(n % 2 != 0);
        let std2 = std * std;
        let a = 1.0 / (2.0 * f64::consts::PI * std2);
        let mut k = Kernel::create(n, n, |i, j| {
            let x = (i * i + j * j) as f64 / (2.0 * std2);
            a * f64::consts::E.powf(-1.0 * x)
        });
        k.normalize();
        k
    }

    /// 3x3 pixel gaussian blur
    pub fn gaussian_3x3() -> Kernel {
        Self::gaussian(3, 1.4)
    }

    /// 5x5 pixel gaussian blur
    pub fn gaussian_5x5() -> Kernel {
        Self::gaussian(5, 1.4)
    }

    /// 7x7 pixel gaussian blur
    pub fn gaussian_7x7() -> Kernel {
        Self::gaussian(7, 1.4)
    }

    /// 9x9 pixel gaussian blur
    pub fn gaussian_9x9() -> Kernel {
        Self::gaussian(9, 1.4)
    }

    /// Sobel X
    pub fn sobel_x() -> Kernel {
        Kernel {
            rows: 3,
            cols: 3,
            data: vec![
                vec![1.0, 0.0, -1.0],
                vec![2.0, 0.0, -2.0],
                vec![1.0, 0.0, -1.0],
            ],
        }
    }

    /// Sobel Y
    pub fn sobel_y() -> Kernel {
        Kernel {
            rows: 3,
            cols: 3,
            data: vec![
                vec![1.0, 2.0, 1.0],
                vec![0.0, 0.0, 0.0],
                vec![-1.0, -2.0, -1.0],
            ],
        }
    }

    /// Laplacian
    pub fn laplacian() -> Kernel {
        Kernel::from([[0., -1., 0.], [-1., 4., -1.], [0., -1., 0.]])
    }

    /// Sobel X and Y combined
    pub fn sobel() -> Kernel {
        Kernel::sobel_x() + Kernel::sobel_y()
    }
}

impl ops::Add for Kernel {
    type Output = Kernel;

    fn add(mut self, other: Kernel) -> Kernel {
        for i in 0..self.rows {
            for j in 0..self.cols {
                self.data[i][j] += other.data[i][j];
            }
        }
        self
    }
}

impl ops::Sub for Kernel {
    type Output = Kernel;

    fn sub(mut self, other: Kernel) -> Kernel {
        for i in 0..self.rows {
            for j in 0..self.cols {
                self.data[i][j] -= other.data[i][j];
            }
        }
        self
    }
}

impl ops::Mul for Kernel {
    type Output = Kernel;

    fn mul(mut self, other: Kernel) -> Kernel {
        for i in 0..self.rows {
            for j in 0..self.cols {
                self.data[i][j] *= other.data[i][j];
            }
        }
        self
    }
}

impl ops::Div for Kernel {
    type Output = Kernel;

    fn div(mut self, other: Kernel) -> Kernel {
        for i in 0..self.rows {
            for j in 0..self.cols {
                self.data[i][j] /= other.data[i][j];
            }
        }
        self
    }
}
