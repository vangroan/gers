//! Vertex buffer object
use super::Vertex;
use crate::{
    collections::U16Array,
    gl_result,
    graphics::{
        device::{Destroy, GraphicDevice},
        utils, GfxError,
    },
};
use glow::HasContext;
use rust_wren::prelude::*;
use std::borrow::Borrow;
use std::{mem, sync::mpsc::Sender};

/// Handle to a vertex buffer object located in video memory.
#[wren_class]
pub struct VertexBuffer {
    vbo: u32,
    vertex_buffer: u32,
    index_buffer: u32,
    destroy: Sender<Destroy>,
}

#[wren_methods]
impl VertexBuffer {
    /// Create a new vertex buffer from Wren.
    #[construct]
    fn new(device: &WrenCell<GraphicDevice>, vertices: Option<()>, indices: &WrenCell<U16Array>) -> Self {
        println!("VertexBuffer.new()");
        println!("  indices = {:?}", indices.borrow().as_slice());

        Self {
            vbo: 0,
            vertex_buffer: 0,
            index_buffer: 0,
            destroy: device.borrow().destroy_sender(),
        }
    }
}

impl VertexBuffer {
    // FIXME: Locations determined by sprite shader.
    const POSITION_LOC: u32 = 0;
    const UV_LOC: u32 = 1;
    const COLOR_LOC: u32 = 2;

    pub fn create(
        device: &GraphicDevice,
        vertices: &[Vertex],
        indices: &[u16],
        freq: UsageFrequency,
        nat: UsageNature,
    ) -> Result<Self, GfxError> {
        unsafe {
            // Vertex Buffer Object
            let vertex_array = device.gl.create_vertex_array().unwrap();
            device.gl.bind_vertex_array(Some(vertex_array));

            // Attached buffer space
            let vertex_buffer = device.gl.create_buffer().unwrap();
            device.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            device
                .gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, utils::as_u8(vertices), Self::mem_hint(freq, nat));
            // assert_gl(&device.gl);
            gl_result!(device.gl);

            // Vertex data is interleaved.
            // Attribute layout positions are determined by shader.
            // Positions
            device.gl.enable_vertex_attrib_array(Self::POSITION_LOC);
            device.gl.vertex_attrib_pointer_f32(
                Self::POSITION_LOC,                             // Attribute location in shader program.
                2,                                              // Size. Components per iteration.
                glow::FLOAT,                                    // Type to get from buffer.
                false,                                          // Normalize.
                mem::size_of::<Vertex>() as i32,                // Stride. Bytes to advance each iteration.
                memoffset::offset_of!(Vertex, position) as i32, // Offset. Bytes from start of buffer.
            );
            // assert_gl(&device.gl);
            gl_result!(device.gl);

            // UVs
            device.gl.enable_vertex_attrib_array(Self::UV_LOC);
            device.gl.vertex_attrib_pointer_f32(
                Self::UV_LOC,                             // Attribute location in shader program.
                2,                                        // Size. Components per iteration.
                glow::FLOAT,                              // Type to get from buffer.
                false,                                    // Normalize.
                mem::size_of::<Vertex>() as i32,          // Stride. Bytes to advance each iteration.
                memoffset::offset_of!(Vertex, uv) as i32, // Offset. Bytes from start of buffer.
            );
            // assert_gl(&device.gl);
            gl_result!(device.gl);

            // Colors
            device.gl.enable_vertex_attrib_array(Self::COLOR_LOC);
            device.gl.vertex_attrib_pointer_f32(
                Self::COLOR_LOC,                             // Attribute location in shader program.
                4,                                           // Size. Components per iteration.
                glow::FLOAT,                                 // Type to get from buffer.
                false,                                       // Normalize.
                mem::size_of::<Vertex>() as i32,             // Stride. Bytes to advance each iteration.
                memoffset::offset_of!(Vertex, color) as i32, // Offset. Bytes from start of buffer.
            );
            // assert_gl(&device.gl);
            gl_result!(device.gl);

            // Indices
            let index_buffer = device.gl.create_buffer().unwrap();
            device.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
            device.gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                utils::as_u8(indices),
                Self::mem_hint(freq, nat),
            );
            gl_result!(device.gl);

            device.gl.bind_buffer(glow::ARRAY_BUFFER, None);
            device.gl.bind_vertex_array(None);

            Ok(Self {
                vbo: vertex_array,
                vertex_buffer,
                index_buffer,
                destroy: device.destroy_sender(),
            })
        }
    }

    fn mem_hint(frequency: UsageFrequency, nature: UsageNature) -> u32 {
        use UsageFrequency as F;
        use UsageNature as N;

        match (frequency, nature) {
            (F::Stream, N::Draw) => glow::STREAM_DRAW,
            (F::Stream, N::Read) => glow::STREAM_READ,
            (F::Stream, N::Copy) => glow::STREAM_COPY,
            (F::Static, N::Draw) => glow::STATIC_DRAW,
            (F::Static, N::Read) => glow::STATIC_READ,
            (F::Static, N::Copy) => glow::STATIC_COPY,
            (F::Dynamic, N::Draw) => glow::DYNAMIC_DRAW,
            (F::Dynamic, N::Read) => glow::DYNAMIC_READ,
            (F::Dynamic, N::Copy) => glow::DYNAMIC_COPY,
        }
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        self.destroy.send(Destroy::VertexArray(self.vbo)).unwrap();
    }
}

/// Memory hint to how frequently the data will be accessed.
#[derive(Debug, Clone, Copy)]
pub enum UsageFrequency {
    /// Data store modified once then used a few times.
    Stream,
    /// Data store modified once then used multiple times.
    Static,
    /// Data store modified repeatedly and used many times.
    Dynamic,
}

/// Memory hint to the nature of the memory access.
#[derive(Debug, Clone, Copy)]
pub enum UsageNature {
    /// Data store will be modified by the application and used
    /// as a source for drawing.
    Draw,
    /// Data store is modified by reading information from OpenGL,
    /// and return it to the application.
    Read,
    /// Data store is modified by reading information from OpenGL,
    /// and used as a source for drawing.
    Copy,
}
