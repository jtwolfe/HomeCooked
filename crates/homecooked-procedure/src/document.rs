//! Procedure document serde types (practical JSON for `docs/standard/procedures.md` §2–4).

use serde::{Deserialize, Serialize};

use homecooked_schema::{ApplianceClassId, SemVer, Value};
use homecooked_thermal::{PortRef, PowerBandW, TransferOffer, TransferTarget};

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

/// Oven bake happy-path: program bake, setpoint 180 °C, wait until ≥ 170 °C.
pub const OVEN_BAKE_180_JSON: &str = include_str!("../examples/oven_bake_180.json");

/// Coffee brew happy-path: power on, program espresso, wait until boiler ≥ 85 °C.
pub const COFFEE_BREW_ESPRESSO_JSON: &str = include_str!("../examples/coffee_brew_espresso.json");

/// Air fryer cook happy-path: program fries, setpoint 200 °C, wait until ≥ 190 °C.
pub const AIR_FRYER_COOK_200_JSON: &str = include_str!("../examples/air_fryer_cook_200.json");

/// Wait on plant DHW reservoir temperature (procedure⇄thermal thin bridge).
pub const WAIT_DHW_RESERVOIR_JSON: &str = include_str!("../examples/wait_dhw_reservoir.json");

/// Offer fridge condenser heat to DHW preheat (procedure⇄thermal thin bridge).
pub const OFFER_FRIDGE_DHW_JSON: &str = include_str!("../examples/offer_fridge_dhw.json");

/// Bundled example documents: `(id, json)`.
pub const BUNDLED_EXAMPLE_PROCEDURES: &[(&str, &str)] = &[
    ("kettle_heat_80", KETTLE_HEAT_80_JSON),
    ("reheat_dominos_microwave", REHEAT_DOMINOS_MICROWAVE_JSON),
    ("wash_then_dry", WASH_THEN_DRY_JSON),
    ("dishwasher_dhw_preheat", DISHWASHER_DHW_PREHEAT_JSON),
    ("oven_bake_180", OVEN_BAKE_180_JSON),
    ("coffee_brew_espresso", COFFEE_BREW_ESPRESSO_JSON),
    ("air_fryer_cook_200", AIR_FRYER_COOK_200_JSON),
    ("wait_dhw_reservoir", WAIT_DHW_RESERVOIR_JSON),
    ("offer_fridge_dhw", OFFER_FRIDGE_DHW_JSON),
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

/// Comparison operator for [`StepAction::ThermalWait`] (numeric °C).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalCmp {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
}

impl ThermalCmp {
    pub fn eval(self, got: f64, threshold: f64) -> bool {
        match self {
            Self::Eq => (got - threshold).abs() < 1e-6,
            Self::Ne => (got - threshold).abs() >= 1e-6,
            Self::Gt => got > threshold,
            Self::Gte => got >= threshold,
            Self::Lt => got < threshold,
            Self::Lte => got <= threshold,
        }
    }
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
    /// Plant reservoir id for [`StepAction::ThermalWait`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reservoir_id: Option<String>,
    /// Comparison for [`StepAction::ThermalWait`] (`gte`, `gt`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cmp: Option<ThermalCmp>,
    /// Threshold °C for [`StepAction::ThermalWait`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp_c: Option<f64>,
    /// Source heat port for [`StepAction::ThermalOffer`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_port: Option<PortRef>,
    /// Destination heat port for [`StepAction::ThermalOffer`] (xor `to_reservoir_id`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_port: Option<PortRef>,
    /// Destination reservoir id for [`StepAction::ThermalOffer`] (xor `to_port`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_reservoir_id: Option<String>,
    /// Offered power band (W) for [`StepAction::ThermalOffer`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power_w: Option<PowerBandW>,
    /// Optional transfer duration (s) for [`StepAction::ThermalOffer`] (`TransferOffer::duration_s`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<u32>,
    /// Offer priority for [`StepAction::ThermalOffer`] (default 1 when omitted at run time).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,
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
    /// Wait until a thermal plant reservoir temperature meets `cmp`/`temp_c`.
    #[serde(alias = "wait_reservoir")]
    ThermalWait,
    /// Submit a [`TransferOffer`] to the attached plant and immediately negotiate
    /// (accept at max allowable, or decline).
    #[serde(alias = "offer_transfer")]
    ThermalOffer,
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

    pub fn reservoir_id(&self) -> Option<&str> {
        self.reservoir_id.as_deref()
    }

    /// Build a [`TransferOffer`] from [`StepAction::ThermalOffer`] fields.
    ///
    /// Prefer [`Procedure::validate`] before run; this helper also errors if
    /// destination fields are missing or both `to_port` and `to_reservoir_id`
    /// are set.
    pub fn transfer_offer(&self) -> Result<TransferOffer, Error> {
        let from_port = self
            .from_port
            .clone()
            .ok_or_else(|| Error::at_step(&self.id, "thermal_offer requires from_port"))?;
        let to = match (&self.to_port, self.to_reservoir_id.as_deref()) {
            (Some(port), None) => TransferTarget::Port {
                device_id: port.device_id.clone(),
                port_id: port.port_id.clone(),
            },
            (None, Some(rid)) if !rid.is_empty() => TransferTarget::Reservoir {
                reservoir_id: rid.to_string(),
            },
            (Some(_), Some(_)) => {
                return Err(Error::at_step(
                    &self.id,
                    "thermal_offer requires exactly one of to_port or to_reservoir_id",
                ));
            }
            _ => {
                return Err(Error::at_step(
                    &self.id,
                    "thermal_offer requires to_port or to_reservoir_id",
                ));
            }
        };
        let power_w = self
            .power_w
            .ok_or_else(|| Error::at_step(&self.id, "thermal_offer requires power_w"))?;
        let priority = self.priority.unwrap_or(1);
        Ok(TransferOffer::new(
            from_port,
            to,
            power_w,
            self.duration_s,
            priority,
        ))
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
