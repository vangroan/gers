//! Collection of vertices.
//!
//! # TODO
//!
//! Because this is a container of foreign types, the
//! array macro won't work.
//!
//! We need a new macro specifically for contains of
//! foreign types, because of WrenCell<T>.
use super::vertex::Vertex;
use crate::collections::OutOfBounds;
use rust_wren::prelude::*;
use std::fmt;

#[wren_class]
pub struct VertexArray(Vec<Vertex>);

#[wren_methods]
impl VertexArray {
    #[construct]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Copies vertex into array.
    #[inline]
    pub fn add(&mut self, value: &WrenCell<Vertex>) {
        self.0.push(value.borrow().clone())
    }

    /// Copies vertex out of array.
    #[inline]
    pub fn get(&self, index: i32) -> rust_wren::Result<Vertex> {
        self.0.get(self.convert_index(index)).cloned().ok_or_else(|| {
            foreign_error!(OutOfBounds {
                index,
                size: self.0.len()
            })
        })
    }

    // TODO: Prop getter
    #[method(name = toString)]
    #[inline]
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl VertexArray {
    #[inline(always)]
    fn convert_index(&self, index: i32) -> usize {
        if index >= 0 {
            index as usize
        } else {
            self.0.len() - index as usize
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[Vertex] {
        &self.0
    }

    /// Borrows the contents of the array as an mutable slice.
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [Vertex] {
        &mut self.0
    }
}

impl fmt::Debug for VertexArray {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}
