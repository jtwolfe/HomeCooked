//! Thin in-browser conformance lab checks (Stream 7).
//!
//! Reimplements a **wasm-safe subset** of `homecooked-conformance` scenarios
//! using schema / sim / procedure / thermal only — **no** bridge, TCP, hub, or
//! controller. Full suite remains `cargo test -p homecooked-conformance`.
//!
//! Catalog JSON: [`docs/conformance/scenarios.json`](../../../docs/conformance/scenarios.json).

use homecooked_core::DeviceId;
use homecooked_procedure::{run, DeviceBindings, Procedure, KETTLE_HEAT_80_JSON};
use homecooked_schema::{
    class_table, static_class_tables, trait_table, typical_capability, ApplianceClassId, ErrorCode,
    TraitId, Value, STATIC_CLASS_IDS, TIER_A_CLASS_IDS, TIER_B_CLASS_IDS,
};
use homecooked_sim::Simulator;
use homecooked_thermal::{
    energy_kwh, PortRef, PowerBandW, ThermalPlant, TransferOffer, TransferReply, TransferTarget,
};
use serde::{Deserialize, Serialize};

use crate::api::ApiError;

const SCENARIOS_JSON: &str = include_str!("../../../docs/conformance/scenarios.json");

/// One row from the checked-in scenario catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioInfo {
    pub name: String,
    pub tags: Vec<String>,
    pub native_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// Result of [`run_conformance_lab_check`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabCheckResult {
    pub name: String,
    pub passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// True when the scenario exists but is listed `native_only` (not run).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub native_only: bool,
}

fn fail(name: &str, message: impl Into<String>) -> LabCheckResult {
    LabCheckResult {
        name: name.to_string(),
        passed: false,
        message: Some(message.into()),
        native_only: false,
    }
}

fn pass(name: &str) -> LabCheckResult {
    LabCheckResult {
        name: name.to_string(),
        passed: true,
        message: None,
        native_only: false,
    }
}

/// Parse + return the scenario catalog as a JSON array.
pub fn list_conformance_scenarios() -> String {
    // Validate once at call time so bad JSON fails loudly in tests / boot.
    let rows: Vec<ScenarioInfo> =
        serde_json::from_str(SCENARIOS_JSON).expect("docs/conformance/scenarios.json parses");
    serde_json::to_string(&rows).expect("ScenarioInfo serializes")
}

fn catalog_rows() -> Vec<ScenarioInfo> {
    serde_json::from_str(SCENARIOS_JSON).expect("docs/conformance/scenarios.json parses")
}

fn lookup(name: &str) -> Option<ScenarioInfo> {
    catalog_rows().into_iter().find(|s| s.name == name)
}

/// Run one in-process lab check by scenario name.
///
/// Native-only names return `{ passed: false, native_only: true, message: hint }`.
/// Unknown names return [`ApiError`].
pub fn run_conformance_lab_check(name: &str) -> Result<String, ApiError> {
    let Some(info) = lookup(name) else {
        return Err(ApiError::invalid_request(format!(
            "unknown conformance scenario {name:?}"
        )));
    };

    if info.native_only {
        let out = LabCheckResult {
            name: info.name,
            passed: false,
            message: Some(
                "native_only — run `cargo test -p homecooked-conformance` (TCP / bridge / controller / hub)"
                    .into(),
            ),
            native_only: true,
        };
        return Ok(serde_json::to_string(&out).expect("LabCheckResult serializes"));
    }

    let result = match name {
        "catalog_hygiene" => check_catalog_hygiene(),
        "tier_a_catalog_sim_describe" => check_tier_describe(TIER_A_CLASS_IDS, 25, name),
        "tier_b_catalog_sim_describe" => check_tier_describe(TIER_B_CLASS_IDS, 31, name),
        "write_denial_matrix" => check_write_denial_matrix(),
        "water_heater_thermal_ports" => check_water_heater_thermal_ports(),
        "procedure_kettle_happy_path" => check_procedure_kettle_happy_path(),
        "thermal_fridge_dhw_demo" => check_thermal_fridge_dhw_demo(),
        other => fail(
            other,
            "listed runnable but no wasm lab-check implementation (bug)",
        ),
    };
    Ok(serde_json::to_string(&result).expect("LabCheckResult serializes"))
}

