//! Modbus register/coil → HomeCooked point mapping table.

use std::collections::HashSet;
use std::fmt;
use std::path::Path;

use homecooked_schema::{catalog_point, ApplianceClassId, QualifiedPointId, Value};
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Example `water_heater` map: setpoint, current temp, power state.
pub const WATER_HEATER_MAP_YAML: &str = include_str!("../../examples/water_heater_map.yaml");

/// Modbus object table kind (function-code family, not a wire stack).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegisterKind {
    Holding,
    Input,
    Coil,
    Discrete,
}

impl RegisterKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Holding => "holding",
            Self::Input => "input",
            Self::Coil => "coil",
            Self::Discrete => "discrete",
        }
    }

    pub const fn is_bit(self) -> bool {
        matches!(self, Self::Coil | Self::Discrete)
    }

    pub const fn is_register(self) -> bool {
        matches!(self, Self::Holding | Self::Input)
    }
}

impl fmt::Display for RegisterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

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

/// One register or coil mapped to a catalog point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModbusEntry {
    pub point: String,
    pub kind: RegisterKind,
    pub address: u16,
    /// Multiplier from raw register units to HomeCooked numeric units.
    /// Tenths of a degree C: `scale: 0.1` so raw `550` → `55.0`.
    #[serde(default = "default_scale")]
    pub scale: f64,
    /// Interpret the 16-bit register as `i16` before scaling.
    #[serde(default)]
    pub signed: bool,
    #[serde(default)]
    pub access: MapAccess,
    /// Coil/discrete `true` → this enum token (e.g. `on`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub true_token: Option<String>,
    /// Coil/discrete `false` → this enum token (e.g. `off`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub false_token: Option<String>,
    /// Seed the in-memory slave. Coils: `0` = false, nonzero = true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_raw: Option<u16>,
}

fn default_scale() -> f64 {
    1.0
}

impl ModbusEntry {
    pub fn uses_enum_tokens(&self) -> bool {
        self.true_token.is_some() || self.false_token.is_some()
    }

    pub fn encode_value(&self, value: &Value) -> Result<ForeignBits, Error> {
        if self.kind.is_bit() {
            return self.encode_bit(value);
        }
        self.encode_register(value)
    }

    pub fn decode_bits(&self, bits: ForeignBits) -> Result<Value, Error> {
        match bits {
            ForeignBits::Register(raw) if self.kind.is_register() => self.decode_register(raw),
            ForeignBits::Coil(on) if self.kind.is_bit() => self.decode_bit(on),
            ForeignBits::Register(_) => Err(Error::TypeMismatch {
                point_id: self.point.clone(),
                expected: "coil".into(),
                detail: "got a 16-bit register".into(),
            }),
            ForeignBits::Coil(_) => Err(Error::TypeMismatch {
                point_id: self.point.clone(),
                expected: "register".into(),
                detail: "got a coil bit".into(),
            }),
        }
    }

    fn encode_bit(&self, value: &Value) -> Result<ForeignBits, Error> {
        if let (Some(on_tok), Some(off_tok)) = (&self.true_token, &self.false_token) {
            let token = match value {
                Value::Enum(s) | Value::String(s) => s.as_str(),
                other => {
                    return Err(Error::TypeMismatch {
                        point_id: self.point.clone(),
                        expected: "enum".into(),
                        detail: format!("{other:?}"),
                    })
                }
            };
            let on = if token == on_tok {
                true
            } else if token == off_tok {
                false
            } else {
                return Err(Error::TypeMismatch {
                    point_id: self.point.clone(),
                    expected: format!("enum {on_tok}|{off_tok}"),
                    detail: token.into(),
                });
            };
            return Ok(ForeignBits::Coil(on));
        }
        match value {
            Value::Bool(on) => Ok(ForeignBits::Coil(*on)),
            other => Err(Error::TypeMismatch {
                point_id: self.point.clone(),
                expected: "bool".into(),
                detail: format!("{other:?}"),
            }),
        }
    }

    fn decode_bit(&self, on: bool) -> Result<Value, Error> {
        match (&self.true_token, &self.false_token) {
            (Some(on_tok), Some(off_tok)) => Ok(Value::Enum(if on {
                on_tok.clone()
            } else {
                off_tok.clone()
            })),
            (None, None) => Ok(Value::Bool(on)),
            _ => Err(Error::InvalidMap(format!(
                "point {} must set both true_token and false_token",
                self.point
            ))),
        }
    }

