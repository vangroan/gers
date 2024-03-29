use crate::util::crate_version;
use rust_wren::prelude::*;

#[wren_class(name = WindowConf)]
#[derive(Debug, Clone)]
pub struct WrenWindowConfig {
    pub size: [f64; 2],
    pub title: String,
}

#[wren_methods]
impl WrenWindowConfig {
    #[construct]
    pub fn new() -> Self {
        Self {
            size: [512., 512.],
            title: format!("Game Engine v{}", crate_version().full),
        }
    }

    pub fn set_size(&mut self, width: f64, height: f64) {
        self.size = [width, height];
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }
}
