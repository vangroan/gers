//! Miscellaneous utilities.
use std::{mem, slice};

/// Cast a slice to a slice of bytes.
///
/// Result will be native endianness.
///
/// # Safety
///
/// There should be no undefined behaviour with the cast.
pub(crate) unsafe fn as_u8<T>(buf: &[T]) -> &[u8] {
    let ptr = buf.as_ptr() as *const u8;
    let size = buf.len() * mem::size_of::<T>();
    // SAFETY: The required invariants will be met
    //         because we're working from a valid &[T].
    //         - We trust the borrow points to non-null and non-junk data.
    //         - Length arithmetic should be good according to Rust's
    //           underlying representation of slices.
    //         - Allocation size restrictions would have been applied
    //           to the array backing the slice.
    slice::from_raw_parts(ptr, size)
}
