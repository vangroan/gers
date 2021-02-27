//! Texture handle.
use crate::{
    gl_result,
    graphics::{
        device::{Destroy, DestroyQueue, GraphicDevice},
        errors::{debug_assert_gl, GfxError, GfxResult},
        rect::Rect,
    },
    marker::Invariant,
};
use glow::HasContext;
use rust_wren::{prelude::*, ForeignError, WrenResult};
use std::{cell::RefCell, rc::Rc};

/// Handle to a texture located in video memory.
#[wren_class]
pub struct Texture {
    /// Handle to texture allocated in video memory.
    texture: glow::Texture,
    /// Total size in texels of the whole texture in video memory.
    /// We need to keep this around for UVs coordinates calculations.
    orig_size: [u32; 2],
    /// Sub-rectangle representing the view of this texture into
    /// the complete texture.
    ///
    /// Must be equal or smaller than `orig_size`.
    rect: Rect<u32>,
    /// Queue for releasing the resource handle on drop.
    destroy: DestroyQueue,
    /// Internal handle belongs to OpenGL context.
    _invariant: Invariant,
}

#[wren_methods]
impl Texture {
    /// Required but unimplemented constructor.
    ///
    /// FIXME: `rust-wren` is currently limited to constructors
    ///        that only return `Self`. Because graphics allocations
    ///        can fail, we need to return `Result<Self>`.
    #[construct]
    fn new_() -> Self {
        unimplemented!()
    }

    fn new(device: &WrenCell<GraphicDevice>, width: u32, height: u32) -> Result<Self, ForeignError> {
        Self::create(&*device.borrow(), width, height).map_err(|err| foreign_error!(err))
    }
}

