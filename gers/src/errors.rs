//! GERS errors.
use std::fmt;

#[derive(Debug)]
pub struct GersError {
    pub kind: ErrorKind,
    /// Optional further context that decorates the error.
    pub context: Vec<String>,
}

#[derive(Debug)]
pub enum ErrorKind {
    Window(winit::error::OsError),
}

impl fmt::Display for GersError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "engine error: {}", self.kind)?;

        // Further optional context
        if !self.context.is_empty() {
            writeln!(f)?;
            writeln!(f, "Further context:")?;
            writeln!(f)?;

            for ctx_msg in &self.context {
                writeln!(f, "- {ctx_msg}")?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for ErrorKind {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "engine error: ")?;

        match self {
            Self::Window(err) => write!(f, "{err}"),
        }
    }
}

impl GersError {
    pub fn with_context(mut self, context: impl ToString) -> Self {
        self.context.push(context.to_string());
        self
    }
}

impl From<winit::error::OsError> for GersError {
    fn from(err: winit::error::OsError) -> Self {
        Self {
            kind: ErrorKind::Window(err),
            context: vec![],
        }
    }
}
