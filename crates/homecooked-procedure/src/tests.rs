use std::collections::HashMap;

use homecooked_schema::{typical_capability, ApplianceClassId, ErrorCode};
use homecooked_sim::Simulator;

use homecooked_thermal::{
    PortRef, PowerBandW, ThermalPlant, TransferOffer, TransferReply, TransferTarget,
};

use crate::{
    run, run_with_config, DeviceBindings, FailReason, Procedure, RunConfig, RunStatus,
    SimulatorBackend, StepAction, ThermalCmp, COFFEE_BREW_ESPRESSO_JSON,
    DISHWASHER_DHW_PREHEAT_JSON, KETTLE_HEAT_80_JSON, OVEN_BAKE_180_JSON,
    REHEAT_DOMINOS_MICROWAVE_JSON, WAIT_DHW_RESERVOIR_JSON, WASH_THEN_DRY_JSON,
};

fn kettle_bindings(sim: &mut Simulator) -> DeviceBindings {
    let id = sim.spawn(ApplianceClassId::Kettle).unwrap();
    DeviceBindings::new().bind("kettle", id.as_str())
}

fn microwave_bindings(sim: &mut Simulator) -> DeviceBindings {
    let id = sim.spawn(ApplianceClassId::Microwave).unwrap();
    DeviceBindings::new().bind("microwave", id.as_str())
}

fn oven_bindings(sim: &mut Simulator) -> DeviceBindings {
    let id = sim.spawn(ApplianceClassId::Oven).unwrap();
    DeviceBindings::new().bind("oven", id.as_str())
}

fn coffee_bindings(sim: &mut Simulator) -> DeviceBindings {
    let id = sim.spawn(ApplianceClassId::CoffeeMachine).unwrap();
    DeviceBindings::new().bind("coffee_machine", id.as_str())
}

fn laundry_bindings(sim: &mut Simulator) -> DeviceBindings {
    let washer = sim.spawn(ApplianceClassId::Washer).unwrap();
    let dryer = sim.spawn(ApplianceClassId::Dryer).unwrap();
    DeviceBindings::new()
        .bind("washer", washer.as_str())
        .bind("dryer", dryer.as_str())
}

#[test]
fn bundled_example_constants_parse() {
    let kettle = Procedure::load_json(crate::KETTLE_HEAT_80_JSON).unwrap();
    assert_eq!(kettle.id, "kettle_heat_80");
    assert_eq!(kettle.devices.len(), 1);
    assert_eq!(kettle.steps.len(), 4);

    let listed: Vec<&str> = crate::BUNDLED_EXAMPLE_PROCEDURES
        .iter()
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(
        listed,
        [
            "kettle_heat_80",
            "reheat_dominos_microwave",
            "wash_then_dry",
            "dishwasher_dhw_preheat",
            "oven_bake_180",
            "coffee_brew_espresso",
            "wait_dhw_reservoir",
        ]
    );
}

#[test]
fn parse_dominos_microwave_fixture() {
    let doc = Procedure::load_json(REHEAT_DOMINOS_MICROWAVE_JSON).unwrap();
    assert_eq!(doc.id, "reheat_dominos_microwave");
    assert_eq!(doc.name, "Reheat 2 Domino's supreme slices (microwave)");
    assert_eq!(doc.devices.len(), 3);
    assert!(
        doc.devices[1].optional,
        "crisp is optional in microwave-only v1"
    );
    assert!(doc.devices[2].optional, "fridge is optional");
    assert_eq!(doc.steps.len(), 5);
    assert_eq!(doc.steps[2].action, StepAction::Command);
    assert_eq!(doc.steps[2].id, "mw_start");
    assert_eq!(doc.steps[2].point(), Some("trait.cycle.start"));
    assert_eq!(doc.steps[3].action, StepAction::Wait);
    assert!(!doc.steps[3].guards().is_empty());

    let json = doc.to_json_string().unwrap();
    let back = Procedure::from_json_str(&json).unwrap();
    assert_eq!(back.id, doc.id);
    assert_eq!(back.steps.len(), doc.steps.len());
}

