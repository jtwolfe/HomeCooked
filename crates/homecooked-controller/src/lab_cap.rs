//! Build lab washer/dryer advertised capabilities from catalog
//! [`typical_capability`], then merge lab-only HAL / sim_tick / cycle extras.
//!
//! Full HAL binding for every typical point is **not** required: endpoints
//! store last-write / return stable defaults for points the host does not
//! drive yet.

use std::collections::BTreeMap;

use homecooked_schema::{
    class_table, trait_table, typical_capability, AccessMode, ApplianceClassId, CapabilityModel,
    CommandArg, PointCapability, TraitCapability, TraitId, Value, ValueRange, ValueType,
    DEFAULT_CLASS_VERSION,
};

/// Cycle points the lab TCP path already wires (pause/resume/phase) that
/// catalog typical omits because they are optional.
const LAB_CYCLE_EXTRAS: &[&str] = &["pause", "resume", "cycle_phase"];

/// Start from catalog typical, then merge lab-only points the host still needs.
pub fn lab_washer_capability() -> CapabilityModel {
    let mut cap = typical_capability(ApplianceClassId::Washer).expect("washer typical");
    ensure_cycle_lab_extras(&mut cap);
    push_lab_hal_washer(&mut cap);
    push_sim_tick(&mut cap, "class.washer.sim_tick");
    cap
}

/// Start from catalog typical, then merge lab-only points (incl. DryOptions
/// setpoints typical does not advertise).
pub fn lab_dryer_capability() -> CapabilityModel {
    let mut cap = typical_capability(ApplianceClassId::Dryer).expect("dryer typical");
    ensure_cycle_lab_extras(&mut cap);
    push_lab_hal_dryer(&mut cap);
    // dryness / heat_level are optional catalog points omitted from typical;
    // lab DryOptions-over-TCP still needs them advertised.
    let dryer = class_table(ApplianceClassId::Dryer).expect("dryer class table");
    push_class_point_if_missing(
        &mut cap,
        PointCapability::from_catalog(
            "class.dryer.dryness",
            dryer.class_point("dryness").expect("dryer dryness"),
        ),
    );
    push_class_point_if_missing(
        &mut cap,
        PointCapability::from_catalog(
            "class.dryer.heat_level",
            dryer.class_point("heat_level").expect("dryer heat_level"),
        ),
    );
    push_sim_tick(&mut cap, "class.dryer.sim_tick");
    cap
}

fn ensure_cycle_lab_extras(cap: &mut CapabilityModel) {
    let cycle = trait_table(TraitId::Cycle).expect("cycle trait table");
    let trait_cap = match cap.traits.iter_mut().find(|t| t.trait_id == TraitId::Cycle) {
        Some(t) => t,
        None => {
            cap.traits.push(TraitCapability {
                trait_id: TraitId::Cycle,
                trait_version: DEFAULT_CLASS_VERSION,
                points: Vec::new(),
            });
            cap.traits.last_mut().unwrap()
        }
    };
    for id in LAB_CYCLE_EXTRAS {
        let qid = format!("trait.cycle.{id}");
        if trait_cap.points.iter().any(|p| p.id == qid) {
            continue;
        }
        let p = cycle
            .point(id)
            .unwrap_or_else(|| panic!("missing trait.cycle.{id}"));
        trait_cap.points.push(PointCapability::from_catalog(qid, p));
    }
}

fn push_sim_tick(cap: &mut CapabilityModel, id: &str) {
    push_class_point_if_missing(
        cap,
        PointCapability {
            id: id.into(),
            value_type: ValueType::Command,
            unit: None,
            access: AccessMode::W,
            required: false,
            range: Some(ValueRange::CommandArg {
                arg: CommandArg::Void,
            }),
            resolution: None,
            zones: None,
        },
    );
}

fn push_lab_hal_washer(cap: &mut CapabilityModel) {
    for (id, value_type, access) in [
        (
            "class.washer.heater_enable",
            ValueType::Bool,
            AccessMode::RW,
        ),
        ("class.washer.door_lock", ValueType::Bool, AccessMode::RW),
        (
            "class.washer.water_level_pa",
            ValueType::F32,
            AccessMode::RW,
        ),
        ("class.washer.door_lock_fb", ValueType::Bool, AccessMode::R),
    ] {
        push_class_point_if_missing(
            cap,
            PointCapability {
                id: id.into(),
                value_type,
                unit: None,
                access,
                required: true,
                range: None,
                resolution: None,
                zones: None,
            },
        );
    }
}

