use glow::HasContext;

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
