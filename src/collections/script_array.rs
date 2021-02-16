//! Statically typed arrays for Wren.
use rust_wren::{prelude::*, WrenContext, WrenResult};
use std::{error::Error, fmt};

#[wren_class]
pub struct U16Array(Vec<u16>);

#[wren_methods]
impl U16Array {
    #[construct]
    pub fn new() -> Self {
        U16Array(Vec::new())
    }

    // TODO: Support subscript operator
    #[inline]
    pub fn get(&self, index: i32) -> rust_wren::Result<u16> {
        self.0.get(self.convert_index(index)).cloned().ok_or_else(|| {
            foreign_error!(OutOfBounds {
                index,
                size: self.0.len()
            })
        })
    }

    #[inline]
    pub fn add(&mut self, value: u16) {
        self.0.push(value)
    }

    #[inline]
    pub fn insert(&mut self, index: i32, value: u16) {
        self.0.insert(self.convert_index(index), value);
    }

    #[inline]
    #[method(name = removeAt)]
    pub fn remove_at(&mut self, index: i32) -> rust_wren::Result<u16> {
        if !self.in_bounds(index) {
            Err(foreign_error!(OutOfBounds {
                index,
                size: self.0.len()
            }))
        } else {
            // Vec panics on out of bounds remove.
            Ok(self.0.remove(self.convert_index(index)))
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    // TODO: count property
    // TODO: toString property

    #[inline]
    pub fn iterate(&self, index: Option<i32>) -> ArrayIterator {
        match index {
            None => ArrayIterator::Index(0),
            Some(index) => {
                if index < (self.0.len() as i32) - 1 {
                    ArrayIterator::Index(index + 1)
                } else {
                    ArrayIterator::Done
                }
            }
        }
    }

    #[inline]
    #[method(name = iteratorValue)]
    pub fn iterator_value(&self, index: i32) -> rust_wren::Result<u16> {
        self.get(index)
    }
}

impl U16Array {
    /// Convert a Wren index, which can be negative, to an
    /// unsigned Rust index.
    ///
    /// Wren allows indexing froms the back of a list.
    #[inline(always)]
    fn convert_index(&self, index: i32) -> usize {
        if index >= 0 {
            index as usize
        } else {
            self.0.len() - index as usize
        }
    }

    #[inline(always)]
    fn in_bounds(&self, index: i32) -> bool {
        index >= -(self.0.len() as i32) && index < self.0.len() as i32
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[u16] {
        &self.0
    }

    #[inline(always)]
    pub fn as_slice_mut(&mut self) -> &mut [u16] {
        &mut self.0
    }
}

impl Default for U16Array {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct OutOfBounds {
    index: i32,
    size: usize,
}

impl Error for OutOfBounds {}

impl fmt::Display for OutOfBounds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Index out of bounds. Index {}, array size is {}.",
            self.index, self.size
        )
    }
}

/// Wren iterator protocol.
pub enum ArrayIterator {
    Index(i32),
    Done,
}

impl ToWren for ArrayIterator {
    fn put(self, ctx: &mut WrenContext, slot: i32) {
        ctx.ensure_slots(1);

        match self {
            Self::Index(index) => index.put(ctx, slot),
            Self::Done => false.put(ctx, slot), // Iterator protocol expects false on done.
        }
    }

    fn size_hint(&self) -> usize {
        1
    }
}
