//! Data collections.
mod script_array;

pub use self::script_array::{ArrayIterator, F32Array, F64Array, OutOfBounds, U16Array, U32Array, U8Array};

use rust_wren::{prelude::*, ModuleBuilder, WrenResult};

pub const COLLECTIONS_MODULE: &str = "collections";

pub fn register_collections(vm: &mut WrenVm) -> WrenResult<()> {
    vm.interpret(COLLECTIONS_MODULE, include_str!("script_array.wren"))?;
    vm.interpret(COLLECTIONS_MODULE, &U8Array::script())?;
    vm.interpret(COLLECTIONS_MODULE, &U32Array::script())?;
    vm.interpret(COLLECTIONS_MODULE, &F32Array::script())?;
    vm.interpret(COLLECTIONS_MODULE, &F64Array::script())?;

    Ok(())
}

pub fn bind_collections(module: &mut ModuleBuilder) {
    module.register::<U16Array>();
    module.register::<U8Array>();
    module.register::<U32Array>();
    module.register::<F32Array>();
    module.register::<F64Array>();
}