fn check_catalog_hygiene() -> LabCheckResult {
    const NAME: &str = "catalog_hygiene";
    if STATIC_CLASS_IDS.len() != static_class_tables().len() {
        return fail(
            NAME,
            format!(
                "STATIC_CLASS_IDS len {} != static_class_tables len {}",
                STATIC_CLASS_IDS.len(),
                static_class_tables().len()
            ),
        );
    }

    for &id in STATIC_CLASS_IDS {
        let Some(table) = class_table(id) else {
            return fail(
                NAME,
                format!("STATIC_CLASS_ID {} has no ClassTable", id.as_str()),
            );
        };
        if table.class_id != id {
            return fail(
                NAME,
                format!(
                    "ClassTable class_id {} != STATIC_CLASS_ID {}",
                    table.class_id.as_str(),
                    id.as_str()
                ),
            );
        }
        let mut seen = std::collections::BTreeSet::new();
        for p in table.class_points {
            if !seen.insert(p.id) {
                return fail(
                    NAME,
                    format!("duplicate class point id {}.{}", id.as_str(), p.id),
                );
            }
        }
    }

    for &trait_id in TraitId::ALL {
        let Some(table) = trait_table(trait_id) else {
            return fail(
                NAME,
                format!("TraitId {} has no TraitTable", trait_id.as_str()),
            );
        };
        let mut seen = std::collections::BTreeSet::new();
        for p in table.points {
            if !seen.insert(p.id) {
                return fail(
                    NAME,
                    format!(
                        "duplicate trait point id trait.{}.{}",
                        trait_id.as_str(),
                        p.id
                    ),
                );
            }
        }
    }

    pass(NAME)
}

fn check_tier_describe(
    ids: &[ApplianceClassId],
    expected_len: usize,
    name: &str,
) -> LabCheckResult {
    if ids.len() != expected_len {
        return fail(
            name,
            format!("expected {expected_len} ids, got {}", ids.len()),
        );
    }
    let mut sim = Simulator::new();
    for &class_id in ids {
        let Some(cap) = typical_capability(class_id) else {
            return fail(
                name,
                format!("typical_capability missing for {}", class_id.as_str()),
            );
        };
        if cap.class_id != class_id {
            return fail(
                name,
                format!(
                    "capability class_id mismatch for {}: got {}",
                    class_id.as_str(),
                    cap.class_id.as_str()
                ),
            );
        }
        let id = match sim.spawn(class_id) {
            Ok(id) => id,
            Err(e) => {
                return fail(
                    name,
                    format!("sim spawn failed for {}: {e}", class_id.as_str()),
                );
            }
        };
        let Some(dev) = sim.hub().registry.get(&id) else {
            return fail(
                name,
                format!("spawned {} missing from registry", class_id.as_str()),
            );
        };
        if dev.capability.class_id != class_id {
            return fail(
                name,
                format!(
                    "describe class_id for {}: got {}",
                    class_id.as_str(),
                    dev.capability.class_id.as_str()
                ),
            );
        }
    }
    pass(name)
}