    fn encode_register(&self, value: &Value) -> Result<ForeignBits, Error> {
        let hc = value.as_f64().ok_or_else(|| Error::TypeMismatch {
            point_id: self.point.clone(),
            expected: "numeric".into(),
            detail: format!("{value:?}"),
        })?;
        if !self.scale.is_finite() || self.scale == 0.0 {
            return Err(Error::InvalidScale {
                point_id: self.point.clone(),
            });
        }
        let raw_f = (hc / self.scale).round();
        let raw = if self.signed {
            if raw_f < i16::MIN as f64 || raw_f > i16::MAX as f64 {
                return Err(Error::ScaleOverflow {
                    point_id: self.point.clone(),
                    detail: format!("{raw_f} outside i16"),
                });
            }
            raw_f as i16 as u16
        } else {
            if raw_f < 0.0 || raw_f > u16::MAX as f64 {
                return Err(Error::ScaleOverflow {
                    point_id: self.point.clone(),
                    detail: format!("{raw_f} outside u16"),
                });
            }
            raw_f as u16
        };
        Ok(ForeignBits::Register(raw))
    }

    fn decode_register(&self, raw: u16) -> Result<Value, Error> {
        if !self.scale.is_finite() || self.scale == 0.0 {
            return Err(Error::InvalidScale {
                point_id: self.point.clone(),
            });
        }
        let n = if self.signed {
            raw as i16 as f64
        } else {
            f64::from(raw)
        };
        Ok(Value::F32((n * self.scale) as f32))
    }
}

/// Bits stored in the mock slave for one mapping entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForeignBits {
    Register(u16),
    Coil(bool),
}

/// Loadable Modbus ↔ HomeCooked mapping table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModbusMap {
    pub version: String,
    pub class_id: String,
    pub device_id: String,
    #[serde(default = "default_slave_id")]
    pub slave_id: u8,
    #[serde(default)]
    pub entries: Vec<ModbusEntry>,
}

fn default_slave_id() -> u8 {
    1
}

impl ModbusMap {
    pub fn from_yaml_str(s: &str) -> Result<Self, Error> {
        let yaml: serde_yaml::Value = serde_yaml::from_str(s)?;
        let json = yaml_to_json(yaml)?;
        let map: Self = serde_json::from_value(json)?;
        map.validate()?;
        Ok(map)
    }

    pub fn from_json_str(s: &str) -> Result<Self, Error> {
        let map: Self = serde_json::from_str(s)?;
        map.validate()?;
        Ok(map)
    }

    /// Load YAML or JSON. `.json` uses the JSON parser; anything else is YAML.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)?;
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => Self::from_json_str(&text),
            _ => Self::from_yaml_str(&text),
        }
    }

    pub fn water_heater_example() -> Result<Self, Error> {
        Self::from_yaml_str(WATER_HEATER_MAP_YAML)
    }

    pub fn entry_for_point(&self, point_id: &str) -> Option<&ModbusEntry> {
        self.entries.iter().find(|e| e.point == point_id)
    }

    pub fn entry_for_address(&self, kind: RegisterKind, address: u16) -> Option<&ModbusEntry> {
        self.entries
            .iter()
            .find(|e| e.kind == kind && e.address == address)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.device_id.is_empty() {
            return Err(Error::EmptyId("device_id"));
        }
        if self.class_id.is_empty() {
            return Err(Error::EmptyId("class_id"));
        }
        if ApplianceClassId::from_str_id(&self.class_id).is_none() {
            return Err(Error::InvalidMap(format!(
                "unknown class_id {}",
                self.class_id
            )));
        }

        let mut points = HashSet::new();
        let mut addresses = HashSet::new();
        for entry in &self.entries {
            if entry.point.is_empty() {
                return Err(Error::EmptyId("point"));
            }
            let qid = QualifiedPointId::parse(&entry.point)?;
            if catalog_point(&qid.namespace, &qid.id).is_none() {
                return Err(Error::UnknownCatalogPoint(entry.point.clone()));
            }
            if !points.insert(entry.point.as_str()) {
                return Err(Error::DuplicatePoint(entry.point.clone()));
            }
            if !addresses.insert((entry.kind, entry.address)) {
                return Err(Error::DuplicateAddress {
                    kind: entry.kind,
                    address: entry.address,
                });
            }
            if !entry.scale.is_finite() || entry.scale == 0.0 {
                return Err(Error::InvalidScale {
                    point_id: entry.point.clone(),
                });
            }
            match (entry.kind.is_bit(), entry.uses_enum_tokens()) {
                (false, true) => {
                    return Err(Error::InvalidMap(format!(
                        "enum tokens are only valid on coil/discrete ({})",
                        entry.point
                    )));
                }
                (true, true) if entry.true_token.is_none() || entry.false_token.is_none() => {
                    return Err(Error::InvalidMap(format!(
                        "point {} must set both true_token and false_token",
                        entry.point
                    )));
                }
                _ => {}
            }
        }
        Ok(())
    }
}

