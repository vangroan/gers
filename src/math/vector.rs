use rust_wren::prelude::*;

#[wren_class]
pub struct Vector2f(nalgebra::Vector2<f32>);

#[wren_methods]
impl Vector2f {
    #[construct]
    pub fn new(x: f32, y: f32) -> Self {
        Vector2f(nalgebra::Vector2::new(x, y))
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl Into<nalgebra::Vector2<f32>> for Vector2f {
    fn into(self) -> nalgebra::Vector2<f32> {
        self.0
    }
}

impl From<nalgebra::Vector2<f32>> for Vector2f {
    fn from(vector: nalgebra::Vector2<f32>) -> Self {
        Vector2f(vector)
    }
}
