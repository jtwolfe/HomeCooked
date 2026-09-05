//! Parse, validation, and backend errors.

use std::fmt;

use homecooked_core::CoreError;
use homecooked_schema::{ErrorCode, ValidationError};

/// Failure while loading, validating, or talking to a device backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Json(String),
    Invalid {
        step_id: Option<String>,
        message: String,
    },
    Capability(ValidationError),
    Backend {
        code: ErrorCode,
        message: String,
        point_id: Option<String>,
    },
}

impl Error {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid {
            step_id: None,
            message: message.into(),
        }
    }

    pub fn at_step(step_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Invalid {
            step_id: Some(step_id.into()),
            message: message.into(),
        }
    }

    pub fn json(message: impl Into<String>) -> Self {
        Self::Json(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(msg) => write!(f, "json: {msg}"),
            Self::Invalid { step_id, message } => {
                if let Some(id) = step_id {
                    write!(f, "step {id}: {message}")
                } else {
                    write!(f, "{message}")
                }
            }
            Self::Capability(err) => write!(f, "{err}"),
            Self::Backend {
                code,
                message,
                point_id,
            } => {
                write!(f, "{code}")?;
                if let Some(id) = point_id {
                    write!(f, " ({id})")?;
                }
                if !message.is_empty() {
                    write!(f, ": {message}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

impl From<ValidationError> for Error {
    fn from(err: ValidationError) -> Self {
        Self::Capability(err)
    }
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Self::Backend {
            code: err.code,
            message: err.message,
            point_id: err.point_id,
        }
    }
}
