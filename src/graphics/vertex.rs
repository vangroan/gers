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

    #[method(name = setPos)]
    #[inline]
    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.position = [x, y];
    }

    #[method(name = setUv)]
    #[inline]
    pub fn set_uv(&mut self, u: f32, v: f32) {
        self.uv = [u, v];
    }

    #[method(name = setColor)]
    #[inline]
    pub fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.color = [r, g, b, a];
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new()
    }
}