#[test]
fn parse_inline_kettle_and_alias_fields() {
    let doc = Procedure::load_json(KETTLE_HEAT_80_JSON).unwrap();
    assert_eq!(doc.steps[3].action, StepAction::Assert);
    assert_eq!(doc.steps[3].guards().len(), 1);
}

#[test]
fn parse_oven_bake_180_fixture() {
    let doc = Procedure::load_json(OVEN_BAKE_180_JSON).unwrap();
    assert_eq!(doc.id, "oven_bake_180");
    assert_eq!(doc.name, "Oven bake at 180C");
    assert_eq!(doc.devices.len(), 1);
    assert_eq!(doc.devices[0].role, "oven");
    assert_eq!(doc.steps.len(), 5);
    assert_eq!(doc.steps[0].id, "program");
    assert_eq!(doc.steps[0].point(), Some("trait.program.program"));
    assert_eq!(doc.steps[1].id, "setpoint");
    assert_eq!(doc.steps[2].action, StepAction::Command);
    assert_eq!(doc.steps[3].action, StepAction::Wait);
    assert_eq!(doc.steps[4].action, StepAction::Assert);
}

#[test]
fn oven_bake_180_happy_path_against_sim() {
    let doc = Procedure::load_json(OVEN_BAKE_180_JSON).unwrap();
    let mut sim = Simulator::new();
    let bindings = oven_bindings(&mut sim);
    let result = run(&doc, &mut sim, &bindings);
    assert!(
        result.is_completed(),
        "expected completed, got {:?}",
        result.status
    );
    assert_eq!(result.outcomes.len(), 5);
    assert!(result.outcomes.iter().all(|o| o.ok));

    let oven = bindings.get("oven").unwrap();
    let current = sim
        .read_value(
            &homecooked_core::DeviceId::new(oven),
            "trait.temperature.current_c",
        )
        .unwrap();
    let c = current.as_f64().expect("current_c numeric");
    assert!(c >= 170.0, "current_c={c}");
    let program = sim
        .read_value(
            &homecooked_core::DeviceId::new(oven),
            "trait.program.program",
        )
        .unwrap();
    assert_eq!(program, homecooked_schema::Value::Enum("bake".into()));
}

#[test]
fn parse_coffee_brew_espresso_fixture() {
    let doc = Procedure::load_json(COFFEE_BREW_ESPRESSO_JSON).unwrap();
    assert_eq!(doc.id, "coffee_brew_espresso");
    assert_eq!(doc.name, "Brew espresso");
    assert_eq!(doc.devices.len(), 1);
    assert_eq!(doc.devices[0].role, "coffee_machine");
    assert_eq!(doc.steps.len(), 5);
    assert_eq!(doc.steps[0].id, "power");
    assert_eq!(doc.steps[0].point(), Some("trait.power.power_on"));
    assert_eq!(doc.steps[1].id, "program");
    assert_eq!(doc.steps[1].point(), Some("trait.program.program"));
    assert_eq!(doc.steps[2].action, StepAction::Command);
    assert_eq!(doc.steps[3].action, StepAction::Wait);
    assert_eq!(doc.steps[4].action, StepAction::Assert);
}

#[test]
fn coffee_brew_espresso_happy_path_against_sim() {
    let doc = Procedure::load_json(COFFEE_BREW_ESPRESSO_JSON).unwrap();
    let mut sim = Simulator::new();
    let bindings = coffee_bindings(&mut sim);
    let result = run(&doc, &mut sim, &bindings);
    assert!(
        result.is_completed(),
        "expected completed, got {:?}",
        result.status
    );
    assert_eq!(result.outcomes.len(), 5);
    assert!(result.outcomes.iter().all(|o| o.ok));

    let coffee = bindings.get("coffee_machine").unwrap();
    let boiler = sim
        .read_value(
            &homecooked_core::DeviceId::new(coffee),
            "class.coffee_machine.boiler_c",
        )
        .unwrap();
    let c = boiler.as_f64().expect("boiler_c numeric");
    assert!(c >= 85.0, "boiler_c={c}");
    let program = sim
        .read_value(
            &homecooked_core::DeviceId::new(coffee),
            "trait.program.program",
        )
        .unwrap();
    assert_eq!(program, homecooked_schema::Value::Enum("espresso".into()));
}

