use std::f64;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Angle(f64);

impl Angle {
    pub fn of_degrees(deg: f64) -> Angle {
        Angle(deg * f64::consts::PI / 180.)
    }

    pub fn of_radians(rad: f64) -> Angle {
        Angle(rad)
    }

    pub fn degrees(&self) -> f64 {
        180. * self.0 / f64::consts::PI
    }

    pub fn radians(&self) -> f64 {
        self.0
    }
}
