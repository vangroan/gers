//! Statically typed arrays for Wren.
use gers_codegen::impl_array;
use rust_wren::{prelude::*, WrenContext};
use std::{error::Error, fmt};

impl_array!(U8, u8);
impl_array!(U16, u16);
impl_array!(U32, u32);
impl_array!(I8, i8);
impl_array!(I16, i16);
impl_array!(I32, i32);
impl_array!(F32, f32);
impl_array!(F64, f64);

#[derive(Debug)]
pub struct OutOfBounds {
    pub index: i32,
    pub size: usize,
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
