//! TODO: WIP
#![allow(dead_code)]
use crate::graphics::angle::Rad;
use nalgebra::{Matrix4, Vector2};

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

impl Transform2D {
    pub fn transform_vertex(&self, vertex: &Vector2<f32>) -> Vector2<f32> {
        // Offset by anchor.
        // Scaling and rotating happens around the origin shifted by the anchor.
        let offset = vertex - &self.anchor;

        // Apply scale.
        let scaled = Vector2::new(offset.x * self.scale.x, offset.y * self.scale.y);

        // TODO: Apply rotation.
        let rotated = scaled;

        // Return to before anchor.
        let transformed = rotated + &self.anchor;

        // Translate by position.
        transformed + &self.pos
    }

    // Create a matrix from the transform suitable to
    // be passed to a shader.
    pub fn to_matrix4(&self) -> Matrix4<f32> {
        let _m = Matrix4::<f32>::from_rows(&[
            [1.0, 0.0, 0.0, 0.0].into(),
            [0.0, 1.0, 0.0, 0.0].into(),
            [0.0, 0.0, 1.0, 0.0].into(),
            [0.0, 0.0, 0.0, 1.0].into(),
        ]);
        todo!()
    }
}