fn check_write_denial_matrix() -> LabCheckResult {
    const NAME: &str = "write_denial_matrix";
    let cases: &[(&str, ApplianceClassId, &str, Value, ErrorCode)] = &[
        (
            "kettle_setpoint_out_of_range",
            ApplianceClassId::Kettle,
            "trait.temperature.setpoint_c",
            Value::F32(20.0),
            ErrorCode::OutOfRange,
        ),
        (
            "oven_setpoint_out_of_range",
            ApplianceClassId::Oven,
            "trait.temperature.setpoint_c",
            Value::F32(300.0),
            ErrorCode::OutOfRange,
        ),
        (
            "unknown_point_id",
            ApplianceClassId::Kettle,
            "trait.temperature.not_a_real_point",
            Value::F32(80.0),
            ErrorCode::UnknownVariable,
        ),
        (
            "read_only_current_c",
            ApplianceClassId::Kettle,
            "trait.temperature.current_c",
            Value::F32(55.0),
            ErrorCode::NotWritable,
        ),
        (
            "bad_enum_token_program",
            ApplianceClassId::Washer,
            "trait.program.program",
            Value::Enum("not_a_program".into()),
            ErrorCode::InvalidEnum,
        ),
        (
            "wrong_type_bool_into_f32_setpoint",
            ApplianceClassId::Kettle,
            "trait.temperature.setpoint_c",
            Value::Bool(true),
            ErrorCode::InvalidType,
        ),
        (
            "class_lacks_foreign_point",
            ApplianceClassId::Kettle,
            "class.washer.spin_rpm",
            Value::U16(800),
            ErrorCode::UnsupportedCapability,
        ),
    ];

    let mut failures = Vec::new();
    for &(case, class_id, point_id, ref value, expected) in cases {
        let mut sim = Simulator::new();
        let id = match sim.spawn(class_id) {
            Ok(id) => id,
            Err(e) => {
                failures.push(format!("{case}: spawn {}: {e}", class_id.as_str()));
                continue;
            }
        };
        match sim.write(&id, point_id, value.clone()) {
            Ok(_) => failures.push(format!(
                "{case}: expected {expected}, write succeeded on {}",
                class_id.as_str()
            )),
            Err(e) if e.code == expected => {}
            Err(e) => failures.push(format!(
                "{case}: got {}, expected {expected} ({})",
                e.code, e.message
            )),
        }
    }

    if failures.is_empty() {
        pass(NAME)
    } else {
        fail(
            NAME,
            format!(
                "{} case(s) failed:\n  - {}",
                failures.len(),
                failures.join("\n  - ")
            ),
        )
    }
}

fn check_procedure_kettle_happy_path() -> LabCheckResult {
    const NAME: &str = "procedure_kettle_happy_path";
    let doc = match Procedure::load_json(KETTLE_HEAT_80_JSON) {
        Ok(d) => d,
        Err(e) => return fail(NAME, format!("load procedure: {e}")),
    };
    let mut sim = Simulator::new();
    let id = match sim.spawn(ApplianceClassId::Kettle) {
        Ok(id) => id,
        Err(e) => return fail(NAME, format!("spawn kettle: {e}")),
    };
    let bindings = DeviceBindings::new().bind("kettle", id.as_str());
    let result = run(&doc, &mut sim, &bindings);
    if !result.is_completed() {
        return fail(NAME, format!("expected completed, got {:?}", result.status));
    }
    let current = match sim.read_value(&DeviceId::new(id.as_str()), "trait.temperature.current_c") {
        Ok(v) => v,
        Err(e) => return fail(NAME, format!("read current_c: {e}")),
    };
    let Some(c) = current.as_f64() else {
        return fail(NAME, format!("current_c not numeric: {current:?}"));
    };
    if c < 75.0 {
        return fail(NAME, format!("current_c={c}, expected >= 75"));
    }
    pass(NAME)
}

