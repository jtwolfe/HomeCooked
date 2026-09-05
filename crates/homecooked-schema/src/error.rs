//! Catalog / protocol error codes used by schema-layer validation.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Stable error tokens from the standard overview and variables catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    UnknownDevice,
    UnknownKind,
    UnknownVariable,
    UnknownCapability,
    UnsupportedCapability,
    UnsupportedOperation,
    NotWritable,
    NotReadable,
    InvalidType,
    InvalidEnum,
    InvalidRequest,
    OutOfRange,
    Busy,
    SafetyInterlock,
    RemoteDisabled,
    Unauthorized,
    Timeout,
    VersionMismatch,
    Internal,
}

impl ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownDevice => "unknown_device",
            Self::UnknownKind => "unknown_kind",
            Self::UnknownVariable => "unknown_variable",
            Self::UnknownCapability => "unknown_capability",
            Self::UnsupportedCapability => "unsupported_capability",
            Self::UnsupportedOperation => "unsupported_operation",
            Self::NotWritable => "not_writable",
            Self::NotReadable => "not_readable",
            Self::InvalidType => "invalid_type",
            Self::InvalidEnum => "invalid_enum",
            Self::InvalidRequest => "invalid_request",
            Self::OutOfRange => "out_of_range",
            Self::Busy => "busy",
            Self::SafetyInterlock => "safety_interlock",
            Self::RemoteDisabled => "remote_disabled",
            Self::Unauthorized => "unauthorized",
            Self::Timeout => "timeout",
            Self::VersionMismatch => "version_mismatch",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Schema-layer validation failure. `code` matches catalog error names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: ErrorCode,
    pub message: String,
    pub point_id: Option<String>,
    pub expected: Option<String>,
}

impl ValidationError {
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

    pub fn unknown_variable(point_id: &str) -> Self {
        Self::new(
            ErrorCode::UnknownVariable,
            format!("unknown point {point_id}"),
        )
        .at_point(point_id)
    }

    pub fn unsupported_capability(point_id: &str) -> Self {
        Self::new(
            ErrorCode::UnsupportedCapability,
            format!("point {point_id} is not advertised"),
        )
        .at_point(point_id)
    }

    pub fn not_writable(point_id: &str) -> Self {
        Self::new(
            ErrorCode::NotWritable,
            format!("point {point_id} is not writable"),
        )
        .at_point(point_id)
    }

    pub fn invalid_enum(point_id: &str, token: &str) -> Self {
        Self::new(
            ErrorCode::InvalidEnum,
            format!("invalid enum token {token:?}"),
        )
        .at_point(point_id)
    }

    pub fn out_of_range(point_id: &str, expected: impl Into<String>) -> Self {
        Self::new(ErrorCode::OutOfRange, "value outside advertised range")
            .at_point(point_id)
            .expected(expected)
    }

    pub fn invalid_type(point_id: &str, expected: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidType, "value type does not match point")
            .at_point(point_id)
            .expected(expected)
    }
}

impl fmt::Display for ValidationError {
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

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_tokens() {
        assert_eq!(ErrorCode::OutOfRange.as_str(), "out_of_range");
        assert_eq!(ErrorCode::InvalidEnum.as_str(), "invalid_enum");
        assert_eq!(ErrorCode::NotWritable.as_str(), "not_writable");
        assert_eq!(
            ErrorCode::UnsupportedCapability.as_str(),
            "unsupported_capability"
        );
        assert_eq!(ErrorCode::UnknownVariable.as_str(), "unknown_variable");
        let json = serde_json::to_string(&ErrorCode::OutOfRange).unwrap();
        assert_eq!(json, "\"out_of_range\"");
    }
}
