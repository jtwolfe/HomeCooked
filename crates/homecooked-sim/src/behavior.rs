//! Simple per-class simulation: kettle heat, washer/microwave cycle progress.

use homecooked_core::{DeviceState, RegisteredDevice};
use homecooked_protocol::WriteOp;
use homecooked_schema::{ApplianceClassId, Value};

/// Degrees celsius per simulated second while the kettle is on.
pub const KETTLE_HEAT_RATE_C_PER_S: f32 = 5.0;
/// Stub washer cycle length.
pub const WASHER_CYCLE_S: u32 = 60;
/// Fallback microwave cook duration when `class.microwave.cook_s` is missing.
pub const DEFAULT_MICROWAVE_COOK_S: u32 = 60;

pub fn apply_writes(dev: &mut RegisteredDevice, writes: &[WriteOp]) {
    for op in writes {
        apply_command(dev, &op.id.to_string(), &op.value);
    }
}

fn apply_command(dev: &mut RegisteredDevice, point_id: &str, _value: &Value) {
    match point_id {
        "trait.power.power_on" => {
            set_enum(&mut dev.state, "trait.power.power_state", "on");
            if dev.identity.class_id == ApplianceClassId::Kettle {
                set_enum(&mut dev.state, "trait.heater.heater_state", "on");
                set_enum(&mut dev.state, "trait.cycle.cycle_state", "running");
            }
        }
        "trait.power.power_off" => {
            set_enum(&mut dev.state, "trait.power.power_state", "off");
            set_enum(&mut dev.state, "trait.heater.heater_state", "off");
            idle_cycle(&mut dev.state);
        }
        "trait.power.power_standby" => {
            set_enum(&mut dev.state, "trait.power.power_state", "standby");
            set_enum(&mut dev.state, "trait.heater.heater_state", "off");
        }
        "trait.cycle.start" => start_cycle(dev),
        "trait.cycle.cancel" => idle_cycle(&mut dev.state),
        "trait.cycle.pause" => pause_cycle(&mut dev.state),
        "trait.cycle.resume" => resume_cycle(&mut dev.state),
        _ => {}
    }
}

fn start_cycle(dev: &mut RegisteredDevice) {
    set_enum(&mut dev.state, "trait.cycle.cycle_state", "running");
    set_percent(&mut dev.state, "trait.cycle.progress_percent", 0.0);
    set_duration(&mut dev.state, "trait.cycle.elapsed_s", 0);
    match dev.identity.class_id {
        ApplianceClassId::Washer | ApplianceClassId::WasherDryer => {
            set_duration(&mut dev.state, "trait.cycle.remaining_s", WASHER_CYCLE_S);
            set_string(&mut dev.state, "trait.cycle.cycle_phase", "fill");
        }
        ApplianceClassId::Microwave => {
            let cook = duration_of(&dev.state, "class.microwave.cook_s");
            let cook = if cook == 0 {
                DEFAULT_MICROWAVE_COOK_S
            } else {
                cook
            };
            set_duration(&mut dev.state, "trait.cycle.remaining_s", cook);
            set_string(&mut dev.state, "trait.cycle.cycle_phase", "cook");
            set_enum(&mut dev.state, "trait.power.power_state", "on");
        }
        ApplianceClassId::Kettle => {
            set_enum(&mut dev.state, "trait.power.power_state", "on");
            set_enum(&mut dev.state, "trait.heater.heater_state", "on");
        }
        _ => {}
    }
}

fn pause_cycle(state: &mut DeviceState) {
    if enum_is(state, "trait.cycle.cycle_state", "running") {
        set_enum(state, "trait.cycle.cycle_state", "paused");
    }
}

fn resume_cycle(state: &mut DeviceState) {
    if enum_is(state, "trait.cycle.cycle_state", "paused") {
        set_enum(state, "trait.cycle.cycle_state", "running");
    }
}

fn idle_cycle(state: &mut DeviceState) {
    set_enum(state, "trait.cycle.cycle_state", "idle");
    set_percent(state, "trait.cycle.progress_percent", 0.0);
    set_duration(state, "trait.cycle.elapsed_s", 0);
    set_duration(state, "trait.cycle.remaining_s", 0);
    set_string(state, "trait.cycle.cycle_phase", "idle");
}

pub fn tick_device(dev: &mut RegisteredDevice, dt_ms: u64) {
    if dt_ms == 0 {
        return;
    }
    match dev.identity.class_id {
        ApplianceClassId::Kettle => tick_kettle(&mut dev.state, dt_ms),
        ApplianceClassId::Washer | ApplianceClassId::WasherDryer => {
            tick_washer(&mut dev.state, dt_ms)
        }
        ApplianceClassId::Microwave => tick_microwave(&mut dev.state, dt_ms),
        _ => {}
    }
}

