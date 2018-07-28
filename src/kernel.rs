use color::Color;
use filter::Filter;
use image::Image;
use ty::Type;

use std::ops;

#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Kernel {
    data: Vec<Vec<f64>>,
    rows: usize,
    cols: usize,
}

impl From<[[f64; 3]; 3]> for Kernel {
    fn from(data: [[f64; 3]; 3]) -> Kernel {
        let data = data.into_iter().map(|d| d.to_vec()).collect();
        Kernel {
            data,
            rows: 3,
            cols: 3,
        }
    }
}

impl From<[[f64; 5]; 5]> for Kernel {
    fn from(data: [[f64; 5]; 5]) -> Kernel {
        let data = data.into_iter().map(|d| d.to_vec()).collect();
        Kernel {
            data,
            rows: 5,
            cols: 5,
        }
    }
}

impl From<[[f64; 7]; 7]> for Kernel {
    fn from(data: [[f64; 7]; 7]) -> Kernel {
        let data = data.into_iter().map(|d| d.to_vec()).collect();
        Kernel {
            data,
            rows: 7,
            cols: 7,
        }
    }
}

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
                f += input[0].get((x as isize + kx) as usize, (y as isize + ky) as usize, c)
                    * kr[(kx + c2) as usize]
            }
        }
        f
    }
}

impl Kernel {
    pub fn new(rows: usize, cols: usize) -> Kernel {
        let data = vec![vec![0.0; cols]; rows];
        Kernel { data, rows, cols }
    }

    pub fn create<F: Fn(usize, usize) -> f64>(rows: usize, cols: usize, f: F) -> Kernel {
        let mut k = Self::new(rows, cols);
        for j in 0..rows {
            let mut d = &mut k.data[j];
            for i in 0..cols {
                d[i] = f(i, j);
            }
        }
        k
    }
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
                        let x = input[0].get((x as isize + kx) as usize, (y as isize + ky) as usize, c);
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

