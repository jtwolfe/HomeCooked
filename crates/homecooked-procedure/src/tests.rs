use std::collections::HashMap;

use homecooked_schema::{typical_capability, ApplianceClassId, ErrorCode};
use homecooked_sim::Simulator;

use crate::{
    run, DeviceBindings, FailReason, Procedure, RunStatus, StepAction,
    REHEAT_DOMINOS_MICROWAVE_JSON,
};

const KETTLE_HEAT_JSON: &str = r#"
{
  "id": "kettle_heat_80",
  "name": "Heat kettle to 80C",
  "catalog_version": "0.1.0",
  "steps": [
    {
      "id": "setpoint",
      "action": "write",
      "target": { "role": "kettle", "point": "trait.temperature.setpoint_c" },
      "value": { "type": "f32", "value": 80.0 }
    },
    {
      "id": "start",
      "action": "command",
      "target": { "role": "kettle", "point": "trait.cycle.start" },
      "value": { "type": "void" }
    },
    {
      "id": "wait_heat",
      "action": "wait",
      "target": { "role": "kettle" },
      "timeout_s": 20,
      "guards": [
        { "point": "trait.temperature.current_c", "gte": { "type": "f32", "value": 75.0 } }
      ]
    },
    {
      "id": "assert_temp",
      "op": "guard",
      "target": { "role": "kettle" },
      "guard": { "point": "trait.temperature.current_c", "gte": { "type": "f32", "value": 75.0 } }
    }
  ]
}
"#;

fn kettle_bindings(sim: &mut Simulator) -> DeviceBindings {
    let id = sim.spawn(ApplianceClassId::Kettle).unwrap();
    DeviceBindings::new().bind("kettle", id.as_str())
}

#[test]
fn parse_dominos_microwave_fixture() {
    let doc = Procedure::load_json(REHEAT_DOMINOS_MICROWAVE_JSON).unwrap();
    assert_eq!(doc.id, "reheat_dominos_microwave");
    assert_eq!(doc.name, "Reheat 2 Domino's supreme slices (microwave)");
    assert_eq!(doc.devices.len(), 3);
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
    let doc = Procedure::load_json(KETTLE_HEAT_JSON).unwrap();
    assert_eq!(doc.steps[3].action, StepAction::Assert);
    assert_eq!(doc.steps[3].guards().len(), 1);
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
    let doc = Procedure::load_json(KETTLE_HEAT_JSON).unwrap();
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
