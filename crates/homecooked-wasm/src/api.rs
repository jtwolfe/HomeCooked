//! JSON-string API over [`homecooked_sim::Simulator`], shared by wasm-bindgen
//! wrappers and native tests.

use homecooked_core::{CoreError, DeviceId};
use homecooked_schema::{
    AccessMode, ApplianceClassId, DeviceIdentity, ErrorCode, Unit, Value, ValueRange, ValueType,
    STATIC_CLASS_IDS,
};
use homecooked_sim::Simulator;
use serde::{Deserialize, Serialize};

/// In-memory simulator world exposed to JS as JSON strings.
#[derive(Debug, Default)]
pub struct WasmApi {
    sim: Simulator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub point_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
}

impl ApiError {
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::InvalidRequest,
            message: message.into(),
            point_id: None,
            expected: None,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::Internal,
            message: message.into(),
            point_id: None,
            expected: None,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"code":"internal","message":"failed to serialize error"}"#.to_string()
        })
    }
}

impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        Self {
            code: err.code,
            message: err.message,
            point_id: err.point_id,
            expected: err.expected,
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_json())
    }
}

impl std::error::Error for ApiError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub class_id: ApplianceClassId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PointView {
    pub id: String,
    #[serde(rename = "type")]
    pub value_type: ValueType,
    pub access: AccessMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<Unit>,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<ValueRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zones: Option<Vec<String>>,
    pub writable: bool,
    pub readable: bool,
    pub command: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DescribeOut {
    pub identity: DeviceIdentity,
    pub capability: homecooked_schema::CapabilityModel,
    pub points: Vec<PointView>,
}

impl WasmApi {
    pub fn new() -> Self {
        Self {
            sim: Simulator::new(),
        }
    }

    pub fn list_appliance_classes() -> String {
        let classes: Vec<ClassInfo> = STATIC_CLASS_IDS
            .iter()
            .map(|id| ClassInfo {
                id: id.as_str().to_string(),
                label: class_label(*id),
            })
            .collect();
        serde_json::to_string(&classes).expect("ClassInfo serializes")
    }

    pub fn create_device(&mut self, class_id: &str) -> Result<String, ApiError> {
        let class = ApplianceClassId::from_str_id(class_id).ok_or_else(|| {
            ApiError::invalid_request(format!("unknown appliance class {class_id:?}"))
        })?;
        let id = self.sim.spawn(class)?;
        Ok(id.as_str().to_string())
    }

    pub fn list_devices(&self) -> String {
        let devices: Vec<DeviceInfo> = self
            .sim
            .hub()
            .registry
            .list()
            .into_iter()
            .map(|dev| DeviceInfo {
                device_id: dev.identity.device_id.clone(),
                class_id: dev.identity.class_id,
                display_name: dev.identity.display_name.clone(),
            })
            .collect();
        serde_json::to_string(&devices).expect("DeviceInfo serializes")
    }

    pub fn describe(&self, device_id: &str) -> Result<String, ApiError> {
        let id = DeviceId::new(device_id);
        let dev = self
            .sim
            .hub()
            .registry
            .get(&id)
            .ok_or_else(|| CoreError::unknown_device(device_id))?;
        let points: Vec<PointView> = dev
            .capability
            .iter_points()
            .map(|p| PointView {
                id: p.id.clone(),
                value_type: p.value_type,
                access: p.access,
                unit: p.unit,
                required: p.required,
                range: p.range.clone(),
                zones: p.zones.clone(),
                writable: p.access.is_writable(),
                readable: p.access.is_readable(),
                command: p.value_type == ValueType::Command,
            })
            .collect();
        let out = DescribeOut {
            identity: dev.identity.clone(),
            capability: dev.capability.clone(),
            points,
        };
        serde_json::to_string(&out).map_err(|e| ApiError::internal(e.to_string()))
    }

