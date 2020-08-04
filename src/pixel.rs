use crate::*;

pub struct Pixel<C: Color> {
    pub color: C,
    pub data: Vec<f32>,
}
