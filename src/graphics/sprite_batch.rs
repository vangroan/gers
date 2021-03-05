//! Batch sprites together for optimised draw calls.
use crate::graphics::{
    device::GraphicDevice,
    errors::{debug_assert_gl, GfxResult},
    shader::Shader,
    texture::{Texture, TextureHandle},
    transform::Transform2D,
    utils,
    vao::{UsageFrequency, UsageNature, VertexArrayObject},
    vertex::Vertex,
};
use glow::HasContext;
use nalgebra::{Point3};
use rust_wren::{prelude::*, ForeignError};
use std::rc::Rc;

#[wren_class]
#[derive(Debug)]
pub struct SpriteBatch {
    items: Vec<BatchItem>,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    vao: VertexArrayObject,
}

#[wren_methods]
impl SpriteBatch {
    #[construct]
    fn new_() -> Self {
        unimplemented!()
    }

    fn new(device: &WrenCell<GraphicDevice>) -> Result<Self, ForeignError> {
        Self::create(&*device.borrow()).map_err(|err| foreign_error!(err))
    }

    #[method(name = add_)]
    fn add_6(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        texture: Option<&WrenCell<Texture>>,
        transform: &WrenCell<Transform2D>,
    ) {
        if let Some(texture) = texture.and_then(|t| t.try_borrow().ok()) {
            // Copies stuff needed for drawing to the internal batch item buffer.
            self.items.push(BatchItem {
                pos: [x, y],
                size: [width, height],
                texture_raw: texture.raw_handle(),
                texture_handle: texture.handle(),
                transform: transform.borrow().clone(),
            });
        }
    }

    #[method(name = draw)]
    fn draw_2(&mut self, device: &WrenCell<GraphicDevice>, shader: &WrenCell<Shader>) -> Result<(), ForeignError> {
        self.draw(&*device.borrow(), &*shader.borrow(), &Transform2D::default())
            .map_err(|err| foreign_error!(err))
    }

    #[method(name = draw)]
    fn draw_3(
        &mut self,
        device: &WrenCell<GraphicDevice>,
        shader: &WrenCell<Shader>,
        transform: &WrenCell<Transform2D>,
    ) -> Result<(), ForeignError> {
        self.draw(&*device.borrow(), &*shader.borrow(), &*transform.borrow())
            .map_err(|err| foreign_error!(err))
    }
}

impl SpriteBatch {
    // Vertex buffer size is unbounded, but uniform buffers have a size limit.
    // If we ever send matrices via uniform buffers, we would need to limit
    // the batch size accordingly.
    // https://www.khronos.org/opengl/wiki/Uniform_Buffer_Object#Limitations
    pub const BATCH_SIZE: usize = 2048;

    pub fn create(device: &GraphicDevice) -> GfxResult<Self> {
        // 4 vertices per sprite
        let vertices = (0..Self::BATCH_SIZE * 4)
            .map(|_| Vertex {
                position: [0.0, 0.0],
                uv: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            })
            .collect::<Vec<_>>();

        // 2 triangles, 6 indices per sprite
        let mut indices: Vec<u16> = vec![];
        for i in 0..Self::BATCH_SIZE as u16 {
            indices.push(i);
            indices.push(i + 1);
            indices.push(i + 2);

            indices.push(i);
            indices.push(i + 2);
            indices.push(i + 3);
        }

        Ok(Self {
            items: Vec::with_capacity(Self::BATCH_SIZE),
            vertices: Vec::with_capacity(Self::BATCH_SIZE * 4),
            indices: Vec::with_capacity(Self::BATCH_SIZE * 6),
            vao: VertexArrayObject::create(device, &vertices, &indices, UsageFrequency::Dynamic, UsageNature::Draw)?,
        })
    }

