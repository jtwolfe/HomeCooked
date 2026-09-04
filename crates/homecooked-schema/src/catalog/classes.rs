//! Static tables for the nine PR2 classes.

use crate::access::AccessMode;
use crate::ids::{ApplianceClassId, TraitId};
use crate::spec::{CatalogPoint, CatalogRange};
use crate::types::{Unit, ValueType};

use super::ClassTable;

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

const WASHER_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::TimeSchedule,
    TraitId::DoorLid,
    TraitId::ChildLock,
    TraitId::Cycle,
    TraitId::Program,
    TraitId::Water,
    TraitId::Temperature,
    TraitId::Motor,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Remote,
    TraitId::Maintenance,
    TraitId::Audio,
    TraitId::Safety,
];

const WASHER_PROGRAMS: &[&str] = &[
    "cotton",
    "eco",
    "wool",
    "delicates",
    "quick",
    "rinse",
    "spin",
    "bedding",
    "allergy",
    "outdoor",
    "synthetic",
    "handwash",
    "drum_clean",
    "custom",
];

const WASHER_PHASES: &[&str] = &[
    "fill", "prewash", "wash", "rinse", "spin", "drain", "soak", "steam", "complete",
];

const WASH_TEMP_BAND: &[&str] = &["cold", "warm", "hot", "90"];
const SOIL_LEVEL: &[&str] = &["light", "normal", "heavy"];
const LOAD_SIZE: &[&str] = &["small", "medium", "large", "auto"];

static WASHER_POINTS: &[CatalogPoint] = &[
    s(
        "wash_temp_c",
        ValueType::F32,
        Some(Unit::Celsius),
        num(0.0, 95.0),
        AccessMode::RW,
        true,
    ),
    s(
        "wash_temp_band",
        ValueType::Enum,
        None,
        en(WASH_TEMP_BAND),
        AccessMode::RW,
        false,
    ),
    s(
        "spin_rpm",
        ValueType::U16,
        Some(Unit::Rpm),
        int(0, 1600),
        AccessMode::RW,
        true,
    ),
    s(
        "spin_off",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "soil_level",
        ValueType::Enum,
        None,
        en(SOIL_LEVEL),
        AccessMode::RW,
        false,
    ),
    s(
        "load_size",
        ValueType::Enum,
        None,
        en(LOAD_SIZE),
        AccessMode::RW,
        false,
    ),
    s(
        "extra_rinse",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "prewash",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s("steam", ValueType::Bool, None, None, AccessMode::RW, false),
    s(
        "rinse_hold",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "auto_dose",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "detergent_ml",
        ValueType::U16,
        Some(Unit::Milliliter),
        int(0, 200),
        AccessMode::RW,
        false,
    ),
    s(
        "softener_ml",
        ValueType::U16,
        Some(Unit::Milliliter),
        int(0, 100),
        AccessMode::RW,
        false,
    ),
    v(
        "detergent_level_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RE,
        false,
    ),
    v(
        "softener_level_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RE,
        false,
    ),
    v(
        "bleach_level_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RE,
        false,
    ),
    v(
        "unbalance",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "drum_rpm",
        ValueType::U16,
        Some(Unit::Rpm),
        int(0, 1600),
        AccessMode::RE,
        false,
    ),
];

const DRYER_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::TimeSchedule,
    TraitId::DoorLid,
    TraitId::ChildLock,
    TraitId::Cycle,
    TraitId::Program,
    TraitId::Temperature,
    TraitId::Humidity,
    TraitId::Heater,
    TraitId::Fan,
    TraitId::Filter,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Remote,
    TraitId::Maintenance,
    TraitId::Audio,
    TraitId::Safety,
];

const DRYER_PROGRAMS: &[&str] = &[
    "cotton",
    "synthetic",
    "delicates",
    "wool",
    "timed",
    "air_fluff",
    "bedding",
    "hygiene",
    "rack",
    "eco",
    "custom",
];

const DRYER_PHASES: &[&str] = &["heating", "drying", "cooling", "anti_crease", "complete"];
const DRYNESS: &[&str] = &["iron", "cupboard", "extra", "damp"];
const HEAT_LEVEL: &[&str] = &["low", "medium", "high", "air"];
const LINT_FILTER: &[&str] = &["ok", "missing", "clogged"];
const DRAIN_TANK: &[&str] = &["ok", "full", "missing", "na"];

