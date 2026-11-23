use std::{error::Error, fmt};

/// An application lifecycle, action-decoding, or view error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplicationError {
    AlreadyStarted,
    NotStarted,
    UnexpectedSurface { expected: String, actual: String },
    InvalidAction(String),
    InvalidState(String),
    InvalidView(String),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyStarted => f.write_str("the application is already started"),
            Self::NotStarted => f.write_str("the application is not started"),
            Self::UnexpectedSurface { expected, actual } => write!(
                f,
                "action targets surface {actual:?}, expected {expected:?}"
            ),
            Self::InvalidAction(message) => write!(f, "invalid A2UI action: {message}"),
            Self::InvalidState(message) => write!(f, "invalid application state: {message}"),
            Self::InvalidView(message) => write!(f, "invalid A2UI view: {message}"),
        }
    }
}

impl Error for ApplicationError {}
