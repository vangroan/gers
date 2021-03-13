//! Data collections.
mod script_array;

pub use self::script_array::{
    ArrayIterator, F32Array, F64Array, I16Array, I32Array, I8Array, OutOfBounds, U16Array, U32Array, U8Array,
};

use rust_wren::{prelude::*, ModuleBuilder, WrenResult};

pub const COLLECTIONS_MODULE: &str = "gers.collections";

pub fn register_collections(vm: &mut WrenVm) -> WrenResult<()> {
    // vm.interpret(COLLECTIONS_MODULE, include_str!("script_array.wren"))?;
    vm.interpret(COLLECTIONS_MODULE, &U8Array::script())?;
    vm.interpret(COLLECTIONS_MODULE, &U16Array::script())?;
    vm.interpret(COLLECTIONS_MODULE, &U32Array::script())?;
    vm.interpret(COLLECTIONS_MODULE, &I8Array::script())?;
    vm.interpret(COLLECTIONS_MODULE, &I16Array::script())?;
    vm.interpret(COLLECTIONS_MODULE, &I32Array::script())?;
    vm.interpret(COLLECTIONS_MODULE, &F32Array::script())?;
    vm.interpret(COLLECTIONS_MODULE, &F64Array::script())?;

    Ok(())
}

pub fn bind_collections(module: &mut ModuleBuilder) {
    module.register::<U8Array>();
    module.register::<U16Array>();
    module.register::<U32Array>();
    module.register::<I8Array>();
    module.register::<I16Array>();
    module.register::<I32Array>();
    module.register::<F32Array>();
    module.register::<F64Array>();
}
