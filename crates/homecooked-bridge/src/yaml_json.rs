//! YAML → JSON value conversion for map loaders (keeps serde_json as the
//! canonical deserialize path for both YAML and JSON fixtures).

use crate::error::Error;

pub(crate) fn yaml_to_json(value: serde_yaml::Value) -> Result<serde_json::Value, Error> {
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
