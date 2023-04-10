//! Graphics renderer.
// use winit::dpi::PhysicalSize;

mod backend;
pub mod opengl;

pub use backend::GfxBackend;

use crate::color::Color;

pub struct Render {
    backend: Box<dyn GfxBackend>,
}

impl Render {
    pub fn new(backend: impl GfxBackend + 'static) -> Self {
        Self {
            backend: Box::new(backend),
        }
    }

    pub fn clear_color(&self, color: Color) {
        self.backend.clear_color(color)
    }
}

// pub struct Viewport {
//     pub size: PhysicalSize<f32>,
// }
