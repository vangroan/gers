use super::{errors::debug_assert_gl, GRAPHICS_MODULE};
use crate::marker::Invariant;
use glow::HasContext;
use glutin::{dpi::PhysicalSize, PossiblyCurrent, WindowedContext};
use rust_wren::{
    handle::{FnSymbolRef, WrenCallHandle, WrenCallRef},
    prelude::*,
    ModuleBuilder, WrenContext, WrenResult, WrenVm,
};
use std::{cell::Cell, collections::HashSet, fmt, sync::mpsc};

pub fn init_graphic_device(ctx: &mut WrenContext, device: GraphicDevice) -> GraphicDeviceHooks {
    // Move graphic device to Wren to own it.
    let set_instance = ctx
        .make_call_ref(GRAPHICS_MODULE, GraphicDevice::NAME, "instance_=(_)")
        .unwrap();
    set_instance.call::<_, ()>(ctx, device).unwrap();

    // Now that the graphics device lives in Wren memory, we
    // should retrieve a handle to it so we can communicate
    // from Rust.
    let get_handle = ctx
        .make_call_ref(GRAPHICS_MODULE, GraphicDevice::NAME, "instance")
        .unwrap();

    let set_viewport_handle = {
        let handle = get_handle.call::<_, WrenRef>(ctx, ()).unwrap();
        let set_viewport_size_ref = FnSymbolRef::compile(ctx, "setViewport_(_,_)").unwrap();
        WrenCallRef::new(handle, set_viewport_size_ref).leak().unwrap()
    };

    let maintain_handle = {
        let handle = get_handle.call::<_, WrenRef>(ctx, ()).unwrap();
        let set_viewport_size_ref = FnSymbolRef::compile(ctx, "maintain()").unwrap();
        WrenCallRef::new(handle, set_viewport_size_ref).leak().unwrap()
    };

    GraphicDeviceHooks {
        set_viewport_handle,
        maintain_handle,
    }
}

pub fn register_graphic_device(vm: &mut WrenVm) -> WrenResult<()> {
    vm.interpret(GRAPHICS_MODULE, include_str!("device.wren"))
}

pub fn bind_graphic_device(module: &mut ModuleBuilder) {
    module.register::<GraphicDevice>();
}

#[wren_class]
pub struct GraphicDevice {
    pub(crate) gl: glow::Context,
    extensions: HashSet<String>,
    tx: mpsc::Sender<Destroy>,
    rx: mpsc::Receiver<Destroy>,
    viewport_size: Cell<PhysicalSize<u32>>,
    /// Inner OpenGL context has inner mutability, and is not thread safe.
    _invariant: Invariant,
}

#[wren_methods]
impl GraphicDevice {
    #[construct]
    pub fn new_() -> Self {
        unimplemented!("Graphics device must be created from Rust")
    }

    /// TODO: Return List or Map
    // #[method(name = getViewport)]
    // #[inline]
    // pub fn get_viewport(&self) ->  {
    //     self.viewport_size.set(PhysicalSize::new(width, height));
    // }

    /// TODO: Bind associated function to Wren property
    #[method(name = setViewport_)]
    #[inline]
    pub fn set_viewport_2(&self, width: u32, height: u32) {
        self.viewport_size.set(PhysicalSize::new(width, height));
    }

    #[method(name = clearScreen)]
    pub fn clear_screen_4(&self, r: u8, g: u8, b: u8, a: u8) {
        // println!("Clear screen");
        self.clear_screen([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0]);
    }

    #[method(name = draw)]
    pub fn draw_1(&self, batch: f64) {
        todo!()
    }

    #[method(name = draw)]
    pub fn draw_2(&self, batch: f64, transform: f64) {
        todo!()
    }

    #[method(name = maintain)]
    pub fn script_maintain(&self) {
        self.maintain();
    }
}

