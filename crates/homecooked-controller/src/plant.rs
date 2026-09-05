//! Deterministic laundry plant model for the host controller sim.
//!
//! Updates sensor channels from actuator state each tick. Not physics —
//! just enough feedback for washer/dryer cotton state machines and interlocks.

use homecooked_hal::{bridge, ChannelId, Hal, HalValue, MockHal};

use crate::error::Error;

/// Pressure (Pa) at or above which `water_present` is true for interlocks.
pub const WATER_PRESENT_PA: f64 = 500.0;

/// Pa added per tick while cold inlet is open.
pub const FILL_RATE_PA: f64 = 800.0;

/// Pa removed per tick while drain pump is on.
pub const DRAIN_RATE_PA: f64 = 1000.0;

/// °C added per tick while heater is on and water is present.
pub const HEAT_RATE_C: f64 = 15.0;

/// Ambient cool-down when heater is off (°C per tick).
pub const COOL_RATE_C: f64 = 1.0;

/// Ambient drum temperature (°C) used as the cool floor.
pub const DRYER_AMBIENT_C: f64 = 20.0;

/// °C added per tick while dryer heater + blower are on.
pub const DRYER_HEAT_RATE_C: f64 = 12.0;

/// °C removed per tick while cooler (heater off, blower on).
pub const DRYER_COOL_RATE_C: f64 = 8.0;

/// RH percent removed per tick while heating with blower.
pub const DRYER_DRY_RATE_RH: f64 = 15.0;

/// Advance the fake washer plant one tick from current actuator outputs.
pub fn step_plant(hal: &mut MockHal) -> Result<(), Error> {
    let inlet_on = read_aout_bool(hal, "aout.cold_inlet")?;
    let drain_on = read_aout_bool(hal, "aout.drain_pump")?;
    let heater_on = read_aout_bool(hal, "aout.heater_enable")?;
    let lock_cmd = read_aout_bool(hal, "aout.door_lock")?;

    let level_id = ChannelId::new("ain.water_level_pa")?;
    let mut level = hal.read_ai(&level_id).unwrap_or(0.0);
    if inlet_on {
        level = (level + FILL_RATE_PA).min(4000.0);
    }
    if drain_on {
        level = (level - DRAIN_RATE_PA).max(0.0);
    }
    hal.inject(&level_id, level)?;

    let temp_id = ChannelId::new("ain.tub_temp_c")?;
    let mut temp = hal.read_ai(&temp_id).unwrap_or(20.0);
    let water_present = level >= WATER_PRESENT_PA;
    if heater_on && water_present {
        temp += HEAT_RATE_C;
    } else if temp > 20.0 {
        temp = (temp - COOL_RATE_C).max(20.0);
    }
    hal.inject(&temp_id, temp)?;

    // Lock feedback follows the lock command (one-tick lag is enough: we set
    // fb in the same plant step after the cycle wrote the command last tick).
    let fb = ChannelId::new("din.door_lock_fb")?;
    hal.inject(&fb, lock_cmd)?;

    // Drum tach tracks motor speed command when enabled.
    let enable = read_motor_bool(hal, "motor.enable")?;
    let rpm_cmd = read_motor_number(hal, "motor.speed_rpm_cmd")?;
    let rpm_id = ChannelId::new("ain.drum_rpm")?;
    let measured = if enable { rpm_cmd } else { 0.0 };
    hal.inject(&rpm_id, measured)?;

    Ok(())
}

/// Refresh derived interlock keys on the mock (`water_present`, `door_locked`).
pub fn refresh_derived(hal: &mut MockHal) -> Result<(), Error> {
    let level = bridge::read_channel(hal, "ain.water_level_pa")?
        .as_number()
        .unwrap_or(0.0);
    let water_present = level >= WATER_PRESENT_PA;
    hal.set_derived("water_present", water_present);

    let lock_fb = bridge::read_channel(hal, "din.door_lock_fb")?
        .as_bool()
        .unwrap_or(false);
    let lock_cmd = read_aout_bool(hal, "aout.door_lock")?;
    // Prefer feedback; fall back to command for the tick before plant runs.
    let door_locked = lock_fb || lock_cmd;
    hal.set_derived("door_locked", door_locked);
    Ok(())
}

/// Advance the fake dryer plant one tick from current actuator outputs.
pub fn step_dryer_plant(hal: &mut MockHal) -> Result<(), Error> {
    let heater_on = read_aout_bool(hal, "aout.heater_enable")?;
    let blower_on = read_aout_bool(hal, "aout.blower")?;
    let lock_cmd = read_aout_bool(hal, "aout.door_lock")?;

    let temp_id = ChannelId::new("ain.drum_temp_c")?;
    let mut temp = hal.read_ai(&temp_id).unwrap_or(DRYER_AMBIENT_C);
    if heater_on && blower_on {
        temp += DRYER_HEAT_RATE_C;
    } else if blower_on && temp > DRYER_AMBIENT_C {
        temp = (temp - DRYER_COOL_RATE_C).max(DRYER_AMBIENT_C);
    } else if temp > DRYER_AMBIENT_C {
        temp = (temp - COOL_RATE_C).max(DRYER_AMBIENT_C);
    }
    hal.inject(&temp_id, temp)?;

    let rh_id = ChannelId::new("ain.humidity_rh")?;
    let mut rh = hal.read_ai(&rh_id).unwrap_or(60.0);
    if heater_on && blower_on {
        rh = (rh - DRYER_DRY_RATE_RH).max(5.0);
    }
    hal.inject(&rh_id, rh)?;

    let fb = ChannelId::new("din.door_lock_fb")?;
    hal.inject(&fb, lock_cmd)?;

    let enable = read_motor_bool(hal, "motor.enable")?;
    let rpm_cmd = read_motor_number(hal, "motor.speed_rpm_cmd")?;
    let rpm_id = ChannelId::new("ain.drum_rpm")?;
    let measured = if enable { rpm_cmd } else { 0.0 };
    hal.inject(&rpm_id, measured)?;

    Ok(())
}

/// Refresh derived dryer interlock keys (`door_locked`, `blower_on`).
pub fn refresh_dryer_derived(hal: &mut MockHal) -> Result<(), Error> {
    let lock_fb = bridge::read_channel(hal, "din.door_lock_fb")?
        .as_bool()
        .unwrap_or(false);
    let lock_cmd = read_aout_bool(hal, "aout.door_lock")?;
    let door_locked = lock_fb || lock_cmd;
    hal.set_derived("door_locked", door_locked);

    let blower = read_aout_bool(hal, "aout.blower")?;
    hal.set_derived("blower_on", blower);
    Ok(())
}

fn read_aout_bool(hal: &MockHal, channel: &str) -> Result<bool, Error> {
    let id = ChannelId::new(channel)?;
    match hal.get(&id)? {
        HalValue::Bool(b) => Ok(*b),
        HalValue::Number(n) => Ok(*n != 0.0),
    }
}

fn read_motor_bool(hal: &MockHal, channel: &str) -> Result<bool, Error> {
    read_aout_bool(hal, channel)
}

fn read_motor_number(hal: &MockHal, channel: &str) -> Result<f64, Error> {
    let id = ChannelId::new(channel)?;
    Ok(hal.get(&id)?.as_number().unwrap_or(0.0))
}