    pub fn read(&self, device_id: &str, points_json: Option<&str>) -> Result<String, ApiError> {
        let id = DeviceId::new(device_id);
        if self.sim.hub().registry.get(&id).is_none() {
            return Err(CoreError::unknown_device(device_id).into());
        }
        let points = parse_points_arg(points_json)?;
        if points.is_empty() {
            return self.get_state(device_id);
        }
        let refs: Vec<&str> = points.iter().map(String::as_str).collect();
        let rows = self.sim.read(&id, &refs)?;
        let values: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|(pid, value)| {
                serde_json::json!({
                    "id": pid,
                    "value": value,
                })
            })
            .collect();
        serde_json::to_string(&serde_json::json!({ "values": values }))
            .map_err(|e| ApiError::internal(e.to_string()))
    }

    pub fn write(
        &mut self,
        device_id: &str,
        point: &str,
        value_json: &str,
    ) -> Result<String, ApiError> {
        let id = DeviceId::new(device_id);
        let expected = self
            .sim
            .hub()
            .registry
            .get(&id)
            .ok_or_else(|| CoreError::unknown_device(device_id))?
            .capability
            .point(point)
            .map(|p| p.value_type);
        let value = parse_write_value(value_json, expected)?;
        let resp = self.sim.write(&id, point, value)?;
        serde_json::to_string(&serde_json::json!({
            "ok": true,
            "accepted": resp.accepted,
        }))
        .map_err(|e| ApiError::internal(e.to_string()))
    }

    pub fn tick(&mut self, device_id: &str, dt_ms: u32) -> Result<String, ApiError> {
        let id = DeviceId::new(device_id);
        self.sim.tick(&id, u64::from(dt_ms))?;
        self.get_state(device_id)
    }

    pub fn get_state(&self, device_id: &str) -> Result<String, ApiError> {
        let id = DeviceId::new(device_id);
        let state = self
            .sim
            .state(&id)
            .ok_or_else(|| CoreError::unknown_device(device_id))?;
        let mut map = serde_json::Map::new();
        let mut rows: Vec<_> = state.iter().collect();
        rows.sort_by(|a, b| a.0.cmp(b.0));
        for (k, v) in rows {
            let json = serde_json::to_value(v).map_err(|e| ApiError::internal(e.to_string()))?;
            map.insert(k.clone(), json);
        }
        serde_json::to_string(&serde_json::Value::Object(map))
            .map_err(|e| ApiError::internal(e.to_string()))
    }
}

fn class_label(id: ApplianceClassId) -> String {
    id.as_str()
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_points_arg(points_json: Option<&str>) -> Result<Vec<String>, ApiError> {
    match points_json {
        None | Some("") | Some("null") => Ok(Vec::new()),
        Some(raw) => serde_json::from_str(raw)
            .map_err(|e| ApiError::invalid_request(format!("points must be a JSON array: {e}"))),
    }
}

fn parse_write_value(raw: &str, expected: Option<ValueType>) -> Result<Value, ApiError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ApiError::invalid_request("write value must not be empty"));
    }
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        return Ok(v);
    }
    let json: serde_json::Value = serde_json::from_str(trimmed).map_err(|e| {
        ApiError::invalid_request(format!(
            "write value is not valid JSON or a tagged Value: {e}"
        ))
    })?;
    match expected {
        Some(ty) => json_to_value(&json, ty),
        None => guess_value(&json),
    }
}

