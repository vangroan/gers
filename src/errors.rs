use rust_wren::{WrenCompileError, WrenError, WrenStackFrame};
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
            GersError::InvalidCmdArgs => write!(f, "Invalid command line arguments"),
        }
    }
}

pub type GersResult<T> = std::result::Result<T, GersError>;

impl From<WrenError> for GersError {
    fn from(wren_error: WrenError) -> Self {
        GersError::Wren(wren_error)
    }
}

/// Utility for pretty printing a Wren error, with its stack trace, to logging output.
pub fn log_wren_error(logger: &slog::Logger, err: &WrenError) {
    match err {
        WrenError::CompileError(errors) => {
            for WrenCompileError { module, message, line } in errors {
                error!(logger, "Compile [{} line {}] {}", module, line, message);
            }
        }
        WrenError::RuntimeError {
            message,
            foreign,
            stack,
        } => {
            let mut msg = String::new();

            if let Some(err) = foreign {
                msg.push_str(&format!("Foreign Runtime Error: {}\n", err));
            } else {
                msg.push_str(&format!("Script Runtime Error: {}\n", message));
            };

            msg.push_str("Stack Trace:\n");

            let count = stack.len();
            for (idx, frame) in stack.into_iter().enumerate() {
                let WrenStackFrame {
                    module,
                    line,
                    function,
                    is_foreign,
                } = frame;

                if *is_foreign {
                    msg.push_str(&format!("\t{}. *foreign {}:{}\n", count - idx, module, line,));
                } else {
                    msg.push_str(&format!("\t{}. {} {}:{}\n", count - idx, module, function, line,));
                }
            }

            error!(logger, "{}", msg);
        }
        _ => error!(logger, "{}", err),
    }
}