impl Texture {
    pub fn create(device: &GraphicDevice, width: u32, height: u32) -> GfxResult<Self> {
        // Upfront validations.
        Self::validate_size(width, height)?;

        // When non-power-of-two textures are not available, several
        // bad things can happen from degraded performance to OpenGL
        // errors.
        if !Self::is_npot_available(device) {
            if !Self::is_power_of_two(width) || !Self::is_power_of_two(height) {
                return Err(GfxError::InvalidTextureSize(width, height));
            }
        }

        // Important: Non power of two textures may not have mipmaps

        unsafe {
            // Create texture returns `Result` but error should be `Infallible`.
            let handle = device.gl.create_texture().unwrap();
            gl_result!(&device.gl);
            device.gl.bind_texture(glow::TEXTURE_2D, Some(handle));

            // Allocate video memory for texture
            device.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,                   // Mip level
                glow::RGBA8 as i32,  // Internal colour format
                width as i32,        // Width in pixels
                height as i32,       // Height in pixels
                0,                   // Border
                glow::RGBA,          // Format
                glow::UNSIGNED_BYTE, // Color data type.
                None,                // Actual data can be uploaded later.
            );
            // gl_error(&device.gl, ())?;
            gl_result!(device.gl);

            device
                .gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            device
                .gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            device
                .gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            device
                .gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            device.gl.bind_texture(glow::TEXTURE_2D, None);

            // Match the allocated texture.
            let rect = Rect {
                pos: [0, 0],
                size: [width, height],
            };

            Ok(Self {
                texture: handle,
                orig_size: [width, height],
                rect,
                destroy: device.destroy_queue(),
                _invariant: Default::default(),
            })
        }
    }

    /// Create a sub texture from the given texture view.
    ///
    /// Does not allocate new texture space in video memory.
    /// Instead creates a view into the same memory backed
    /// by `source`.
    ///
    /// # Errors
    ///
    /// Returns `InvalidSubTexture` if the given position and
    /// size do not fit inside the source texture.
    ///
    /// Returns `InvalidTextureSize` if any given dimension is 0
    /// or invalid for the current graphic device.
    pub fn new_sub(&self, pos: [u32; 2], size: [u32; 2]) -> GfxResult<Self> {
        let target_rect = Rect { pos, size };

        if !self.rect.can_fit(&target_rect) {
            return Err(GfxError::InvalidSubTexture {
                outer: self.rect,
                inner: target_rect,
            });
        }

        Self::validate_size(size[0], size[1])?;

        // We can probably get away without checking power-of-two since we're not
        // allocating video memory.

        // Ok(Self {
        //     texture: self.texture,
        //     orig_size: self.orig_size,
        //     rect: target_rect,
        //     destroy: self.destroy_queue(),
        //     _invariant: Default::default(),
        // });

        todo!("Decide what to do about sub textures and sharing the handle")
    }

    fn validate_size(width: u32, height: u32) -> GfxResult<()> {
        if width == 0 || height == 0 {
            return Err(GfxError::InvalidTextureSize(width, height));
        }

        Ok(())
    }

    /// Checks whether the given number is power-of-two.
    #[inline(always)]
    fn is_power_of_two(n: u32) -> bool {
        // This bitwise test does not work on the number zero.
        n != 0 && ((n & n - 1) == 0)
    }

    /// Queries the device support for non-power-of-two-textures.
    pub fn is_npot_available(device: &GraphicDevice) -> bool {
        device.has_extension("GL_ARB_texture_non_power_of_two")
    }

    #[inline(always)]
    pub fn raw_handle(&self) -> glow::Texture {
        self.texture
    }

    pub fn update_data(&mut self, device: &GraphicDevice, data: &[u8]) -> GfxResult<()> {
        let size = self.orig_size;
        self.update_sub_data(device, [0, 0], size, data)
    }

    /// Uploads image data to the texture's storage on the GPU device.
    pub fn update_sub_data(
        &mut self,
        device: &GraphicDevice,
        pos: [u32; 2],
        size: [u32; 2],
        data: &[u8],
    ) -> GfxResult<()> {
        // TODO: Unbind GL_PIXEL_UNPACK_BUFFER
        //       https://www.khronos.org/opengl/wiki/GLAPI/glTexSubImage2D
        //       If a non-zero named buffer object is bound to the
        //       GL_PIXEL_UNPACK_BUFFER target (see glBindBuffer)
        //       while a texture image is specified, data is
        //       treated as a byte offset into the buffer object's
        //       data store.

        // TODO: Validate given pos and size against target texture rectangle. Must fit.

        // Upfront validation
        let expected_len = size[0] as usize * size[1] as usize * 4;
        if data.len() != expected_len {
            return Err(GfxError::InvalidImageData {
                expected: expected_len,
                actual: data.len(),
            });
        }

        // Borrow mut to enforce runtime borrow rules.
        // TODO: Do we need Texture to have an internal RefCell?
        // let handle = self.handle.borrow_mut();

        unsafe {
            let _save = TextureSave::new(&device);

            device.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            device.gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,                   // level
                pos[0] as i32,       // x_offset
                pos[1] as i32,       // y_offset
                size[0] as i32,      // width
                size[1] as i32,      // height
                glow::RGBA,          // pixel format
                glow::UNSIGNED_BYTE, // color data type
                glow::PixelUnpackData::Slice(data),
            );
            gl_result!(&device.gl);
        }

        Ok(())
    }

    /// Returns the number of bytes contained in the texture's storage.
    pub fn data_len(&self) -> usize {
        // TODO: Does texture need an internal RefCell?
        // let size = self.handle.borrow().size;
        // Each pixel is 4 bytes, RGBA
        self.orig_size[0] as usize * self.orig_size[1] as usize * 4
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        // self.destroy.send(Destroy::Texture(self.handle)).unwrap();
        self.destroy.send(Destroy::Texture(self.texture));
    }
}

/// Utility for saving the currently bound texture onto the call stack, and
/// restoring the binding on drop.
///
/// Used so that editing a texture does not disrupt a currently bound texture.
pub(crate) struct TextureSave<'a> {
    gl: &'a glow::Context,
    texture_handle: u32,
}

impl<'a> TextureSave<'a> {
    pub(crate) fn new(device: &'a GraphicDevice) -> Self {
        Self {
            gl: &device.gl,
            texture_handle: unsafe {
                debug_assert_gl(&device.gl, device.gl.get_parameter_i32(glow::TEXTURE_BINDING_2D) as u32)
            },
        }
    }
}

impl<'a> Drop for TextureSave<'a> {
    fn drop(&mut self) {
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture_handle));
        }
    }
}