impl GraphicDevice {
    /// Creates a new graphic device with the given OpenGL context.
    pub fn new(gl: glow::Context) -> Self {
        let mut extensions = HashSet::new();

        // This implementation is taken from glow::Context::from_loader_function.
        let num_extensions = unsafe { gl.get_parameter_i32(glow::NUM_EXTENSIONS) };
        for i in 0..num_extensions {
            let extension_name = unsafe { gl.get_parameter_indexed_string(glow::EXTENSIONS, i as u32) };
            extensions.insert(extension_name);
        }

        // Ensure our preferred settings.
        unsafe {
            // Counter-clockwise winding.
            gl.front_face(glow::CCW);

            // In 2D sprites can be flipped.
            gl.disable(glow::CULL_FACE);
        }

        // Dropped resources need to be deallocated via the OpenGL context.
        let (tx, rx) = mpsc::channel();

        Self {
            gl,
            extensions,
            tx,
            rx,
            viewport_size: Cell::new(PhysicalSize::new(640, 480)),
            _invariant: Default::default(),
        }
    }

    /// Creates a graphic device from the given `glutin::WindowedContext`.
    pub unsafe fn from_windowed_context(windowed_context: &WindowedContext<PossiblyCurrent>) -> Self {
        // Glue glow and glutin together.
        let gl = glow::Context::from_loader_function(|s| windowed_context.get_proc_address(s) as *const _);

        let device = Self::new(gl);
        device.set_viewport_size(windowed_context.window().inner_size());

        device
    }

    #[inline]
    pub fn viewport_size(&self) -> PhysicalSize<u32> {
        self.viewport_size.get()
    }

    #[inline]
    pub fn set_viewport_size(&self, size: PhysicalSize<u32>) {
        self.viewport_size.set(size);
    }

    #[inline]
    pub fn clear_screen(&self, color: [f32; 4]) {
        unsafe {
            let physical_size_i32 = self.viewport_size.get().cast::<i32>();

            // Tell openGL how to map normalised device coordinates to pixels.
            self.gl
                .viewport(0, 0, physical_size_i32.width, physical_size_i32.height);

            self.gl.clear_color(color[0], color[1], color[2], color[3]);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            debug_assert_gl(&self.gl, ());
        }
    }

    /// Query the graphics driver for hardware information.
    #[inline]
    pub fn opengl_info(&self) -> OpenGlInfo {
        unsafe {
            let version = self.gl.get_parameter_string(glow::VERSION);
            let vendor = self.gl.get_parameter_string(glow::VENDOR);
            let renderer = self.gl.get_parameter_string(glow::RENDERER);
            debug_assert_gl(&self.gl, ());

            OpenGlInfo {
                version,
                vendor,
                renderer,
            }
        }
    }

    /// Release graphics resources.
    pub fn maintain(&self) {
        while let Ok(resource) = self.rx.try_recv() {
            match resource {
                Destroy::Texture(handle) => unsafe {
                    println!("destroying texture");
                    self.gl.delete_texture(handle);
                },
                Destroy::Shader(program) => unsafe {
                    println!("destroying texture");
                    self.gl.delete_program(program);
                },
                Destroy::VertexArray(handle) => unsafe {
                    println!("destroying texture");
                    self.gl.delete_vertex_array(handle);
                },
            }
        }
    }
}

impl Drop for GraphicDevice {
    fn drop(&mut self) {
        self.maintain();
    }
}

/// Message sent from dropped resource handles, instructing
/// device context to release the underlying resource.
pub(crate) enum Destroy {
    Texture(u32),
    Shader(u32),
    VertexArray(u32),
}

/// Hardware information supplied by the graphics device.
pub struct OpenGlInfo {
    pub version: String,
    pub vendor: String,
    pub renderer: String,
}

impl fmt::Display for OpenGlInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "OpenGL Info:")?;
        writeln!(f, "    Version: {}", self.version)?;
        writeln!(f, "    Vendor: {}", self.vendor)?;
        writeln!(f, "    Renderer: {}", self.renderer)?;

        Ok(())
    }
}

/// Handles to Wren functions.
pub struct GraphicDeviceHooks {
    pub set_viewport_handle: WrenCallHandle,
    pub maintain_handle: WrenCallHandle,
}
