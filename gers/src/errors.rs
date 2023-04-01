//! GERS errors.
use std::{borrow::Cow, fmt};

#[derive(Debug)]
pub struct GersError {
    pub message: Option<Cow<'static, str>>,
    pub kind: ErrorKind,
    /// Optional further context that decorates the error.
    pub context: Vec<String>,
}

#[derive(Debug)]
pub enum ErrorKind {
    /// Unspecified error occurred.
    Generic,
    Window(winit::error::OsError),
}

pub trait GersResultExt {
    /// Adds additional context, if the result contains an error.
    fn with_context<S>(self, context: impl FnOnce() -> S) -> Self
    where
        S: ToString;

    /// Replace the error message, if the result contains an error.
    fn with_message<S>(self, message: S) -> Self
    where
        S: Into<Cow<'static, str>>;
}

impl fmt::Display for GersError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { message, kind, context } = self;

        match message {
            Some(message) => write!(f, "Engine error: {message}; {kind}")?,
            None => write!(f, "Engine error: {kind}")?,
        }

        // Further optional context
        if !context.is_empty() {
            writeln!(f)?;
            writeln!(f)?;
            writeln!(f, "Further context:")?;
            writeln!(f)?;

            for ctx_msg in context {
                writeln!(f, "- {ctx_msg}")?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for ErrorKind {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Window(err) => write!(f, "{err}"),
            Self::Generic => write!(f, "unspecified error"),
        }
    }
}

impl GersError {
    pub fn generic<S>(message: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self {
            kind: ErrorKind::Generic,
            message: Some(message.into()),
            context: vec![],
        }
    }

    pub fn add_context(&mut self, context: impl ToString) {
        self.context.push(context.to_string());
    }
}

impl<T> GersResultExt for Result<T, GersError> {
    fn with_context<S>(self, context: impl FnOnce() -> S) -> Result<T, GersError>
    where
        S: ToString,
    {
        self.map_err(|mut err| {
            err.add_context(context());
            err
        })
    }

    fn with_message<S>(self, message: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.map_err(|mut err| {
            if let Some(previous) = err.message.replace(message.into()) {
                log::warn!("replacing error message: {previous}");
            }
            err
        })
    }
}

impl From<winit::error::OsError> for GersError {
    fn from(err: winit::error::OsError) -> Self {
        Self {
            message: None,
            kind: ErrorKind::Window(err),
            context: vec![],
        }
    }
}
