//! Logical HAL channel ids and kinds.
//!
//! Channel ids are `snake_case` with a kind prefix, matching
//! [`docs/standard/control-system.md`](../../../docs/standard/control-system.md)
//! §4.3 and `homecooked-io-map` (`din.*`, `dout.*`, `ain.*`, `aout.*`,
//! `relay.*`, `motor.*`).

use core::fmt;
use core::str::FromStr;

use crate::error::Error;

/// Allowed HAL channel kinds, aligned with `homecooked-io-map::IoKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChannelKind {
    /// Digital input (`din.*`).
    DigitalIn,
    /// Digital / LV output (`dout.*`).
    DigitalOut,
    /// Analog input (`ain.*`).
    AnalogIn,
    /// Analog / gated HV actuator out (`aout.*`).
    AnalogOut,
    /// Relay / discrete HV coil (`relay.*`).
    Relay,
    /// Motor-drive interface (`motor.*`).
    Motor,
}

impl ChannelKind {
    pub const ALL: [ChannelKind; 6] = [
        ChannelKind::DigitalIn,
        ChannelKind::DigitalOut,
        ChannelKind::AnalogIn,
        ChannelKind::AnalogOut,
        ChannelKind::Relay,
        ChannelKind::Motor,
    ];

    /// Prefix token used in channel ids (`din`, `motor`, …).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DigitalIn => "din",
            Self::DigitalOut => "dout",
            Self::AnalogIn => "ain",
            Self::AnalogOut => "aout",
            Self::Relay => "relay",
            Self::Motor => "motor",
        }
    }

    pub fn from_token(s: &str) -> Option<Self> {
        match s {
            "din" => Some(Self::DigitalIn),
            "dout" => Some(Self::DigitalOut),
            "ain" => Some(Self::AnalogIn),
            "aout" => Some(Self::AnalogOut),
            "relay" => Some(Self::Relay),
            "motor" => Some(Self::Motor),
            _ => None,
        }
    }

    /// Inputs are readable; outputs / actuators are writable.
    pub const fn is_input(self) -> bool {
        matches!(self, Self::DigitalIn | Self::AnalogIn)
    }

    pub const fn is_output(self) -> bool {
        !self.is_input()
    }
}

impl fmt::Display for ChannelKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// First `snake_case` segment of a channel id, or the whole id if it has no `.`.
pub fn channel_prefix(channel: &str) -> &str {
    match channel.split_once('.') {
        Some((prefix, _)) => prefix,
        None => channel,
    }
}

/// Logical HAL channel id (`din.door_closed`, `aout.heater_enable`, …).
///
/// Compatible with `homecooked-io-map` binding `channel` strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChannelId {
    raw: String,
    kind: ChannelKind,
}

impl ChannelId {
    /// Parse and validate a channel string (`prefix.name`).
    pub fn new(s: impl AsRef<str>) -> Result<Self, Error> {
        let raw = s.as_ref().trim();
        if raw.is_empty() {
            return Err(Error::InvalidChannel {
                channel: raw.to_string(),
                detail: "empty channel id".into(),
            });
        }
        let prefix = channel_prefix(raw);
        let kind = ChannelKind::from_token(prefix).ok_or_else(|| Error::UnknownKind {
            channel: raw.to_string(),
            prefix: prefix.to_string(),
        })?;
        if !raw.contains('.') || channel_prefix(raw) == raw {
            return Err(Error::InvalidChannel {
                channel: raw.to_string(),
                detail: "expected kind.suffix (e.g. din.door_closed)".into(),
            });
        }
        let suffix = &raw[prefix.len() + 1..];
        if suffix.is_empty()
            || !suffix
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(Error::InvalidChannel {
                channel: raw.to_string(),
                detail: "suffix must be snake_case ascii".into(),
            });
        }
        Ok(Self {
            raw: raw.to_string(),
            kind,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn kind(&self) -> ChannelKind {
        self.kind
    }
}

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

impl AsRef<str> for ChannelId {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}

impl FromStr for ChannelId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<&str> for ChannelId {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for ChannelId {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_roundtrip() {
        for kind in ChannelKind::ALL {
            assert_eq!(ChannelKind::from_token(kind.as_str()), Some(kind));
        }
        assert!(ChannelKind::from_token("pwm").is_none());
    }

    #[test]
    fn parse_ok() {
        let id = ChannelId::new("din.door_closed").unwrap();
        assert_eq!(id.kind(), ChannelKind::DigitalIn);
        assert_eq!(id.as_str(), "din.door_closed");
        assert_eq!(
            ChannelId::new("aout.heater_enable").unwrap().kind(),
            ChannelKind::AnalogOut
        );
        assert_eq!(
            ChannelId::new("motor.speed_rpm_cmd").unwrap().kind(),
            ChannelKind::Motor
        );
        assert_eq!(
            ChannelId::new("relay.heater").unwrap().kind(),
            ChannelKind::Relay
        );
    }

    #[test]
    fn parse_rejects() {
        assert!(ChannelId::new("").is_err());
        assert!(ChannelId::new("din").is_err());
        assert!(ChannelId::new("din.").is_err());
        assert!(ChannelId::new("foo.bar").is_err());
        assert!(ChannelId::new("din.Door").is_err());
    }
}