fn json_to_value(v: &serde_json::Value, ty: ValueType) -> Result<Value, ApiError> {
    let mismatch = || {
        ApiError::invalid_request(format!(
            "could not coerce JSON {v} to catalog type {}",
            ty.as_str()
        ))
    };
    match ty {
        ValueType::Bool => v.as_bool().map(Value::Bool).ok_or_else(mismatch),
        ValueType::U8 => json_u64(v)
            .and_then(|n| u8::try_from(n).ok())
            .map(Value::U8)
            .ok_or_else(mismatch),
        ValueType::U16 => json_u64(v)
            .and_then(|n| u16::try_from(n).ok())
            .map(Value::U16)
            .ok_or_else(mismatch),
        ValueType::U32 => json_u64(v)
            .and_then(|n| u32::try_from(n).ok())
            .map(Value::U32)
            .ok_or_else(mismatch),
        ValueType::I16 => json_i64(v)
            .and_then(|n| i16::try_from(n).ok())
            .map(Value::I16)
            .ok_or_else(mismatch),
        ValueType::I32 => json_i64(v)
            .and_then(|n| i32::try_from(n).ok())
            .map(Value::I32)
            .ok_or_else(mismatch),
        ValueType::F32 => json_f64(v)
            .map(|n| Value::F32(n as f32))
            .ok_or_else(mismatch),
        ValueType::Percent => json_f64(v)
            .map(|n| Value::Percent(n as f32))
            .ok_or_else(mismatch),
        ValueType::Enum => v
            .as_str()
            .map(|s| Value::Enum(s.to_string()))
            .ok_or_else(mismatch),
        ValueType::String => v
            .as_str()
            .map(|s| Value::String(s.to_string()))
            .ok_or_else(mismatch),
        ValueType::TimestampMs => json_u64(v).map(Value::TimestampMs).ok_or_else(mismatch),
        ValueType::DurationS => json_u64(v)
            .and_then(|n| u32::try_from(n).ok())
            .map(Value::DurationS)
            .ok_or_else(mismatch),
        ValueType::Command => {
            if v.is_null() || v.as_object().is_some_and(|o| o.is_empty()) {
                Ok(Value::Void)
            } else {
                serde_json::from_value(v.clone()).map_err(|_| mismatch())
            }
        }
        ValueType::List(item) => {
            let arr = v.as_array().ok_or_else(mismatch)?;
            let mut out = Vec::with_capacity(arr.len());
            for el in arr {
                out.push(json_to_value(el, item.to_value_type())?);
            }
            Ok(Value::List(out))
        }
    }
}

fn guess_value(v: &serde_json::Value) -> Result<Value, ApiError> {
    match v {
        serde_json::Value::Null => Ok(Value::Void),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                if u <= u32::MAX as u64 {
                    return Ok(Value::U32(u as u32));
                }
                return Ok(Value::TimestampMs(u));
            }
            if let Some(i) = n.as_i64() {
                return i32::try_from(i)
                    .map(Value::I32)
                    .map_err(|_| ApiError::invalid_request("integer out of i32 range"));
            }
            json_f64(v)
                .map(|n| Value::F32(n as f32))
                .ok_or_else(|| ApiError::invalid_request("invalid number"))
        }
        serde_json::Value::String(s) => Ok(Value::Enum(s.clone())),
        serde_json::Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for el in items {
                out.push(guess_value(el)?);
            }
            Ok(Value::List(out))
        }
        serde_json::Value::Object(_) => Err(ApiError::invalid_request(
            "object write values must be tagged {\"type\":...,\"value\":...}",
        )),
    }
}

fn json_u64(v: &serde_json::Value) -> Option<u64> {
    v.as_u64().or_else(|| {
        v.as_f64().and_then(|f| {
            if f.is_finite() && f >= 0.0 && f.fract() == 0.0 && f <= u64::MAX as f64 {
                Some(f as u64)
            } else {
                None
            }
        })
    })
}

fn json_i64(v: &serde_json::Value) -> Option<i64> {
    v.as_i64().or_else(|| {
        v.as_f64().and_then(|f| {
            if f.is_finite() && f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                Some(f as i64)
            } else {
                None
            }
        })
    })
}

