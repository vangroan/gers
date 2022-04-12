use crate::graphics::transform::Transform2D;
use rust_wren::prelude::*;

#[wren_class(name = Sprite)]
pub struct Sprite {
    pub pos: [i32; 2],
    pub size: [u32; 2],
    pub transform: Transform2D,
    // pub texture: Option<Texture>,
}

#[wren_methods]
impl Sprite {
    #[construct]
    pub fn new_() -> Self {
        todo!()
    }

    #[method(name = new)]
    pub fn from_size(width: u32, height: u32) -> Self {
        Self {
            pos: [0, 0],
            size: [width, height],
            transform: Transform2D::default(),
        }
    }
}
