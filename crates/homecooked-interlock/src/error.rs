//! Rule load errors.

use std::fmt;

/// Failure while loading interlock rules from YAML or JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Yaml(String),
    Json(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yaml(msg) => write!(f, "yaml: {msg}"),
            Self::Json(msg) => write!(f, "json: {msg}"),
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