fn json_f64(v: &serde_json::Value) -> Option<f64> {
    v.as_f64()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f32_of(state: &serde_json::Value, point: &str) -> f32 {
        state[point]["value"].as_f64().unwrap() as f32
    }

    #[test]
    fn list_classes_is_the_static_catalog_tables() {
        let classes: Vec<ClassInfo> =
            serde_json::from_str(&WasmApi::list_appliance_classes()).unwrap();
        assert_eq!(classes.len(), STATIC_CLASS_IDS.len());
        assert!(classes
            .iter()
            .any(|c| c.id == "kettle" && c.label == "Kettle"));
        assert!(classes
            .iter()
            .any(|c| c.id == "induction_hob" && c.label == "Induction Hob"));
        assert!(classes
            .iter()
            .any(|c| c.id == "steam_oven" && c.label == "Steam Oven"));
        assert!(classes
            .iter()
            .any(|c| c.id == "coffee_machine" && c.label == "Coffee Machine"));
    }

    #[test]
    fn create_describe_read_write_tick_kettle() {
        let mut api = WasmApi::new();
        let id = api.create_device("kettle").unwrap();
        assert!(id.starts_with("sim-kettle-"));

        let devices: Vec<DeviceInfo> = serde_json::from_str(&api.list_devices()).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_id, id);
        assert_eq!(devices[0].class_id, ApplianceClassId::Kettle);

        let desc: serde_json::Value = serde_json::from_str(&api.describe(&id).unwrap()).unwrap();
        assert_eq!(desc["identity"]["class_id"], "kettle");
        assert!(desc["points"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| { p["id"] == "trait.temperature.setpoint_c" && p["writable"] == true }));

        let state: serde_json::Value = serde_json::from_str(&api.get_state(&id).unwrap()).unwrap();
        assert!((f32_of(&state, "trait.temperature.current_c") - 20.0).abs() < f32::EPSILON);

        let written = api
            .write(&id, "trait.temperature.setpoint_c", "80")
            .unwrap();
        let body: serde_json::Value = serde_json::from_str(&written).unwrap();
        assert_eq!(body["ok"], true);

        let err = api
            .write(&id, "trait.temperature.setpoint_c", "20")
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfRange);

        api.write(&id, "trait.power.power_on", "null").unwrap();
        api.tick(&id, 5_000).unwrap();
        let state: serde_json::Value = serde_json::from_str(&api.get_state(&id).unwrap()).unwrap();
        let current = f32_of(&state, "trait.temperature.current_c");
        assert!(
            (current - 45.0).abs() < 0.05,
            "current={current}, expected 45 after 5s at 5 C/s"
        );

        let read = api
            .read(&id, Some(r#"["trait.power.power_state"]"#))
            .unwrap();
        let body: serde_json::Value = serde_json::from_str(&read).unwrap();
        assert_eq!(body["values"][0]["value"]["value"], "on");
    }

    #[test]
    fn unknown_class_and_device() {
        let mut api = WasmApi::new();
        assert_eq!(
            api.create_device("not_a_class").unwrap_err().code,
            ErrorCode::InvalidRequest
        );
        assert_eq!(
            api.create_device("beverage_cooler").unwrap_err().code,
            ErrorCode::InvalidRequest
        );
        assert_eq!(
            api.describe("missing").unwrap_err().code,
            ErrorCode::UnknownDevice
        );
        assert_eq!(
            api.get_state("missing").unwrap_err().code,
            ErrorCode::UnknownDevice
        );
    }

    #[test]
    fn tagged_value_write_and_command() {
        let mut api = WasmApi::new();
        let id = api.create_device("washer").unwrap();
        api.write(
            &id,
            "class.washer.spin_rpm",
            r#"{"type":"u16","value":1200}"#,
        )
        .unwrap();
        let read = api.read(&id, Some(r#"["class.washer.spin_rpm"]"#)).unwrap();
        let body: serde_json::Value = serde_json::from_str(&read).unwrap();
        assert_eq!(body["values"][0]["value"]["value"], 1200);

        api.write(&id, "trait.cycle.start", "null").unwrap();
        let state: serde_json::Value = serde_json::from_str(&api.get_state(&id).unwrap()).unwrap();
        assert_eq!(state["trait.cycle.cycle_state"]["value"], "running");
    }

    #[test]
    fn empty_read_returns_full_state() {
        let mut api = WasmApi::new();
        let id = api.create_device("fridge").unwrap();
        let all = api.read(&id, None).unwrap();
        let state = api.get_state(&id).unwrap();
        assert_eq!(all, state);
        assert!(state.contains("trait.temperature.setpoint_c"));
    }
}
