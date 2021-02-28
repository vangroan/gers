//! TODO: WIP
#![allow(dead_code)]
use crate::graphics::angle::Rad;
use nalgebra::{Matrix4, Vector2};
use rust_wren::prelude::*;

#[wren_class]
pub struct Transform2D {
    pub pos: Vector2<f32>,
    pub anchor: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub rot: Rad<f32>,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            pos: Vector2::new(0.0, 0.0),
            anchor: Vector2::new(0.0, 0.0),
            rot: Rad(0.0),
            scale: Vector2::new(1.0, 1.0),
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
        self.pos = Vector2::new(x, y);
    }

    #[inline(always)]
    pub fn translate(&mut self, x: f32, y: f32) {
        self.pos += Vector2::new(x, y);
    }
}

impl Transform2D {
    pub fn transform_vertex(&self, vertex: &Vector2<f32>) -> Vector2<f32> {
        // Offset by anchor.
        // Scaling and rotating happens around the origin shifted by the anchor.
        let offset = vertex - self.anchor;

        // Apply scale.
        let scaled = Vector2::new(offset.x * self.scale.x, offset.y * self.scale.y);

        // TODO: Apply rotation.
        let rotated = scaled;

        // Return to before anchor.
        let transformed = rotated + self.anchor;

        // Translate by position.
        transformed + self.pos
    }

    // Create a matrix from the transform suitable to
    // be passed to a shader.
    #[rustfmt::skip]
    pub fn to_matrix4(&self) -> Matrix4<f32> {
        // Translation
        let tx = self.pos.x;
        let ty = self.pos.y;
        let tz = 1.0;


        Matrix4::<f32>::from_rows(&[
            [1.0, 0.0, 0.0,  tx].into(),
            [0.0, 1.0, 0.0,  ty].into(),
            [0.0, 0.0, 1.0,  tz].into(),
            [0.0, 0.0, 0.0, 1.0].into(),
        ])
    }
}
