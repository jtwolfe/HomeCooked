//! Chassis I/O map types, loaders, and validation.

use std::collections::HashSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::kind::{channel_prefix, IoKind};

/// Washer fragment from `docs/standard/examples/washer-dryer-io.md` §5.
pub const WASHER_FRAGMENT_YAML: &str = include_str!("../examples/washer-fragment.yaml");

/// Per-chassis HAL bindings (`chassis.io_map.yaml`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IoMap {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class_id: Option<String>,
    #[serde(default)]
    pub bindings: Vec<Binding>,
}

/// One logical channel bound to hardware and optionally a HomeCooked point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    pub channel: String,
    /// Explicit kind. When omitted, inferred from the channel prefix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sink: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub point: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encode: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gated_by: Option<serde_json::Value>,
}

impl Binding {
    /// Kind from the explicit field, or inferred from the channel prefix.
    ///
    /// An explicit `kind` may authorize a channel without a known prefix
    /// (`heater_enable` + `kind: aout`). When both prefix and kind are known
    /// kinds, they must agree.
    pub fn resolved_kind(&self) -> Result<IoKind, Error> {
        let prefix = channel_prefix(&self.channel);
        let prefix_kind = IoKind::from_token(prefix);

        if let Some(kind_tok) = &self.kind {
            let kind = IoKind::from_token(kind_tok).ok_or_else(|| Error::UnknownKind {
                channel: self.channel.clone(),
                kind: kind_tok.clone(),
            })?;
            if let Some(pk) = prefix_kind {
                if pk != kind {
                    return Err(Error::KindMismatch {
                        channel: self.channel.clone(),
                        kind: kind_tok.clone(),
                        prefix: prefix.to_string(),
                    });
                }
            }
            return Ok(kind);
        }

        prefix_kind.ok_or_else(|| Error::UnknownPrefix {
            channel: self.channel.clone(),
            prefix: prefix.to_string(),
        })
    }
}

impl IoMap {
    pub fn from_yaml_str(s: &str) -> Result<Self, Error> {
        let yaml: serde_yaml::Value = serde_yaml::from_str(s)?;
        let json = yaml_to_json(yaml)?;
        let map: IoMap = serde_json::from_value(json)?;
        map.validate()?;
        Ok(map)
    }

    pub fn from_json_str(s: &str) -> Result<Self, Error> {
        let map: IoMap = serde_json::from_str(s)?;
        map.validate()?;
        Ok(map)
    }

