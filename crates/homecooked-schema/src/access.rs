//! Access modes: read / write / event.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ids::ParseIdError;

/// Point access flags. Serialized as catalog tokens (`r`, `r/w`, `r/w/e`, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccessMode {
    pub read: bool,
    pub write: bool,
    pub event: bool,
}

impl AccessMode {
    pub const R: Self = Self {
        read: true,
        write: false,
        event: false,
    };
    pub const W: Self = Self {
        read: false,
        write: true,
        event: false,
    };
    pub const E: Self = Self {
        read: false,
        write: false,
        event: true,
    };
    pub const RE: Self = Self {
        read: true,
        write: false,
        event: true,
    };
    pub const RW: Self = Self {
        read: true,
        write: true,
        event: false,
    };
    pub const RWE: Self = Self {
        read: true,
        write: true,
        event: true,
    };
    pub const WE: Self = Self {
        read: false,
        write: true,
        event: true,
    };

    pub const fn new(read: bool, write: bool, event: bool) -> Self {
        Self { read, write, event }
    }

    pub const fn is_writable(self) -> bool {
        self.write
    }

    pub const fn is_readable(self) -> bool {
        self.read
    }

    pub fn as_str(self) -> &'static str {
        match (self.read, self.write, self.event) {
            (true, false, false) => "r",
            (false, true, false) => "w",
            (false, false, true) => "e",
            (true, true, false) => "r/w",
            (true, false, true) => "r/e",
            (true, true, true) => "r/w/e",
            (false, true, true) => "w/e",
            (false, false, false) => "",
        }
    }
}

impl fmt::Display for AccessMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AccessMode {
    type Err = ParseIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "r" => Ok(Self::R),
            "w" => Ok(Self::W),
            "e" => Ok(Self::E),
            "r/w" => Ok(Self::RW),
            "r/e" => Ok(Self::RE),
            "r/w/e" => Ok(Self::RWE),
            "w/e" => Ok(Self::WE),
            _ => Err(ParseIdError {
                kind: "access_mode",
                value: s.to_string(),
            }),
        }
    }
}

impl Serialize for AccessMode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AccessMode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_roundtrip() {
        for mode in [
            AccessMode::R,
            AccessMode::W,
            AccessMode::E,
            AccessMode::RW,
            AccessMode::RE,
            AccessMode::RWE,
            AccessMode::WE,
        ] {
            let json = serde_json::to_string(&mode).unwrap();
            let back: AccessMode = serde_json::from_str(&json).unwrap();
            assert_eq!(back, mode);
        }
    }
}
