mod app;
mod errors;
mod input;
mod intern;

pub use self::{
    app::{App, GersControl, WindowConf},
    errors::GersError,
    intern::InternStr,
};

pub mod prelude {
    pub use super::errors::{GersExpectExt, GersResultExt};
}
