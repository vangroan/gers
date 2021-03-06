mod config;

pub const WINDOW_MODULE: &str = "gers.window";
pub use config::WrenWindowConfig;

use rust_wren::{prelude::*, ModuleBuilder, WrenResult};

pub fn register_window(vm: &mut WrenVm) -> WrenResult<()> {
    vm.interpret(WINDOW_MODULE, include_str!("config.wren"))?;
    vm.interpret(WINDOW_MODULE, include_str!("signal.wren"))?;

    Ok(())
}

pub fn bind_window(module: &mut ModuleBuilder) {
    module.register::<WrenWindowConfig>();
}
