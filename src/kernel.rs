use color::Color;
use filter::Filter;
use image::Image;
use ty::Type;

use std::ops;
use std::f64;

#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Kernel {
    rows: usize,
    cols: usize,
    data: Vec<Vec<f64>>,
}

macro_rules! kernel_from {
    ($n:expr) => {
        impl From<[[f64; $n]; $n]> for Kernel {
            fn from(data: [[f64; $n]; $n]) -> Kernel {
                let data = data.into_iter().map(|d| d.to_vec()).collect();
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

kernel_from!(2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
             16, 17, 18, 19, 20,);

impl From<Vec<Vec<f64>>> for Kernel {
    fn from(data: Vec<Vec<f64>>) -> Kernel {
        let rows = data.len();
        let cols = data[0].len();
        Kernel { data, rows, cols }
    }
}

impl Filter for Kernel {
    fn compute_at<T: Type, C: Color, I: Image<T, C>>(
        &self,
        x: usize,
        y: usize,
        c: usize,
        input: &[&I],
    ) -> f64 {
        let r2 = (self.rows / 2) as isize;
        let c2 = (self.cols / 2) as isize;
        let mut f = 0.0;
        for ky in -r2..r2 + 1 {
            let kr = &self.data[(ky + r2) as usize];
            for kx in -c2..c2 + 1 {
                let x = input[0].get_f((x as isize + kx) as usize, (y as isize + ky) as usize, c);
                f += x * kr[(kx + c2) as usize];
            }
        }
        f
    }
}

impl Kernel {
    pub fn new(rows: usize, cols: usize) -> Kernel {
        let data = vec![vec![0.0; cols]; rows];
        let mut k = Kernel { data, rows, cols };
        k.normalize();
        k
    }

    pub fn normalize(&mut self) {
        let sum: f64 = self.data.iter().map(|x| -> f64 { x.iter().sum() }).sum();
        if sum == 0.0 {
            return
        }

        for j in 0..self.rows {
            for i in 0..self.cols {
                self.data[j][i] /= sum
            }
        }
    }

    pub fn create<F: Fn(usize, usize) -> f64>(rows: usize, cols: usize, f: F) -> Kernel {
        let mut k = Self::new(rows, cols);
        for j in 0..rows {
            let mut d = &mut k.data[j];
            for i in 0..cols {
                d[i] = f(i, j);
            }
        }
        k.normalize();
        k
    }
}

pub fn gaussian(n: usize, std: f64) -> Kernel {
    assert!(n % 2 != 0);
    let std2 = std * std;
    let a = 1.0 / (2.0 * f64::consts::PI * std2);
    Kernel::create(n, n, |i, j| {
        let x = (i * i + j * j) as f64 / (2.0 * std2);
        a * f64::consts::E.powf(-1.0 * x)

    })
}

lazy_static! {
    pub static ref GAUSSIAN_3X3: Kernel = gaussian(3, 1.4);
}

lazy_static! {
    pub static ref GAUSSIAN_5X5: Kernel = gaussian(5, 1.4);
}

lazy_static! {
    pub static ref GAUSSIAN_7X7: Kernel = gaussian(7, 1.4);
}

lazy_static! {
    pub static ref GAUSSIAN_9X9: Kernel = gaussian(9, 1.4);
}

lazy_static! {
    pub static ref SOBEL_X: Kernel = Kernel {
        rows: 3,
        cols: 3,
        data: vec![
            vec![1.0, 0.0, -1.0],
            vec![2.0, 0.0, -2.0],
            vec![1.0, 0.0, -1.0],
        ]
    };
}

lazy_static! {
    pub static ref SOBEL_Y: Kernel = Kernel {
        rows: 3,
        cols: 3,
        data: vec![
            vec![ 1.0,  2.0,  1.0],
            vec![ 0.0,  0.0,  0.0],
            vec![-1.0, -2.0, -1.0],
        ]
    };
}

macro_rules! op {
    ($name:ident, $fx:ident, $f:expr) => {
        pub struct $name {
            a: Kernel,
            b: Kernel,
        }

        impl Filter for $name {
            fn compute_at<T: Type, C: Color, I: Image<T, C>>(
                &self,
                x: usize,
                y: usize,
                c: usize,
                input: &[&I],
            ) -> f64 {
                let r2 = (self.a.rows / 2) as isize;
                let c2 = (self.a.cols / 2) as isize;
                let mut f = 0.0;
                for ky in -r2..r2 + 1 {
                    let kr = &self.a.data[(ky + r2) as usize];
                    let kr1 = &self.b.data[(ky + r2) as usize];
                    for kx in -c2..c2 + 1 {
                        let x = input[0].get_f((x as isize + kx) as usize, (y as isize + ky) as usize, c);
                        f += $f(x * kr[(kx + c2) as usize], x * kr1[(kx + c2) as usize]);
                    }
                }
                f
            }
        }

        impl ops::$name for Kernel {
            type Output = $name;

            fn $fx(self, other: Kernel) -> $name {
                $name {
                    a: self,
                    b: other,
                }
            }
        }
    }
}

op!(Add, add, |a, b| a + b);
op!(Sub, sub, |a, b| a - b);
op!(Mul, mul, |a, b| a * b);
op!(Div, div, |a, b| a / b);
op!(Rem, rem, |a, b| a % b);

pub fn sobel() -> Add {
    SOBEL_X.clone() + SOBEL_Y.clone()
}

pub fn gaussian_3x3() -> Kernel {
    GAUSSIAN_3X3.clone()
}

pub fn gaussian_5x5() -> Kernel {
    GAUSSIAN_5X5.clone()
}

pub fn gaussian_7x7() -> Kernel {
    GAUSSIAN_7X7.clone()
}

pub fn gaussian_9x9() -> Kernel {
    GAUSSIAN_9X9.clone()
}

