//! Typed values exchanged on HAL channels.

use core::fmt;

/// Bool or numeric sample / command on a logical channel.
#[derive(Debug, Clone, PartialEq)]
pub enum HalValue {
    Bool(bool),
    Number(f64),
}

impl HalValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            Self::Bool(true) => Some(1.0),
            Self::Bool(false) => Some(0.0),
        }
    }
}

impl fmt::Display for HalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
        }
    }
}

impl From<bool> for HalValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<f64> for HalValue {
    fn from(v: f64) -> Self {
        Self::Number(v)
    }
}

impl From<i32> for HalValue {
    fn from(v: i32) -> Self {
        Self::Number(f64::from(v))
    }
}

/// Motor-drive command surface (`motor.enable`, `motor.speed_rpm_cmd`, …).
///
/// Individual motor channels still use [`HalValue`]; this struct is a
/// convenience for callers that want to set several fields at once on a mock
/// or firmware backend that owns a motor IF block.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MotorCommand {
    pub enable: Option<bool>,
    pub speed_rpm: Option<f64>,
    /// `-1.0` / `0.0` / `1.0` or device-specific encoding.
    pub direction: Option<f64>,
}