static DRYER_POINTS: &[CatalogPoint] = &[
    s(
        "dryness",
        ValueType::Enum,
        None,
        en(DRYNESS),
        AccessMode::RW,
        false,
    ),
    s(
        "timed_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(0, 18000),
        AccessMode::RW,
        false,
    ),
    s(
        "heat_level",
        ValueType::Enum,
        None,
        en(HEAT_LEVEL),
        AccessMode::RW,
        false,
    ),
    s(
        "anti_crease",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "steam_refresh",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    v(
        "lint_filter",
        ValueType::Enum,
        None,
        en(LINT_FILTER),
        AccessMode::RE,
        true,
    ),
    v(
        "drain_tank",
        ValueType::Enum,
        None,
        en(DRAIN_TANK),
        AccessMode::RE,
        false,
    ),
    v(
        "dryness_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RE,
        false,
    ),
    v(
        "vent_blocked",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
];

const FRIDGE_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::DoorLid,
    TraitId::Temperature,
    TraitId::Zone,
    TraitId::Lighting,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Remote,
    TraitId::Maintenance,
];

const FRIDGE_OPTIONAL_TRAITS: &[TraitId] = &[
    TraitId::Ice,
    TraitId::Dispense,
    TraitId::Filter,
    TraitId::ChildLock,
    TraitId::Audio,
];

const FRIDGE_ZONES: &[&str] = &["fridge"];

static FRIDGE_POINTS: &[CatalogPoint] = &[
    s(
        "vacation_mode",
        ValueType::Bool,
        None,
        None,
        AccessMode::RWE,
        false,
    ),
    s(
        "sabbath_mode",
        ValueType::Bool,
        None,
        None,
        AccessMode::RWE,
        false,
    ),
    s(
        "eco_mode",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    v(
        "defrost_active",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "compressor_on",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "high_temp_alarm",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "power_fail_ms",
        ValueType::TimestampMs,
        None,
        None,
        AccessMode::RE,
        false,
    ),
];

const DISHWASHER_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::TimeSchedule,
    TraitId::DoorLid,
    TraitId::ChildLock,
    TraitId::Cycle,
    TraitId::Program,
    TraitId::Water,
    TraitId::Temperature,
    TraitId::Heater,
    TraitId::Filter,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Remote,
    TraitId::Maintenance,
    TraitId::Audio,
    TraitId::Safety,
];

const DISHWASHER_PROGRAMS: &[&str] = &[
    "auto",
    "eco",
    "intensive",
    "quick",
    "glass",
    "rinse",
    "hygiene",
    "night",
    "custom",
];

const DISHWASHER_PHASES: &[&str] = &["prewash", "wash", "rinse", "dry", "complete"];
const ZONE_WASH: &[&str] = &["all", "upper", "lower"];
const RINSE_AID_LEVEL: &[&str] = &["empty", "low", "ok"];
const SALT_LEVEL: &[&str] = &["empty", "low", "ok", "na"];

static DISHWASHER_POINTS: &[CatalogPoint] = &[
    s(
        "extra_dry",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "half_load",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "sanitize",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "zone_wash",
        ValueType::Enum,
        None,
        en(ZONE_WASH),
        AccessMode::RW,
        false,
    ),
    s(
        "tab_mode",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    v(
        "rinse_aid_level",
        ValueType::Enum,
        None,
        en(RINSE_AID_LEVEL),
        AccessMode::RE,
        false,
    ),
    v(
        "salt_level",
        ValueType::Enum,
        None,
        en(SALT_LEVEL),
        AccessMode::RE,
        false,
    ),
    s(
        "rinse_aid_dose",
        ValueType::U8,
        None,
        int(0, 6),
        AccessMode::RW,
        false,
    ),
    v(
        "turbidity",
        ValueType::U16,
        None,
        int(0, 1000),
        AccessMode::R,
        false,
    ),
    s(
        "wash_temp_c",
        ValueType::F32,
        Some(Unit::Celsius),
        num(30.0, 75.0),
        AccessMode::RW,
        false,
    ),
];

const MICROWAVE_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::DoorLid,
    TraitId::Cycle,
    TraitId::Program,
    TraitId::Lighting,
    TraitId::Audio,
    TraitId::ChildLock,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Remote,
    TraitId::Safety,
];

const MICROWAVE_OPTIONAL_TRAITS: &[TraitId] = &[TraitId::Heater];

const MICROWAVE_PROGRAMS: &[&str] = &[
    "manual",
    "sensor_reheat",
    "sensor_cook",
    "defrost",
    "popcorn",
    "beverage",
    "potato",
    "grill",
    "convection",
    "combo",
    "custom",
];

static MICROWAVE_POINTS: &[CatalogPoint] = &[
    s(
        "cook_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(1, 3600),
        AccessMode::RW,
        true,
    ),
    s(
        "power_level_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RW,
        true,
    ),
    s(
        "power_w",
        ValueType::U16,
        Some(Unit::Watt),
        int(0, 2000),
        AccessMode::RW,
        false,
    ),
    s(
        "defrost_g",
        ValueType::U16,
        Some(Unit::Gram),
        int(50, 4000),
        AccessMode::RW,
        false,
    ),
    s(
        "turntable",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    v(
        "inverter",
        ValueType::Bool,
        None,
        None,
        AccessMode::R,
        false,
    ),
    cmd("add_30s", Some(CatalogRange::CommandVoid), false),
];

const OVEN_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::DoorLid,
    TraitId::Temperature,
    TraitId::Cycle,
    TraitId::Program,
    TraitId::Heater,
    TraitId::Fan,
    TraitId::Lighting,
    TraitId::ChildLock,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Remote,
    TraitId::TimeSchedule,
    TraitId::Safety,
    TraitId::Audio,
];

const OVEN_PROGRAMS: &[&str] = &[
    "bake",
    "convection_bake",
    "roast",
    "convection_roast",
    "broil",
    "convection_broil",
    "proof",
    "keep_warm",
    "self_clean",
    "pyrolytic",
    "air_fry",
    "steam_assist",
    "sabbath",
    "off",
];

const BROIL_LEVEL: &[&str] = &["low", "high"];

static OVEN_POINTS: &[CatalogPoint] = &[
    s(
        "broil_level",
        ValueType::Enum,
        None,
        en(BROIL_LEVEL),
        AccessMode::RW,
        false,
    ),
    s(
        "convection_fan",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "steam_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RW,
        false,
    ),
    s(
        "cook_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(0, 43200),
        AccessMode::RW,
        false,
    ),
    v(
        "door_locked_clean",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "element_bake",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "element_broil",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
];

const HOB_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::ChildLock,
    TraitId::Heater,
    TraitId::Zone,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Safety,
];

const HOB_OPTIONAL_TRAITS: &[TraitId] = &[TraitId::Lighting];

const HOB_ZONES: &[&str] = &["hob_1", "hob_2", "hob_3", "hob_4"];
const PAN_SIZE: &[&str] = &["none", "small", "medium", "large", "unknown"];

static INDUCTION_HOB_POINTS: &[CatalogPoint] = &[
    CatalogPoint::setting(
        "level",
        ValueType::U8,
        None,
        int(0, 9),
        AccessMode::RWE,
        true,
    )
    .zoned(),
    CatalogPoint::setting("boost", ValueType::Bool, None, None, AccessMode::RWE, false).zoned(),
    CatalogPoint::setting(
        "timer_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(0, 10800),
        AccessMode::RWE,
        false,
    )
    .zoned(),
    CatalogPoint::setting("bridge", ValueType::Bool, None, None, AccessMode::RW, false).zoned(),
    CatalogPoint::variable(
        "residual_heat",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        true,
    )
    .zoned(),
    v(
        "flame_out",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "ignition_fail",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    s(
        "power_limit_w",
        ValueType::U32,
        Some(Unit::Watt),
        None,
        AccessMode::RW,
        false,
    ),
    cmd("pause_all", Some(CatalogRange::CommandVoid), false),
    cmd("resume_all", Some(CatalogRange::CommandVoid), false),
    CatalogPoint::variable(
        "pan_present",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        true,
    )
    .zoned(),
    CatalogPoint::variable(
        "pan_size",
        ValueType::Enum,
        None,
        en(PAN_SIZE),
        AccessMode::RE,
        false,
    )
    .zoned(),
    CatalogPoint::variable(
        "power_w",
        ValueType::U16,
        Some(Unit::Watt),
        int(0, 4000),
        AccessMode::RE,
        false,
    )
    .zoned(),
    v(
        "limiter_active",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "cookware_ok",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    s(
        "temp_mode",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "flex_group",
        ValueType::String,
        None,
        None,
        AccessMode::RW,
        false,
    ),
];

const KETTLE_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::Temperature,
    TraitId::Heater,
    TraitId::Cycle,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Safety,
    TraitId::Audio,
];

const KETTLE_OPTIONAL_TRAITS: &[TraitId] = &[TraitId::ChildLock];

static KETTLE_POINTS: &[CatalogPoint] = &[
    s(
        "keep_warm",
        ValueType::Bool,
        None,
        None,
        AccessMode::RWE,
        false,
    ),
    s(
        "keep_warm_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(0, 3600),
        AccessMode::RW,
        false,
    ),
    v("on_base", ValueType::Bool, None, None, AccessMode::RE, true),
    v(
        "boil_dry",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
];

const AIR_FRYER_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::Temperature,
    TraitId::Cycle,
    TraitId::Program,
    TraitId::Heater,
    TraitId::Fan,
    TraitId::DoorLid,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Audio,
    TraitId::Safety,
];

const AIR_FRYER_PROGRAMS: &[&str] = &[
    "manual",
    "fries",
    "wings",
    "reheat",
    "bake",
    "dehydrate",
    "fish",
    "veg",
    "custom",
];

static AIR_FRYER_POINTS: &[CatalogPoint] = &[
    s(
        "cook_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(1, 36000),
        AccessMode::RW,
        true,
    ),
    s(
        "shake_enable",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    v(
        "shake_due",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    s(
        "preheat",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    CatalogPoint::variable(
        "basket_present",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    )
    .zoned(),
    s(
        "sync_finish",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
];

pub const STATIC_CLASS_IDS: &[ApplianceClassId] = &[
    ApplianceClassId::Washer,
    ApplianceClassId::Dryer,
    ApplianceClassId::Fridge,
    ApplianceClassId::Dishwasher,
    ApplianceClassId::Microwave,
    ApplianceClassId::Oven,
    ApplianceClassId::InductionHob,
    ApplianceClassId::Kettle,
    ApplianceClassId::AirFryer,
];

pub const STATIC_CLASS_TABLES: &[ClassTable] = &[
    ClassTable {
        class_id: ApplianceClassId::Washer,
        typical_traits: WASHER_TRAITS,
        optional_traits: &[],
        class_points: WASHER_POINTS,
        program_tokens: WASHER_PROGRAMS,
        cycle_phase_tokens: WASHER_PHASES,
        typical_setpoint_c: None,
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::Dryer,
        typical_traits: DRYER_TRAITS,
        optional_traits: &[],
        class_points: DRYER_POINTS,
        program_tokens: DRYER_PROGRAMS,
        cycle_phase_tokens: DRYER_PHASES,
        typical_setpoint_c: None,
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::Fridge,
        typical_traits: FRIDGE_TRAITS,
        optional_traits: FRIDGE_OPTIONAL_TRAITS,
        class_points: FRIDGE_POINTS,
        program_tokens: &[],
        cycle_phase_tokens: &[],
        typical_setpoint_c: Some((1.0, 7.0)),
        typical_zones: FRIDGE_ZONES,
    },
    ClassTable {
        class_id: ApplianceClassId::Dishwasher,
        typical_traits: DISHWASHER_TRAITS,
        optional_traits: &[],
        class_points: DISHWASHER_POINTS,
        program_tokens: DISHWASHER_PROGRAMS,
        cycle_phase_tokens: DISHWASHER_PHASES,
        typical_setpoint_c: None,
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::Microwave,
        typical_traits: MICROWAVE_TRAITS,
        optional_traits: MICROWAVE_OPTIONAL_TRAITS,
        class_points: MICROWAVE_POINTS,
        program_tokens: MICROWAVE_PROGRAMS,
        cycle_phase_tokens: &[],
        typical_setpoint_c: None,
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::Oven,
        typical_traits: OVEN_TRAITS,
        optional_traits: &[],
        class_points: OVEN_POINTS,
        program_tokens: OVEN_PROGRAMS,
        cycle_phase_tokens: &[],
        typical_setpoint_c: Some((50.0, 250.0)),
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::InductionHob,
        typical_traits: HOB_TRAITS,
        optional_traits: HOB_OPTIONAL_TRAITS,
        class_points: INDUCTION_HOB_POINTS,
        program_tokens: &[],
        cycle_phase_tokens: &[],
        typical_setpoint_c: None,
        typical_zones: HOB_ZONES,
    },
    ClassTable {
        class_id: ApplianceClassId::Kettle,
        typical_traits: KETTLE_TRAITS,
        optional_traits: KETTLE_OPTIONAL_TRAITS,
        class_points: KETTLE_POINTS,
        program_tokens: &[],
        cycle_phase_tokens: &[],
        typical_setpoint_c: Some((40.0, 100.0)),
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::AirFryer,
        typical_traits: AIR_FRYER_TRAITS,
        optional_traits: &[],
        class_points: AIR_FRYER_POINTS,
        program_tokens: AIR_FRYER_PROGRAMS,
        cycle_phase_tokens: &[],
        typical_setpoint_c: Some((80.0, 200.0)),
        typical_zones: &[],
    },
];
