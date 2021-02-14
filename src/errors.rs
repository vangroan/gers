use rust_wren::WrenError;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum GersError {
    /// Error in Wren VM or deeper within foreign function calls.
    Wren(WrenError),

    /// Error when the program was executed with incorrect command line arguments.
    InvalidCmdArgs,
}

impl Error for GersError {}

impl fmt::Display for GersError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GersError::Wren(err) => fmt::Display::fmt(err, f),
            InvalidCmdArgs => write!(f, "Invalid command line arguments"),
        }
    }
}

pub type GersResult<T> = std::result::Result<T, GersError>;

impl From<WrenError> for GersError {
    fn from(wren_error: WrenError) -> Self {
        GersError::Wren(wren_error)
    }
}
