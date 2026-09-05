//! Catalog-driven capability extras and default point values.

use homecooked_core::DeviceState;
use homecooked_protocol::PROTOCOL_VERSION;
use homecooked_schema::{
    trait_table, typical_capability, ApplianceClassId, CapabilityModel, DeviceIdentity,
    PointCapability, TraitId, Value, ValueRange, ValueType,
};

/// Typical capability plus optional cycle telemetry the sim exposes.
pub fn sim_capability(class_id: ApplianceClassId) -> Option<CapabilityModel> {
    let mut cap = typical_capability(class_id)?;
    if let Some(cycle) = cap.traits.iter_mut().find(|t| t.trait_id == TraitId::Cycle) {
        if let Some(table) = trait_table(TraitId::Cycle) {
            for extra in [
                "cycle_phase",
                "progress_percent",
                "remaining_s",
                "elapsed_s",
            ] {
                if cycle.points.iter().any(|p| p.base_id().ends_with(extra)) {
                    continue;
                }
                if let Some(point) = table.point(extra) {
                    cycle.points.push(PointCapability::from_catalog(
                        format!("trait.cycle.{extra}"),
                        point,
                    ));
                }
            }
        }
    }
    Some(cap)
}

pub fn seed_identity(device_id: &str, class_id: ApplianceClassId) -> DeviceIdentity {
    let mut identity = DeviceIdentity::new(
        device_id,
        "HomeCooked",
        format!("sim-{}", class_id.as_str()),
        "0.1.0",
        class_id,
    );
    identity.display_name = Some(format!("Simulated {}", class_id.as_str().replace('_', " ")));
    identity.protocol_version = PROTOCOL_VERSION;
    identity
}

pub fn seed_state(identity: &DeviceIdentity, cap: &CapabilityModel) -> DeviceState {
    let ctx = SeedCtx::from_identity(identity);
    let mut state = DeviceState::new();
    for point in cap.iter_points() {
        if point.value_type == ValueType::Command {
            continue;
        }
        if let Some(zones) = &point.zones {
            for zone in zones {
                let id = format!("{}#{zone}", point.base_id());
                state.insert(id, default_value(point, &ctx, Some(zone)));
            }
        } else {
            state.insert(point.id.clone(), default_value(point, &ctx, None));
        }
    }
    state
}

struct SeedCtx {
    identity: DeviceIdentity,
    ambient_c: f32,
    setpoint_c: f32,
    power_state: &'static str,
}

impl SeedCtx {
    fn from_identity(identity: &DeviceIdentity) -> Self {
        let (ambient_c, setpoint_c, power_state) = match identity.class_id {
            ApplianceClassId::Kettle => (20.0, 100.0, "standby"),
            ApplianceClassId::Fridge => (4.0, 4.0, "on"),
            ApplianceClassId::Freezer => (-18.0, -18.0, "on"),
            ApplianceClassId::FridgeFreezer => (4.0, 4.0, "on"),
            ApplianceClassId::WineCooler => (12.0, 12.0, "on"),
            ApplianceClassId::BeverageCooler | ApplianceClassId::Kegerator => (5.0, 5.0, "on"),
            ApplianceClassId::Oven
            | ApplianceClassId::SteamOven
            | ApplianceClassId::ToasterOven
            | ApplianceClassId::Range => (20.0, 180.0, "on"),
            ApplianceClassId::AirFryer => (20.0, 180.0, "on"),
            ApplianceClassId::WarmingDrawer => (20.0, 60.0, "on"),
            ApplianceClassId::PizzaOven => (20.0, 350.0, "on"),
            ApplianceClassId::ElectricGrill => (20.0, 180.0, "on"),
            ApplianceClassId::ElectricSmoker => (20.0, 100.0, "on"),
            ApplianceClassId::Microwave => (20.0, 20.0, "standby"),
            ApplianceClassId::InductionHob | ApplianceClassId::Cooktop => (20.0, 20.0, "on"),
            ApplianceClassId::WaterHeater => (60.0, 60.0, "on"),
            ApplianceClassId::Hvac => (21.0, 21.0, "on"),
            ApplianceClassId::SousVide => (20.0, 55.0, "standby"),
            ApplianceClassId::CoffeeMachine | ApplianceClassId::EspressoMachine => {
                (90.0, 92.0, "standby")
            }
            ApplianceClassId::MultiCooker => (20.0, 80.0, "on"),
            ApplianceClassId::Dehydrator => (20.0, 55.0, "on"),
            ApplianceClassId::YogurtMaker => (20.0, 42.0, "on"),
            ApplianceClassId::WaffleMaker => (20.0, 190.0, "on"),
            _ => (20.0, 40.0, "on"),
        };
        Self {
            identity: identity.clone(),
            ambient_c,
            setpoint_c: clamp_to_typical(identity.class_id, setpoint_c),
            power_state,
        }
    }
}

