use std::{ffi::CString, fmt};

use glow::{Context, HasContext};
use glutin::display::{Display, GlDisplay};

use crate::color::Color;

use super::GfxBackend;

pub struct OpenGLBackend {
    gl: Context,
    info: OpenGLInfo,
}

impl OpenGLBackend {
    pub fn new(gl_display: &Display) -> Self {
        let gl = unsafe {
            glow::Context::from_loader_function(|symbol| {
                let cstring = CString::new(symbol).unwrap();
                gl_display.get_proc_address(cstring.as_c_str()) as *const _
            })
        };

        let info = OpenGLInfo::new(&gl);

        Self { gl, info }
    }

    pub fn info(&self) -> &OpenGLInfo {
        &self.info
    }
}

impl GfxBackend for OpenGLBackend {
    fn clear_color(&self, color: Color) {
        unsafe {
            let [r, g, b, a] = color.as_f32();
            self.gl.clear_color(r, g, b, a);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }
}

#[derive(Debug)]
pub struct OpenGLInfo {
    pub version: String,
    pub vendor: String,
    pub renderer: String,
}

impl OpenGLInfo {
    pub fn new(gl: &Context) -> Self {
        unsafe {
            let version = gl.get_parameter_string(glow::VERSION);
            let vendor = gl.get_parameter_string(glow::VENDOR);
            let renderer = gl.get_parameter_string(glow::RENDERER);

            Self {
                version,
                vendor,
                renderer,
            }
        }
    }
}

impl fmt::Display for OpenGLInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "version: {}, ", self.version)?;
        write!(f, "vendor: {}, ", self.vendor)?;
        write!(f, "renderer: {}", self.renderer)?;
        Ok(())
    }
}
