//! Catalog primitive types, units, values, and ranges.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ids::ParseIdError;

/// Item type inside a `list<T>` catalog type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListItemType {
    Bool,
    U8,
    U16,
    U32,
    I16,
    I32,
    F32,
    Enum,
    String,
}

impl ListItemType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::F32 => "f32",
            Self::Enum => "enum",
            Self::String => "string",
        }
    }

    pub const fn to_value_type(self) -> ValueType {
        match self {
            Self::Bool => ValueType::Bool,
            Self::U8 => ValueType::U8,
            Self::U16 => ValueType::U16,
            Self::U32 => ValueType::U32,
            Self::I16 => ValueType::I16,
            Self::I32 => ValueType::I32,
            Self::F32 => ValueType::F32,
            Self::Enum => ValueType::Enum,
            Self::String => ValueType::String,
        }
    }
}

/// Catalog type tag (`bool`, `u16`, `enum`, `list<enum>`, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    Bool,
    U8,
    U16,
    U32,
    I16,
    I32,
    F32,
    Percent,
    Enum,
    String,
    TimestampMs,
    DurationS,
    Command,
    List(ListItemType),
}

impl ValueType {
    pub fn as_str(self) -> String {
        match self {
            Self::Bool => "bool".to_string(),
            Self::U8 => "u8".to_string(),
            Self::U16 => "u16".to_string(),
            Self::U32 => "u32".to_string(),
            Self::I16 => "i16".to_string(),
            Self::I32 => "i32".to_string(),
            Self::F32 => "f32".to_string(),
            Self::Percent => "percent".to_string(),
            Self::Enum => "enum".to_string(),
            Self::String => "string".to_string(),
            Self::TimestampMs => "timestamp_ms".to_string(),
            Self::DurationS => "duration_s".to_string(),
            Self::Command => "command".to_string(),
            Self::List(item) => format!("list<{}>", item.as_str()),
        }
    }

    pub fn is_numeric(self) -> bool {
        matches!(
            self,
            Self::U8
                | Self::U16
                | Self::U32
                | Self::I16
                | Self::I32
                | Self::F32
                | Self::Percent
                | Self::DurationS
                | Self::TimestampMs
        )
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_str())
    }
}

impl FromStr for ValueType {
    type Err = ParseIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || ParseIdError {
            kind: "value_type",
            value: s.to_string(),
        };
        match s {
            "bool" => Ok(Self::Bool),
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "i16" => Ok(Self::I16),
            "i32" => Ok(Self::I32),
            "f32" => Ok(Self::F32),
            "percent" => Ok(Self::Percent),
            "enum" => Ok(Self::Enum),
            "string" => Ok(Self::String),
            "timestamp_ms" => Ok(Self::TimestampMs),
            "duration_s" => Ok(Self::DurationS),
            "command" => Ok(Self::Command),
            other => {
                let inner = other
                    .strip_prefix("list<")
                    .and_then(|rest| rest.strip_suffix('>'))
                    .ok_or_else(err)?;
                let item = match inner {
                    "bool" => ListItemType::Bool,
                    "u8" => ListItemType::U8,
                    "u16" => ListItemType::U16,
                    "u32" => ListItemType::U32,
                    "i16" => ListItemType::I16,
                    "i32" => ListItemType::I32,
                    "f32" => ListItemType::F32,
                    "enum" => ListItemType::Enum,
                    "string" => ListItemType::String,
                    _ => return Err(err()),
                };
                Ok(Self::List(item))
            }
        }
    }
}

impl Serialize for ValueType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.as_str())
    }
}

