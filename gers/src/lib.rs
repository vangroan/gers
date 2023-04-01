mod app;
mod errors;
mod input;

pub use self::{
    app::{App, GersControl, WindowConf},
    errors::GersError,
};

pub mod prelude {
    pub use super::errors::{GersExpectExt, GersResultExt};
}