fn push_lab_hal_dryer(cap: &mut CapabilityModel) {
    for (id, value_type, access) in [
        ("class.dryer.heater_enable", ValueType::Bool, AccessMode::RW),
        ("class.dryer.door_lock", ValueType::Bool, AccessMode::RW),
        ("class.dryer.blower", ValueType::Bool, AccessMode::RW),
        ("class.dryer.door_lock_fb", ValueType::Bool, AccessMode::R),
    ] {
        push_class_point_if_missing(
            cap,
            PointCapability {
                id: id.into(),
                value_type,
                unit: None,
                access,
                required: true,
                range: None,
                resolution: None,
                zones: None,
            },
        );
    }
}

fn push_class_point_if_missing(cap: &mut CapabilityModel, point: PointCapability) {
    if cap.class_points.iter().any(|p| p.id == point.id) {
        return;
    }
    cap.class_points.push(point);
}

/// Stable default for an advertised point the host does not drive yet.
pub fn stable_default(point: &PointCapability) -> Value {
    let seg = point
        .id
        .rsplit('.')
        .next()
        .unwrap_or(point.id.as_str())
        .split('#')
        .next()
        .unwrap_or(point.id.as_str());
    match seg {
        "power_state" => Value::Enum("on".into()),
        "cycle_state" => Value::Enum("idle".into()),
        "cycle_phase" => Value::String("idle".into()),
        "door_state" => Value::Enum("closed".into()),
        "link_state" => Value::Enum("online".into()),
        "transport" => Value::Enum("ip".into()),
        "motor_state" | "heater_state" | "fan_state" => Value::Enum("off".into()),
        "fault_present" => Value::Bool(false),
        "interlock_ok" => Value::Bool(true),
        "remote_control_enabled" => Value::Bool(true),
        "child_lock" | "sound_enable" | "sabbath_mode" | "eco_mode" => Value::Bool(false),
        "program" => first_enum(point).unwrap_or_else(|| Value::Enum("cotton".into())),
        "available_programs" => enum_list(point),
        "wash_temp_c" => Value::F32(40.0),
        "spin_rpm" => Value::U16(800),
        "dryness" => Value::Enum("cupboard".into()),
        "heat_level" => Value::Enum("medium".into()),
        "current_c" => Value::F32(20.0),
        "detergent_level_percent" | "dryness_percent" => Value::Percent(80.0),
        "progress_percent" => Value::Percent(0.0),
        "remaining_s" | "elapsed_s" | "timer_s" | "delay_start_s" => Value::DurationS(0),
        "device_id" => Value::String("lab".into()),
        "manufacturer" => Value::String("HomeCooked".into()),
        "model" => Value::String("lab".into()),
        "fw_version" => Value::String("0.1.0".into()),
        "class_id" => first_enum(point).unwrap_or_else(|| Value::Enum("unknown".into())),
        "catalog_version" | "protocol_version" => {
            Value::String(homecooked_schema::CATALOG_VERSION.to_string())
        }
        _ => generic_default(point),
    }
}

fn first_enum(point: &PointCapability) -> Option<Value> {
    match &point.range {
        Some(ValueRange::Enum { tokens }) => tokens.first().map(|t| Value::Enum(t.clone())),
        _ => None,
    }
}

fn enum_list(point: &PointCapability) -> Value {
    match &point.range {
        Some(ValueRange::List {
            item: Some(inner), ..
        }) => {
            if let ValueRange::Enum { tokens } = inner.as_ref() {
                return Value::List(tokens.iter().cloned().map(Value::Enum).collect());
            }
        }
        Some(ValueRange::Enum { tokens }) => {
            return Value::List(tokens.iter().cloned().map(Value::Enum).collect());
        }
        _ => {}
    }
    Value::List(Vec::new())
}

fn generic_default(point: &PointCapability) -> Value {
    match point.value_type {
        ValueType::Bool => Value::Bool(false),
        ValueType::U8 => Value::U8(int_min(point) as u8),
        ValueType::U16 => Value::U16(int_min(point) as u16),
        ValueType::U32 => Value::U32(int_min(point) as u32),
        ValueType::I16 => Value::I16(int_min(point) as i16),
        ValueType::I32 => Value::I32(int_min(point) as i32),
        ValueType::F32 => Value::F32(numeric_default(point)),
        ValueType::Percent => Value::Percent(0.0),
        ValueType::Enum => first_enum(point).unwrap_or_else(|| Value::Enum("unknown".into())),
        ValueType::String => Value::String(String::new()),
        ValueType::TimestampMs => Value::TimestampMs(0),
        ValueType::DurationS => Value::DurationS(0),
        ValueType::List(_) => Value::List(Vec::new()),
        ValueType::Command => Value::Void,
    }
}

fn int_min(point: &PointCapability) -> i64 {
    match &point.range {
        Some(ValueRange::Integer { min, .. }) => (*min).max(0),
        Some(ValueRange::Numeric { min, .. }) => (*min as i64).max(0),
        _ => 0,
    }
}