#[test]
fn structural_validation_rejects_bad_docs() {
    let empty = Procedure::from_json_str(r#"{ "id": "x", "name": "n", "steps": [] }"#).unwrap();
    assert!(empty
        .validate()
        .unwrap_err()
        .to_string()
        .contains("at least one"));

    let dup = Procedure::from_json_str(
        r#"{
          "id": "x", "name": "n",
          "steps": [
            { "id": "a", "action": "wait", "timeout_s": 1 },
            { "id": "a", "action": "wait", "timeout_s": 1 }
          ]
        }"#,
    )
    .unwrap();
    let err = dup.validate().unwrap_err();
    assert!(err.to_string().contains("duplicate"));

    let write = Procedure::from_json_str(
        r#"{
          "id": "x", "name": "n",
          "steps": [{ "id": "w", "action": "write", "target": { "point": "trait.cycle.start" } }]
        }"#,
    )
    .unwrap();
    assert!(write.validate().unwrap_err().to_string().contains("write"));

    let wait = Procedure::from_json_str(
        r#"{ "id": "x", "name": "n", "steps": [{ "id": "w", "action": "wait" }] }"#,
    )
    .unwrap();
    assert!(wait.validate().unwrap_err().to_string().contains("wait"));

    let timeout = Procedure::from_json_str(
        r#"{
          "id": "x", "name": "n",
          "steps": [{ "id": "w", "action": "wait", "timeout_s": 0 }]
        }"#,
    )
    .unwrap();
    assert!(timeout
        .validate()
        .unwrap_err()
        .to_string()
        .contains("positive"));

    let assert_step = Procedure::from_json_str(
        r#"{ "id": "x", "name": "n", "steps": [{ "id": "a", "action": "assert" }] }"#,
    )
    .unwrap();
    assert!(assert_step
        .validate()
        .unwrap_err()
        .to_string()
        .contains("assert"));

    let read = Procedure::from_json_str(
        r#"{ "id": "x", "name": "n", "steps": [{ "id": "r", "action": "read" }] }"#,
    )
    .unwrap();
    assert!(read.validate().unwrap_err().to_string().contains("point"));
}

#[test]
fn capability_validate_rejects_out_of_range_write() {
    let doc = Procedure::load_json(
        r#"{
          "id": "bad_setpoint",
          "name": "too cold",
          "devices": [{ "role": "kettle", "class_id": "kettle" }],
          "steps": [{
            "id": "setpoint",
            "action": "write",
            "target": { "role": "kettle", "point": "trait.temperature.setpoint_c" },
            "value": { "type": "f32", "value": 20.0 }
          }]
        }"#,
    )
    .unwrap();
    let cap = typical_capability(ApplianceClassId::Kettle).unwrap();
    let mut caps = HashMap::new();
    caps.insert("kettle".to_string(), &cap);
    let err = doc.validate_with_capabilities(Some(&caps)).unwrap_err();
    match err {
        crate::Error::Capability(v) => assert_eq!(v.code, ErrorCode::OutOfRange),
        other => panic!("expected capability error, got {other}"),
    }
}

#[test]
fn kettle_happy_path_against_sim() {
    let doc = Procedure::load_json(KETTLE_HEAT_80_JSON).unwrap();
    let mut sim = Simulator::new();
    let bindings = kettle_bindings(&mut sim);
    let result = run(&doc, &mut sim, &bindings);
    assert!(
        result.is_completed(),
        "expected completed, got {:?}",
        result.status
    );
    assert_eq!(result.outcomes.len(), 4);
    assert!(result.outcomes.iter().all(|o| o.ok));

    let kettle = bindings.get("kettle").unwrap();
    let current = sim
        .read_value(
            &homecooked_core::DeviceId::new(kettle),
            "trait.temperature.current_c",
        )
        .unwrap();
    let c = current.as_f64().unwrap();
    assert!(c >= 75.0, "current_c={c}");
}

