//! Registry and request-handling errors.

use homecooked_protocol::ErrorBody;
use homecooked_schema::{ErrorCode, ValidationError};

/// High-level core failure. Convertible to a protocol `error` body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreError {
    pub code: ErrorCode,
    pub message: String,
    pub point_id: Option<String>,
    pub expected: Option<String>,
}

impl CoreError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            point_id: None,
            expected: None,
        }
    }

    pub fn at_point(mut self, point_id: impl Into<String>) -> Self {
        self.point_id = Some(point_id.into());
        self
    }

    pub fn expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    pub fn unknown_device(device_id: &str) -> Self {
        Self::new(
            ErrorCode::UnknownDevice,
            format!("unknown device {device_id}"),
        )
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidRequest, message)
    }
}

impl From<ValidationError> for CoreError {
    fn from(err: ValidationError) -> Self {
        Self {
            code: err.code,
            message: err.message,
            point_id: err.point_id,
            expected: err.expected,
        }
    }
}

impl From<ErrorBody> for CoreError {
    fn from(err: ErrorBody) -> Self {
        Self {
            code: err.code,
            message: err.message,
            point_id: err.point_id,
            expected: err.expected,
        }
    }
}

impl From<CoreError> for ErrorBody {
    fn from(err: CoreError) -> Self {
        let mut body = ErrorBody::new(err.code, err.message);
        if let Some(id) = err.point_id {
            body = body.at_point(id);
        }
        if let Some(expected) = err.expected {
            body = body.expected(expected);
        }
        body
    }
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)?;
        if let Some(id) = &self.point_id {
            write!(f, " ({id})")?;
        }
        if !self.message.is_empty() {
            write!(f, ": {}", self.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for CoreError {}