fn numeric_default(point: &PointCapability) -> f32 {
    match &point.range {
        Some(ValueRange::Numeric { min, .. }) if *min > 0.0 => *min as f32,
        Some(ValueRange::Integer { min, .. }) if *min > 0 => *min as f32,
        _ => 0.0,
    }
}

/// Seed readable advertised points with stable defaults; override identity.
pub fn seed_identity_store(
    store: &mut BTreeMap<String, Value>,
    capability: &CapabilityModel,
    device_id: &str,
    class_id: ApplianceClassId,
) {
    for point in capability.iter_points() {
        if point.value_type == ValueType::Command {
            continue;
        }
        if !point.access.is_readable() {
            continue;
        }
        store
            .entry(point.id.clone())
            .or_insert_with(|| stable_default(point));
    }
    store.insert(
        "trait.identity.device_id".into(),
        Value::String(device_id.to_string()),
    );
    store.insert(
        "trait.identity.manufacturer".into(),
        Value::String("HomeCooked".into()),
    );
    store.insert(
        "trait.identity.model".into(),
        Value::String(format!("lab-{}", class_id.as_str())),
    );
    store.insert(
        "trait.identity.class_id".into(),
        Value::Enum(class_id.as_str().to_string()),
    );
}

/// Read from last-write cache, else stable default for an advertised point.
pub fn read_store_or_default(
    store: &BTreeMap<String, Value>,
    capability: &CapabilityModel,
    point_id: &str,
) -> Option<Value> {
    if let Some(v) = store.get(point_id) {
        return Some(v.clone());
    }
    capability.point(point_id).map(stable_default)
}

/// Accept a validated write into the last-write cache (commands are no-ops).
pub fn store_write(
    store: &mut BTreeMap<String, Value>,
    capability: &CapabilityModel,
    point_id: &str,
    value: Value,
) -> Result<(), homecooked_protocol::ErrorBody> {
    let Some(cap) = capability.point(point_id) else {
        return Err(homecooked_protocol::ErrorBody::new(
            homecooked_schema::ErrorCode::UnknownVariable,
            format!("unknown point {point_id}"),
        )
        .at_point(point_id));
    };
    if cap.value_type == ValueType::Command {
        // Unmapped void/arg command: accept after capability validate.
        return Ok(());
    }
    store.insert(point_id.to_string(), value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use homecooked_schema::typical_capability;

    #[test]
    fn washer_lab_includes_typical_and_lab_extras() {
        let cap = lab_washer_capability();
        let typical = typical_capability(ApplianceClassId::Washer).unwrap();
        for id in [
            "trait.program.program",
            "trait.program.available_programs",
            "class.washer.wash_temp_c",
            "class.washer.spin_rpm",
            "class.washer.door_locked",
            "trait.power.power_state",
            "trait.cycle.cycle_state",
            "trait.cycle.start",
            "trait.cycle.cancel",
        ] {
            assert!(
                cap.point(id).is_some(),
                "lab washer missing typical point {id}"
            );
            assert!(
                typical.point(id).is_some(),
                "sanity: typical should have {id}"
            );
        }
        // Lab extras thin typical lacked.
        for id in [
            "class.washer.sim_tick",
            "class.washer.heater_enable",
            "class.washer.door_lock",
            "class.washer.water_level_pa",
            "class.washer.door_lock_fb",
            "trait.cycle.pause",
            "trait.cycle.resume",
            "trait.cycle.cycle_phase",
        ] {
            assert!(cap.point(id).is_some(), "lab washer missing lab extra {id}");
        }
        assert!(typical.point("class.washer.sim_tick").is_none());
        assert!(typical.point("trait.cycle.pause").is_none());
    }

    #[test]
    fn dryer_lab_includes_typical_and_lab_extras() {
        let cap = lab_dryer_capability();
        let typical = typical_capability(ApplianceClassId::Dryer).unwrap();
        for id in [
            "trait.program.program",
            "class.dryer.lint_filter",
            "class.dryer.door_locked",
            "trait.power.power_state",
            "trait.cycle.cycle_state",
            "trait.cycle.start",
            "trait.cycle.cancel",
        ] {
            assert!(
                cap.point(id).is_some(),
                "lab dryer missing typical point {id}"
            );
            assert!(
                typical.point(id).is_some(),
                "sanity: typical should have {id}"
            );
        }
        for id in [
            "class.dryer.sim_tick",
            "class.dryer.heater_enable",
            "class.dryer.door_lock",
            "class.dryer.blower",
            "class.dryer.door_lock_fb",
            "class.dryer.dryness",
            "class.dryer.heat_level",
            "trait.cycle.pause",
            "trait.cycle.resume",
            "trait.cycle.cycle_phase",
        ] {
            assert!(cap.point(id).is_some(), "lab dryer missing lab extra {id}");
        }
        assert!(typical.point("class.dryer.dryness").is_none());
        assert!(typical.point("class.dryer.sim_tick").is_none());
    }
}
