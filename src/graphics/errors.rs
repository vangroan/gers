use glow::HasContext;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum GfxError {
    /// Error code from OpenGL.
    /// Contextual depending on which commands
    /// were run.
    OpenGl(u32),
}

impl GfxError {
    pub fn opengl_error_name(&self) -> Option<&str> {
        match self {
            Self::OpenGl(code) => match *code {
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
        }
    }
}

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
