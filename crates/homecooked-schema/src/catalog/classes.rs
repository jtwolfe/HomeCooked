//! Static tables for encoded catalog classes (Tier-A batches).
//!
//! Point ids, types, units, ranges, and required flags are copied from
//! `docs/catalog/variables-and-settings.md`. Do not invent core point ids.

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

const fn dummy_point() -> CatalogPoint {
    CatalogPoint::variable("", ValueType::Bool, None, None, AccessMode::R, false)
}

const fn append_points<const N: usize>(
    dest: &mut [CatalogPoint; N],
    mut i: usize,
    src: &[CatalogPoint],
) -> usize {
    let mut j = 0;
    while j < src.len() {
        dest[i] = src[j];
        i += 1;
        j += 1;
    }
    i
}

const fn concat2<const N: usize>(a: &[CatalogPoint], b: &[CatalogPoint]) -> [CatalogPoint; N] {
    concat3(a, b, &[])
}

const fn concat3<const N: usize>(
    a: &[CatalogPoint],
    b: &[CatalogPoint],
    c: &[CatalogPoint],
) -> [CatalogPoint; N] {
    let mut dest = [dummy_point(); N];
    let mut i = 0;
    i = append_points(&mut dest, i, a);
    i = append_points(&mut dest, i, b);
    let _ = append_points(&mut dest, i, c);
    dest
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

const WASHER_POINTS: &[CatalogPoint] = &[
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

const DRYER_POINTS: &[CatalogPoint] = &[
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

/// Shared extras for fridge-like cold cabinets (fridge, freezer, fridge_freezer).
const COLD_CABINET_POINTS: &[CatalogPoint] = &[
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

const OVEN_POINTS: &[CatalogPoint] = &[
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

const COOKTOP_POINTS: &[CatalogPoint] = &[
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
];

const INDUCTION_HOB_EXTRA: &[CatalogPoint] = &[
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

const INDUCTION_HOB_MERGED: [CatalogPoint; 17] = concat2(COOKTOP_POINTS, INDUCTION_HOB_EXTRA);
const INDUCTION_HOB_POINTS: &[CatalogPoint] = &INDUCTION_HOB_MERGED;

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

const WASHER_DRYER_TRAITS: &[TraitId] = &[
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

const WASHER_DRYER_PROGRAMS: &[&str] = &[
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
    "timed",
    "air_fluff",
    "hygiene",
    "rack",
    "custom",
];

const WASHER_DRYER_PHASES: &[&str] = &[
    "fill",
    "prewash",
    "wash",
    "rinse",
    "spin",
    "drain",
    "soak",
    "steam",
    "heating",
    "drying",
    "cooling",
    "anti_crease",
    "complete",
];

const COMBO_MODE: &[&str] = &["wash_only", "dry_only", "wash_and_dry"];

const WASHER_DRYER_EXTRA: &[CatalogPoint] = &[
    s(
        "combo_mode",
        ValueType::Enum,
        None,
        en(COMBO_MODE),
        AccessMode::RW,
        true,
    ),
    s(
        "dry_after_wash",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "max_dry_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(0, 18000),
        AccessMode::RW,
        false,
    ),
];

const WASHER_DRYER_POINT_COUNT: usize = 30;
const WASHER_DRYER_MERGED: [CatalogPoint; WASHER_DRYER_POINT_COUNT] =
    concat3(WASHER_POINTS, DRYER_POINTS, WASHER_DRYER_EXTRA);
const WASHER_DRYER_POINTS: &[CatalogPoint] = &WASHER_DRYER_MERGED;

const FREEZER_TRAITS: &[TraitId] = &[
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
    TraitId::ChildLock,
    TraitId::Audio,
];

const FREEZER_ZONES: &[&str] = &["freezer"];

const FRIDGE_FREEZER_TRAITS: &[TraitId] = &[
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
    TraitId::ChildLock,
    TraitId::Audio,
];

const FRIDGE_FREEZER_OPTIONAL_TRAITS: &[TraitId] = &[
    TraitId::Ice,
    TraitId::Dispense,
    TraitId::Filter,
    TraitId::Humidity,
];

const FRIDGE_FREEZER_ZONES: &[&str] = &["fridge", "freezer"];

const WINE_COOLER_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::DoorLid,
    TraitId::Temperature,
    TraitId::Zone,
    TraitId::Humidity,
    TraitId::Lighting,
    TraitId::ChildLock,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Audio,
];

const WINE_COOLER_ZONES: &[&str] = &["upper", "lower"];

static WINE_COOLER_POINTS: &[CatalogPoint] = &[
    s(
        "vibration_reduce",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "uv_protect",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
];

const ICE_MAKER_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::DoorLid,
    TraitId::Ice,
    TraitId::Water,
    TraitId::Filter,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Maintenance,
    TraitId::ChildLock,
];

static ICE_MAKER_POINTS: &[CatalogPoint] = &[
    v(
        "clean_cycle_needed",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "water_temp_c",
        ValueType::F32,
        Some(Unit::Celsius),
        num(0.0, 40.0),
        AccessMode::R,
        false,
    ),
];

const WATER_HEATER_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::Temperature,
    TraitId::Heater,
    TraitId::Water,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::TimeSchedule,
    TraitId::Safety,
    TraitId::Maintenance,
];

const WATER_HEATER_MODE: &[&str] = &[
    "heat_pump",
    "hybrid",
    "electric",
    "vacation",
    "high_demand",
    "off",
];
const WATER_HEATER_FORM: &[&str] = &["tank", "tankless", "heat_pump"];

static WATER_HEATER_POINTS: &[CatalogPoint] = &[
    s(
        "mode",
        ValueType::Enum,
        None,
        en(WATER_HEATER_MODE),
        AccessMode::RW,
        false,
    ),
    v(
        "inlet_c",
        ValueType::F32,
        Some(Unit::Celsius),
        None,
        AccessMode::R,
        false,
    ),
    v(
        "outlet_c",
        ValueType::F32,
        Some(Unit::Celsius),
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "hot_remaining_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RE,
        false,
    ),
    v("leak", ValueType::Bool, None, None, AccessMode::RE, false),
    v(
        "dry_fire",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    s(
        "recirc_on",
        ValueType::Bool,
        None,
        None,
        AccessMode::RWE,
        false,
    ),
    v(
        "form_factor",
        ValueType::Enum,
        None,
        en(WATER_HEATER_FORM),
        AccessMode::R,
        false,
    ),
];

const HVAC_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::Temperature,
    TraitId::Humidity,
    TraitId::Fan,
    TraitId::Filter,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::TimeSchedule,
    TraitId::Zone,
    TraitId::Safety,
    TraitId::Maintenance,
];

const HVAC_MODE: &[&str] = &[
    "off",
    "heat",
    "cool",
    "auto",
    "fan_only",
    "dry",
    "emergency_heat",
];
const REVERSING_VALVE: &[&str] = &["heat", "cool", "unknown"];

static HVAC_POINTS: &[CatalogPoint] = &[
    s(
        "hvac_mode",
        ValueType::Enum,
        None,
        en(HVAC_MODE),
        AccessMode::RWE,
        true,
    ),
    s(
        "heat_setpoint_c",
        ValueType::F32,
        Some(Unit::Celsius),
        num(10.0, 32.0),
        AccessMode::RWE,
        false,
    ),
    s(
        "cool_setpoint_c",
        ValueType::F32,
        Some(Unit::Celsius),
        num(10.0, 32.0),
        AccessMode::RWE,
        false,
    ),
    s(
        "deadband_c",
        ValueType::F32,
        Some(Unit::Celsius),
        num(0.5, 5.0),
        AccessMode::RW,
        false,
    ),
    v(
        "space_c",
        ValueType::F32,
        Some(Unit::Celsius),
        None,
        AccessMode::RE,
        true,
    ),
    v(
        "outdoor_c",
        ValueType::F32,
        Some(Unit::Celsius),
        None,
        AccessMode::R,
        false,
    ),
    s("hold", ValueType::Bool, None, None, AccessMode::RWE, false),
    s("quiet", ValueType::Bool, None, None, AccessMode::RW, false),
    s("eco", ValueType::Bool, None, None, AccessMode::RW, false),
    v(
        "compressor_on",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "aux_heat",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "defrost",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "reversing_valve",
        ValueType::Enum,
        None,
        en(REVERSING_VALVE),
        AccessMode::R,
        false,
    ),
];

const DEHUMIDIFIER_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::Humidity,
    TraitId::Fan,
    TraitId::Water,
    TraitId::Filter,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::TimeSchedule,
];

static DEHUMIDIFIER_POINTS: &[CatalogPoint] = &[
    v(
        "tank_full",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    s(
        "pump_mode",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    v(
        "defrost",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
];

const RANGE_HOOD_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::Fan,
    TraitId::Lighting,
    TraitId::Filter,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Remote,
    TraitId::Audio,
];

const GREASE_FILTER: &[&str] = &["ok", "clogged", "missing"];
const CHARCOAL_FILTER: &[&str] = &["ok", "replace", "na"];

static RANGE_HOOD_POINTS: &[CatalogPoint] = &[
    s(
        "auto_mode",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    s(
        "delay_off_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(0, 1800),
        AccessMode::RW,
        false,
    ),
    v(
        "voc_index",
        ValueType::U16,
        None,
        int(0, 500),
        AccessMode::RE,
        false,
    ),
    v(
        "grease_filter",
        ValueType::Enum,
        None,
        en(GREASE_FILTER),
        AccessMode::RE,
        false,
    ),
    v(
        "charcoal_filter",
        ValueType::Enum,
        None,
        en(CHARCOAL_FILTER),
        AccessMode::RE,
        false,
    ),
];

const STEAM_OVEN_TRAITS: &[TraitId] = &[
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
    TraitId::Water,
    TraitId::Humidity,
    TraitId::Filter,
];

const STEAM_MODE: &[&str] = &[
    "steam",
    "combi",
    "convection",
    "sous_vide",
    "reheat",
    "descale",
];
const WATER_TANK: &[&str] = &["ok", "low", "empty", "missing"];

const STEAM_OVEN_EXTRA: &[CatalogPoint] = &[
    s(
        "steam_mode",
        ValueType::Enum,
        None,
        en(STEAM_MODE),
        AccessMode::RW,
        true,
    ),
    s(
        "humidity_set_percent",
        ValueType::Percent,
        Some(Unit::Percent),
        num(0.0, 100.0),
        AccessMode::RW,
        false,
    ),
    v(
        "water_tank",
        ValueType::Enum,
        None,
        en(WATER_TANK),
        AccessMode::RE,
        true,
    ),
];

const STEAM_OVEN_MERGED: [CatalogPoint; 10] = concat2(OVEN_POINTS, STEAM_OVEN_EXTRA);
const STEAM_OVEN_POINTS: &[CatalogPoint] = &STEAM_OVEN_MERGED;

const TOASTER_OVEN_PROGRAMS: &[&str] = &[
    "toast",
    "bake",
    "broil",
    "air_fry",
    "keep_warm",
    "convection",
];
const CRUMB_TRAY: &[&str] = &["ok", "missing", "unknown"];

const TOASTER_OVEN_EXTRA: &[CatalogPoint] = &[
    s(
        "toast_shade",
        ValueType::U8,
        None,
        int(1, 7),
        AccessMode::RW,
        false,
    ),
    v(
        "crumb_tray",
        ValueType::Enum,
        None,
        en(CRUMB_TRAY),
        AccessMode::RE,
        false,
    ),
];

const TOASTER_OVEN_MERGED: [CatalogPoint; 9] = concat2(OVEN_POINTS, TOASTER_OVEN_EXTRA);
const TOASTER_OVEN_POINTS: &[CatalogPoint] = &TOASTER_OVEN_MERGED;

const RANGE_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::ChildLock,
    TraitId::Heater,
    TraitId::Zone,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Safety,
    TraitId::DoorLid,
    TraitId::Temperature,
    TraitId::Cycle,
    TraitId::Program,
    TraitId::Fan,
    TraitId::Lighting,
    TraitId::Remote,
    TraitId::TimeSchedule,
    TraitId::Audio,
];

const RANGE_ZONES: &[&str] = &["hob_1", "hob_2", "hob_3", "hob_4", "oven"];
const RANGE_SURFACE: &[&str] = &["gas", "electric", "radiant", "induction", "mixed"];

const RANGE_SURFACE_POINTS: &[CatalogPoint] = &[v(
    "surface",
    ValueType::Enum,
    None,
    en(RANGE_SURFACE),
    AccessMode::R,
    true,
)];

const RANGE_MERGED: [CatalogPoint; 18] = concat3(COOKTOP_POINTS, OVEN_POINTS, RANGE_SURFACE_POINTS);
const RANGE_POINTS: &[CatalogPoint] = &RANGE_MERGED;

const COFFEE_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::Water,
    TraitId::Temperature,
    TraitId::Cycle,
    TraitId::Program,
    TraitId::ChildLock,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::Maintenance,
    TraitId::Filter,
    TraitId::Audio,
];

const COFFEE_OPTIONAL_TRAITS: &[TraitId] = &[TraitId::Lighting];

const COFFEE_PROGRAMS: &[&str] = &[
    "espresso",
    "double_espresso",
    "americano",
    "lungo",
    "cappuccino",
    "latte",
    "macchiato",
    "hot_water",
    "steam",
    "rinse",
    "descale",
    "custom",
];

const COFFEE_STRENGTH: &[&str] = &["mild", "normal", "strong", "extra"];
const DRIP_TRAY: &[&str] = &["ok", "full", "missing"];
const GROUNDS_BIN: &[&str] = &["ok", "full", "missing"];

static COFFEE_MACHINE_POINTS: &[CatalogPoint] = &[
    s(
        "strength",
        ValueType::Enum,
        None,
        en(COFFEE_STRENGTH),
        AccessMode::RW,
        false,
    ),
    s(
        "volume_ml",
        ValueType::U16,
        Some(Unit::Milliliter),
        int(15, 400),
        AccessMode::RW,
        false,
    ),
    s(
        "milk_ml",
        ValueType::U16,
        Some(Unit::Milliliter),
        int(0, 400),
        AccessMode::RW,
        false,
    ),
    s(
        "grind_level",
        ValueType::U8,
        None,
        int(1, 16),
        AccessMode::RW,
        false,
    ),
    s(
        "cups",
        ValueType::U8,
        None,
        int(1, 2),
        AccessMode::RW,
        false,
    ),
    v(
        "water_tank",
        ValueType::Enum,
        None,
        en(WATER_TANK),
        AccessMode::RE,
        true,
    ),
    v(
        "drip_tray",
        ValueType::Enum,
        None,
        en(DRIP_TRAY),
        AccessMode::RE,
        false,
    ),
    v(
        "grounds_bin",
        ValueType::Enum,
        None,
        en(GROUNDS_BIN),
        AccessMode::RE,
        false,
    ),
    v(
        "milk_present",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "capsule_present",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "boiler_c",
        ValueType::F32,
        Some(Unit::Celsius),
        None,
        AccessMode::RE,
        false,
    ),
    v(
        "brew_pressure_bar",
        ValueType::F32,
        Some(Unit::Bar),
        num(0.0, 20.0),
        AccessMode::RE,
        false,
    ),
];

const SOUS_VIDE_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::Temperature,
    TraitId::Cycle,
    TraitId::Heater,
    TraitId::Fan,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::TimeSchedule,
    TraitId::Safety,
    TraitId::Audio,
];

static SOUS_VIDE_POINTS: &[CatalogPoint] = &[
    v(
        "low_water",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        true,
    ),
    v(
        "circulating",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
    s(
        "cook_s",
        ValueType::DurationS,
        Some(Unit::Second),
        int(0, 259200),
        AccessMode::RW,
        false,
    ),
];

const MULTI_COOKER_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    TraitId::Temperature,
    TraitId::Cycle,
    TraitId::Program,
    TraitId::Heater,
    TraitId::DoorLid,
    TraitId::ChildLock,
    TraitId::Fault,
    TraitId::Energy,
    TraitId::TimeSchedule,
    TraitId::Safety,
    TraitId::Audio,
];

const MULTI_COOKER_PROGRAMS: &[&str] = &[
    "pressure",
    "saute",
    "slow",
    "steam",
    "rice",
    "yogurt",
    "sous_vide",
    "keep_warm",
    "sterilize",
    "custom",
];

const MULTI_COOKER_PHASES: &[&str] = &[
    "preheat",
    "pressurizing",
    "at_pressure",
    "cooking",
    "venting",
    "keep_warm",
    "safe_to_open",
];

const PRESSURE_BAND: &[&str] = &["low", "high"];
const FLOAT_VALVE: &[&str] = &["down", "up"];

static MULTI_COOKER_POINTS: &[CatalogPoint] = &[
    s(
        "pressure_band",
        ValueType::Enum,
        None,
        en(PRESSURE_BAND),
        AccessMode::RW,
        false,
    ),
    v(
        "pressure_kpa",
        ValueType::F32,
        Some(Unit::Kilopascal),
        num(0.0, 150.0),
        AccessMode::RE,
        false,
    ),
    v(
        "lid_locked",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        true,
    ),
    v(
        "float_valve",
        ValueType::Enum,
        None,
        en(FLOAT_VALVE),
        AccessMode::RE,
        false,
    ),
    v(
        "safe_to_open",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        true,
    ),
    s(
        "remote_vent_enabled",
        ValueType::Bool,
        None,
        None,
        AccessMode::RW,
        false,
    ),
    cmd("vent", Some(CatalogRange::CommandVoid), false),
    v(
        "burn_detected",
        ValueType::Bool,
        None,
        None,
        AccessMode::RE,
        false,
    ),
];

/// Roadmap §4 Tier-A class set (25 ids). Tables land in batches.
pub const TIER_A_CLASS_IDS: &[ApplianceClassId] = &[
    ApplianceClassId::Washer,
    ApplianceClassId::Dryer,
    ApplianceClassId::WasherDryer,
    ApplianceClassId::Fridge,
    ApplianceClassId::Freezer,
    ApplianceClassId::FridgeFreezer,
    ApplianceClassId::Dishwasher,
    ApplianceClassId::Microwave,
    ApplianceClassId::Oven,
    ApplianceClassId::SteamOven,
    ApplianceClassId::Range,
    ApplianceClassId::Cooktop,
    ApplianceClassId::InductionHob,
    ApplianceClassId::AirFryer,
    ApplianceClassId::Kettle,
    ApplianceClassId::CoffeeMachine,
    ApplianceClassId::WaterHeater,
    ApplianceClassId::Hvac,
    ApplianceClassId::Dehumidifier,
    ApplianceClassId::RangeHood,
    ApplianceClassId::ToasterOven,
    ApplianceClassId::SousVide,
    ApplianceClassId::MultiCooker,
    ApplianceClassId::IceMaker,
    ApplianceClassId::WineCooler,
];

/// Classes with a static `ClassTable` (full Tier-A set).
pub const STATIC_CLASS_IDS: &[ApplianceClassId] = TIER_A_CLASS_IDS;

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
        class_points: COLD_CABINET_POINTS,
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
    ClassTable {
        class_id: ApplianceClassId::WasherDryer,
        typical_traits: WASHER_DRYER_TRAITS,
        optional_traits: &[],
        class_points: WASHER_DRYER_POINTS,
        program_tokens: WASHER_DRYER_PROGRAMS,
        cycle_phase_tokens: WASHER_DRYER_PHASES,
        typical_setpoint_c: None,
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::Freezer,
        typical_traits: FREEZER_TRAITS,
        optional_traits: &[],
        class_points: COLD_CABINET_POINTS,
        program_tokens: &[],
        cycle_phase_tokens: &[],
        typical_setpoint_c: Some((-24.0, -12.0)),
        typical_zones: FREEZER_ZONES,
    },
    ClassTable {
        class_id: ApplianceClassId::FridgeFreezer,
        typical_traits: FRIDGE_FREEZER_TRAITS,
        optional_traits: FRIDGE_FREEZER_OPTIONAL_TRAITS,
        class_points: COLD_CABINET_POINTS,
        program_tokens: &[],
        cycle_phase_tokens: &[],
        typical_setpoint_c: Some((-24.0, 7.0)),
        typical_zones: FRIDGE_FREEZER_ZONES,
    },
    ClassTable {
        class_id: ApplianceClassId::WineCooler,
        typical_traits: WINE_COOLER_TRAITS,
        optional_traits: &[],
        class_points: WINE_COOLER_POINTS,
        program_tokens: &[],
        cycle_phase_tokens: &[],
        typical_setpoint_c: Some((5.0, 20.0)),
        typical_zones: WINE_COOLER_ZONES,
    },
    ClassTable {
        class_id: ApplianceClassId::IceMaker,
        typical_traits: ICE_MAKER_TRAITS,
        optional_traits: &[],
        class_points: ICE_MAKER_POINTS,
        program_tokens: &[],
        cycle_phase_tokens: &[],
        typical_setpoint_c: None,
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::WaterHeater,
        typical_traits: WATER_HEATER_TRAITS,
        optional_traits: &[],
        class_points: WATER_HEATER_POINTS,
        program_tokens: &[],
        cycle_phase_tokens: &[],
        typical_setpoint_c: Some((40.0, 70.0)),
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::Hvac,
        typical_traits: HVAC_TRAITS,
        optional_traits: &[],
        class_points: HVAC_POINTS,
        program_tokens: &[],
        cycle_phase_tokens: &[],
        typical_setpoint_c: None,
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::Dehumidifier,
        typical_traits: DEHUMIDIFIER_TRAITS,
        optional_traits: &[],
        class_points: DEHUMIDIFIER_POINTS,
        program_tokens: &[],
        cycle_phase_tokens: &[],
        typical_setpoint_c: None,
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::RangeHood,
        typical_traits: RANGE_HOOD_TRAITS,
        optional_traits: &[],
        class_points: RANGE_HOOD_POINTS,
        program_tokens: &[],
        cycle_phase_tokens: &[],
        typical_setpoint_c: None,
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::SteamOven,
        typical_traits: STEAM_OVEN_TRAITS,
        optional_traits: &[],
        class_points: STEAM_OVEN_POINTS,
        program_tokens: OVEN_PROGRAMS,
        cycle_phase_tokens: &[],
        typical_setpoint_c: Some((50.0, 250.0)),
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::Range,
        typical_traits: RANGE_TRAITS,
        optional_traits: HOB_OPTIONAL_TRAITS,
        class_points: RANGE_POINTS,
        program_tokens: OVEN_PROGRAMS,
        cycle_phase_tokens: &[],
        typical_setpoint_c: Some((50.0, 250.0)),
        typical_zones: RANGE_ZONES,
    },
    ClassTable {
        class_id: ApplianceClassId::Cooktop,
        typical_traits: HOB_TRAITS,
        optional_traits: HOB_OPTIONAL_TRAITS,
        class_points: COOKTOP_POINTS,
        program_tokens: &[],
        cycle_phase_tokens: &[],
        typical_setpoint_c: None,
        typical_zones: HOB_ZONES,
    },
    ClassTable {
        class_id: ApplianceClassId::ToasterOven,
        typical_traits: OVEN_TRAITS,
        optional_traits: &[],
        class_points: TOASTER_OVEN_POINTS,
        program_tokens: TOASTER_OVEN_PROGRAMS,
        cycle_phase_tokens: &[],
        typical_setpoint_c: Some((50.0, 250.0)),
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::CoffeeMachine,
        typical_traits: COFFEE_TRAITS,
        optional_traits: COFFEE_OPTIONAL_TRAITS,
        class_points: COFFEE_MACHINE_POINTS,
        program_tokens: COFFEE_PROGRAMS,
        cycle_phase_tokens: &[],
        typical_setpoint_c: None,
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::SousVide,
        typical_traits: SOUS_VIDE_TRAITS,
        optional_traits: &[],
        class_points: SOUS_VIDE_POINTS,
        program_tokens: &[],
        cycle_phase_tokens: &[],
        typical_setpoint_c: Some((20.0, 95.0)),
        typical_zones: &[],
    },
    ClassTable {
        class_id: ApplianceClassId::MultiCooker,
        typical_traits: MULTI_COOKER_TRAITS,
        optional_traits: &[],
        class_points: MULTI_COOKER_POINTS,
        program_tokens: MULTI_COOKER_PROGRAMS,
        cycle_phase_tokens: MULTI_COOKER_PHASES,
        typical_setpoint_c: None,
        typical_zones: &[],
    },
];