    #[allow(clippy::many_single_char_names)]
    pub fn draw(&mut self, device: &GraphicDevice, shader: &Shader, transform: &Transform2D) -> GfxResult<()> {
        // Nothing to draw.
        if self.items.is_empty() {
            return Ok(());
        }

        unsafe {
            let canvas_size = device.viewport_size();

            let physical_size_i32 = canvas_size.cast::<i32>();
            device
                .gl
                .viewport(0, 0, physical_size_i32.width, physical_size_i32.height);

            device.gl.use_program(Some(shader.program));

            // FIXME: Specific to the sprite shader.
            device
                .gl
                .uniform_2_f32(Some(&0), canvas_size.width as f32, canvas_size.height as f32);

            let matrix = transform.to_matrix4();
            let matrix_data = matrix.as_slice();
            // FIXME: Uniform location specific to the sprite shader.
            device.gl.uniform_matrix_4_f32_slice(Some(&1), false, matrix_data);
            // device.gl.uniform_matrix_4_f32_slice(Some(&1), false, Transform2D::default().to_matrix4().as_slice());

            device.gl.bind_vertex_array(Some(self.vao.vao));
        }

        let SpriteBatch {
            items,
            vertices,
            indices,
            vao,
        } = self;

        let mut batch_count = 0;
        let mut last_texture = None;

        for item in items.drain(..) {
            // log::info!("### BATCH {} ###", batch_count);

            if batch_count >= Self::BATCH_SIZE {
                Self::flush(device, vao, &vertices, &indices);
                vertices.clear();
                indices.clear();
                batch_count = 0;
            }

            // The buffer is flushed each time we encounter a new texture.
            if last_texture != Some(item.texture_raw) {
                Self::flush(device, vao, &vertices, &indices);
                vertices.clear();
                indices.clear();
                batch_count = 0;
                last_texture = Some(item.texture_raw);
                unsafe {
                    // Map uniform sampler to texture unit.
                    // - First argument is the uniform's location in the shader.
                    // - Second argument is the texture unit eg. `TEXTURE0`
                    device.gl.uniform_1_i32(Some(&2), 0);
                
                    // Texture slot determined by sprite shader.
                    device.gl.active_texture(glow::TEXTURE0);
                    device.gl.bind_texture(glow::TEXTURE_2D, Some(item.texture_raw));
                }
            }

            let BatchItem {
                pos: [x, y],
                size: [w, h],
                transform,
                ..
            } = item;
            // log::info!("{:?} {:?}", [x, y], [w, h]);
            // println!("{:?}", transform);
            let matrix = transform.to_matrix4();

            // Build vertices from sprite parameters.
            // TODO: scale UVs according to texture sub rectangle.
            vertices.push(Vertex {
                position: matrix
                    .transform_point(&Point3::new(x, y, 0.))
                    .to_homogeneous()
                    .xy()
                    .into(),
                uv: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            });
            vertices.push(Vertex {
                position: matrix
                    .transform_point(&Point3::new(x + w, y, 0.))
                    .to_homogeneous()
                    .xy()
                    .into(),
                uv: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            });
            vertices.push(Vertex {
                position: matrix
                    .transform_point(&Point3::new(x + w, y + h, 0.))
                    .to_homogeneous()
                    .xy()
                    .into(),
                uv: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            });
            vertices.push(Vertex {
                position: matrix
                    .transform_point(&Point3::new(x, y + h, 0.))
                    .to_homogeneous()
                    .xy()
                    .into(),
                uv: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            });

            // vertices.push(Vertex {
            //     position: [10. + x, 10. + y],
            //     uv: [0.0, 0.0],
            //     color: [1.0, 1.0, 1.0, 1.0],
            // });
            // vertices.push(Vertex {
            //     position: [10. + x + w, 10. + y],
            //     uv: [1.0, 0.0],
            //     color: [1.0, 1.0, 1.0, 1.0],
            // });
            // vertices.push(Vertex {
            //     position: [10. + x + w, 10. + y + h],
            //     uv: [1.0, 1.0],
            //     color: [1.0, 1.0, 1.0, 1.0],
            // });
            // vertices.push(Vertex {
            //     position: [10. + x, 10. + y + h],
            //     uv: [0.0, 1.0],
            //     color: [1.0, 1.0, 1.0, 1.0],
            // });

            // println!("{:?}", &vertices[vertices.len() - 4..vertices.len()]);

            let i = batch_count as u16 * 4;
            indices.push(i);
            indices.push(i + 1);
            indices.push(i + 2);
            indices.push(i);
            indices.push(i + 2);
            indices.push(i + 3);
            // println!("{:?}", &indices[indices.len() - 6..indices.len()]);

            batch_count += 1;
        }

        // Flush the last sprites that didn't reach the threshold.
        if batch_count > 0 {
            Self::flush(device, vao, &vertices, &indices);
            vertices.clear();
            indices.clear();
        }

        unsafe {
            device.gl.bind_texture(glow::TEXTURE_2D, None);
            device.gl.bind_vertex_array(None);
            device.gl.use_program(None);
        }

        Ok(())
    }

    /// This is where the actual drawing happens.
    fn flush(device: &GraphicDevice, vao: &VertexArrayObject, vertices: &[Vertex], indices: &[u16]) {
        if vertices.is_empty() {
            // Nothing to draw
            return;
        }

        // println!("{:?} {:?} {:?} {:?}", vertices[0].position, vertices[1].position, vertices[2].position, vertices[3].position);
        // println!("{:?}", indices);

        debug_assert!(vertices.len() / 4 == indices.len() / 6);

        unsafe {
            // Upload new data.
            device.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vao.vertex_buf_handle()));
            device
                .gl
                .buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, &utils::as_u8(vertices));
            debug_assert_gl(&device.gl, ());

            device
                .gl
                .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(vao.index_buf_handle()));
            device
                .gl
                .buffer_sub_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, 0, &utils::as_u8(indices));
            debug_assert_gl(&device.gl, ());

            // FIXME: Unsigned short is a detail of the vertex buffer, so drawing should probably happen there.
            device
                .gl
                .draw_elements(glow::TRIANGLES, indices.len() as i32, glow::UNSIGNED_SHORT, 0);
            debug_assert_gl(&device.gl, ());
        }
    }
}

/// The item inlines a handle to the texture in OpenGL,
/// so iteration does not need to hop a pointer. The `Rc`
/// to the texture is kept so the texture is not de-allocated
/// after the sprite is added, but before the batch is flushed.
#[derive(Debug)]
struct BatchItem {
    pos: [f32; 2],
    size: [f32; 2],
    texture_raw: glow::Texture,
    texture_handle: Rc<TextureHandle>,
    transform: Transform2D,
}
