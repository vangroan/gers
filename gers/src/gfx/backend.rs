use crate::color::Color;

pub trait GfxBackend {
    fn clear_color(&self, color: Color);
}
