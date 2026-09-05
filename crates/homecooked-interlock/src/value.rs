//! Snapshot values and proposed actuator commands.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Bool, number, or string value on a channel / point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Number(f64),
    String(String),
}

impl Value {
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
            Self::String(_) => None,
        }
    }

    pub fn equal_to(&self, other: &Value) -> bool {
        match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (a, b) => match (a.as_number(), b.as_number()) {
                (Some(x), Some(y)) => x == y,
                _ => false,
            },
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
        }
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Number(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Self::Number(v as f64)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::String(v.into())
    }
}

/// Channel / point values at evaluate time.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    values: BTreeMap<String, Value>,
}

impl Snapshot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Value>) -> &mut Self {
        self.values.insert(key.into(), value.into());
        self
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.insert(key, value);
        self
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    /// Overlay a proposed command so conditions can see the requested value.
    pub fn with_command(&self, command: &Command) -> Self {
        let mut next = self.clone();
        next.insert(command.channel.clone(), command.value.clone());
        next
    }
}

/// Proposed write to an actuator channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Command {
    pub channel: String,
    pub value: Value,
}

impl Command {
    pub fn new(channel: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            channel: channel.into(),
            value: value.into(),
        }
    }
}
