//! BACnet device instance + object type/instance + property → HomeCooked point mapping.

use std::collections::HashSet;
use std::fmt;
use std::path::Path;

use homecooked_schema::{catalog_point, ApplianceClassId, QualifiedPointId, Value};
use serde::{Deserialize, Serialize};

use super::store::BacnetPropValue;
use crate::access::MapAccess;
use crate::bridge::BacnetRaw;
use crate::error::Error;
use crate::yaml_json::yaml_to_json;

/// Example `kettle` map: Binary Value + Analog Input/Value Present_Value.
pub const KETTLE_BACNET_MAP_YAML: &str = include_str!("../../examples/kettle_bacnet_map.yaml");

/// Illustrative BACnet object type (lab constants, not a stack enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BacnetObjectType {
    AnalogInput,
    AnalogValue,
    BinaryInput,
    BinaryValue,
}

impl BacnetObjectType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AnalogInput => "analog_input",
            Self::AnalogValue => "analog_value",
            Self::BinaryInput => "binary_input",
            Self::BinaryValue => "binary_value",
        }
    }
}

impl fmt::Display for BacnetObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Illustrative BACnet property identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BacnetProperty {
    #[default]
    PresentValue,
}

impl BacnetProperty {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PresentValue => "present_value",
        }
    }
}

impl fmt::Display for BacnetProperty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How the mock device stores one property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttrValueType {
    Bool,
    Int16,
    UInt16,
}

impl AttrValueType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Int16 => "int16",
            Self::UInt16 => "uint16",
        }
    }
}

impl fmt::Display for AttrValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One BACnet property mapped to a catalog point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacnetEntry {
    pub point: String,
    pub object_type: BacnetObjectType,
    pub object_instance: u32,
    #[serde(default)]
    pub property: BacnetProperty,
    pub value_type: AttrValueType,
    /// Multiplier from raw numeric property units to HomeCooked numeric units.
    /// Hundredths of a degree C: `scale: 0.01` so raw `2500` → `25.0`.
    #[serde(default = "default_scale")]
    pub scale: f64,
    #[serde(default)]
    pub access: MapAccess,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub true_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub false_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_bool: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_int: Option<i32>,
}

fn default_scale() -> f64 {
    1.0
}

impl BacnetEntry {
    pub fn uses_enum_tokens(&self) -> bool {
        self.true_token.is_some() || self.false_token.is_some()
    }

    pub fn encode_value(&self, value: &Value) -> Result<BacnetPropValue, Error> {
        match self.value_type {
            AttrValueType::Bool => self.encode_bool(value),
            AttrValueType::Int16 | AttrValueType::UInt16 => self.encode_numeric(value),
        }
    }

    pub fn decode_prop(&self, prop: BacnetPropValue) -> Result<Value, Error> {
        match (self.value_type, prop) {
            (AttrValueType::Bool, BacnetPropValue::Bool(on)) => self.decode_bool(on),
            (AttrValueType::Int16, BacnetPropValue::Int16(raw)) => {
                self.decode_numeric(f64::from(raw))
            }
            (AttrValueType::UInt16, BacnetPropValue::UInt16(raw)) => {
                self.decode_numeric(f64::from(raw))
            }
            (expected, got) => Err(Error::TypeMismatch {
                point_id: self.point.clone(),
                expected: expected.to_string(),
                detail: format!("{got:?}"),
            }),
        }
    }

    pub fn encode_raw(&self, raw: BacnetRaw) -> Result<BacnetPropValue, Error> {
        match (self.value_type, raw) {
            (AttrValueType::Bool, BacnetRaw::Bool(v)) => Ok(BacnetPropValue::Bool(v)),
            (AttrValueType::Int16, BacnetRaw::Int16(v)) => Ok(BacnetPropValue::Int16(v)),
            (AttrValueType::UInt16, BacnetRaw::UInt16(v)) => Ok(BacnetPropValue::UInt16(v)),
            (expected, got) => Err(Error::InvalidRaw {
                detail: format!("{got:?} is not valid for value_type {expected}"),
            }),
        }
    }

