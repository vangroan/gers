//! Shader program.
use crate::graphics::{
    device::{Destroy, DestroyQueue, GraphicDevice},
    errors::{GfxError, GfxResult},
    GRAPHICS_MODULE,
};
use glow::HasContext;
use rust_wren::{prelude::*, ForeignError, WrenContext, WrenError, WrenResult};

/// Compile the built in shaders and place them in static variables in Wren.
pub fn init_default_shaders(ctx: &mut WrenContext, device: &GraphicDevice) -> WrenResult<()> {
    {
        // Default sprite shader
        let vert = include_str!("shaders/sprite.vert");
        let frag = include_str!("shaders/sprite.frag");
        let shader = Shader::from_source(device, vert, frag).map_err(|err| WrenError::Ctx(err.into()))?;

        let default_prop = ctx.make_call_ref(GRAPHICS_MODULE, "Shader", "default_=(_)")?;
        default_prop.call::<_, ()>(ctx, shader)?;
    }

    Ok(())
}

#[wren_class]
#[derive(Debug)]
pub struct Shader {
    pub(crate) program: u32,
    destroy: DestroyQueue,
}

#[wren_methods]
impl Shader {
    #[construct]
    fn new_() -> Self {
        unimplemented!()
    }

    fn compile(device: &WrenCell<GraphicDevice>, vertex: &str, fragment: &str) -> Result<Self, ForeignError> {
        Self::from_source(&*device.borrow(), vertex, fragment).map_err(|err| foreign_error!(err))
    }
}

impl Shader {
    pub fn from_source(device: &GraphicDevice, vertex: &str, fragment: &str) -> GfxResult<Self> {
        // Create Shader program.
        // Call is infallible.
        let program = unsafe { device.gl.create_program().unwrap() };

        // Link shaders.
        let shader_sources = [(glow::VERTEX_SHADER, vertex), (glow::FRAGMENT_SHADER, fragment)];

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            unsafe {
                let shader = device.gl.create_shader(*shader_type).unwrap();
                device.gl.shader_source(shader, shader_source);
                device.gl.compile_shader(shader);
                if !device.gl.get_shader_compile_status(shader) {
                    return Err(GfxError::ShaderCompile(device.gl.get_shader_info_log(shader)));
                }
                device.gl.attach_shader(program, shader);
                shaders.push(shader);
            }
        }

        unsafe {
            device.gl.link_program(program);
            if !device.gl.get_program_link_status(program) {
                return Err(GfxError::ShaderCompile(device.gl.get_program_info_log(program)));
            }
        }

        // Once the shaders are linked to a program, it's safe to detach and delete them.
        for shader in shaders {
            unsafe {
                device.gl.detach_shader(program, shader);
                device.gl.delete_shader(shader);
            }
        }

        Ok(Self {
            program,
            destroy: device.destroy_queue(),
        })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        self.destroy.send(Destroy::Shader(self.program));
    }
}