fn yaml_to_json(value: serde_yaml::Value) -> Result<serde_json::Value, Error> {
    match value {
        serde_yaml::Value::Null => Ok(serde_json::Value::Null),
        serde_yaml::Value::Bool(b) => Ok(serde_json::Value::Bool(b)),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(serde_json::Value::Number(i.into()))
            } else if let Some(u) = n.as_u64() {
                Ok(serde_json::Value::Number(u.into()))
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .ok_or_else(|| Error::Yaml("non-finite yaml number".into()))
            } else {
                Err(Error::Yaml("unrepresentable yaml number".into()))
            }
        }
        serde_yaml::Value::String(s) => Ok(serde_json::Value::String(s)),
        serde_yaml::Value::Sequence(seq) => {
            let items = seq
                .into_iter()
                .map(yaml_to_json)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(serde_json::Value::Array(items))
        }
        serde_yaml::Value::Mapping(map) => {
            let mut obj = serde_json::Map::new();
            for (key, val) in map {
                obj.insert(yaml_key_to_string(key)?, yaml_to_json(val)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(tagged.value),
    }
}

fn yaml_key_to_string(key: serde_yaml::Value) -> Result<String, Error> {
    match key {
        serde_yaml::Value::String(s) => Ok(s),
        serde_yaml::Value::Bool(b) => Ok(b.to_string()),
        serde_yaml::Value::Number(n) => Ok(n.to_string()),
        serde_yaml::Value::Null => Ok("null".into()),
        _ => Err(Error::Yaml("unsupported yaml mapping key".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_water_heater_example() {
        let map = ModbusMap::water_heater_example().unwrap();
        assert_eq!(map.version, "0.1.0");
        assert_eq!(map.class_id, "water_heater");
        assert_eq!(map.device_id, "water-heater-plant");
        assert_eq!(map.slave_id, 1);
        assert_eq!(map.entries.len(), 3);
        let set = map.entry_for_point("trait.temperature.setpoint_c").unwrap();
        assert_eq!(set.kind, RegisterKind::Holding);
        assert_eq!(set.address, 0);
        assert!((set.scale - 0.1).abs() < f64::EPSILON);
        assert!(set.signed);
        assert_eq!(set.initial_raw, Some(550));
        let power = map.entry_for_address(RegisterKind::Coil, 0).unwrap();
        assert_eq!(power.true_token.as_deref(), Some("on"));
        assert_eq!(power.false_token.as_deref(), Some("off"));
    }

    #[test]
    fn json_roundtrip_and_path() {
        let map = ModbusMap::water_heater_example().unwrap();
        let json = serde_json::to_string(&map).unwrap();
        let back = ModbusMap::from_json_str(&json).unwrap();
        assert_eq!(back.entries.len(), 3);

        let path = std::env::temp_dir().join("homecooked-water-heater-map.json");
        std::fs::write(&path, &json).unwrap();
        let from_path = ModbusMap::from_path(&path).unwrap();
        assert_eq!(from_path.device_id, map.device_id);
    }

    #[test]
    fn tenths_scale_encode_decode() {
        let map = ModbusMap::water_heater_example().unwrap();
        let set = map.entry_for_point("trait.temperature.setpoint_c").unwrap();
        let bits = set.encode_value(&Value::F32(55.0)).unwrap();
        assert_eq!(bits, ForeignBits::Register(550));
        assert_eq!(set.decode_bits(bits).unwrap(), Value::F32(55.0));

        let bits = set.encode_value(&Value::F32(-4.5)).unwrap();
        assert_eq!(bits, ForeignBits::Register((-45_i16) as u16));
        assert_eq!(set.decode_bits(bits).unwrap(), Value::F32(-4.5));
    }

    #[test]
    fn rejects_unknown_and_duplicate_points() {
        let bad = r#"
version: "0.1.0"
class_id: water_heater
device_id: d1
entries:
  - point: trait.power.on
    kind: coil
    address: 0
"#;
        let err = ModbusMap::from_yaml_str(bad).unwrap_err();
        assert!(matches!(err, Error::UnknownCatalogPoint(_)));

        let dup = r#"
version: "0.1.0"
class_id: water_heater
device_id: d1
entries:
  - point: trait.temperature.setpoint_c
    kind: holding
    address: 0
  - point: trait.temperature.current_c
    kind: holding
    address: 0
"#;
        let err = ModbusMap::from_yaml_str(dup).unwrap_err();
        assert!(matches!(
            err,
            Error::DuplicateAddress {
                kind: RegisterKind::Holding,
                address: 0
            }
        ));
    }
}
