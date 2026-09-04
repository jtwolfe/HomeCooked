//! Schema, catalog, and generic semver (`MAJOR.MINOR.PATCH`).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ids::ParseIdError;

/// Semver-style `MAJOR.MINOR.PATCH` with no pre-release / build metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl SemVer {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub const V0_1_0: Self = Self::new(0, 1, 0);
    pub const V1_0_0: Self = Self::new(1, 0, 0);
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for SemVer {
    type Err = ParseIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || ParseIdError {
            kind: "semver",
            value: s.to_string(),
        };
        let mut parts = s.split('.');
        let major = parts.next().and_then(|p| p.parse().ok()).ok_or_else(err)?;
        let minor = parts.next().and_then(|p| p.parse().ok()).ok_or_else(err)?;
        let patch = parts.next().and_then(|p| p.parse().ok()).ok_or_else(err)?;
        if parts.next().is_some() {
            return Err(err());
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl Serialize for SemVer {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SemVer {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Schema crate version. Tracks the catalog this crate encodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(pub SemVer);

impl SchemaVersion {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self(SemVer::new(major, minor, patch))
    }

    pub const fn as_semver(self) -> SemVer {
        self.0
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Catalog version advertised by devices (`trait.identity.catalog_version`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CatalogVersion(pub SemVer);

impl CatalogVersion {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self(SemVer::new(major, minor, patch))
    }

    pub const fn as_semver(self) -> SemVer {
        self.0
    }
}

impl fmt::Display for CatalogVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Schema types in this crate (matches catalog 0.1.0).
pub const SCHEMA_VERSION: SchemaVersion = SchemaVersion::new(0, 1, 0);

/// Catalog version encoded by the static tables.
pub const CATALOG_VERSION: CatalogVersion = CatalogVersion::new(0, 1, 0);

/// Default trait version when the catalog does not note otherwise.
pub const DEFAULT_TRAIT_VERSION: SemVer = SemVer::V1_0_0;

/// Default class-slice version for a 0.1.0 catalog class.
pub const DEFAULT_CLASS_VERSION: SemVer = SemVer::V1_0_0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_roundtrip() {
        let v = SemVer::new(0, 1, 0);
        assert_eq!(v.to_string(), "0.1.0");
        assert_eq!("0.1.0".parse::<SemVer>().unwrap(), v);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "\"0.1.0\"");
        let back: SemVer = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }

    #[test]
    fn versions_match_catalog() {
        assert_eq!(SCHEMA_VERSION.to_string(), "0.1.0");
        assert_eq!(CATALOG_VERSION.to_string(), "0.1.0");
    }
}
