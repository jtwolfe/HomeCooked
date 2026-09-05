//! Device identifier newtype.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Stable device id (`trait.identity.device_id`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(String);

impl DeviceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for DeviceId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for DeviceId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl AsRef<str> for DeviceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