fn check_thermal_fridge_dhw_demo() -> LabCheckResult {
    const NAME: &str = "thermal_fridge_dhw_demo";
    let mut plant = match ThermalPlant::fridge_condenser_dhw_demo() {
        Ok(p) => p,
        Err(e) => return fail(NAME, format!("demo plant: {e}")),
    };
    if plant.get_port("fridge-kitchen", "condenser").is_none() {
        return fail(NAME, "missing fridge condenser port");
    }
    let Some(preheat) = plant.get_port("water-heater-plant", "preheat") else {
        return fail(NAME, "missing water-heater preheat port");
    };
    if preheat.attached_reservoir_id.as_deref() != Some("dhw-tank") {
        return fail(
            NAME,
            format!(
                "preheat reservoir={:?}, expected dhw-tank",
                preheat.attached_reservoir_id
            ),
        );
    }
    let Some(start) = plant.get_reservoir("dhw-tank").and_then(|r| r.temp_c) else {
        return fail(NAME, "dhw-tank missing temp");
    };
    if (start - 35.0).abs() >= 1e-4 {
        return fail(NAME, format!("dhw start temp={start}, expected 35"));
    }

    let offer = match (|| {
        Ok::<_, String>(TransferOffer::new(
            PortRef::new("fridge-kitchen", "condenser").map_err(|e| e.to_string())?,
            TransferTarget::port("water-heater-plant", "preheat").map_err(|e| e.to_string())?,
            PowerBandW::new(80, 120).map_err(|e| e.to_string())?,
            None,
            1,
        ))
    })() {
        Ok(o) => o,
        Err(e) => return fail(NAME, e),
    };
    match plant.negotiate(offer) {
        TransferReply::Accept(a) => {
            if a.accepted_power_w != 120 {
                return fail(
                    NAME,
                    format!("accepted_power_w={}, expected 120", a.accepted_power_w),
                );
            }
        }
        other => return fail(NAME, format!("expected Accept, got {other:?}")),
    }

    let results = match plant.step(3_600.0) {
        Ok(r) => r,
        Err(e) => return fail(NAME, format!("step: {e}")),
    };
    if results.len() != 1 {
        return fail(
            NAME,
            format!("expected 1 transfer result, got {}", results.len()),
        );
    }
    if results[0].power_w != 120 {
        return fail(
            NAME,
            format!("power_w={}, expected 120", results[0].power_w),
        );
    }
    let expected = energy_kwh(120, 3_600.0);
    if (results[0].energy_kwh - expected).abs() >= 1e-6 {
        return fail(
            NAME,
            format!("energy_kwh={}, expected {expected}", results[0].energy_kwh),
        );
    }
    if (results[0].delta_temp_c - 1.2).abs() >= 1e-4 {
        return fail(
            NAME,
            format!("delta_temp_c={}, expected 1.2", results[0].delta_temp_c),
        );
    }
    let Some(end) = plant.get_reservoir("dhw-tank").and_then(|r| r.temp_c) else {
        return fail(NAME, "dhw-tank missing temp after step");
    };
    if (end - 36.2).abs() >= 1e-4 {
        return fail(NAME, format!("dhw end temp={end}, expected 36.2"));
    }
    pass(NAME)
}

