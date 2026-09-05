//! Procedure document serde types (practical JSON for `docs/standard/procedures.md` §2–4).

use serde::{Deserialize, Serialize};

use homecooked_schema::{ApplianceClassId, SemVer, Value};

use crate::error::Error;
use crate::guard::{Guard, GuardSet};

/// Kettle happy-path fixture: setpoint 80 °C, start, wait until ≥ 75 °C.
pub const KETTLE_HEAT_80_JSON: &str = include_str!("../examples/kettle_heat_80.json");

/// Microwave-only Domino's reheat sketch from `docs/standard/procedures.md` §4.
pub const REHEAT_DOMINOS_MICROWAVE_JSON: &str =
    include_str!("../examples/reheat_dominos_microwave.json");

/// Multi-device laundry demo: washer cycle then dryer cycle.
pub const WASH_THEN_DRY_JSON: &str = include_str!("../examples/wash_then_dry.json");

/// Dishwasher settings after fridge→DHW thermal preheat (procedure leg only).
pub const DISHWASHER_DHW_PREHEAT_JSON: &str =
    include_str!("../examples/dishwasher_dhw_preheat.json");

/// Bundled example documents: `(id, json)`.
pub const BUNDLED_EXAMPLE_PROCEDURES: &[(&str, &str)] = &[
    ("kettle_heat_80", KETTLE_HEAT_80_JSON),
    ("reheat_dominos_microwave", REHEAT_DOMINOS_MICROWAVE_JSON),
    ("wash_then_dry", WASH_THEN_DRY_JSON),
    ("dishwasher_dhw_preheat", DISHWASHER_DHW_PREHEAT_JSON),
];

/// Ordered recipe / protocol document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Procedure {
    pub id: String,
    #[serde(alias = "title")]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Procedure document version (sketch `version` field).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<SemVer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_version: Option<SemVer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<SemVer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub devices: Vec<DeviceRef>,
    pub steps: Vec<Step>,
}

/// Role binding hint. Multi-device orchestration can stay unbound in v1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceRef {
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class_id: Option<ClassHint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub optional: bool,
}

/// One class or a runner-chosen set (`oven` | `[oven, air_fryer]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClassHint {
    One(ApplianceClassId),
    Any(Vec<ApplianceClassId>),
}

/// One sequential HomeCooked operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    #[serde(alias = "op")]
    pub action: StepAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<StepTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(default, alias = "guard", skip_serializing_if = "GuardSet::is_empty")]
    pub guards: GuardSet,
    #[serde(default, alias = "timeout", skip_serializing_if = "Option::is_none")]
    pub timeout_s: Option<u32>,
}

/// Executable ops. Sketch `guard` deserializes as [`StepAction::Assert`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepAction {
    Read,
    Write,
    Command,
    Wait,
    #[serde(alias = "guard")]
    Assert,
}

/// Device binding plus optional qualified point id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepTarget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub point: Option<String>,
}

impl Procedure {
    pub fn from_json_str(s: &str) -> Result<Self, Error> {
        Ok(serde_json::from_str(s)?)
    }

    pub fn to_json_string(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(self)?)
    }

    /// Parse JSON and run structural validation.
    pub fn load_json(s: &str) -> Result<Self, Error> {
        let doc = Self::from_json_str(s)?;
        doc.validate()?;
        Ok(doc)
    }

    pub fn device_ref(&self, role: &str) -> Option<&DeviceRef> {
        self.devices.iter().find(|d| d.role == role)
    }
}

impl Step {
    pub fn point(&self) -> Option<&str> {
        self.target.as_ref().and_then(|t| t.point.as_deref())
    }

    pub fn role(&self) -> Option<&str> {
        self.target.as_ref().and_then(|t| t.role.as_deref())
    }

    pub fn guards(&self) -> &[Guard] {
        self.guards.as_slice()
    }
}

impl StepTarget {
    pub fn new() -> Self {
        Self {
            device_id: None,
            role: None,
            point: None,
        }
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    pub fn with_device_id(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }

    pub fn with_point(mut self, point: impl Into<String>) -> Self {
        self.point = Some(point.into());
        self
    }
}

impl Default for StepTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassHint {
    pub fn as_slice(&self) -> &[ApplianceClassId] {
        match self {
            Self::One(id) => std::slice::from_ref(id),
            Self::Any(ids) => ids,
        }
    }
}
