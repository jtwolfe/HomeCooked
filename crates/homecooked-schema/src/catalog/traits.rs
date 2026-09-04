//! Shared trait point specs from `docs/catalog/variables-and-settings.md`.

use crate::access::AccessMode;
use crate::ids::TraitId;
use crate::spec::{CatalogPoint, CatalogRange};
use crate::types::{Unit, ValueType};

use super::TraitTable;

const fn v(
    id: &'static str,
    ty: ValueType,
    unit: Option<Unit>,
    range: Option<CatalogRange>,
    access: AccessMode,
    required: bool,
) -> CatalogPoint {
    CatalogPoint::variable(id, ty, unit, range, access, required)
}

const fn s(
    id: &'static str,
    ty: ValueType,
    unit: Option<Unit>,
    range: Option<CatalogRange>,
    access: AccessMode,
    required: bool,
) -> CatalogPoint {
    CatalogPoint::setting(id, ty, unit, range, access, required)
}

const fn cmd(id: &'static str, range: Option<CatalogRange>, required: bool) -> CatalogPoint {
    CatalogPoint::command(id, range, required)
}

const fn num(min: f64, max: f64) -> Option<CatalogRange> {
    Some(CatalogRange::Numeric { min, max })
}

const fn int(min: i64, max: i64) -> Option<CatalogRange> {
    Some(CatalogRange::Integer { min, max })
}

const fn en(tokens: &'static [&'static str]) -> Option<CatalogRange> {
    Some(CatalogRange::Enum(tokens))
}

const fn slen(min: u32, max: u32) -> Option<CatalogRange> {
    Some(CatalogRange::String {
        min_chars: min,
        max_chars: max,
    })
}

const fn list_max(max_len: u32) -> Option<CatalogRange> {
    Some(CatalogRange::List { max_len })
}

const POWER_STATE: &[&str] = &["off", "standby", "on", "fault"];
const POWER_SOURCE: &[&str] = &["mains", "battery", "mains_battery", "unknown"];
const LINK_STATE: &[&str] = &["offline", "connecting", "online", "degraded"];
const TRANSPORT: &[&str] = &["ip", "ble", "thread", "zigbee", "matter", "uart", "unknown"];
const PAIR_STATE: &[&str] = &["unpaired", "pairing", "paired"];
const CLOUD_STATE: &[&str] = &["disabled", "disconnected", "connected"];
const DOOR_STATE: &[&str] = &["open", "closed", "ajar", "unknown"];
const DOOR_LOCK: &[&str] = &["unlocked", "locked", "locking", "unlocking", "fault"];
const END_SIGNAL: &[&str] = &["off", "chime", "repeat"];
const DISPLAY_UNIT: &[&str] = &["celsius", "fahrenheit"];
const CYCLE_STATE: &[&str] = &[
    "idle",
    "delayed",
    "running",
    "paused",
    "complete",
    "canceling",
    "error",
];
const FAULT_SEVERITY: &[&str] = &["info", "warning", "error", "critical"];
const ENERGY_MODE: &[&str] = &["normal", "eco", "off_peak"];
const INLET_VALVE: &[&str] = &["closed", "open", "fault"];
const DRAIN_PUMP: &[&str] = &["off", "on", "fault"];
const TANK_STATE: &[&str] = &["ok", "low", "empty", "full", "missing"];
const FILTER_STATE: &[&str] = &["ok", "low", "replace", "missing", "clogged"];
const INTERLOCK_REASON: &[&str] = &[
    "none",
    "door",
    "lid",
    "pan",
    "water",
    "pressure",
    "tilt",
    "leak",
    "overtemp",
    "child_lock",
    "remote",
    "other",
];
const FAN_STATE: &[&str] = &["off", "on", "auto", "boost"];
const HEATER_STATE: &[&str] = &["off", "on", "fault"];
const HEAT_SOURCE: &[&str] = &[
    "electric",
    "gas",
    "induction",
    "heat_pump",
    "steam",
    "mixed",
    "unknown",
];
const FLAME: &[&str] = &["off", "on", "fault"];
const MOTOR_STATE: &[&str] = &["off", "on", "stall", "fault"];
const DIRECTION: &[&str] = &["forward", "reverse"];
const DISPENSE_TYPE: &[&str] = &[
    "water",
    "ice_cubed",
    "ice_crushed",
    "hot_water",
    "ambient",
    "cold",
    "beer",
    "other",
];
const ICE_STATE: &[&str] = &["off", "making", "harvest", "full", "fault"];
const ICE_TYPE: &[&str] = &["cube", "crushed", "nugget", "clear", "crescent"];
const UPDATE_STATE: &[&str] = &["idle", "downloading", "applying", "reboot", "failed"];
const ZONE_MODE: &[&str] = &["fridge", "freezer", "off", "bar"];

