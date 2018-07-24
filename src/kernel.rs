use ty::Type;
use color::Color;
use image::Image;
use filter::Filter;

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

impl Filter for Kernel {
    fn compute_at<T: Type, C: Color, I: Image<T, C>>(&self, x: usize, y: usize, c: usize, input: &[&I]) -> f64 {
        let r2 = (self.rows / 2) as isize;
        let c2 = (self.cols / 2) as isize;
        let mut f = 0.0;
        for ky in -r2 .. r2 + 1 {
            let kr = &self.data[(ky + r2) as usize];
            for kx in -c2 .. c2 + 1 {
                f += input[0].get((x as isize + kx) as usize, (y as isize + ky) as usize, c) * kr[(kx + c2) as usize]
            }
        }
        f
    }
}

impl Kernel {
    pub fn new(rows: usize, cols: usize) -> Kernel {
        let data = vec![vec![0.0; cols]; rows];
        Kernel {
            data, rows, cols
        }
    }

    pub fn v<F: Fn(usize, usize) -> f64>(rows: usize, cols: usize, f: F) -> Kernel {
        let mut k = Self::new(rows, cols);
        for j in 0 .. rows {
            let mut d = &mut k.data[j];
            for i in 0 .. cols {
                d[i] = f(i, j);
            }
        }
        k
    }
}