impl<'de> Deserialize<'de> for ValueType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Catalog unit tokens. Temperatures on the wire are always celsius.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Unit {
    #[serde(rename = "celsius")]
    Celsius,
    #[serde(rename = "percent")]
    Percent,
    #[serde(rename = "second")]
    Second,
    #[serde(rename = "watt")]
    Watt,
    #[serde(rename = "watt_hour")]
    WattHour,
    #[serde(rename = "volt")]
    Volt,
    #[serde(rename = "ampere")]
    Ampere,
    #[serde(rename = "rpm")]
    Rpm,
    #[serde(rename = "liter")]
    Liter,
    #[serde(rename = "milliliter")]
    Milliliter,
    #[serde(rename = "liter_per_min")]
    LiterPerMin,
    #[serde(rename = "gram")]
    Gram,
    #[serde(rename = "kilogram")]
    Kilogram,
    #[serde(rename = "pascal")]
    Pascal,
    #[serde(rename = "kilopascal")]
    Kilopascal,
    #[serde(rename = "bar")]
    Bar,
    #[serde(rename = "ppm")]
    Ppm,
    #[serde(rename = "gpg")]
    Gpg,
    #[serde(rename = "rh_percent")]
    RhPercent,
    #[serde(rename = "dBm")]
    Dbm,
    #[serde(rename = "dB")]
    Db,
    #[serde(rename = "lux")]
    Lux,
    #[serde(rename = "degree")]
    Degree,
    #[serde(rename = "hertz")]
    Hertz,
}

impl Unit {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Celsius => "celsius",
            Self::Percent => "percent",
            Self::Second => "second",
            Self::Watt => "watt",
            Self::WattHour => "watt_hour",
            Self::Volt => "volt",
            Self::Ampere => "ampere",
            Self::Rpm => "rpm",
            Self::Liter => "liter",
            Self::Milliliter => "milliliter",
            Self::LiterPerMin => "liter_per_min",
            Self::Gram => "gram",
            Self::Kilogram => "kilogram",
            Self::Pascal => "pascal",
            Self::Kilopascal => "kilopascal",
            Self::Bar => "bar",
            Self::Ppm => "ppm",
            Self::Gpg => "gpg",
            Self::RhPercent => "rh_percent",
            Self::Dbm => "dBm",
            Self::Db => "dB",
            Self::Lux => "lux",
            Self::Degree => "degree",
            Self::Hertz => "hertz",
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Argument of a `command` point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum CommandArg {
    Void,
    Typed {
        value_type: ValueType,
        range: Option<Box<ValueRange>>,
        optional: bool,
    },
}

/// Inclusive range / enum subset / string limit / command arg for a point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ValueRange {
    Numeric {
        min: f64,
        max: f64,
    },
    Integer {
        min: i64,
        max: i64,
    },
    Enum {
        tokens: Vec<String>,
    },
    String {
        min_chars: u32,
        max_chars: u32,
    },
    List {
        max_len: u32,
        item: Option<Box<ValueRange>>,
    },
    CommandArg {
        arg: CommandArg,
    },
}

impl ValueRange {
    pub fn numeric(min: f64, max: f64) -> Self {
        Self::Numeric { min, max }
    }

    pub fn integer(min: i64, max: i64) -> Self {
        Self::Integer { min, max }
    }

    pub fn enum_tokens<I, S>(tokens: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::Enum {
            tokens: tokens.into_iter().map(Into::into).collect(),
        }
    }

    pub fn contains_f64(&self, v: f64) -> bool {
        match self {
            Self::Numeric { min, max } => v >= *min && v <= *max,
            Self::Integer { min, max } => {
                if !v.is_finite() || v.fract() != 0.0 {
                    return false;
                }
                let n = v as i64;
                n as f64 == v && n >= *min && n <= *max
            }
            _ => false,
        }
    }

    pub fn contains_i64(&self, v: i64) -> bool {
        match self {
            Self::Integer { min, max } => v >= *min && v <= *max,
            Self::Numeric { min, max } => (v as f64) >= *min && (v as f64) <= *max,
            _ => false,
        }
    }

    pub fn contains_enum(&self, token: &str) -> bool {
        match self {
            Self::Enum { tokens } => tokens.iter().any(|t| t == token),
            _ => false,
        }
    }
}

/// Tagged wire value matching [`ValueType`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Value {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    I16(i16),
    I32(i32),
    F32(f32),
    Percent(f32),
    Enum(String),
    String(String),
    TimestampMs(u64),
    DurationS(u32),
    List(Vec<Value>),
    Void,
}

