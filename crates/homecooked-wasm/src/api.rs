//! JSON-string API over [`homecooked_sim::Simulator`], shared by wasm-bindgen
//! wrappers and native tests.

use std::collections::HashSet;

use homecooked_core::{CoreError, DeviceId};
use homecooked_procedure::{
    run, ClassHint, DeviceBindings, DeviceRef, FailReason, Procedure, RunResult, RunStatus,
    StepAction, BUNDLED_EXAMPLE_PROCEDURES, DISHWASHER_DHW_PREHEAT_JSON,
};
use homecooked_schema::{
    catalog_group, AccessMode, ApplianceClassId, DeviceIdentity, ErrorCode, Unit, Value,
    ValueRange, ValueType, STATIC_CLASS_IDS,
};
use homecooked_sim::Simulator;
use homecooked_thermal::{
    HeatPort, PortRef, PowerBandW, Reservoir, ThermalPlant, TransferOffer, TransferReply,
    TransferResult, TransferTarget,
};
use serde::{Deserialize, Serialize};

/// In-memory simulator world exposed to JS as JSON strings.
#[derive(Debug, Default)]
pub struct WasmApi {
    sim: Simulator,
    /// Optional thermal plant (fridge condenser → DHW demo, or custom later).
    thermal: Option<ThermalPlant>,
    /// Transfers applied by the most recent [`Self::thermal_tick`] / demo transfer.
    last_thermal_transfers: Vec<TransferResult>,
    /// Last offer/accept reply from [`Self::thermal_negotiate_demo`].
    last_thermal_reply: Option<TransferReply>,
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

impl From<homecooked_procedure::Error> for ApiError {
    fn from(err: homecooked_procedure::Error) -> Self {
        match err {
            homecooked_procedure::Error::Json(message) => Self::invalid_request(message),
            homecooked_procedure::Error::Invalid { step_id, message } => {
                let message = match step_id {
                    Some(id) => format!("step {id}: {message}"),
                    None => message,
                };
                Self::invalid_request(message)
            }
            homecooked_procedure::Error::Capability(v) => Self {
                code: v.code,
                message: v.message,
                point_id: v.point_id,
                expected: v.expected,
            },
            homecooked_procedure::Error::Backend {
                code,
                message,
                point_id,
            } => Self {
                code,
                message,
                point_id,
                expected: None,
            },
        }
    }
}

impl From<homecooked_thermal::Error> for ApiError {
    fn from(err: homecooked_thermal::Error) -> Self {
        Self::invalid_request(err.to_string())
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
    /// Catalog Index group from `docs/catalog/appliances.md` (Laundry, Cold, …).
    pub group: String,
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

/// Bundled example listing entry for the simulator-web picker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleProcedureInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub class_hints: Vec<String>,
}

/// Summary returned by [`WasmApi::parse_procedure`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureSummary {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub class_hints: Vec<String>,
    pub step_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub devices: Vec<ProcedureDeviceSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureDeviceSummary {
    pub role: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub class_hints: Vec<String>,
    pub optional: bool,
}

/// JS-facing run result (maps [`RunStatus`] / [`StepOutcome`] / [`FailReason`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureRunOut {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_step_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_reason: Option<FailReasonOut>,
    pub outcomes: Vec<StepOutcomeOut>,
    pub bindings: Vec<BindingOut>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailReasonOut {
    pub kind: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<ErrorCode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOutcomeOut {
    pub step_id: String,
    pub action: StepAction,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingOut {
    pub role: String,
    pub device_id: String,
    pub class_id: ApplianceClassId,
    pub spawned: bool,
}

/// Structured thermal plant snapshot for the simulator-web panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalStateOut {
    pub loaded: bool,
    pub scenario: Option<String>,
    pub reservoirs: Vec<Reservoir>,
    pub ports: Vec<HeatPort>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub last_transfers: Vec<TransferResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_reply: Option<TransferReply>,
}

/// Tick / demo-transfer response: applied transfers plus fresh plant state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalTickOut {
    pub dt_s: f32,
    pub transfers: Vec<TransferResult>,
    pub state: ThermalStateOut,
}

/// Dual-path demo: thermal fridge→DHW then dishwasher preheat procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalThenDishwasherOut {
    pub scenario: String,
    pub dhw_temp_start_c: f32,
    pub dhw_temp_end_c: f32,
    pub thermal: ThermalTickOut,
    pub procedure: ProcedureRunOut,
}

impl WasmApi {
    pub fn new() -> Self {
        Self {
            sim: Simulator::new(),
            thermal: None,
            last_thermal_transfers: Vec::new(),
            last_thermal_reply: None,
        }
    }

