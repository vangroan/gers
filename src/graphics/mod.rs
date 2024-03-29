mod angle;
mod colour;
mod device;
mod errors;
mod rect;
mod shader;
mod sprite;
mod sprite_batch;
mod texture;
mod transform;
mod utils;
mod vao;
mod vertex;
mod vertex_array;

pub const GRAPHICS_MODULE: &str = "gers.graphics";
pub use self::device::{
    bind_graphic_device, init_graphic_device, register_graphic_device, GraphicDevice, GraphicDeviceHooks, OpenGlInfo,
};
pub use self::errors::GfxError;
pub use self::shader::{init_default_shaders, Shader};
pub use self::sprite_batch::SpriteBatch;
pub use self::texture::Texture;
pub use self::transform::Transform2D;
pub use self::vao::{UsageFrequency, UsageNature, VertexArrayObject};
pub use self::vertex::Vertex;
pub use self::vertex_array::VertexArray;

use rust_wren::{prelude::*, ModuleBuilder, WrenResult};

pub fn register_graphics(vm: &mut WrenVm) -> WrenResult<()> {
    vm.interpret(GRAPHICS_MODULE, include_str!("transform.wren"))?;
    vm.interpret(GRAPHICS_MODULE, include_str!("vertex.wren"))?;
    vm.interpret(GRAPHICS_MODULE, include_str!("vao.wren"))?;
    vm.interpret(GRAPHICS_MODULE, include_str!("vertex_array.wren"))?;
    vm.interpret(GRAPHICS_MODULE, include_str!("texture.wren"))?;
    vm.interpret(GRAPHICS_MODULE, include_str!("sprite.wren"))?;
    vm.interpret(GRAPHICS_MODULE, include_str!("sprite_batch.wren"))?;
    vm.interpret(GRAPHICS_MODULE, include_str!("shader.wren"))?;

    Ok(())
}

pub fn bind_graphics(module: &mut ModuleBuilder) {
    module.register::<Transform2D>();
    module.register::<Vertex>();
    module.register::<VertexArrayObject>();
    module.register::<VertexArray>();
    module.register::<Texture>();
    module.register::<SpriteBatch>();
    module.register::<Shader>();
}
