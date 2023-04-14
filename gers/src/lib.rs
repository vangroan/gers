mod app;
mod app_layer;
mod color;
mod errors;
mod gfx;
mod input;
mod intern;

pub mod version {
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
    pub const MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
    pub const PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");
}

pub use self::{
    app::{App, GersControl, WindowConf},
    app_layer::{AppLayer, UpdateCtx},
    errors::GersError,
    input::{ActionInfo, InputMap},
    intern::InternStr,
};

pub mod prelude {
    pub use super::{
        app_layer::AppLayer,
        errors::{GersExpectExt, GersResultExt},
    };
}
