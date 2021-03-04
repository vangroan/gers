//! TODO: WIP
#![allow(dead_code)]
use crate::graphics::angle::{Deg, Rad};
use nalgebra::{Matrix4, Point2, Vector2};
use rust_wren::prelude::*;

#[wren_class]
#[derive(Debug, Clone)]
pub struct Transform2D {
    pub position: Point2<f32>,
    pub offset: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub rotation: Rad<f32>,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Point2::new(0.0, 0.0),
            offset: Vector2::new(0.0, 0.0),
            scale: Vector2::new(1.0, 1.0),
            rotation: Rad(0.0),
        }
    }
}

#[wren_methods]
impl Transform2D {
    #[construct]
    pub fn new() -> Self {
        Default::default()
    }

    #[method(name = setPos)]
    #[inline(always)]
    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.position = Point2::new(x, y);
    }

    #[method(name = setOffset)]
    #[inline(always)]
    pub fn set_offset(&mut self, x: f32, y: f32) {
        self.offset = Vector2::new(x, y);
    }

    #[method(name = setScale)]
    #[inline(always)]
    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.scale = Vector2::new(x, y);
    }

    #[inline(always)]
    pub fn translate(&mut self, x: f32, y: f32) {
        self.position += Vector2::new(x, y);
    }

    #[inline(always)]
    pub fn rotate(&mut self, degrees: f32) {
        self.rotation.0 += Deg(degrees).as_radians();
    }
}

impl Transform2D {
    pub fn transform_vertex(&self, vertex: &Vector2<f32>) -> Vector2<f32> {
        // Offset by anchor.
        // Scaling and rotating happens around the origin shifted by the anchor.
        let offset = vertex - self.offset;

        // Apply scale.
        let scaled = Vector2::new(offset.x * self.scale.x, offset.y * self.scale.y);

        // TODO: Apply rotation.
        let rotated = scaled;

        // Return to before anchor.
        let transformed = rotated + self.offset;

        // Translate by position.
        // transformed + self.position

        todo!()
    }

    /// Create a matrix from the transform suitable to
    /// be passed to a shader.
    ///
    /// Implementation taken from `ggez`.
    /// - [ggez/drawparam.rs](https://github.com/ggez/ggez/blob/master/src/graphics/drawparam.rs)
    #[rustfmt::skip]
    pub fn to_matrix4(&self) -> Matrix4<f32> {
        let Self { position, offset, scale, rotation } = self;

        let (sinr, cosr) = rotation.sin_cos();
        let m00 = cosr * scale.x;
        let m01 = -sinr * scale.y;
        let m10 = sinr * scale.x;
        let m11 = cosr * scale.y;

        let m03 = offset.x * (1.0 - m00) - offset.y * m01 + position.x;
        let m13 = offset.y * (1.0 - m11) - offset.x * m10 + position.y;

        Matrix4::<f32>::from_row_slice(&[
            m00, m01, 0.0, m03,
            m10, m11, 0.0, m13,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }
}