#[test]
fn kettle_assert_guard_fails_when_still_cold() {
    let doc = Procedure::load_json(
        r#"{
          "id": "cold",
          "name": "assert without heat",
          "steps": [{
            "id": "too_hot",
            "action": "assert",
            "target": { "role": "kettle" },
            "guards": [
              { "point": "trait.temperature.current_c", "gte": { "type": "f32", "value": 90.0 } }
            ]
          }]
        }"#,
    )
    .unwrap();
    let mut sim = Simulator::new();
    let bindings = kettle_bindings(&mut sim);
    let result = run(&doc, &mut sim, &bindings);
    match result.status {
        RunStatus::Failed { step_id, reason } => {
            assert_eq!(step_id, "too_hot");
            assert!(matches!(reason, FailReason::GuardFailed(_)));
        }
        other => panic!("expected guard failure, got {other:?}"),
    }
}

#[test]
fn kettle_write_out_of_range_fails_at_step() {
    let doc = Procedure::load_json(
        r#"{
          "id": "oor",
          "name": "out of range",
          "steps": [{
            "id": "setpoint",
            "action": "write",
            "target": { "role": "kettle", "point": "trait.temperature.setpoint_c" },
            "value": { "type": "f32", "value": 20.0 }
          }]
        }"#,
    )
    .unwrap();
    let mut sim = Simulator::new();
    let bindings = kettle_bindings(&mut sim);
    let result = run(&doc, &mut sim, &bindings);
    match result.status {
        RunStatus::Failed { step_id, reason } => {
            assert_eq!(step_id, "setpoint");
            match reason {
                FailReason::Backend { code, .. } => assert_eq!(code, ErrorCode::OutOfRange),
                other => panic!("expected backend out_of_range, got {other:?}"),
            }
        }
        other => panic!("expected failed run, got {other:?}"),
    }
}

#[test]
fn wait_timeout_when_guard_never_true() {
    let doc = Procedure::load_json(
        r#"{
          "id": "timeout",
          "name": "wait forever",
          "steps": [{
            "id": "wait_impossible",
            "action": "wait",
            "target": { "role": "kettle" },
            "timeout_s": 2,
            "guards": [
              { "point": "trait.temperature.current_c", "gte": { "type": "f32", "value": 200.0 } }
            ]
          }]
        }"#,
    )
    .unwrap();
    let mut sim = Simulator::new();
    let bindings = kettle_bindings(&mut sim);
    let result = run(&doc, &mut sim, &bindings);
    match result.status {
        RunStatus::Failed { step_id, reason } => {
            assert_eq!(step_id, "wait_impossible");
            assert_eq!(reason, FailReason::Timeout);
        }
        other => panic!("expected timeout, got {other:?}"),
    }
}

#[test]
fn microwave_fixture_validates_against_typical_caps() {
    let doc = Procedure::load_json(REHEAT_DOMINOS_MICROWAVE_JSON).unwrap();
    let cap = typical_capability(ApplianceClassId::Microwave).unwrap();
    let mut caps = HashMap::new();
    caps.insert("microwave".to_string(), &cap);
    doc.validate_with_capabilities(Some(&caps)).unwrap();
}

#[test]
fn dominos_microwave_completes_against_sim() {
    let doc = Procedure::load_json(REHEAT_DOMINOS_MICROWAVE_JSON).unwrap();
    let mut sim = Simulator::new();
    let bindings = microwave_bindings(&mut sim);
    let result = run(&doc, &mut sim, &bindings);
    assert!(
        result.is_completed(),
        "expected completed, got {:?}",
        result.status
    );
    assert!(result.outcomes.iter().all(|o| o.ok));
    let mw = bindings.get("microwave").unwrap();
    // cancel leaves cycle idle
    assert_eq!(
        sim.read_value(
            &homecooked_core::DeviceId::new(mw),
            "trait.cycle.cycle_state"
        )
        .unwrap(),
        homecooked_schema::Value::Enum("idle".into())
    );
}

