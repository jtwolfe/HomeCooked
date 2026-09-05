//! Protocol error body and decode/version failures.

use std::fmt;

use homecooked_schema::{ErrorCode, SemVer, ValidationError};
use serde::{Deserialize, Serialize};

use crate::PROTOCOL_VERSION;

/// `kind: error` body (overview §9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub point_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(default)]
    pub retryable: bool,
}

impl ErrorBody {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            point_id: None,
            expected: None,
            retryable: is_retryable(code),
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

    pub fn version_mismatch(got: SemVer) -> Self {
        Self::new(
            ErrorCode::VersionMismatch,
            format!(
                "protocol major {} is incompatible with {}",
                got.major, PROTOCOL_VERSION
            ),
        )
        .expected(format!("major {}", PROTOCOL_VERSION.major))
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

    pub fn unknown_kind(kind: &str) -> Self {
        Self::new(
            ErrorCode::UnknownKind,
            format!("unknown message kind {kind}"),
        )
    }
}

impl From<ValidationError> for ErrorBody {
    fn from(err: ValidationError) -> Self {
        Self {
            retryable: is_retryable(err.code),
            code: err.code,
            message: err.message,
            point_id: err.point_id,
            expected: err.expected,
        }
    }
}

impl fmt::Display for ErrorBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl std::error::Error for ErrorBody {}

/// Retryability from overview §9.1. `internal` is treated as not retryable.
pub fn is_retryable(code: ErrorCode) -> bool {
    matches!(code, ErrorCode::Busy | ErrorCode::Timeout)
}

/// Failures while decoding or checking a message (not a wire `error` body).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    VersionMismatch { got: SemVer, expected_major: u64 },
    Json(String),
}

impl ProtocolError {
    pub fn version_mismatch(got: SemVer) -> Self {
        Self::VersionMismatch {
            got,
            expected_major: PROTOCOL_VERSION.major,
        }
    }

    pub fn to_error_body(&self) -> ErrorBody {
        match self {
            Self::VersionMismatch { got, .. } => ErrorBody::version_mismatch(*got),
            Self::Json(msg) => ErrorBody::invalid_request(msg.clone()),
        }
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VersionMismatch {
                got,
                expected_major,
            } => write!(
                f,
                "version_mismatch: got {got}, expected major {expected_major}"
            ),
            Self::Json(msg) => write!(f, "invalid json: {msg}"),
        }
    }
}

impl std::error::Error for ProtocolError {}

impl From<serde_json::Error> for ProtocolError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}