impl Value {
    pub fn value_type(&self) -> Option<ValueType> {
        Some(match self {
            Self::Bool(_) => ValueType::Bool,
            Self::U8(_) => ValueType::U8,
            Self::U16(_) => ValueType::U16,
            Self::U32(_) => ValueType::U32,
            Self::I16(_) => ValueType::I16,
            Self::I32(_) => ValueType::I32,
            Self::F32(_) => ValueType::F32,
            Self::Percent(_) => ValueType::Percent,
            Self::Enum(_) => ValueType::Enum,
            Self::String(_) => ValueType::String,
            Self::TimestampMs(_) => ValueType::TimestampMs,
            Self::DurationS(_) => ValueType::DurationS,
            Self::List(items) => {
                let first = items.first()?;
                let inner = first.value_type()?;
                let item = match inner {
                    ValueType::Bool => ListItemType::Bool,
                    ValueType::U8 => ListItemType::U8,
                    ValueType::U16 => ListItemType::U16,
                    ValueType::U32 => ListItemType::U32,
                    ValueType::I16 => ListItemType::I16,
                    ValueType::I32 => ListItemType::I32,
                    ValueType::F32 => ListItemType::F32,
                    ValueType::Enum => ListItemType::Enum,
                    ValueType::String => ListItemType::String,
                    _ => return None,
                };
                ValueType::List(item)
            }
            Self::Void => ValueType::Command,
        })
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::U8(v) => Some(*v as f64),
            Self::U16(v) => Some(*v as f64),
            Self::U32(v) => Some(*v as f64),
            Self::I16(v) => Some(*v as f64),
            Self::I32(v) => Some(*v as f64),
            Self::F32(v) => Some(*v as f64),
            Self::Percent(v) => Some(*v as f64),
            Self::DurationS(v) => Some(*v as f64),
            Self::TimestampMs(v) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::U8(v) => Some(*v as i64),
            Self::U16(v) => Some(*v as i64),
            Self::U32(v) => Some(*v as i64),
            Self::I16(v) => Some(*v as i64),
            Self::I32(v) => Some(*v as i64),
            Self::DurationS(v) => Some(*v as i64),
            _ => None,
        }
    }

    pub fn matches_type(&self, expected: ValueType) -> bool {
        match (expected, self) {
            (ValueType::Bool, Self::Bool(_))
            | (ValueType::U8, Self::U8(_))
            | (ValueType::U16, Self::U16(_))
            | (ValueType::U32, Self::U32(_))
            | (ValueType::I16, Self::I16(_))
            | (ValueType::I32, Self::I32(_))
            | (ValueType::F32, Self::F32(_))
            | (ValueType::Percent, Self::Percent(_))
            | (ValueType::Enum, Self::Enum(_))
            | (ValueType::String, Self::String(_))
            | (ValueType::TimestampMs, Self::TimestampMs(_))
            | (ValueType::DurationS, Self::DurationS(_))
            | (ValueType::Command, Self::Void) => true,
            (ValueType::Command, other) => !matches!(other, Self::List(_)),
            (ValueType::List(item), Self::List(values)) => {
                values.iter().all(|v| v.matches_type(item.to_value_type()))
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_type_list_parse() {
        assert_eq!(
            "list<enum>".parse::<ValueType>().unwrap(),
            ValueType::List(ListItemType::Enum)
        );
        assert_eq!(ValueType::DurationS.as_str(), "duration_s");
    }

    #[test]
    fn value_roundtrip() {
        let values = [
            Value::Bool(true),
            Value::U16(800),
            Value::F32(40.0),
            Value::Percent(50.0),
            Value::Enum("eco".into()),
            Value::DurationS(60),
            Value::Void,
            Value::List(vec![Value::Enum("cotton".into())]),
        ];
        for v in values {
            let json = serde_json::to_string(&v).unwrap();
            let back: Value = serde_json::from_str(&json).unwrap();
            assert_eq!(back, v);
        }
    }

    #[test]
    fn unit_dbm_token() {
        let json = serde_json::to_string(&Unit::Dbm).unwrap();
        assert_eq!(json, "\"dBm\"");
        let back: Unit = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Unit::Dbm);
    }
}