#[test]
fn short_microwave_wait_fixture_completes() {
    let doc = Procedure::load_json(
        r#"{
          "id": "mw_short",
          "name": "short microwave",
          "devices": [{ "role": "microwave", "class_id": "microwave" }],
          "steps": [
            {
              "id": "cook",
              "action": "write",
              "target": { "role": "microwave", "point": "class.microwave.cook_s" },
              "value": { "type": "duration_s", "value": 3 }
            },
            {
              "id": "start",
              "action": "command",
              "target": { "role": "microwave", "point": "trait.cycle.start" },
              "value": { "type": "void" }
            },
            {
              "id": "wait",
              "action": "wait",
              "target": { "role": "microwave" },
              "timeout_s": 10,
              "guards": [
                { "point": "trait.cycle.elapsed_s", "gte": { "type": "duration_s", "value": 3 } }
              ]
            }
          ]
        }"#,
    )
    .unwrap();
    let mut sim = Simulator::new();
    let bindings = microwave_bindings(&mut sim);
    let result = run(&doc, &mut sim, &bindings);
    assert!(
        result.is_completed(),
        "expected completed, got {:?}",
        result.status
    );
}

#[test]
fn parse_wash_then_dry_fixture() {
    let doc = Procedure::load_json(WASH_THEN_DRY_JSON).unwrap();
    assert_eq!(doc.id, "wash_then_dry");
    assert_eq!(doc.devices.len(), 2);
    assert_eq!(doc.devices[0].role, "washer");
    assert_eq!(doc.devices[1].role, "dryer");
    assert!(!doc.devices[0].optional);
    assert!(!doc.devices[1].optional);
    assert_eq!(doc.steps.len(), 9);
    assert_eq!(doc.steps[3].action, StepAction::Wait);
    assert_eq!(doc.steps[3].id, "wash_wait");
    assert_eq!(doc.steps[7].id, "dry_wait");
}

#[test]
fn wash_then_dry_validates_against_typical_caps() {
    let doc = Procedure::load_json(WASH_THEN_DRY_JSON).unwrap();
    let washer = typical_capability(ApplianceClassId::Washer).unwrap();
    let dryer = typical_capability(ApplianceClassId::Dryer).unwrap();
    let mut caps = HashMap::new();
    caps.insert("washer".to_string(), &washer);
    caps.insert("dryer".to_string(), &dryer);
    doc.validate_with_capabilities(Some(&caps)).unwrap();
}

#[test]
fn wash_then_dry_completes_against_sim() {
    let doc = Procedure::load_json(WASH_THEN_DRY_JSON).unwrap();
    let mut sim = Simulator::new();
    let bindings = laundry_bindings(&mut sim);
    let result = run(&doc, &mut sim, &bindings);
    assert!(
        result.is_completed(),
        "expected completed, got {:?}",
        result.status
    );
    assert_eq!(result.outcomes.len(), 9);
    assert!(result.outcomes.iter().all(|o| o.ok));

    let washer = bindings.get("washer").unwrap();
    let dryer = bindings.get("dryer").unwrap();
    assert_eq!(
        sim.read_value(
            &homecooked_core::DeviceId::new(washer),
            "trait.cycle.cycle_state"
        )
        .unwrap(),
        homecooked_schema::Value::Enum("complete".into())
    );
    assert_eq!(
        sim.read_value(
            &homecooked_core::DeviceId::new(dryer),
            "trait.cycle.cycle_state"
        )
        .unwrap(),
        homecooked_schema::Value::Enum("complete".into())
    );
}

#[test]
fn parse_dishwasher_dhw_preheat_fixture() {
    let doc = Procedure::load_json(DISHWASHER_DHW_PREHEAT_JSON).unwrap();
    assert_eq!(doc.id, "dishwasher_dhw_preheat");
    assert_eq!(doc.devices.len(), 1);
    assert_eq!(doc.devices[0].role, "dishwasher");
    assert_eq!(doc.steps.len(), 4);
    assert_eq!(doc.steps[0].id, "program_eco");
    assert_eq!(doc.steps[1].id, "wash_temp_preheat");
    assert_eq!(doc.steps[2].action, StepAction::Assert);
    assert_eq!(doc.steps[3].action, StepAction::Assert);
}