    pub fn list_appliance_classes() -> String {
        // Catalog Index order so the UI can emit one `<optgroup>` per group
        // without duplicating group membership in JS. Lists every statically
        // tabled class (Tier-A ∪ Tier-B = full catalog).
        let classes: Vec<ClassInfo> = ApplianceClassId::ALL
            .iter()
            .filter(|id| STATIC_CLASS_IDS.contains(id))
            .map(|id| ClassInfo {
                id: id.as_str().to_string(),
                label: class_label(*id),
                group: catalog_group(*id).to_string(),
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

    /// JSON array of bundled examples (`id`, `name`, `description`, `class_hints`).
    pub fn list_example_procedures() -> String {
        let items: Vec<ExampleProcedureInfo> = BUNDLED_EXAMPLE_PROCEDURES
            .iter()
            .filter_map(|(id, json)| {
                let doc = Procedure::load_json(json).ok()?;
                Some(ExampleProcedureInfo {
                    id: (*id).to_string(),
                    name: doc.name,
                    description: doc.description,
                    class_hints: class_hints_of(&doc.devices),
                })
            })
            .collect();
        serde_json::to_string(&items).expect("ExampleProcedureInfo serializes")
    }

    /// Full JSON for a bundled example id (`kettle_heat_80`, `reheat_dominos_microwave`, `wash_then_dry`, `dishwasher_dhw_preheat`, `oven_bake_180`, `coffee_brew_espresso`).
    pub fn get_example_procedure(id: &str) -> Result<String, ApiError> {
        BUNDLED_EXAMPLE_PROCEDURES
            .iter()
            .find(|(known, _)| *known == id)
            .map(|(_, json)| (*json).to_string())
            .ok_or_else(|| ApiError::invalid_request(format!("unknown example procedure {id:?}")))
    }

    /// Parse + structurally validate a procedure document.
    pub fn parse_procedure(json: &str) -> Result<String, ApiError> {
        let doc = Procedure::load_json(json)?;
        let summary = ProcedureSummary {
            id: doc.id.clone(),
            name: doc.name.clone(),
            description: doc.description.clone(),
            class_hints: class_hints_of(&doc.devices),
            step_count: doc.steps.len(),
            devices: doc
                .devices
                .iter()
                .map(|d| ProcedureDeviceSummary {
                    role: d.role.clone(),
                    class_hints: class_ids_of(d.class_id.as_ref()),
                    optional: d.optional,
                })
                .collect(),
        };
        serde_json::to_string(&summary).map_err(|e| ApiError::internal(e.to_string()))
    }

    /// Auto-bind / spawn devices by role, then run the procedure against the sim.
    pub fn run_procedure(&mut self, json: &str) -> Result<String, ApiError> {
        let doc = Procedure::load_json(json)?;
        let (bindings, binding_out) = self.bind_procedure_devices(&doc)?;
        let result = run(&doc, &mut self.sim, &bindings);
        let out = ProcedureRunOut::from_run(result, binding_out);
        serde_json::to_string(&out).map_err(|e| ApiError::internal(e.to_string()))
    }

    /// Create/reset the fridge condenser → DHW water_heater demo plant.
    pub fn create_thermal_demo(&mut self) -> Result<String, ApiError> {
        self.thermal = Some(ThermalPlant::fridge_condenser_dhw_demo()?);
        self.last_thermal_transfers.clear();
        self.last_thermal_reply = None;
        self.thermal_state()
    }

    /// JSON snapshot of the loaded thermal plant (empty if none).
    pub fn thermal_state(&self) -> Result<String, ApiError> {
        serde_json::to_string(&self.thermal_state_out())
            .map_err(|e| ApiError::internal(e.to_string()))
    }

    /// Negotiate the demo offer (fridge condenser → water_heater preheat).
    pub fn thermal_negotiate_demo(&mut self) -> Result<String, ApiError> {
        let offer = demo_fridge_offer()?;
        let reply = self.thermal_mut()?.negotiate(offer);
        self.last_thermal_reply = Some(reply);
        self.thermal_state()
    }

    /// Apply queued accepts over `dt_s` seconds (one plant tick).
    pub fn thermal_tick(&mut self, dt_s: f32) -> Result<String, ApiError> {
        if !dt_s.is_finite() || dt_s <= 0.0 {
            return Err(ApiError::invalid_request("dt_s must be > 0"));
        }
        let transfers = self.thermal_mut()?.step(dt_s)?;
        self.last_thermal_transfers = transfers.clone();
        let out = ThermalTickOut {
            dt_s,
            transfers,
            state: self.thermal_state_out(),
        };
        serde_json::to_string(&out).map_err(|e| ApiError::internal(e.to_string()))
    }

    /// Negotiate the demo offer then tick once (UI one-shot transfer).
    pub fn thermal_demo_transfer(&mut self, dt_s: f32) -> Result<String, ApiError> {
        if !dt_s.is_finite() || dt_s <= 0.0 {
            return Err(ApiError::invalid_request("dt_s must be > 0"));
        }
        let offer = demo_fridge_offer()?;
        let reply = self.thermal_mut()?.negotiate(offer);
        self.last_thermal_reply = Some(reply);
        let transfers = self.thermal_mut()?.step(dt_s)?;
        self.last_thermal_transfers = transfers.clone();
        let out = ThermalTickOut {
            dt_s,
            transfers,
            state: self.thermal_state_out(),
        };
        serde_json::to_string(&out).map_err(|e| ApiError::internal(e.to_string()))
    }

    /// Dual-path demo: load fridge→DHW plant, transfer (`dt_s`), assert DHW rose,
    /// then run `dishwasher_dhw_preheat` (eco + wash_temp reflecting warm inlet).
    ///
    /// Procedures cannot call thermal APIs yet; this helper is the orchestrated
    /// wasm / conformance path documented in `docs/standard/thermal-plant.md` §8.
    pub fn run_thermal_then_dishwasher_preheat(&mut self, dt_s: f32) -> Result<String, ApiError> {
        if !dt_s.is_finite() || dt_s <= 0.0 {
            return Err(ApiError::invalid_request("dt_s must be > 0"));
        }

        self.create_thermal_demo()?;
        let start = self
            .thermal_mut()?
            .get_reservoir("dhw-tank")
            .and_then(|r| r.temp_c)
            .ok_or_else(|| ApiError::internal("dhw-tank missing temp"))?;

        let thermal_raw = self.thermal_demo_transfer(dt_s)?;
        let thermal: ThermalTickOut =
            serde_json::from_str(&thermal_raw).map_err(|e| ApiError::internal(e.to_string()))?;

        let end = thermal
            .state
            .reservoirs
            .iter()
            .find(|r| r.id == "dhw-tank")
            .and_then(|r| r.temp_c)
            .ok_or_else(|| ApiError::internal("dhw-tank missing after transfer"))?;
        if end <= start {
            return Err(ApiError::internal(format!(
                "DHW temp did not rise: start={start}, end={end}"
            )));
        }

        let proc_raw = self.run_procedure(DISHWASHER_DHW_PREHEAT_JSON)?;
        let procedure: ProcedureRunOut =
            serde_json::from_str(&proc_raw).map_err(|e| ApiError::internal(e.to_string()))?;
        if procedure.status != "completed" {
            return Err(ApiError::internal(format!(
                "dishwasher procedure status={}, expected completed",
                procedure.status
            )));
        }

        let out = ThermalThenDishwasherOut {
            scenario: "thermal_then_dishwasher_preheat".into(),
            dhw_temp_start_c: start,
            dhw_temp_end_c: end,
            thermal,
            procedure,
        };
        serde_json::to_string(&out).map_err(|e| ApiError::internal(e.to_string()))
    }

    fn thermal_mut(&mut self) -> Result<&mut ThermalPlant, ApiError> {
        self.thermal.as_mut().ok_or_else(|| {
            ApiError::invalid_request("no thermal plant loaded; call create_thermal_demo first")
        })
    }

    fn thermal_state_out(&self) -> ThermalStateOut {
        match &self.thermal {
            Some(plant) => ThermalStateOut {
                loaded: true,
                scenario: Some("fridge_condenser_dhw".into()),
                reservoirs: plant.list_reservoirs().into_iter().cloned().collect(),
                ports: plant.list_ports().into_iter().cloned().collect(),
                last_transfers: self.last_thermal_transfers.clone(),
                last_reply: self.last_thermal_reply.clone(),
            },
            None => ThermalStateOut {
                loaded: false,
                scenario: None,
                reservoirs: Vec::new(),
                ports: Vec::new(),
                last_transfers: Vec::new(),
                last_reply: None,
            },
        }
    }

    fn bind_procedure_devices(
        &mut self,
        procedure: &Procedure,
    ) -> Result<(DeviceBindings, Vec<BindingOut>), ApiError> {
        let mut bindings = DeviceBindings::new();
        let mut used: HashSet<String> = HashSet::new();
        let mut out = Vec::new();

        for dev_ref in collect_role_refs(procedure) {
            if let Some(bound) = self.try_bind_existing(&dev_ref, &used) {
                used.insert(bound.device_id.clone());
                bindings.insert(&dev_ref.role, &bound.device_id);
                out.push(bound);
                continue;
            }
            if dev_ref.optional {
                continue;
            }
            let class = first_spawn_class(&dev_ref).ok_or_else(|| {
                ApiError::invalid_request(format!(
                    "cannot bind required role {:?}: no class hint and no matching device",
                    dev_ref.role
                ))
            })?;
            let id = self.sim.spawn(class)?;
            used.insert(id.as_str().to_string());
            bindings.insert(&dev_ref.role, id.as_str());
            out.push(BindingOut {
                role: dev_ref.role,
                device_id: id.as_str().to_string(),
                class_id: class,
                spawned: true,
            });
        }

        Ok((bindings, out))
    }

    fn try_bind_existing(&self, dev_ref: &DeviceRef, used: &HashSet<String>) -> Option<BindingOut> {
        if let Some(id) = dev_ref.device_id.as_deref() {
            if let Some(dev) = self.sim.hub().registry.get(&DeviceId::new(id)) {
                return Some(BindingOut {
                    role: dev_ref.role.clone(),
                    device_id: id.to_string(),
                    class_id: dev.identity.class_id,
                    spawned: false,
                });
            }
        }
        let classes = class_ids_enum(dev_ref);
        self.sim
            .hub()
            .registry
            .list()
            .into_iter()
            .find(|dev| {
                !used.contains(&dev.identity.device_id) && classes.contains(&dev.identity.class_id)
            })
            .map(|dev| BindingOut {
                role: dev_ref.role.clone(),
                device_id: dev.identity.device_id.clone(),
                class_id: dev.identity.class_id,
                spawned: false,
            })
    }
}

impl ProcedureRunOut {
    fn from_run(result: RunResult, bindings: Vec<BindingOut>) -> Self {
        let (status, failed_step_id, fail_reason) = match result.status {
            RunStatus::Completed => ("completed".to_string(), None, None),
            RunStatus::Failed { step_id, reason } => (
                "failed".to_string(),
                Some(step_id),
                Some(FailReasonOut::from(reason)),
            ),
        };
        Self {
            status,
            failed_step_id,
            fail_reason,
            outcomes: result
                .outcomes
                .into_iter()
                .map(|o| StepOutcomeOut {
                    step_id: o.step_id,
                    action: o.action,
                    ok: o.ok,
                    read_value: o.read_value,
                    message: o.message,
                })
                .collect(),
            bindings,
        }
    }
}

impl From<FailReason> for FailReasonOut {
    fn from(reason: FailReason) -> Self {
        match reason {
            FailReason::Validation(message) => Self {
                kind: "validation".into(),
                message,
                role: None,
                code: None,
            },
            FailReason::GuardFailed(message) => Self {
                kind: "guard_failed".into(),
                message,
                role: None,
                code: None,
            },
            FailReason::Timeout => Self {
                kind: "timeout".into(),
                message: "timeout".into(),
                role: None,
                code: Some(ErrorCode::Timeout),
            },
            FailReason::UnboundDevice { role } => Self {
                kind: "unbound_device".into(),
                message: match &role {
                    Some(r) => format!("unbound device role {r}"),
                    None => "unbound device".into(),
                },
                role,
                code: None,
            },
            FailReason::Backend { code, message } => Self {
                kind: "backend".into(),
                message,
                role: None,
                code: Some(code),
            },
        }
    }
}

fn class_ids_of(hint: Option<&ClassHint>) -> Vec<String> {
    hint.map(|h| {
        h.as_slice()
            .iter()
            .map(|c| c.as_str().to_string())
            .collect()
    })
    .unwrap_or_default()
}

fn class_hints_of(devices: &[DeviceRef]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for dev in devices {
        for id in class_ids_of(dev.class_id.as_ref()) {
            if seen.insert(id.clone()) {
                out.push(id);
            }
        }
    }
    out
}

fn class_ids_enum(dev_ref: &DeviceRef) -> Vec<ApplianceClassId> {
    if let Some(hint) = &dev_ref.class_id {
        return hint.as_slice().to_vec();
    }
    ApplianceClassId::from_str_id(&dev_ref.role)
        .into_iter()
        .collect()
}

fn first_spawn_class(dev_ref: &DeviceRef) -> Option<ApplianceClassId> {
    class_ids_enum(dev_ref).into_iter().next()
}

/// Declared `devices` plus any step roles that were omitted from the document.
fn demo_fridge_offer() -> Result<TransferOffer, ApiError> {
    Ok(TransferOffer::new(
        PortRef::new("fridge-kitchen", "condenser")?,
        TransferTarget::port("water-heater-plant", "preheat")?,
        PowerBandW::new(80, 120)?,
        None,
        1,
    ))
}

fn collect_role_refs(procedure: &Procedure) -> Vec<DeviceRef> {
    let mut refs = procedure.devices.clone();
    let mut seen: HashSet<String> = refs.iter().map(|d| d.role.clone()).collect();
    for step in &procedure.steps {
        if let Some(role) = step.role() {
            if seen.insert(role.to_string()) {
                refs.push(DeviceRef {
                    role: role.to_string(),
                    class_id: ApplianceClassId::from_str_id(role).map(ClassHint::One),
                    device_id: None,
                    optional: false,
                });
            }
        }
    }
    refs
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
    use homecooked_schema::TIER_A_CLASS_IDS;

    fn f32_of(state: &serde_json::Value, point: &str) -> f32 {
        state[point]["value"].as_f64().unwrap() as f32
    }

    #[test]
    fn list_appliance_classes_covers_tier_a() {
        let classes: Vec<ClassInfo> =
            serde_json::from_str(&WasmApi::list_appliance_classes()).unwrap();
        assert_eq!(classes.len(), 56);
        assert_eq!(classes.len(), STATIC_CLASS_IDS.len());
        assert!(classes.len() >= TIER_A_CLASS_IDS.len());

        let listed: std::collections::BTreeSet<&str> =
            classes.iter().map(|c| c.id.as_str()).collect();
        let expected: std::collections::BTreeSet<&str> =
            STATIC_CLASS_IDS.iter().map(|id| id.as_str()).collect();
        assert_eq!(listed, expected);
        for id in TIER_A_CLASS_IDS {
            assert!(listed.contains(id.as_str()));
        }

        assert!(classes
            .iter()
            .any(|c| c.id == "kettle" && c.label == "Kettle" && c.group == "Beverage"));
        assert!(classes.iter().any(|c| c.id == "induction_hob"
            && c.label == "Induction Hob"
            && c.group == "Cooking"));
        assert!(classes
            .iter()
            .any(|c| c.id == "steam_oven" && c.label == "Steam Oven" && c.group == "Cooking"));
        assert!(classes
            .iter()
            .any(|c| c.id == "wine_cooler" && c.label == "Wine Cooler" && c.group == "Cold"));
        assert!(classes
            .iter()
            .any(|c| c.id == "hvac" && c.label == "Hvac" && c.group == "Climate"));

        let groups: Vec<&str> = classes.iter().map(|c| c.group.as_str()).collect();
        let mut seen = Vec::new();
        for group in groups {
            if seen.last() != Some(&group) {
                assert!(
                    !seen.contains(&group),
                    "class list is not grouped: {group} appears twice"
                );
                seen.push(group);
            }
        }
    }

    #[test]
    fn create_describe_get_state_for_every_tier_a_class() {
        let mut api = WasmApi::new();
        for class in STATIC_CLASS_IDS {
            let id = api
                .create_device(class.as_str())
                .unwrap_or_else(|e| panic!("create {} failed: {e}", class.as_str()));
            assert!(
                id.starts_with(&format!("sim-{}-", class.as_str())),
                "unexpected device id {id} for {}",
                class.as_str()
            );
            let desc: serde_json::Value =
                serde_json::from_str(&api.describe(&id).unwrap()).unwrap();
            assert_eq!(desc["identity"]["class_id"], class.as_str());
            assert!(desc["points"].as_array().is_some_and(|p| !p.is_empty()));
            let state: serde_json::Value =
                serde_json::from_str(&api.get_state(&id).unwrap()).unwrap();
            assert!(state.as_object().is_some_and(|o| !o.is_empty()));
        }
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
        // Every catalog id is tabled (Tier-A ∪ Tier-B); spawn succeeds.
        let id = api.create_device("beverage_cooler").unwrap();
        assert!(id.starts_with("sim-beverage_cooler-"));
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

    #[test]
    fn list_example_procedures_includes_kettle_dominos_laundry_dishwasher_oven_and_coffee() {
        let items: Vec<ExampleProcedureInfo> =
            serde_json::from_str(&WasmApi::list_example_procedures()).unwrap();
        assert_eq!(items.len(), 6);
        assert_eq!(items[0].id, "kettle_heat_80");
        assert_eq!(items[0].name, "Heat kettle to 80C");
        assert!(items[0].class_hints.iter().any(|c| c == "kettle"));
        assert_eq!(items[1].id, "reheat_dominos_microwave");
        assert!(items[1].class_hints.iter().any(|c| c == "microwave"));
        assert_eq!(items[2].id, "wash_then_dry");
        assert!(items[2].class_hints.iter().any(|c| c == "washer"));
        assert!(items[2].class_hints.iter().any(|c| c == "dryer"));
        assert_eq!(items[3].id, "dishwasher_dhw_preheat");
        assert!(items[3].class_hints.iter().any(|c| c == "dishwasher"));
        assert_eq!(items[4].id, "oven_bake_180");
        assert_eq!(items[4].name, "Oven bake at 180C");
        assert!(items[4].class_hints.iter().any(|c| c == "oven"));
        assert_eq!(items[5].id, "coffee_brew_espresso");
        assert_eq!(items[5].name, "Brew espresso");
        assert!(items[5].class_hints.iter().any(|c| c == "coffee_machine"));
    }

    #[test]
    fn get_and_parse_example_procedures() {
        let kettle = WasmApi::get_example_procedure("kettle_heat_80").unwrap();
        let summary: ProcedureSummary =
            serde_json::from_str(&WasmApi::parse_procedure(&kettle).unwrap()).unwrap();
        assert_eq!(summary.id, "kettle_heat_80");
        assert_eq!(summary.step_count, 4);
        assert_eq!(summary.devices[0].role, "kettle");

        let mw = WasmApi::get_example_procedure("reheat_dominos_microwave").unwrap();
        let summary: ProcedureSummary =
            serde_json::from_str(&WasmApi::parse_procedure(&mw).unwrap()).unwrap();
        assert_eq!(summary.id, "reheat_dominos_microwave");
        assert_eq!(summary.step_count, 5);

        let laundry = WasmApi::get_example_procedure("wash_then_dry").unwrap();
        let summary: ProcedureSummary =
            serde_json::from_str(&WasmApi::parse_procedure(&laundry).unwrap()).unwrap();
        assert_eq!(summary.id, "wash_then_dry");
        assert_eq!(summary.step_count, 9);
        assert_eq!(summary.devices.len(), 2);
        assert_eq!(summary.devices[0].role, "washer");
        assert_eq!(summary.devices[1].role, "dryer");

        let dw = WasmApi::get_example_procedure("dishwasher_dhw_preheat").unwrap();
        let summary: ProcedureSummary =
            serde_json::from_str(&WasmApi::parse_procedure(&dw).unwrap()).unwrap();
        assert_eq!(summary.id, "dishwasher_dhw_preheat");
        assert_eq!(summary.step_count, 4);
        assert_eq!(summary.devices[0].role, "dishwasher");

        let oven = WasmApi::get_example_procedure("oven_bake_180").unwrap();
        let summary: ProcedureSummary =
            serde_json::from_str(&WasmApi::parse_procedure(&oven).unwrap()).unwrap();
        assert_eq!(summary.id, "oven_bake_180");
        assert_eq!(summary.step_count, 5);
        assert_eq!(summary.devices[0].role, "oven");

        let coffee = WasmApi::get_example_procedure("coffee_brew_espresso").unwrap();
        let summary: ProcedureSummary =
            serde_json::from_str(&WasmApi::parse_procedure(&coffee).unwrap()).unwrap();
        assert_eq!(summary.id, "coffee_brew_espresso");
        assert_eq!(summary.step_count, 5);
        assert_eq!(summary.devices[0].role, "coffee_machine");

        let err = WasmApi::get_example_procedure("not_a_recipe").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidRequest);

        let err = WasmApi::parse_procedure("{").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidRequest);
    }

    #[test]
    fn run_kettle_procedure_auto_spawns_and_completes() {
        let mut api = WasmApi::new();
        let json = WasmApi::get_example_procedure("kettle_heat_80").unwrap();
        let raw = api.run_procedure(&json).unwrap();
        let result: ProcedureRunOut = serde_json::from_str(&raw).unwrap();

        assert_eq!(result.status, "completed");
        assert!(result.failed_step_id.is_none());
        assert!(result.fail_reason.is_none());
        assert_eq!(result.outcomes.len(), 4);
        assert!(result.outcomes.iter().all(|o| o.ok));
        assert_eq!(result.outcomes[0].step_id, "setpoint");
        assert_eq!(result.outcomes[1].step_id, "start");
        assert_eq!(result.outcomes[2].step_id, "wait_heat");
        assert_eq!(result.outcomes[3].step_id, "assert_temp");

        assert_eq!(result.bindings.len(), 1);
        assert_eq!(result.bindings[0].role, "kettle");
        assert_eq!(result.bindings[0].class_id, ApplianceClassId::Kettle);
        assert!(result.bindings[0].spawned);
        assert!(result.bindings[0].device_id.starts_with("sim-kettle-"));

        let devices: Vec<DeviceInfo> = serde_json::from_str(&api.list_devices()).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_id, result.bindings[0].device_id);

        let state: serde_json::Value =
            serde_json::from_str(&api.get_state(&result.bindings[0].device_id).unwrap()).unwrap();
        assert!(f32_of(&state, "trait.temperature.current_c") >= 75.0);
    }

    #[test]
    fn run_dominos_microwave_procedure_auto_spawns_and_completes() {
        let mut api = WasmApi::new();
        let json = WasmApi::get_example_procedure("reheat_dominos_microwave").unwrap();
        let raw = api.run_procedure(&json).unwrap();
        let result: ProcedureRunOut = serde_json::from_str(&raw).unwrap();

        assert_eq!(
            result.status, "completed",
            "dominos run failed: {:?}",
            result.fail_reason
        );
        assert!(result.failed_step_id.is_none());
        assert!(result.fail_reason.is_none());
        assert_eq!(result.outcomes.len(), 5);
        assert!(result.outcomes.iter().all(|o| o.ok));
        assert_eq!(result.outcomes[0].step_id, "mw_power");
        assert_eq!(result.outcomes[1].step_id, "mw_cook_time");
        assert_eq!(result.outcomes[2].step_id, "mw_start");
        assert_eq!(result.outcomes[3].step_id, "mw_wait");
        assert_eq!(result.outcomes[4].step_id, "mw_stop");

        // Microwave-only v1: optional crisp/fridge roles are not auto-spawned.
        assert_eq!(result.bindings.len(), 1);
        assert_eq!(result.bindings[0].role, "microwave");
        assert_eq!(result.bindings[0].class_id, ApplianceClassId::Microwave);
        assert!(result.bindings[0].spawned);
        assert!(result.bindings[0].device_id.starts_with("sim-microwave-"));

        let state: serde_json::Value =
            serde_json::from_str(&api.get_state(&result.bindings[0].device_id).unwrap()).unwrap();
        // Final cancel/`idle_cycle` resets elapsed_s; wait step already required >= 45.
        assert_eq!(state["trait.cycle.cycle_state"]["value"], "idle");
        assert_eq!(state["trait.cycle.elapsed_s"]["value"], 0);
    }

    #[test]
    fn run_oven_bake_180_procedure_auto_spawns_and_completes() {
        let mut api = WasmApi::new();
        let json = WasmApi::get_example_procedure("oven_bake_180").unwrap();
        let raw = api.run_procedure(&json).unwrap();
        let result: ProcedureRunOut = serde_json::from_str(&raw).unwrap();

        assert_eq!(
            result.status, "completed",
            "oven_bake_180 run failed: {:?}",
            result.fail_reason
        );
        assert!(result.failed_step_id.is_none());
        assert!(result.fail_reason.is_none());
        assert_eq!(result.outcomes.len(), 5);
        assert!(result.outcomes.iter().all(|o| o.ok));
        assert_eq!(result.outcomes[0].step_id, "program");
        assert_eq!(result.outcomes[1].step_id, "setpoint");
        assert_eq!(result.outcomes[2].step_id, "start");
        assert_eq!(result.outcomes[3].step_id, "wait_heat");
        assert_eq!(result.outcomes[4].step_id, "assert_temp");

        assert_eq!(result.bindings.len(), 1);
        assert_eq!(result.bindings[0].role, "oven");
        assert_eq!(result.bindings[0].class_id, ApplianceClassId::Oven);
        assert!(result.bindings[0].spawned);
        assert!(result.bindings[0].device_id.starts_with("sim-oven-"));

        let state: serde_json::Value =
            serde_json::from_str(&api.get_state(&result.bindings[0].device_id).unwrap()).unwrap();
        assert!(f32_of(&state, "trait.temperature.current_c") >= 170.0);
        assert_eq!(state["trait.program.program"]["value"], "bake");
        assert!((f32_of(&state, "trait.temperature.setpoint_c") - 180.0).abs() < 0.05);
    }

    #[test]
    fn run_coffee_brew_espresso_procedure_auto_spawns_and_completes() {
        let mut api = WasmApi::new();
        let json = WasmApi::get_example_procedure("coffee_brew_espresso").unwrap();
        let raw = api.run_procedure(&json).unwrap();
        let result: ProcedureRunOut = serde_json::from_str(&raw).unwrap();

        assert_eq!(
            result.status, "completed",
            "coffee_brew_espresso run failed: {:?}",
            result.fail_reason
        );
        assert!(result.failed_step_id.is_none());
        assert!(result.fail_reason.is_none());
        assert_eq!(result.outcomes.len(), 5);
        assert!(result.outcomes.iter().all(|o| o.ok));
        assert_eq!(result.outcomes[0].step_id, "power");
        assert_eq!(result.outcomes[1].step_id, "program");
        assert_eq!(result.outcomes[2].step_id, "start");
        assert_eq!(result.outcomes[3].step_id, "wait_boiler");
        assert_eq!(result.outcomes[4].step_id, "assert_boiler");

        assert_eq!(result.bindings.len(), 1);
        assert_eq!(result.bindings[0].role, "coffee_machine");
        assert_eq!(result.bindings[0].class_id, ApplianceClassId::CoffeeMachine);
        assert!(result.bindings[0].spawned);
        assert!(result.bindings[0]
            .device_id
            .starts_with("sim-coffee_machine-"));

        let state: serde_json::Value =
            serde_json::from_str(&api.get_state(&result.bindings[0].device_id).unwrap()).unwrap();
        assert!(f32_of(&state, "class.coffee_machine.boiler_c") >= 85.0);
        assert_eq!(state["trait.program.program"]["value"], "espresso");
    }

    #[test]
    fn run_wash_then_dry_procedure_auto_spawns_and_completes() {
        let mut api = WasmApi::new();
        let json = WasmApi::get_example_procedure("wash_then_dry").unwrap();
        let raw = api.run_procedure(&json).unwrap();
        let result: ProcedureRunOut = serde_json::from_str(&raw).unwrap();

        assert_eq!(
            result.status, "completed",
            "wash_then_dry run failed: {:?}",
            result.fail_reason
        );
        assert!(result.failed_step_id.is_none());
        assert!(result.fail_reason.is_none());
        assert_eq!(result.outcomes.len(), 9);
        assert!(result.outcomes.iter().all(|o| o.ok));
        assert_eq!(result.outcomes[0].step_id, "wash_spin");
        assert_eq!(result.outcomes[3].step_id, "wash_wait");
        assert_eq!(result.outcomes[6].step_id, "dry_start");
        assert_eq!(result.outcomes[7].step_id, "dry_wait");

        assert_eq!(result.bindings.len(), 2);
        assert_eq!(result.bindings[0].role, "washer");
        assert_eq!(result.bindings[0].class_id, ApplianceClassId::Washer);
        assert!(result.bindings[0].spawned);
        assert!(result.bindings[0].device_id.starts_with("sim-washer-"));
        assert_eq!(result.bindings[1].role, "dryer");
        assert_eq!(result.bindings[1].class_id, ApplianceClassId::Dryer);
        assert!(result.bindings[1].spawned);
        assert!(result.bindings[1].device_id.starts_with("sim-dryer-"));

        let washer_state: serde_json::Value =
            serde_json::from_str(&api.get_state(&result.bindings[0].device_id).unwrap()).unwrap();
        let dryer_state: serde_json::Value =
            serde_json::from_str(&api.get_state(&result.bindings[1].device_id).unwrap()).unwrap();
        assert_eq!(washer_state["trait.cycle.cycle_state"]["value"], "complete");
        assert_eq!(dryer_state["trait.cycle.cycle_state"]["value"], "complete");
    }

    #[test]
    fn run_kettle_reuses_existing_matching_device() {
        let mut api = WasmApi::new();
        let existing = api.create_device("kettle").unwrap();
        let json = WasmApi::get_example_procedure("kettle_heat_80").unwrap();
        let result: ProcedureRunOut =
            serde_json::from_str(&api.run_procedure(&json).unwrap()).unwrap();
        assert_eq!(result.status, "completed");
        assert_eq!(result.bindings.len(), 1);
        assert_eq!(result.bindings[0].device_id, existing);
        assert!(!result.bindings[0].spawned);
        assert_eq!(
            serde_json::from_str::<Vec<DeviceInfo>>(&api.list_devices())
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn create_thermal_demo_lists_reservoirs_and_ports() {
        let mut api = WasmApi::new();
        let empty: ThermalStateOut = serde_json::from_str(&api.thermal_state().unwrap()).unwrap();
        assert!(!empty.loaded);
        assert!(empty.reservoirs.is_empty());

        let raw = api.create_thermal_demo().unwrap();
        let state: ThermalStateOut = serde_json::from_str(&raw).unwrap();
        assert!(state.loaded);
        assert_eq!(state.scenario.as_deref(), Some("fridge_condenser_dhw"));
        assert_eq!(state.reservoirs.len(), 1);
        assert_eq!(state.reservoirs[0].id, "dhw-tank");
        assert!((state.reservoirs[0].temp_c.unwrap() - 35.0).abs() < 1e-4);
        assert_eq!(state.ports.len(), 2);
        assert!(state
            .ports
            .iter()
            .any(|p| p.device_id == "fridge-kitchen" && p.port_id == "condenser"));
        assert!(state
            .ports
            .iter()
            .any(|p| p.device_id == "water-heater-plant" && p.port_id == "preheat"));
    }

    #[test]
    fn thermal_demo_transfer_raises_dhw_temp() {
        let mut api = WasmApi::new();
        api.create_thermal_demo().unwrap();
        let raw = api.thermal_demo_transfer(3_600.0).unwrap();
        let tick: ThermalTickOut = serde_json::from_str(&raw).unwrap();
        assert_eq!(tick.dt_s, 3_600.0);
        assert_eq!(tick.transfers.len(), 1);
        assert_eq!(tick.transfers[0].power_w, 120);
        assert!((tick.transfers[0].delta_temp_c - 1.2).abs() < 1e-4);
        assert_eq!(
            tick.transfers[0].heated_reservoir_id.as_deref(),
            Some("dhw-tank")
        );
        let dhw = tick
            .state
            .reservoirs
            .iter()
            .find(|r| r.id == "dhw-tank")
            .unwrap();
        assert!((dhw.temp_c.unwrap() - 36.2).abs() < 1e-4);

        // Separate negotiate + tick path matches one-shot.
        let mut api2 = WasmApi::new();
        api2.create_thermal_demo().unwrap();
        let after_offer: ThermalStateOut =
            serde_json::from_str(&api2.thermal_negotiate_demo().unwrap()).unwrap();
        assert!(after_offer.last_reply.as_ref().unwrap().is_accept());
        let tick2: ThermalTickOut =
            serde_json::from_str(&api2.thermal_tick(3_600.0).unwrap()).unwrap();
        assert_eq!(tick2.transfers.len(), 1);
        let dhw2 = tick2
            .state
            .reservoirs
            .iter()
            .find(|r| r.id == "dhw-tank")
            .unwrap();
        assert!((dhw2.temp_c.unwrap() - 36.2).abs() < 1e-4);
    }

    #[test]
    fn thermal_tick_without_plant_errors() {
        let mut api = WasmApi::new();
        let err = api.thermal_tick(1.0).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidRequest);
        assert!(err.message.contains("no thermal plant"));
    }

    #[test]
    fn run_thermal_then_dishwasher_preheat_raises_dhw_and_sets_eco() {
        let mut api = WasmApi::new();
        let raw = api.run_thermal_then_dishwasher_preheat(3_600.0).unwrap();
        let out: ThermalThenDishwasherOut = serde_json::from_str(&raw).unwrap();
        assert_eq!(out.scenario, "thermal_then_dishwasher_preheat");
        assert!((out.dhw_temp_start_c - 35.0).abs() < 1e-4);
        assert!((out.dhw_temp_end_c - 36.2).abs() < 1e-4);
        assert!(out.dhw_temp_end_c > out.dhw_temp_start_c);
        assert_eq!(out.thermal.transfers.len(), 1);
        assert_eq!(out.thermal.transfers[0].power_w, 120);
        assert_eq!(out.procedure.status, "completed");
        assert_eq!(out.procedure.outcomes.len(), 4);
        assert!(out.procedure.outcomes.iter().all(|o| o.ok));
        assert_eq!(out.procedure.bindings[0].role, "dishwasher");
        assert_eq!(
            out.procedure.bindings[0].class_id,
            ApplianceClassId::Dishwasher
        );

        let dw_id = &out.procedure.bindings[0].device_id;
        let state = api.get_state(dw_id).unwrap();
        assert!(state.contains("\"eco\"") || state.contains("eco"));
        assert!(state.contains("45") || state.contains("45.0"));
    }
}