    pub fn seed_prop(&self) -> Result<BacnetPropValue, Error> {
        match self.value_type {
            AttrValueType::Bool => Ok(BacnetPropValue::Bool(self.initial_bool.unwrap_or(false))),
            AttrValueType::Int16 => {
                let n = self.initial_int.unwrap_or(0);
                if n < i32::from(i16::MIN) || n > i32::from(i16::MAX) {
                    return Err(Error::InvalidMap(format!(
                        "initial_int {n} outside i16 for {}",
                        self.point
                    )));
                }
                Ok(BacnetPropValue::Int16(n as i16))
            }
            AttrValueType::UInt16 => {
                let n = self.initial_int.unwrap_or(0);
                if n < 0 || n > i32::from(u16::MAX) {
                    return Err(Error::InvalidMap(format!(
                        "initial_int {n} outside u16 for {}",
                        self.point
                    )));
                }
                Ok(BacnetPropValue::UInt16(n as u16))
            }
        }
    }

    fn encode_bool(&self, value: &Value) -> Result<BacnetPropValue, Error> {
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
            return Ok(BacnetPropValue::Bool(on));
        }
        match value {
            Value::Bool(on) => Ok(BacnetPropValue::Bool(*on)),
            other => Err(Error::TypeMismatch {
                point_id: self.point.clone(),
                expected: "bool".into(),
                detail: format!("{other:?}"),
            }),
        }
    }

    fn decode_bool(&self, on: bool) -> Result<Value, Error> {
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

    fn encode_numeric(&self, value: &Value) -> Result<BacnetPropValue, Error> {
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
        match self.value_type {
            AttrValueType::Int16 => {
                if raw_f < f64::from(i16::MIN) || raw_f > f64::from(i16::MAX) {
                    return Err(Error::ScaleOverflow {
                        point_id: self.point.clone(),
                        detail: format!("{raw_f} outside i16"),
                    });
                }
                Ok(BacnetPropValue::Int16(raw_f as i16))
            }
            AttrValueType::UInt16 => {
                if raw_f < 0.0 || raw_f > f64::from(u16::MAX) {
                    return Err(Error::ScaleOverflow {
                        point_id: self.point.clone(),
                        detail: format!("{raw_f} outside u16"),
                    });
                }
                Ok(BacnetPropValue::UInt16(raw_f as u16))
            }
            AttrValueType::Bool => unreachable!("encode_numeric only for int types"),
        }
    }

    fn decode_numeric(&self, raw: f64) -> Result<Value, Error> {
        if !self.scale.is_finite() || self.scale == 0.0 {
            return Err(Error::InvalidScale {
                point_id: self.point.clone(),
            });
        }
        Ok(Value::F32((raw * self.scale) as f32))
    }
}

/// Loadable BACnet ↔ HomeCooked mapping table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BacnetMap {
    pub version: String,
    pub class_id: String,
    pub device_id: String,
    #[serde(default = "default_device_instance")]
    pub device_instance: u32,
    #[serde(default)]
    pub entries: Vec<BacnetEntry>,
}

fn default_device_instance() -> u32 {
    1
}

impl BacnetMap {
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

    pub fn kettle_example() -> Result<Self, Error> {
        Self::from_yaml_str(KETTLE_BACNET_MAP_YAML)
    }

    pub fn entry_for_point(&self, point_id: &str) -> Option<&BacnetEntry> {
        self.entries.iter().find(|e| e.point == point_id)
    }

