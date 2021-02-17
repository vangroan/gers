use rust_wren::prelude::*;

#[wren_class]
#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

#[wren_methods]
impl Vertex {
    #[construct]
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.0],
            uv: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new()
    }
}
