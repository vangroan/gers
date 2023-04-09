mod app;
mod app_layer;
mod errors;
mod input;
mod intern;

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