#[test]
fn dishwasher_dhw_preheat_validates_against_typical_caps() {
    let doc = Procedure::load_json(DISHWASHER_DHW_PREHEAT_JSON).unwrap();
    let dw = typical_capability(ApplianceClassId::Dishwasher).unwrap();
    let mut caps = HashMap::new();
    caps.insert("dishwasher".to_string(), &dw);
    doc.validate_with_capabilities(Some(&caps)).unwrap();
}

#[test]
fn dishwasher_dhw_preheat_completes_against_sim() {
    let doc = Procedure::load_json(DISHWASHER_DHW_PREHEAT_JSON).unwrap();
    let mut sim = Simulator::new();
    let id = sim.spawn(ApplianceClassId::Dishwasher).unwrap();
    let bindings = DeviceBindings::new().bind("dishwasher", id.as_str());
    let result = run(&doc, &mut sim, &bindings);
    assert!(
        result.is_completed(),
        "expected completed, got {:?}",
        result.status
    );
    assert_eq!(result.outcomes.len(), 4);
    assert!(result.outcomes.iter().all(|o| o.ok));

    let program = sim
        .read_value(
            &homecooked_core::DeviceId::new(id.as_str()),
            "trait.program.program",
        )
        .unwrap();
    assert_eq!(program, homecooked_schema::Value::Enum("eco".into()));
    let wash_temp = sim
        .read_value(
            &homecooked_core::DeviceId::new(id.as_str()),
            "class.dishwasher.wash_temp_c",
        )
        .unwrap();
    assert_eq!(wash_temp, homecooked_schema::Value::F32(45.0));
}

fn demo_fridge_offer() -> TransferOffer {
    TransferOffer::new(
        PortRef::new("fridge-kitchen", "condenser").unwrap(),
        TransferTarget::port("water-heater-plant", "preheat").unwrap(),
        PowerBandW::new(80, 120).unwrap(),
        None,
        1,
    )
}

#[test]
fn parse_wait_dhw_reservoir_fixture() {
    let doc = Procedure::load_json(WAIT_DHW_RESERVOIR_JSON).unwrap();
    assert_eq!(doc.id, "wait_dhw_reservoir");
    assert!(doc.devices.is_empty());
    assert_eq!(doc.steps.len(), 1);
    assert_eq!(doc.steps[0].action, StepAction::ThermalWait);
    assert_eq!(doc.steps[0].reservoir_id(), Some("dhw-tank"));
    assert_eq!(doc.steps[0].cmp, Some(ThermalCmp::Gte));
    assert_eq!(doc.steps[0].temp_c, Some(36.0));
    assert_eq!(doc.steps[0].timeout_s, Some(7200));
}

#[test]
fn thermal_wait_alias_wait_reservoir_parses() {
    let raw = r#"{
      "id": "alias",
      "name": "alias",
      "steps": [{
        "id": "w",
        "op": "wait_reservoir",
        "reservoir_id": "dhw-tank",
        "cmp": "gte",
        "temp_c": 36.0,
        "timeout_s": 60
      }]
    }"#;
    let doc = Procedure::load_json(raw).unwrap();
    assert_eq!(doc.steps[0].action, StepAction::ThermalWait);
}

#[test]
fn thermal_wait_validation_requires_fields() {
    let missing = r#"{
      "id": "bad",
      "name": "bad",
      "steps": [{ "id": "w", "action": "thermal_wait", "timeout_s": 10 }]
    }"#;
    let err = Procedure::from_json_str(missing)
        .unwrap()
        .validate()
        .unwrap_err()
        .to_string();
    assert!(err.contains("reservoir_id"), "{err}");
}

#[test]
fn thermal_wait_fails_without_plant_backend() {
    let doc = Procedure::load_json(WAIT_DHW_RESERVOIR_JSON).unwrap();
    let mut sim = Simulator::new();
    let bindings = DeviceBindings::new();
    let result = run(&doc, &mut sim, &bindings);
    assert!(!result.is_completed());
    match result.fail_reason() {
        Some(FailReason::Backend { code, .. }) => {
            assert_eq!(*code, ErrorCode::UnsupportedOperation);
        }
        other => panic!("expected backend unsupported, got {other:?}"),
    }
}