static IDENTITY: &[CatalogPoint] = &[
    s(
        "device_id",
        ValueType::String,
        None,
        slen(1, 128),
        AccessMode::R,
        true,
    ),
    s(
        "manufacturer",
        ValueType::String,
        None,
        slen(1, 64),
        AccessMode::R,
        true,
    ),
    s(
        "model",
        ValueType::String,
        None,
        slen(1, 64),
        AccessMode::R,
        true,
    ),
    v(
        "serial",
        ValueType::String,
        None,
        slen(0, 64),
        AccessMode::R,
        false,
    ),
    v(
        "hw_version",
        ValueType::String,
        None,
        slen(0, 32),
        AccessMode::R,
        false,
    ),
    s(
        "fw_version",
        ValueType::String,
        None,
        slen(1, 32),
        AccessMode::R,
        true,
    ),
    v("class_id", ValueType::Enum, None, None, AccessMode::R, true),
    v(
        "secondary_class_ids",
        ValueType::List(crate::types::ListItemType::Enum),
        None,
        list_max(8),
        AccessMode::R,
        false,
    ),
    v(
        "catalog_version",
        ValueType::String,
        None,
        None,
        AccessMode::R,
        true,
    ),
    v(
        "protocol_version",
        ValueType::String,
        None,
        None,
        AccessMode::R,
        true,
    ),
    s(
        "display_name",
        ValueType::String,
        None,
        slen(0, 64),
        AccessMode::RW,
        false,
    ),
    s(
        "room",
        ValueType::String,
        None,
        slen(0, 64),
        AccessMode::RW,
        false,
    ),
];

static POWER: &[CatalogPoint] = &[
    v(
        "power_state",
        ValueType::Enum,
        None,
        en(POWER_STATE),
        AccessMode::RE,
        true,
    ),
    v(
        "power_source",
        ValueType::Enum,
        None,
        en(POWER_SOURCE),
        AccessMode::R,
        false,
    ),
    v(
        "battery_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RE,
        false,
    ),
    s(
        "auto_off_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(0, 86400),
        AccessMode::RW,
        false,
    ),
    cmd("power_on", Some(CatalogRange::CommandVoid), true),
    cmd("power_off", Some(CatalogRange::CommandVoid), true),
    cmd("power_standby", Some(CatalogRange::CommandVoid), false),
];

static CONNECTIVITY: &[CatalogPoint] = &[
    v(
        "link_state",
        ValueType::Enum,
        None,
        en(LINK_STATE),
        AccessMode::RE,
        true,
    ),
    v(
        "transport",
        ValueType::Enum,
        None,
        en(TRANSPORT),
        AccessMode::R,
        true,
    ),
    v(
        "rssi_dbm",
        ValueType::I16,
        Some(Unit::Dbm),
        int(-120, 0),
        AccessMode::RE,
        false,
    ),
    v(
        "ip_address",
        ValueType::String,
        None,
        None,
        AccessMode::R,
        false,
    ),
    v(
        "mac_address",
        ValueType::String,
        None,
        slen(17, 17),
        AccessMode::R,
        false,
    ),
    v(
        "pair_state",
        ValueType::Enum,
        None,
        en(PAIR_STATE),
        AccessMode::RE,
        false,
    ),
    v(
        "cloud_state",
        ValueType::Enum,
        None,
        en(CLOUD_STATE),
        AccessMode::RE,
        false,
    ),
    cmd(
        "identify",
        Some(CatalogRange::CommandTyped {
            value_type: ValueType::DurationS,
            min: Some(1.0),
            max: Some(60.0),
            optional: false,
        }),
        false,
    ),
    cmd("reprovision", Some(CatalogRange::CommandVoid), false),
];

