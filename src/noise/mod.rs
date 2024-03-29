mod perm;
mod poisson;
mod voronoi;

pub const NOISE_MODULE: &str = "gers.noise";
pub use self::poisson::{PoissonDisc, PoissonOptions};
pub use self::voronoi::{Polygons, Voronoi2D};

use rust_wren::{prelude::*, ModuleBuilder, WrenResult};

pub fn register_noise(vm: &mut WrenVm) -> WrenResult<()> {
    vm.interpret(NOISE_MODULE, include_str!("voronoi.wren"))?;
    vm.interpret(NOISE_MODULE, include_str!("poisson.wren"))?;
    Ok(())
}

pub fn bind_noise(module: &mut ModuleBuilder) {
    module.register::<Voronoi2D>();
    module.register::<Polygons>();
    module.register::<PoissonDisc>();
}
