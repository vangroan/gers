use rust_wren::prelude::*;

#[wren_class]
#[derive(Debug, Clone, Copy)]
pub struct Rgba8 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[wren_methods]
impl Rgba8 {
    #[construct]
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}