fn check_water_heater_thermal_ports() -> LabCheckResult {
    const NAME: &str = "water_heater_thermal_ports";
    let mut sim = Simulator::new();

    let id = match sim.spawn(ApplianceClassId::WaterHeater) {
        Ok(id) => id,
        Err(e) => return fail(NAME, format!("spawn water_heater: {e}")),
    };
    for (point, expected) in [
        (
            "class.water_heater.thermal_port_direction",
            Value::Enum("sink".into()),
        ),
        (
            "class.water_heater.thermal_port_media",
            Value::Enum("water".into()),
        ),
        (
            "class.water_heater.thermal_port_max_power_w",
            Value::F32(2_000.0),
        ),
    ] {
        match sim.read_value(&id, point) {
            Ok(v) if v == expected => {}
            Ok(v) => return fail(NAME, format!("{point}={v:?}, expected {expected:?}")),
            Err(e) => return fail(NAME, format!("read {point}: {e}")),
        }
    }
    if let Err(e) = sim.write(
        &id,
        "class.water_heater.thermal_port_attached_reservoir_id",
        Value::String("dhw-tank".into()),
    ) {
        return fail(NAME, format!("write water_heater attach: {e}"));
    }

    let fridge = match sim.spawn(ApplianceClassId::Fridge) {
        Ok(id) => id,
        Err(e) => return fail(NAME, format!("spawn fridge: {e}")),
    };
    match sim.read_value(&fridge, "class.fridge.thermal_port_direction") {
        Ok(Value::Enum(ref d)) if d == "source" => {}
        Ok(v) => return fail(NAME, format!("fridge direction={v:?}, expected source")),
        Err(e) => return fail(NAME, format!("read fridge direction: {e}")),
    }
    if let Err(e) = sim.write(
        &fridge,
        "class.fridge.thermal_port_attached_reservoir_id",
        Value::String("dhw-tank".into()),
    ) {
        return fail(NAME, format!("write fridge attach: {e}"));
    }

    // HVAC / dishwasher / dryer: seed + attach smoke (same assertions as native suite).
    let hvac = match sim.spawn(ApplianceClassId::Hvac) {
        Ok(id) => id,
        Err(e) => return fail(NAME, format!("spawn hvac: {e}")),
    };
    match sim.read_value(&hvac, "class.hvac.thermal_port_id") {
        Ok(Value::String(ref s)) if s == "coil" => {}
        Ok(v) => return fail(NAME, format!("hvac port_id={v:?}, expected coil")),
        Err(e) => return fail(NAME, format!("read hvac port_id: {e}")),
    }
    if let Err(e) = sim.write(
        &hvac,
        "class.hvac.thermal_port_attached_reservoir_id",
        Value::String("chw-buffer".into()),
    ) {
        return fail(NAME, format!("write hvac attach: {e}"));
    }

    let dw = match sim.spawn(ApplianceClassId::Dishwasher) {
        Ok(id) => id,
        Err(e) => return fail(NAME, format!("spawn dishwasher: {e}")),
    };
    match sim.read_value(&dw, "class.dishwasher.thermal_port_id") {
        Ok(Value::String(ref s)) if s == "inlet_preheat" => {}
        Ok(v) => {
            return fail(
                NAME,
                format!("dishwasher port_id={v:?}, expected inlet_preheat"),
            )
        }
        Err(e) => return fail(NAME, format!("read dishwasher port_id: {e}")),
    }
    if let Err(e) = sim.write(
        &dw,
        "class.dishwasher.thermal_port_attached_reservoir_id",
        Value::String("dhw-tank".into()),
    ) {
        return fail(NAME, format!("write dishwasher attach: {e}"));
    }

    let dryer = match sim.spawn(ApplianceClassId::Dryer) {
        Ok(id) => id,
        Err(e) => return fail(NAME, format!("spawn dryer: {e}")),
    };
    match sim.read_value(&dryer, "class.dryer.thermal_port_id") {
        Ok(Value::String(ref s)) if s == "exhaust" => {}
        Ok(v) => return fail(NAME, format!("dryer port_id={v:?}, expected exhaust")),
        Err(e) => return fail(NAME, format!("read dryer port_id: {e}")),
    }
    if let Err(e) = sim.write(
        &dryer,
        "class.dryer.thermal_port_attached_reservoir_id",
        Value::String("air-buffer".into()),
    ) {
        return fail(NAME, format!("write dryer attach: {e}"));
    }

    pass(NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_conformance_scenarios_is_complete() {
        let rows: Vec<ScenarioInfo> = serde_json::from_str(&list_conformance_scenarios()).unwrap();
        assert_eq!(rows.len(), 36);
        let runnable: Vec<_> = rows.iter().filter(|r| !r.native_only).collect();
        assert_eq!(runnable.len(), 7);
        assert!(rows.iter().any(|r| r.name == "catalog_hygiene"));
        assert!(rows
            .iter()
            .any(|r| r.name == "hub_lab_set_discover_describe"
                && r.native_only
                && r.tags.iter().any(|t| t == "hub")));
        assert!(rows.iter().any(|r| r.name == "modbus_tcp_water_heater_lab"
            && r.native_only
            && r.tags.iter().any(|t| t == "tcp")));
    }

    #[test]
    fn lab_check_catalog_hygiene_passes() {
        let raw = run_conformance_lab_check("catalog_hygiene").unwrap();
        let out: LabCheckResult = serde_json::from_str(&raw).unwrap();
        assert!(out.passed, "{out:?}");
        assert_eq!(out.name, "catalog_hygiene");
    }

    #[test]
    fn lab_check_write_denial_matrix_passes() {
        let raw = run_conformance_lab_check("write_denial_matrix").unwrap();
        let out: LabCheckResult = serde_json::from_str(&raw).unwrap();
        assert!(out.passed, "{out:?}");
    }

    #[test]
    fn lab_check_kettle_and_tier_and_thermal() {
        for name in [
            "tier_a_catalog_sim_describe",
            "procedure_kettle_happy_path",
            "thermal_fridge_dhw_demo",
            "water_heater_thermal_ports",
        ] {
            let raw = run_conformance_lab_check(name).unwrap();
            let out: LabCheckResult = serde_json::from_str(&raw).unwrap();
            assert!(out.passed, "{name}: {out:?}");
        }
    }

    #[test]
    fn lab_check_native_only_hint() {
        let raw = run_conformance_lab_check("tcp_kettle_discover_describe_read_write").unwrap();
        let out: LabCheckResult = serde_json::from_str(&raw).unwrap();
        assert!(!out.passed);
        assert!(out.native_only);
        assert!(out.message.as_deref().unwrap_or("").contains("cargo test"));
    }

    #[test]
    fn lab_check_unknown_errors() {
        let err = run_conformance_lab_check("not_a_scenario").unwrap_err();
        assert!(err.message.contains("unknown"));
    }
}