    pub fn entry_for_prop(
        &self,
        object_type: BacnetObjectType,
        object_instance: u32,
        property: BacnetProperty,
    ) -> Option<&BacnetEntry> {
        self.entries.iter().find(|e| {
            e.object_type == object_type
                && e.object_instance == object_instance
                && e.property == property
        })
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
        let mut props = HashSet::new();
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
            let key = (entry.object_type, entry.object_instance, entry.property);
            if !props.insert(key) {
                return Err(Error::DuplicateBacnetProperty {
                    object_type: entry.object_type.to_string(),
                    object_instance: entry.object_instance,
                    property: entry.property.to_string(),
                });
            }
            if matches!(
                entry.value_type,
                AttrValueType::Int16 | AttrValueType::UInt16
            ) && (!entry.scale.is_finite() || entry.scale == 0.0)
            {
                return Err(Error::InvalidScale {
                    point_id: entry.point.clone(),
                });
            }
            match (entry.value_type, entry.uses_enum_tokens()) {
                (AttrValueType::Bool, true)
                    if entry.true_token.is_none() || entry.false_token.is_none() =>
                {
                    return Err(Error::InvalidMap(format!(
                        "point {} must set both true_token and false_token",
                        entry.point
                    )));
                }
                (AttrValueType::Int16 | AttrValueType::UInt16, true) => {
                    return Err(Error::InvalidMap(format!(
                        "enum tokens are only valid on bool properties ({})",
                        entry.point
                    )));
                }
                _ => {}
            }
            match entry.value_type {
                AttrValueType::Bool if entry.initial_int.is_some() => {
                    return Err(Error::InvalidMap(format!(
                        "point {} is bool; use initial_bool not initial_int",
                        entry.point
                    )));
                }
                AttrValueType::Int16 | AttrValueType::UInt16 if entry.initial_bool.is_some() => {
                    return Err(Error::InvalidMap(format!(
                        "point {} is numeric; use initial_int not initial_bool",
                        entry.point
                    )));
                }
                _ => {}
            }
            let _ = entry.seed_prop()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_kettle_example() {
        let map = BacnetMap::kettle_example().unwrap();
        assert_eq!(map.version, "0.1.0");
        assert_eq!(map.class_id, "kettle");
        assert_eq!(map.device_id, "kettle-lab-1");
        assert_eq!(map.device_instance, 1);
        assert_eq!(map.entries.len(), 3);
        let power = map.entry_for_point("trait.power.power_state").unwrap();
        assert_eq!(power.object_type, BacnetObjectType::BinaryValue);
        assert_eq!(power.object_instance, 1);
        assert_eq!(power.property, BacnetProperty::PresentValue);
        assert_eq!(power.value_type, AttrValueType::Bool);
        assert_eq!(power.true_token.as_deref(), Some("on"));
        let current = map
            .entry_for_prop(
                BacnetObjectType::AnalogInput,
                1,
                BacnetProperty::PresentValue,
            )
            .unwrap();
        assert_eq!(current.point, "trait.temperature.current_c");
        assert!((current.scale - 0.01).abs() < f64::EPSILON);
        assert_eq!(current.initial_int, Some(2500));
        let set = map.entry_for_point("trait.temperature.setpoint_c").unwrap();
        assert_eq!(set.object_type, BacnetObjectType::AnalogValue);
        assert_eq!(set.access, MapAccess::Rw);
    }

    #[test]
    fn hundredths_scale_encode_decode() {
        let map = BacnetMap::kettle_example().unwrap();
        let set = map.entry_for_point("trait.temperature.setpoint_c").unwrap();
        let prop = set.encode_value(&Value::F32(80.0)).unwrap();
        assert_eq!(prop, BacnetPropValue::Int16(8000));
        assert_eq!(set.decode_prop(prop).unwrap(), Value::F32(80.0));
    }

    #[test]
    fn rejects_unknown_and_duplicate_props() {
        let bad = r#"
version: "0.1.0"
class_id: kettle
device_id: d1
entries:
  - point: trait.power.on
    object_type: binary_value
    object_instance: 1
    property: present_value
    value_type: bool
"#;
        let err = BacnetMap::from_yaml_str(bad).unwrap_err();
        assert!(matches!(err, Error::UnknownCatalogPoint(_)));

        let dup = r#"
version: "0.1.0"
class_id: kettle
device_id: d1
entries:
  - point: trait.temperature.setpoint_c
    object_type: analog_value
    object_instance: 1
    property: present_value
    value_type: int16
    scale: 0.01
  - point: trait.temperature.current_c
    object_type: analog_value
    object_instance: 1
    property: present_value
    value_type: int16
    scale: 0.01
"#;
        let err = BacnetMap::from_yaml_str(dup).unwrap_err();
        assert!(matches!(
            err,
            Error::DuplicateBacnetProperty {
                object_instance: 1,
                ..
            }
        ));
    }
}
