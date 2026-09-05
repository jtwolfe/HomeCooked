//! Shared map access mode (HomeCooked write permission on a mapping entry).

use serde::{Deserialize, Serialize};

/// Whether HomeCooked writes may update this mapping entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapAccess {
    R,
    #[default]
    Rw,
}

impl MapAccess {
    pub const fn is_writable(self) -> bool {
        matches!(self, Self::Rw)
    }
}