fn clamp_to_typical(class_id: ApplianceClassId, setpoint: f32) -> f32 {
    match class_id {
        ApplianceClassId::Kettle => setpoint.clamp(40.0, 100.0),
        ApplianceClassId::Fridge => setpoint.clamp(1.0, 7.0),
        ApplianceClassId::Freezer => setpoint.clamp(-24.0, -12.0),
        ApplianceClassId::FridgeFreezer => setpoint.clamp(-24.0, 7.0),
        ApplianceClassId::WineCooler => setpoint.clamp(5.0, 20.0),
        ApplianceClassId::BeverageCooler | ApplianceClassId::Kegerator => setpoint.clamp(1.0, 10.0),
        ApplianceClassId::Oven
        | ApplianceClassId::SteamOven
        | ApplianceClassId::ToasterOven
        | ApplianceClassId::Range => setpoint.clamp(50.0, 250.0),
        ApplianceClassId::AirFryer => setpoint.clamp(80.0, 200.0),
        ApplianceClassId::WarmingDrawer => setpoint.clamp(40.0, 90.0),
        ApplianceClassId::PizzaOven => setpoint.clamp(200.0, 450.0),
        ApplianceClassId::ElectricGrill => setpoint.clamp(100.0, 250.0),
        ApplianceClassId::ElectricSmoker => setpoint.clamp(50.0, 150.0),
        ApplianceClassId::WaterHeater => setpoint.clamp(40.0, 70.0),
        ApplianceClassId::SousVide => setpoint.clamp(20.0, 95.0),
        ApplianceClassId::Dehydrator => setpoint.clamp(30.0, 75.0),
        ApplianceClassId::YogurtMaker => setpoint.clamp(35.0, 50.0),
        ApplianceClassId::WaffleMaker => setpoint.clamp(150.0, 220.0),
        _ => setpoint,
    }
}

fn last_segment(qualified: &str) -> &str {
    let base = qualified.split('#').next().unwrap_or(qualified);
    base.rsplit('.').next().unwrap_or(base)
}

fn zoned_temp_c(class_id: ApplianceClassId, zone: &str) -> Option<f32> {
    match (class_id, zone) {
        (ApplianceClassId::FridgeFreezer, "fridge") => Some(4.0),
        (ApplianceClassId::FridgeFreezer, "freezer") => Some(-18.0),
        (ApplianceClassId::WineCooler, "upper" | "lower") => Some(12.0),
        (ApplianceClassId::Freezer, "freezer") => Some(-18.0),
        _ => None,
    }
}