fn tick_kettle(state: &mut DeviceState, dt_ms: u64) {
    let on = enum_is(state, "trait.power.power_state", "on")
        || enum_is(state, "trait.cycle.cycle_state", "running");
    if !on {
        return;
    }
    let mut current = f32_of(state, "trait.temperature.current_c").unwrap_or(20.0);
    let setpoint = f32_of(state, "trait.temperature.setpoint_c").unwrap_or(100.0);
    let dt_s = dt_ms as f32 / 1000.0;
    current = (current + KETTLE_HEAT_RATE_C_PER_S * dt_s).min(setpoint);
    state.insert("trait.temperature.current_c", Value::F32(current));
    if current >= setpoint {
        set_enum(state, "trait.heater.heater_state", "off");
        set_enum(state, "trait.cycle.cycle_state", "complete");
        set_enum(state, "trait.power.power_state", "standby");
        set_percent(state, "trait.cycle.progress_percent", 100.0);
        set_duration(state, "trait.cycle.remaining_s", 0);
    } else {
        set_enum(state, "trait.heater.heater_state", "on");
        let remaining = ((setpoint - current) / KETTLE_HEAT_RATE_C_PER_S).ceil() as u32;
        set_duration(state, "trait.cycle.remaining_s", remaining);
        let span = (setpoint - 20.0).max(1.0);
        let progress = ((current - 20.0) / span * 100.0).clamp(0.0, 99.0);
        set_percent(state, "trait.cycle.progress_percent", progress);
    }
}

fn tick_washer(state: &mut DeviceState, dt_ms: u64) {
    if !enum_is(state, "trait.cycle.cycle_state", "running") {
        return;
    }
    let add_s = (dt_ms / 1000) as u32;
    if add_s == 0 {
        return;
    }
    let elapsed = duration_of(state, "trait.cycle.elapsed_s").saturating_add(add_s);
    let total = WASHER_CYCLE_S.max(1);
    let elapsed = elapsed.min(total);
    let remaining = total.saturating_sub(elapsed);
    let progress = (elapsed as f32 / total as f32) * 100.0;
    set_duration(state, "trait.cycle.elapsed_s", elapsed);
    set_duration(state, "trait.cycle.remaining_s", remaining);
    set_percent(state, "trait.cycle.progress_percent", progress.min(100.0));
    set_string(state, "trait.cycle.cycle_phase", washer_phase(progress));
    if elapsed >= total {
        set_enum(state, "trait.cycle.cycle_state", "complete");
        set_percent(state, "trait.cycle.progress_percent", 100.0);
        set_string(state, "trait.cycle.cycle_phase", "complete");
    }
}

fn tick_microwave(state: &mut DeviceState, dt_ms: u64) {
    if !enum_is(state, "trait.cycle.cycle_state", "running") {
        return;
    }
    let add_s = (dt_ms / 1000) as u32;
    if add_s == 0 {
        return;
    }
    let cook = duration_of(state, "class.microwave.cook_s");
    let total = if cook == 0 {
        DEFAULT_MICROWAVE_COOK_S
    } else {
        cook
    }
    .max(1);
    let elapsed = duration_of(state, "trait.cycle.elapsed_s").saturating_add(add_s);
    let elapsed = elapsed.min(total);
    let remaining = total.saturating_sub(elapsed);
    let progress = (elapsed as f32 / total as f32) * 100.0;
    set_duration(state, "trait.cycle.elapsed_s", elapsed);
    set_duration(state, "trait.cycle.remaining_s", remaining);
    set_percent(state, "trait.cycle.progress_percent", progress.min(100.0));
    set_string(state, "trait.cycle.cycle_phase", "cook");
    if elapsed >= total {
        set_enum(state, "trait.cycle.cycle_state", "complete");
        set_percent(state, "trait.cycle.progress_percent", 100.0);
        set_string(state, "trait.cycle.cycle_phase", "complete");
        set_enum(state, "trait.power.power_state", "standby");
    }
}

fn washer_phase(progress: f32) -> &'static str {
    if progress < 15.0 {
        "fill"
    } else if progress < 45.0 {
        "wash"
    } else if progress < 70.0 {
        "rinse"
    } else if progress < 95.0 {
        "spin"
    } else {
        "complete"
    }
}

fn set_enum(state: &mut DeviceState, id: &str, token: &str) {
    if state.contains(id) {
        state.insert(id, Value::Enum(token.to_string()));
    }
}

fn set_percent(state: &mut DeviceState, id: &str, v: f32) {
    if state.contains(id) {
        state.insert(id, Value::Percent(v));
    }
}

fn set_duration(state: &mut DeviceState, id: &str, v: u32) {
    if state.contains(id) {
        state.insert(id, Value::DurationS(v));
    }
}

fn set_string(state: &mut DeviceState, id: &str, v: &str) {
    if state.contains(id) {
        state.insert(id, Value::String(v.to_string()));
    }
}

fn enum_is(state: &DeviceState, id: &str, token: &str) -> bool {
    matches!(state.get(id), Some(Value::Enum(s)) if s == token)
}

fn f32_of(state: &DeviceState, id: &str) -> Option<f32> {
    match state.get(id) {
        Some(Value::F32(v)) => Some(*v),
        Some(Value::Percent(v)) => Some(*v),
        _ => None,
    }
}

fn duration_of(state: &DeviceState, id: &str) -> u32 {
    match state.get(id) {
        Some(Value::DurationS(v)) => *v,
        _ => 0,
    }
}
