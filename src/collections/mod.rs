//! Data collections.
mod script_array;

pub use self::script_array::{OutOfBounds, U16Array};

use rust_wren::{prelude::*, ModuleBuilder, WrenResult};

pub const COLLECTIONS_MODULE: &str = "collections";

pub fn register_collections(vm: &mut WrenVm) -> WrenResult<()> {
    vm.interpret(COLLECTIONS_MODULE, include_str!("script_array.wren"))
}

pub fn bind_collections(module: &mut ModuleBuilder) {
    module.register::<U16Array>();
}
