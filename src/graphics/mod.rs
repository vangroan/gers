mod angle;
mod colour;
mod device;
mod errors;
mod sprite;
mod transform;
mod utils;
mod vertex;
mod vertex_buf;

pub const GRAPHICS_MODULE: &'static str = "graphics";
pub use self::device::{
    bind_graphic_device, init_graphic_device, register_graphic_device, GraphicDevice, GraphicDeviceHooks, OpenGlInfo,
};
pub use self::errors::GfxError;
pub use self::vertex::Vertex;
pub use self::vertex_buf::{UsageFrequency, UsageNature, VertexBuffer};

use rust_wren::{prelude::*, ModuleBuilder, WrenResult};

pub fn register_graphics(vm: &mut WrenVm) -> WrenResult<()> {
    vm.interpret(GRAPHICS_MODULE, include_str!("vertex_buf.wren"))
}

pub fn bind_graphics(module: &mut ModuleBuilder) {
    module.register::<VertexBuffer>();
}
