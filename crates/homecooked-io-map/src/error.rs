//! I/O map parse and validation errors.

use std::fmt;

/// Failure while loading or validating a chassis I/O map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Yaml(String),
    Json(String),
    Io(String),
    DuplicateChannel(String),
    UnknownKind { channel: String, kind: String },
    UnknownPrefix { channel: String, prefix: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yaml(msg) => write!(f, "yaml: {msg}"),
            Self::Json(msg) => write!(f, "json: {msg}"),
            Self::Io(msg) => write!(f, "io: {msg}"),
            Self::DuplicateChannel(channel) => {
                write!(f, "duplicate channel {channel}")
            }
            Self::UnknownKind { channel, kind } => {
                write!(f, "unknown kind {kind:?} on channel {channel}")
            }
            Self::UnknownPrefix { channel, prefix } => {
                write!(f, "unknown channel prefix {prefix:?} on {channel}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Yaml(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}
