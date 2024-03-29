use super::rect::Rect;
use glow::HasContext;
use std::{error::Error, fmt};

pub type GfxResult<T> = Result<T, GfxError>;

#[derive(Debug)]
pub enum GfxError {
    /// Error code from OpenGL.
    /// Contextual depending on which commands
    /// were run.
    OpenGl(u32),

    /// Error when creating a vertex-array-object with
    /// indices that are out of bounds of the given vertices.
    InvalidVertexArray {
        index: usize,
        vertex_count: usize,
    },

    /// Texture size is not valid for the graphics hardware.
    InvalidTextureSize(u32, u32),

    /// Inner-texture does not fit in outer-texture.
    InvalidSubTexture {
        outer: Rect<u32>,
        inner: Rect<u32>,
    },

    /// Error when a buffer of image data does not fit inside
    /// the specific texture rectangle.
    ///
    /// Data length would be something like `width * height * rgba`.
    InvalidImageData {
        expected: usize,
        actual: usize,
    },

    // Shader compilation error.
    ShaderCompile(String),
}

impl GfxError {
    pub fn opengl_error_name(&self) -> Option<&str> {
        match self {
            GfxError::OpenGl(code) => match *code {
                glow::NO_ERROR => Some("GL_NO_ERROR"),
                glow::INVALID_ENUM => Some("GL_INVALID_ENUM"),
                glow::INVALID_VALUE => Some("GL_INVALID_VALUE"),
                glow::INVALID_OPERATION => Some("GL_INVALID_OPERATION"),
                glow::INVALID_FRAMEBUFFER_OPERATION => Some("GL_INVALID_FRAMEBUFFER_OPERATION"),
                glow::OUT_OF_MEMORY => Some("GL_INVALID_FRAMEBUFFER_OPERATION"),
                glow::STACK_UNDERFLOW => Some("GL_STACK_UNDERFLOW"),
                glow::STACK_OVERFLOW => Some("GL_STACK_OVERFLOW"),
                _ => None,
            },
            _ => None,
        }
    }
}

impl Error for GfxError {}

impl fmt::Display for GfxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use GfxError as E;
        match self {
            E::OpenGl(error_code) => {
                write!(f, "OpenGL Error: 0x{:x} ", error_code)?;

                if let Some(error_name) = self.opengl_error_name() {
                    write!(f, "{} ", error_name)?;
                }

                match *error_code {
                    glow::NO_ERROR => write!(f, "No error has been recorded"),
                    glow::INVALID_ENUM => write!(f, "An unacceptable value is specified for an enumerated argument."),
                    glow::INVALID_VALUE => write!(f, "A numeric argument is out of range."),
                    glow::INVALID_OPERATION => write!(f, "The specified operation is not allowed in the current state."),
                    glow::INVALID_FRAMEBUFFER_OPERATION => write!(f, "The framebuffer object is not complete."),
                    glow::OUT_OF_MEMORY => write!(f, "There is not enough memory left to execute the command."),
                    glow::STACK_UNDERFLOW => write!(f, "An attempt has been made to perform an operation that would cause an internal stack to underflow."),
                    glow::STACK_OVERFLOW => write!(f, "An attempt has been made to perform an operation that would cause an internal stack to overflow."),
                    _ => Ok(()),
                }
            }
            E::InvalidVertexArray { index, vertex_count } => write!(
                f,
                "Invalid vertex array object. Index {} is out of bounds for the given {} vertices.",
                index, vertex_count
            ),
            E::InvalidTextureSize(width, height) => write!(
                f,
                "Invalid texture size ({}, {}). Ensure that neither dimension is zero, and is power-of-two.",
                width, height
            ),
            E::InvalidSubTexture { inner, outer } => {
                write!(f, "Sub-texture rectangle {} does not fit in {}.", inner, outer)
            }
            E::InvalidImageData { expected, actual } => write!(
                f,
                "Image data does not match texture storage size. Expected bytes {}. Actual bytes {}.",
                expected, actual
            ),
            E::ShaderCompile(message) => write!(f, "Shader compile error: {}", message),
        }
    }
}

/// Check OpenGL for an error, using `glGetError`, and returns
/// it as a `Result`.
#[macro_export]
macro_rules! gl_result {
    ($gl:expr) => {{
        use glow::HasContext;
        let gl_error = $gl.get_error();
        if gl_error != glow::NO_ERROR {
            return Err($crate::graphics::GfxError::OpenGl(gl_error).into());
        }
    }};
}

#[inline(always)]
pub unsafe fn debug_assert_gl<T>(gl: &glow::Context, value: T) -> T {
    #[cfg(debug_assertions)]
    {
        let gl_err = gl.get_error();
        if gl_err != glow::NO_ERROR {
            panic!("OpenGL Error: 0x{:x}", gl_err);
        }
    }

    value
}