    /// Load YAML or JSON from a file. `.json` uses the JSON parser; anything
    /// else is treated as YAML.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)?;
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => Self::from_json_str(&text),
            _ => Self::from_yaml_str(&text),
        }
    }

    /// Reject duplicate `channel` ids and unknown / mismatched kinds.
    ///
    /// Allowed kinds are `din`, `dout`, `ain`, `aout`, `relay`, and `motor`.
    /// Kind may be inferred from the channel prefix or set explicitly; see
    /// [`Binding::resolved_kind`].
    pub fn validate(&self) -> Result<(), Error> {
        let mut seen = HashSet::new();
        for binding in &self.bindings {
            if !seen.insert(binding.channel.as_str()) {
                return Err(Error::DuplicateChannel(binding.channel.clone()));
            }
            binding.resolved_kind()?;
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

    fn washer() -> IoMap {
        IoMap::from_yaml_str(WASHER_FRAGMENT_YAML).unwrap()
    }

    #[test]
    fn load_washer_fragment() {
        let map = washer();
        assert_eq!(map.version, "0.1.0");
        assert_eq!(map.class_id.as_deref(), Some("washer"));
        assert_eq!(map.bindings.len(), 10);
        assert_eq!(map.bindings[0].channel, "din.door_closed");
        assert_eq!(
            map.bindings[0].point.as_deref(),
            Some("trait.door_lid.door_state")
        );
        assert_eq!(map.bindings[0].resolved_kind().unwrap(), IoKind::Din);
        assert_eq!(map.bindings.last().unwrap().channel, "motor.speed_rpm_cmd");
        assert_eq!(
            map.bindings.last().unwrap().resolved_kind().unwrap(),
            IoKind::Motor
        );

        let heater = map
            .bindings
            .iter()
            .find(|b| b.channel == "aout.heater_enable")
            .unwrap();
        assert_eq!(heater.resolved_kind().unwrap(), IoKind::Aout);
        assert_eq!(heater.gated_by, Some(serde_json::json!(["il.heater"])));
        assert_eq!(heater.sink.as_ref().unwrap()["board"], "hv_actuator");
        assert_eq!(heater.sink.as_ref().unwrap()["circuit"], "heater");

        let door = &map.bindings[0];
        assert_eq!(door.source.as_ref().unwrap()["pin"], "di_0");
        assert_eq!(door.encode.as_ref().unwrap()["true"], "closed");
        assert_eq!(door.encode.as_ref().unwrap()["false"], "open");

        let level = map
            .bindings
            .iter()
            .find(|b| b.channel == "ain.water_level_pa")
            .unwrap();
        assert_eq!(level.scale.as_ref().unwrap()["full"], 4000);
        assert_eq!(level.scale.as_ref().unwrap()["to"], "percent");
    }

    #[test]
    fn from_path_loads_example() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/washer-fragment.yaml");
        let map = IoMap::from_path(&path).unwrap();
        assert_eq!(map.bindings.len(), 10);
    }

    #[test]
    fn json_roundtrip() {
        let map = washer();
        let json = serde_json::to_string_pretty(&map).unwrap();
        let back = IoMap::from_json_str(&json).unwrap();
        assert_eq!(back, map);
        assert!(json.contains("\"din.door_closed\""));
        assert!(json.contains("\"motor.speed_rpm_cmd\""));
    }

    #[test]
    fn duplicate_channel_fails() {
        let err = IoMap::from_yaml_str(
            r#"
version: "0.1.0"
bindings:
  - channel: din.door_closed
  - channel: din.door_closed
"#,
        )
        .unwrap_err();
        assert_eq!(err, Error::DuplicateChannel("din.door_closed".into()));
    }

    #[test]
    fn unknown_prefix_fails() {
        let err = IoMap::from_yaml_str(
            r#"
version: "0.1.0"
bindings:
  - channel: pwm.heater
"#,
        )
        .unwrap_err();
        assert_eq!(
            err,
            Error::UnknownPrefix {
                channel: "pwm.heater".into(),
                prefix: "pwm".into(),
            }
        );
    }

    #[test]
    fn unknown_kind_fails() {
        let err = IoMap {
            version: "0.1.0".into(),
            class_id: None,
            bindings: vec![Binding {
                channel: "din.door_closed".into(),
                kind: Some("pwm".into()),
                source: None,
                sink: None,
                point: None,
                encode: None,
                scale: None,
                gated_by: None,
            }],
        }
        .validate()
        .unwrap_err();
        assert_eq!(
            err,
            Error::UnknownKind {
                channel: "din.door_closed".into(),
                kind: "pwm".into(),
            }
        );
    }

    #[test]
    fn explicit_kind_and_relay_prefix() {
        let map = IoMap::from_yaml_str(
            r#"
version: "0.1.0"
bindings:
  - channel: din.door_closed
    kind: din
  - channel: relay.spare
    sink: { board: hv_actuator, circuit: spare }
"#,
        )
        .unwrap();
        assert_eq!(map.bindings[0].resolved_kind().unwrap(), IoKind::Din);
        assert_eq!(map.bindings[1].resolved_kind().unwrap(), IoKind::Relay);
    }

    #[test]
    fn explicit_kind_without_known_prefix() {
        let map = IoMap::from_yaml_str(
            r#"
version: "0.1.0"
bindings:
  - channel: heater_enable
    kind: aout
"#,
        )
        .unwrap();
        assert_eq!(map.bindings[0].resolved_kind().unwrap(), IoKind::Aout);
    }

    #[test]
    fn kind_prefix_mismatch_fails() {
        let err = IoMap {
            version: "0.1.0".into(),
            class_id: None,
            bindings: vec![Binding {
                channel: "motor.speed_rpm_cmd".into(),
                kind: Some("din".into()),
                source: None,
                sink: None,
                point: None,
                encode: None,
                scale: None,
                gated_by: None,
            }],
        }
        .validate()
        .unwrap_err();
        assert_eq!(
            err,
            Error::KindMismatch {
                channel: "motor.speed_rpm_cmd".into(),
                kind: "din".into(),
                prefix: "motor".into(),
            }
        );
    }
}
