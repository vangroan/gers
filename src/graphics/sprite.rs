use rust_wren::prelude::*;

#[wren_class(name = Sprite)]
pub struct Sprite {
    pub pos: [i32; 2],
    pub size: [u32; 2],
    // pub texture: Option<Texture>,
}

#[wren_methods]
impl Sprite {
    #[construct]
    pub fn new() -> Self {
        todo!()
    }
}