fn default_value(point: &PointCapability, ctx: &SeedCtx, zone: Option<&str>) -> Value {
    let seg = last_segment(&point.id);
    if matches!(seg, "current_c" | "setpoint_c") {
        if let Some(z) = zone {
            if let Some(temp) = zoned_temp_c(ctx.identity.class_id, z) {
                return Value::F32(temp);
            }
        }
    }
    match seg {
        "device_id" => Value::String(ctx.identity.device_id.clone()),
        "manufacturer" => Value::String(ctx.identity.manufacturer.clone()),
        "model" => Value::String(ctx.identity.model.clone()),
        "fw_version" => Value::String(ctx.identity.fw_version.clone()),
        "display_name" => Value::String(
            ctx.identity
                .display_name
                .clone()
                .unwrap_or_else(|| ctx.identity.device_id.clone()),
        ),
        "class_id" => Value::Enum(ctx.identity.class_id.as_str().to_string()),
        "catalog_version" => Value::String(ctx.identity.catalog_version.to_string()),
        "protocol_version" => Value::String(ctx.identity.protocol_version.to_string()),
        "current_c" => Value::F32(ctx.ambient_c),
        "setpoint_c" => Value::F32(ctx.setpoint_c),
        "space_c" => Value::F32(21.0),
        "hvac_mode" => Value::Enum("auto".into()),
        "combo_mode" => Value::Enum("wash_and_dry".into()),
        "ch_enable" => Value::Bool(true),
        "ch_setpoint_c" => Value::F32(60.0),
        "flow_c" => Value::F32(55.0),
        "pressure_bar" => Value::F32(1.5),
        "brew_setpoint_c" => Value::F32(93.0),
        "boiler_c" => Value::F32(20.0),
        "heat_level" => Value::Enum("low".into()),
        "water_empty" => Value::Bool(false),
        "incubate_s" => Value::DurationS(28800),
        "power_state" => Value::Enum(ctx.power_state.to_string()),
        "cycle_state" => Value::Enum("idle".into()),
        "cycle_phase" => Value::String("idle".into()),
        "door_state" => Value::Enum("closed".into()),
        "link_state" => Value::Enum("online".into()),
        "transport" => Value::Enum("ip".into()),
        "heater_state" => Value::Enum("off".into()),
        "motor_state" => Value::Enum("off".into()),
        "fan_state" => Value::Enum("off".into()),
        "ice_state" => Value::Enum("off".into()),
        "fault_present" => Value::Bool(false),
        "interlock_ok" => Value::Bool(true),
        "remote_control_enabled" => Value::Bool(true),
        "on_base" => Value::Bool(true),
        "progress_percent" => Value::Percent(0.0),
        "remaining_s" | "elapsed_s" => Value::DurationS(0),
        "wash_temp_c" => Value::F32(40.0),
        "spin_rpm" => Value::U16(800),
        "cook_s" => Value::DurationS(600),
        "level" => Value::U8(0),
        "program" => first_enum(point).unwrap_or_else(|| Value::Enum("custom".into())),
        "available_programs" => enum_list(point),
        "thermal_port_id" => match ctx.identity.class_id {
            ApplianceClassId::WaterHeater => Value::String("preheat".into()),
            ApplianceClassId::Fridge => Value::String("condenser".into()),
            // Hydronic space-heating coil (not the air-condenser reject port used in plant media-mismatch demos).
            ApplianceClassId::Hvac => Value::String("coil".into()),
            _ => Value::String(String::new()),
        },
        "thermal_port_direction" => match ctx.identity.class_id {
            ApplianceClassId::WaterHeater => Value::Enum("sink".into()),
            ApplianceClassId::Fridge => Value::Enum("source".into()),
            // Sink: space heating drawing from a hot plant reservoir (thermal-plant.md comfort priority).
            ApplianceClassId::Hvac => Value::Enum("sink".into()),
            _ => first_enum(point).unwrap_or_else(|| Value::Enum("unknown".into())),
        },
        // Water_heater/fridge: plant demo Media::Water. HVAC: hydronic coil loop (water).
        "thermal_port_media" => Value::Enum("water".into()),
        "thermal_port_max_power_w" => match ctx.identity.class_id {
            ApplianceClassId::WaterHeater => Value::F32(2_000.0),
            ApplianceClassId::Fridge => Value::F32(120.0),
            ApplianceClassId::Hvac => Value::F32(5_000.0),
            _ => Value::F32(0.0),
        },
        "thermal_port_attached_reservoir_id" => Value::String(String::new()),
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
        ValueType::U8 => Value::U8(int_min(point, 0) as u8),
        ValueType::U16 => Value::U16(int_min(point, 0) as u16),
        ValueType::U32 => Value::U32(int_min(point, 0) as u32),
        ValueType::I16 => Value::I16(int_min(point, 0) as i16),
        ValueType::I32 => Value::I32(int_min(point, 0) as i32),
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

fn int_min(point: &PointCapability, fallback: i64) -> i64 {
    match &point.range {
        Some(ValueRange::Integer { min, .. }) => (*min).max(0),
        Some(ValueRange::Numeric { min, .. }) => (*min as i64).max(0),
        _ => fallback,
    }
}

fn numeric_default(point: &PointCapability) -> f32 {
    match &point.range {
        Some(ValueRange::Numeric { min, .. }) if *min > 0.0 => *min as f32,
        Some(ValueRange::Integer { min, .. }) if *min > 0 => *min as f32,
        _ => 0.0,
    }
}