static TIME_SCHEDULE: &[CatalogPoint] = &[
    s(
        "clock_ms",
        ValueType::TimestampMs,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "timezone",
        ValueType::String,
        None,
        slen(0, 64),
        AccessMode::RW,
        false,
    ),
    s(
        "delay_start_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(0, 86400),
        AccessMode::RWE,
        false,
    ),
    v(
        "delay_end_ms",
        ValueType::TimestampMs,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    s(
        "schedule_enabled",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
];

static DOOR_LID: &[CatalogPoint] = &[
    CatalogPoint::variable(
        "door_state",
        ValueType::Enum,
        None,
        en(DOOR_STATE),
        AccessMode::RE,
        true,
    )
    .zoned(),
    v(
        "door_lock_state",
        ValueType::Enum,
        None,
        en(DOOR_LOCK),
        AccessMode::RE,
        false,
    ),
    v(
        "door_open_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(0, 86400),
        AccessMode::RE,
        false,
    ),
    v(
        "door_alarm",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    s(
        "door_alarm_enable",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "door_alarm_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(5, 3600),
        AccessMode::RW,
        false,
    ),
    cmd("lock_door", Some(CatalogRange::CommandVoid), false),
    cmd("unlock_door", Some(CatalogRange::CommandVoid), false),
];

static CHILD_LOCK: &[CatalogPoint] = &[s(
    "child_lock",
    ValueType::Bool,
    None,
    None,
    AccessMode::RWE,
    true,
)];

static LIGHTING: &[CatalogPoint] = &[
    CatalogPoint::setting(
        "light_on",
        ValueType::Bool,
        None,
        None,
        AccessMode::RWE,
        true,
    )
    .zoned(),
    s(
        "light_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RW,
        false,
    ),
    s(
        "light_auto",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
];

static AUDIO: &[CatalogPoint] = &[
    s(
        "sound_enable",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        true,
    ),
    s(
        "volume_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RW,
        false,
    ),
    s(
        "end_signal",
        ValueType::Enum,
        None,
        en(END_SIGNAL),
        AccessMode::RW,
        false,
    ),
];

static TEMPERATURE: &[CatalogPoint] = &[
    CatalogPoint::variable(
        "current_c",
        ValueType::F32,
        Some(Unit::Celsius),
        num(-40.0, 500.0),
        AccessMode::RE,
        true,
    )
    .zoned(),
    CatalogPoint::setting(
        "setpoint_c",
        ValueType::F32,
        Some(Unit::Celsius),
        None,
        AccessMode::RWE,
        false,
    )
    .zoned(),
    v(
        "setpoint_min_c",
        ValueType::F32,
        Some(Unit::Celsius),
        None,
        AccessMode::R,
        false,
    ),
    v(
        "setpoint_max_c",
        ValueType::F32,
        Some(Unit::Celsius),
        None,
        AccessMode::R,
        false,
    ),
    v(
        "resolution_c",
        ValueType::F32,
        Some(Unit::Celsius),
        num(0.01, 1.0),
        AccessMode::R,
        false,
    ),
    s(
        "display_unit",
        ValueType::Enum,
        None,
        en(DISPLAY_UNIT),
        AccessMode::RW,
        false,
    ),
    CatalogPoint::variable(
        "probe_c",
        ValueType::F32,
        Some(Unit::Celsius),
        num(-40.0, 300.0),
        AccessMode::RE,
        false,
    )
    .zoned(),
    s(
        "probe_target_c",
        ValueType::F32,
        Some(Unit::Celsius),
        num(0.0, 100.0),
        AccessMode::RW,
        false,
    ),
    v(
        "probe_connected",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "preheat_complete",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    s(
        "super_mode",
        ValueType::Bool,
        None,
        None,
        AccessMode::RWE,
        false,
    ),
    v(
        "heater_active",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
];

static HUMIDITY: &[CatalogPoint] = &[
    v(
        "current_rh",
        ValueType::Percent,
        Some(Unit::RhPercent),
        num(0.0, 100.0),
        AccessMode::RE,
        true,
    ),
    s(
        "setpoint_rh",
        ValueType::Percent,
        Some(Unit::RhPercent),
        num(0.0, 100.0),
        AccessMode::RWE,
        false,
    ),
    v(
        "dew_point_c",
        ValueType::F32,
        Some(Unit::Celsius),
        None,
        AccessMode::R,
        false,
    ),
];

static CYCLE: &[CatalogPoint] = &[
    v(
        "cycle_state",
        ValueType::Enum,
        None,
        en(CYCLE_STATE),
        AccessMode::RE,
        true,
    ),
    v(
        "cycle_phase",
        ValueType::String,
        None,
        slen(1, 32),
        AccessMode::RE,
        false,
    ),
    v(
        "progress_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RE,
        false,
    ),
    v(
        "remaining_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(0, 86400),
        AccessMode::RE,
        false,
    ),
    v(
        "elapsed_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(0, 86400),
        AccessMode::RE,
        false,
    ),
    v(
        "cycle_id",
        ValueType::U32,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "end_ms",
        ValueType::TimestampMs,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    cmd("start", Some(CatalogRange::CommandVoid), true),
    cmd("pause", Some(CatalogRange::CommandVoid), false),
    cmd("resume", Some(CatalogRange::CommandVoid), false),
    cmd("cancel", Some(CatalogRange::CommandVoid), true),
];

static PROGRAM: &[CatalogPoint] = &[
    s(
        "program",
        ValueType::Enum,
        None,
        None,
        AccessMode::RWE,
        true,
    ),
    v(
        "available_programs",
        ValueType::List(crate::types::ListItemType::Enum),
        None,
        list_max(64),
        AccessMode::R,
        true,
    ),
    s(
        "option_flags",
        ValueType::List(crate::types::ListItemType::Enum),
        None,
        list_max(32),
        AccessMode::RW,
        false,
    ),
    s(
        "custom_name",
        ValueType::String,
        None,
        slen(0, 32),
        AccessMode::RW,
        false,
    ),
];

static FAULT: &[CatalogPoint] = &[
    v(
        "fault_present",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        true,
    ),
    v(
        "fault_code",
        ValueType::String,
        None,
        slen(0, 32),
        AccessMode::RE,
        false,
    ),
    v(
        "fault_severity",
        ValueType::Enum,
        None,
        en(FAULT_SEVERITY),
        AccessMode::RE,
        false,
    ),
    v(
        "fault_message",
        ValueType::String,
        None,
        slen(0, 128),
        AccessMode::R,
        false,
    ),
    v(
        "alert_list",
        ValueType::List(crate::types::ListItemType::String),
        None,
        list_max(16),
        AccessMode::RE,
        false,
    ),
    v(
        "last_fault_ms",
        ValueType::TimestampMs,
        None,
        None,
        AccessMode::R,
        false,
    ),
    cmd("ack_fault", Some(CatalogRange::CommandVoid), false),
    cmd("mute_alert", Some(CatalogRange::CommandVoid), false),
];

static ENERGY: &[CatalogPoint] = &[
    v(
        "power_w",
        ValueType::F32,
        Some(Unit::Watt),
        num(0.0, 50000.0),
        AccessMode::RE,
        false,
    ),
    v(
        "energy_wh",
        ValueType::F32,
        Some(Unit::WattHour),
        num(0.0, 1.0e12),
        AccessMode::RE,
        false,
    ),
    v(
        "cycle_energy_wh",
        ValueType::F32,
        Some(Unit::WattHour),
        num(0.0, 1.0e12),
        AccessMode::RE,
        false,
    ),
    v(
        "voltage_v",
        ValueType::F32,
        Some(Unit::Volt),
        num(0.0, 500.0),
        AccessMode::R,
        false,
    ),
    v(
        "current_a",
        ValueType::F32,
        Some(Unit::Ampere),
        num(0.0, 200.0),
        AccessMode::R,
        false,
    ),
    s(
        "energy_mode",
        ValueType::Enum,
        None,
        en(ENERGY_MODE),
        AccessMode::RW,
        false,
    ),
];

static WATER: &[CatalogPoint] = &[
    v(
        "inlet_present",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "inlet_valve",
        ValueType::Enum,
        None,
        en(INLET_VALVE),
        AccessMode::RE,
        false,
    ),
    v(
        "drain_pump",
        ValueType::Enum,
        None,
        en(DRAIN_PUMP),
        AccessMode::RE,
        false,
    ),
    v(
        "level_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RE,
        false,
    ),
    v(
        "flow_l_min",
        ValueType::F32,
        Some(Unit::LiterPerMin),
        num(0.0, 50.0),
        AccessMode::RE,
        false,
    ),
    v(
        "used_l",
        ValueType::F32,
        Some(Unit::Liter),
        num(0.0, 1.0e12),
        AccessMode::RE,
        false,
    ),
    v(
        "cycle_used_l",
        ValueType::F32,
        Some(Unit::Liter),
        num(0.0, 1.0e12),
        AccessMode::RE,
        false,
    ),
    s(
        "hardness_ppm",
        ValueType::U16,
        Some(Unit::Ppm),
        int(0, 1000),
        AccessMode::RW,
        false,
    ),
    s(
        "hardness_gpg",
        ValueType::F32,
        Some(Unit::Gpg),
        num(0.0, 50.0),
        AccessMode::RW,
        false,
    ),
    v("leak", ValueType::Bool, None, None, AccessMode::RE, false),
    v(
        "tank_state",
        ValueType::Enum,
        None,
        en(TANK_STATE),
        AccessMode::RE,
        false,
    ),
    v(
        "tds_ppm",
        ValueType::U16,
        Some(Unit::Ppm),
        int(0, 2000),
        AccessMode::RE,
        false,
    ),
];

static FILTER: &[CatalogPoint] = &[
    CatalogPoint::variable(
        "filter_state",
        ValueType::Enum,
        None,
        en(FILTER_STATE),
        AccessMode::RE,
        true,
    )
    .zoned(),
    v(
        "life_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RE,
        false,
    ),
    v(
        "life_s",
        ValueType::DurationS,
        Some(Unit::Second),
        None,
        AccessMode::R,
        false,
    ),
    v(
        "stage_id",
        ValueType::String,
        None,
        slen(1, 32),
        AccessMode::R,
        false,
    ),
    cmd(
        "reset_filter",
        Some(CatalogRange::CommandTyped {
            value_type: ValueType::String,
            min: None,
            max: None,
            optional: true,
        }),
        true,
    ),
];

static REMOTE: &[CatalogPoint] = &[
    s(
        "remote_control_enabled",
        ValueType::Bool,
        None,
        None,
        AccessMode::RWE,
        true,
    ),
    s(
        "remote_start_enabled",
        ValueType::Bool,
        None,
        None,
        AccessMode::RWE,
        false,
    ),
    v(
        "local_only",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
];

static MAINTENANCE: &[CatalogPoint] = &[
    v(
        "needs_clean",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "needs_descale",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "cycle_count",
        ValueType::U32,
        None,
        None,
        AccessMode::R,
        false,
    ),
    v(
        "last_clean_ms",
        ValueType::TimestampMs,
        None,
        None,
        AccessMode::R,
        false,
    ),
    v(
        "service_due",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    cmd("start_clean", Some(CatalogRange::CommandVoid), false),
    cmd("ack_clean", Some(CatalogRange::CommandVoid), false),
];

static SAFETY: &[CatalogPoint] = &[
    v(
        "interlock_ok",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        true,
    ),
    v(
        "interlock_reason",
        ValueType::Enum,
        None,
        en(INTERLOCK_REASON),
        AccessMode::RE,
        false,
    ),
    v(
        "hot_surface",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v("tilt", ValueType::Bool, None, None, AccessMode::RE, false),
];

static FAN: &[CatalogPoint] = &[
    s(
        "fan_state",
        ValueType::Enum,
        None,
        en(FAN_STATE),
        AccessMode::RWE,
        true,
    ),
    s(
        "fan_speed",
        ValueType::U8,
        None,
        int(0, 5),
        AccessMode::RWE,
        false,
    ),
    s(
        "fan_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RW,
        false,
    ),
    v(
        "fan_remaining_s",
        ValueType::DurationS,
        Some(Unit::Second),
        None,
        AccessMode::RE,
        false,
    ),
    s(
        "swing_on",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "louver_deg",
        ValueType::U16,
        Some(Unit::Degree),
        int(0, 180),
        AccessMode::RW,
        false,
    ),
    cmd(
        "boost",
        Some(CatalogRange::CommandTyped {
            value_type: ValueType::DurationS,
            min: Some(30.0),
            max: Some(900.0),
            optional: false,
        }),
        false,
    ),
];

static HEATER: &[CatalogPoint] = &[
    CatalogPoint::variable(
        "heater_state",
        ValueType::Enum,
        None,
        en(HEATER_STATE),
        AccessMode::RE,
        true,
    )
    .zoned(),
    s(
        "heater_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RW,
        false,
    ),
    v(
        "heat_source",
        ValueType::Enum,
        None,
        en(HEAT_SOURCE),
        AccessMode::R,
        false,
    ),
    v(
        "flame",
        ValueType::Enum,
        None,
        en(FLAME),
        AccessMode::RE,
        false,
    ),
];

static MOTOR: &[CatalogPoint] = &[
    v(
        "motor_state",
        ValueType::Enum,
        None,
        en(MOTOR_STATE),
        AccessMode::RE,
        true,
    ),
    v(
        "rpm",
        ValueType::U16,
        Some(Unit::Rpm),
        int(0, 20000),
        AccessMode::RE,
        false,
    ),
    s(
        "rpm_setpoint",
        ValueType::U16,
        Some(Unit::Rpm),
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "speed_level",
        ValueType::U8,
        None,
        None,
        AccessMode::RWE,
        false,
    ),
    s(
        "direction",
        ValueType::Enum,
        None,
        en(DIRECTION),
        AccessMode::RW,
        false,
    ),
];

static ZONE: &[CatalogPoint] = &[
    v(
        "zones",
        ValueType::List(crate::types::ListItemType::String),
        None,
        list_max(16),
        AccessMode::R,
        true,
    ),
    CatalogPoint::setting(
        "zone_mode",
        ValueType::Enum,
        None,
        en(ZONE_MODE),
        AccessMode::RW,
        false,
    )
    .zoned(),
    CatalogPoint::setting(
        "zone_enable",
        ValueType::Bool,
        None,
        None,
        AccessMode::RWE,
        false,
    )
    .zoned(),
];

static DISPENSE: &[CatalogPoint] = &[
    s(
        "dispense_type",
        ValueType::Enum,
        None,
        en(DISPENSE_TYPE),
        AccessMode::RW,
        false,
    ),
    s(
        "portion_ml",
        ValueType::U16,
        Some(Unit::Milliliter),
        int(10, 2000),
        AccessMode::RW,
        false,
    ),
    s(
        "hot_lock",
        ValueType::Bool,
        None,
        None,
        AccessMode::RWE,
        false,
    ),
    v(
        "dispensing",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        true,
    ),
    cmd(
        "dispense",
        Some(CatalogRange::CommandTyped {
            value_type: ValueType::U16,
            min: Some(10.0),
            max: Some(2000.0),
            optional: true,
        }),
        true,
    ),
    cmd("stop_dispense", Some(CatalogRange::CommandVoid), true),
];

static ICE: &[CatalogPoint] = &[
    s(
        "ice_enabled",
        ValueType::Bool,
        None,
        None,
        AccessMode::RWE,
        true,
    ),
    v(
        "ice_state",
        ValueType::Enum,
        None,
        en(ICE_STATE),
        AccessMode::RE,
        true,
    ),
    s(
        "ice_type",
        ValueType::Enum,
        None,
        en(ICE_TYPE),
        AccessMode::RW,
        false,
    ),
    v(
        "bin_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RE,
        false,
    ),
    v(
        "bin_full",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
];

static OTA: &[CatalogPoint] = &[
    v(
        "update_available",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        true,
    ),
    v(
        "update_version",
        ValueType::String,
        None,
        slen(0, 32),
        AccessMode::R,
        false,
    ),
    v(
        "update_state",
        ValueType::Enum,
        None,
        en(UPDATE_STATE),
        AccessMode::RE,
        false,
    ),
    v(
        "update_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RE,
        false,
    ),
    cmd("start_update", Some(CatalogRange::CommandVoid), false),
];

const TABLES: &[TraitTable] = &[
    TraitTable {
        trait_id: TraitId::Identity,
        points: IDENTITY,
    },
    TraitTable {
        trait_id: TraitId::Power,
        points: POWER,
    },
    TraitTable {
        trait_id: TraitId::Connectivity,
        points: CONNECTIVITY,
    },
    TraitTable {
        trait_id: TraitId::TimeSchedule,
        points: TIME_SCHEDULE,
    },
    TraitTable {
        trait_id: TraitId::DoorLid,
        points: DOOR_LID,
    },
    TraitTable {
        trait_id: TraitId::ChildLock,
        points: CHILD_LOCK,
    },
    TraitTable {
        trait_id: TraitId::Lighting,
        points: LIGHTING,
    },
    TraitTable {
        trait_id: TraitId::Audio,
        points: AUDIO,
    },
    TraitTable {
        trait_id: TraitId::Temperature,
        points: TEMPERATURE,
    },
    TraitTable {
        trait_id: TraitId::Humidity,
        points: HUMIDITY,
    },
    TraitTable {
        trait_id: TraitId::Cycle,
        points: CYCLE,
    },
    TraitTable {
        trait_id: TraitId::Program,
        points: PROGRAM,
    },
    TraitTable {
        trait_id: TraitId::Fault,
        points: FAULT,
    },
    TraitTable {
        trait_id: TraitId::Energy,
        points: ENERGY,
    },
    TraitTable {
        trait_id: TraitId::Water,
        points: WATER,
    },
    TraitTable {
        trait_id: TraitId::Filter,
        points: FILTER,
    },
    TraitTable {
        trait_id: TraitId::Remote,
        points: REMOTE,
    },
    TraitTable {
        trait_id: TraitId::Maintenance,
        points: MAINTENANCE,
    },
    TraitTable {
        trait_id: TraitId::Safety,
        points: SAFETY,
    },
    TraitTable {
        trait_id: TraitId::Fan,
        points: FAN,
    },
    TraitTable {
        trait_id: TraitId::Heater,
        points: HEATER,
    },
    TraitTable {
        trait_id: TraitId::Motor,
        points: MOTOR,
    },
    TraitTable {
        trait_id: TraitId::Zone,
        points: ZONE,
    },
    TraitTable {
        trait_id: TraitId::Dispense,
        points: DISPENSE,
    },
    TraitTable {
        trait_id: TraitId::Ice,
        points: ICE,
    },
    TraitTable {
        trait_id: TraitId::Ota,
        points: OTA,
    },
];

pub fn trait_table(trait_id: TraitId) -> Option<&'static TraitTable> {
    TABLES.iter().find(|t| t.trait_id == trait_id)
}
