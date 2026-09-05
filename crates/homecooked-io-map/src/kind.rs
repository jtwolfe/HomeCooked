//! Channel kinds and HAL prefixes.

use std::fmt;

/// Allowed I/O kinds (and matching channel prefixes).
///
/// Prefixes are the first `snake_case` segment of a channel id (`din.door_closed`
/// → `din`, `motor.speed_rpm_cmd` → `motor`). `relay` is accepted as a kind
/// and as a `relay.*` prefix even though the control-system sketch prefers
/// `aout.*` / `dout.*` for actuators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IoKind {
    Din,
    Dout,
    Ain,
    Aout,
    Relay,
    Motor,
}

impl IoKind {
    /// Every accepted kind / prefix token.
    pub const ALL: [IoKind; 6] = [
        IoKind::Din,
        IoKind::Dout,
        IoKind::Ain,
        IoKind::Aout,
        IoKind::Relay,
        IoKind::Motor,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Din => "din",
            Self::Dout => "dout",
            Self::Ain => "ain",
            Self::Aout => "aout",
            Self::Relay => "relay",
            Self::Motor => "motor",
        }
    }

    /// Parse a kind or channel-prefix token (`din`, `motor`, …).
    pub fn from_token(s: &str) -> Option<Self> {
        match s {
            "din" => Some(Self::Din),
            "dout" => Some(Self::Dout),
            "ain" => Some(Self::Ain),
            "aout" => Some(Self::Aout),
            "relay" => Some(Self::Relay),
            "motor" => Some(Self::Motor),
            _ => None,
        }
    }
}

impl fmt::Display for IoKind {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_roundtrip() {
        for kind in IoKind::ALL {
            assert_eq!(IoKind::from_token(kind.as_str()), Some(kind));
        }
        assert!(IoKind::from_token("pwm").is_none());
        assert!(IoKind::from_token("thermal").is_none());
    }

    #[test]
    fn prefixes() {
        assert_eq!(channel_prefix("din.door_closed"), "din");
        assert_eq!(channel_prefix("motor.speed_rpm_cmd"), "motor");
        assert_eq!(channel_prefix("relay"), "relay");
        assert_eq!(channel_prefix(""), "");
    }
}