#[test]
fn thermal_wait_succeeds_when_preseeded() {
    let doc = Procedure::load_json(WAIT_DHW_RESERVOIR_JSON).unwrap();
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    match plant.negotiate(demo_fridge_offer()) {
        TransferReply::Accept(_) => {}
        other => panic!("expected Accept, got {other:?}"),
    }
    plant.step(3_600.0).unwrap();
    let end = plant.get_reservoir("dhw-tank").unwrap().temp_c.unwrap();
    assert!((end - 36.2).abs() < 1e-3, "end={end}");

    let mut backend = SimulatorBackend::with_plant(Simulator::new(), plant);
    let result = run(&doc, &mut backend, &DeviceBindings::new());
    assert!(
        result.is_completed(),
        "expected completed, got {:?}",
        result.status
    );
    assert_eq!(result.outcomes.len(), 1);
    assert!(result.outcomes[0].ok);
    let got = result.outcomes[0]
        .read_value
        .as_ref()
        .and_then(|v| v.as_f64())
        .unwrap();
    assert!(got >= 36.0, "got={got}");
}

/// Plant accepts are applied once per `step` (not continuous). The wait loop
/// itself is covered here with a tiny rising-temp backend; real plant
/// integration uses the pre-seeded fixture test above.
struct RisingTempBackend {
    temp_c: f64,
    tick_delta: f64,
}

impl crate::DeviceBackend for RisingTempBackend {
    fn read(
        &mut self,
        _device_id: &str,
        _point_id: &str,
    ) -> Result<homecooked_schema::Value, crate::Error> {
        Err(crate::Error::Backend {
            code: ErrorCode::UnsupportedOperation,
            message: "device I/O unused".into(),
            point_id: None,
        })
    }

    fn write(
        &mut self,
        _device_id: &str,
        _point_id: &str,
        _value: &homecooked_schema::Value,
    ) -> Result<(), crate::Error> {
        Err(crate::Error::Backend {
            code: ErrorCode::UnsupportedOperation,
            message: "device I/O unused".into(),
            point_id: None,
        })
    }

    fn thermal_read_reservoir_temp(&mut self, reservoir_id: &str) -> Result<f64, crate::Error> {
        assert_eq!(reservoir_id, "dhw-tank");
        Ok(self.temp_c)
    }

    fn thermal_tick(&mut self, _dt_ms: u64) -> Result<(), crate::Error> {
        self.temp_c += self.tick_delta;
        Ok(())
    }
}

#[test]
fn thermal_wait_polls_until_cmp_via_thermal_tick() {
    let doc = Procedure::load_json(WAIT_DHW_RESERVOIR_JSON).unwrap();
    let mut backend = RisingTempBackend {
        temp_c: 35.0,
        tick_delta: 0.5,
    };
    let config = RunConfig {
        poll_interval_ms: 1_000,
    };
    let result = run_with_config(&doc, &mut backend, &DeviceBindings::new(), &config);
    assert!(
        result.is_completed(),
        "expected completed, got {:?}",
        result.status
    );
    assert!(backend.temp_c >= 36.0, "temp={}", backend.temp_c);
}

#[test]
fn thermal_wait_times_out_if_plant_idle() {
    let doc = Procedure::load_json(
        r#"{
      "id": "wait_hot",
      "name": "wait",
      "steps": [{
        "id": "w",
        "action": "thermal_wait",
        "reservoir_id": "dhw-tank",
        "cmp": "gte",
        "temp_c": 50.0,
        "timeout_s": 5
      }]
    }"#,
    )
    .unwrap();
    let plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    let mut backend = SimulatorBackend::with_plant(Simulator::new(), plant);
    let config = RunConfig {
        poll_interval_ms: 1_000,
    };
    let result = run_with_config(&doc, &mut backend, &DeviceBindings::new(), &config);
    assert_eq!(
        result.status,
        RunStatus::Failed {
            step_id: "w".into(),
            reason: FailReason::Timeout,
        }
    );
}
