mod vector;

pub const MATH_MODULE: &str = "gers.math";
pub use self::vector::Vector2f;

use rust_wren::{prelude::*, ModuleBuilder, WrenResult};

pub fn register_math(vm: &mut WrenVm) -> WrenResult<()> {
    vm.interpret(MATH_MODULE, include_str!("vector.wren"))?;
    Ok(())
}

pub fn bind_math(module: &mut ModuleBuilder) {
    module.register::<Vector2f>();
}
