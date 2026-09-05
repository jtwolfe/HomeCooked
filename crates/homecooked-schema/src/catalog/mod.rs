//! Static catalog tables derived from `docs/catalog`.

mod classes;
mod traits;

use crate::access::AccessMode;
use crate::capability::{CapabilityModel, PointCapability, TraitCapability};
use crate::ids::{ApplianceClassId, PointNamespace, TraitId};
use crate::spec::CatalogPoint;
use crate::types::{ValueRange, ValueType};
use crate::version::{CATALOG_VERSION, DEFAULT_CLASS_VERSION, DEFAULT_TRAIT_VERSION};

pub use classes::{STATIC_CLASS_IDS, TIER_A_CLASS_IDS, TIER_B_CLASS_IDS};
pub use traits::trait_table;

/// Catalog Index groups from `docs/catalog/appliances.md`, in table order.
pub const CATALOG_GROUP_ORDER: &[&str] = &[
    "Laundry",
    "Cold",
    "Wash",
    "Cooking",
    "Ventilation",
    "Beverage",
    "Countertop",
    "Utility",
    "Climate",
];

/// Index table group for a class id (`docs/catalog/appliances.md`).
pub fn catalog_group(class_id: ApplianceClassId) -> &'static str {
    use ApplianceClassId::*;
    match class_id {
        Washer | Dryer | WasherDryer => "Laundry",
        Fridge | Freezer | FridgeFreezer | WineCooler | BeverageCooler | IceMaker | Kegerator => {
            "Cold"
        }
        Dishwasher => "Wash",
        Microwave | Oven | SteamOven | ToasterOven | Range | Cooktop | InductionHob
        | WarmingDrawer | PizzaOven | AirFryer | ElectricGrill | ElectricSmoker => "Cooking",
        RangeHood => "Ventilation",
        CoffeeMachine | EspressoMachine | DripCoffeeMaker | CoffeeGrinder | Kettle
        | WaterDispenser => "Beverage",
        Toaster | Blender | FoodProcessor | StandMixer | Juicer | RiceCooker | SlowCooker
        | MultiCooker | SousVide | BreadMaker | Dehydrator | VacuumSealer | IceCreamMaker
        | YogurtMaker | WaffleMaker | PastaMaker | SteamCooker => "Countertop",
        GarbageDisposal | TrashCompactor | WaterHeater | Boiler | WaterSoftener | WaterFilter => {
            "Utility"
        }
        Hvac | Dehumidifier | Humidifier => "Climate",
    }
}

/// Shared-trait table: required and optional points for one trait.
#[derive(Debug, Clone, Copy)]
pub struct TraitTable {
    pub trait_id: TraitId,
    pub points: &'static [CatalogPoint],
}

impl TraitTable {
    pub fn point(&self, id: &str) -> Option<&'static CatalogPoint> {
        self.points.iter().find(|p| p.id == id)
    }

    pub fn required_points(&self) -> impl Iterator<Item = &'static CatalogPoint> {
        self.points.iter().filter(|p| p.required)
    }
}

/// Static table for a class with an encoding in this crate (Tier-A / Tier-B).
#[derive(Debug, Clone, Copy)]
pub struct ClassTable {
    pub class_id: ApplianceClassId,
    pub typical_traits: &'static [TraitId],
    pub optional_traits: &'static [TraitId],
    pub class_points: &'static [CatalogPoint],
    pub program_tokens: &'static [&'static str],
    pub cycle_phase_tokens: &'static [&'static str],
    /// Typical `trait.temperature.setpoint_c` range when the class is closed-loop.
    pub typical_setpoint_c: Option<(f32, f32)>,
    /// Default zone ids advertised when the class uses `trait.zone`.
    pub typical_zones: &'static [&'static str],
    /// Static heat-port advertisement metadata (`HeatPortSpec`). Empty for
    /// classes without a thermal-port surface; catalog `thermal_port_*` points
    /// remain the device RW surface when present.
    pub thermal_ports: &'static [crate::HeatPortSpec],
}

impl ClassTable {
    pub fn class_point(&self, id: &str) -> Option<&'static CatalogPoint> {
        self.class_points.iter().find(|p| p.id == id)
    }

    pub fn required_class_points(&self) -> impl Iterator<Item = &'static CatalogPoint> {
        self.class_points.iter().filter(|p| p.required)
    }
}

/// Every class id in the appliances.md index table (washer through humidifier).
pub fn list_all_class_ids() -> &'static [ApplianceClassId] {
    ApplianceClassId::ALL
}

/// Static table for one of the fully-encoded (static) classes, if present.
pub fn class_table(class_id: ApplianceClassId) -> Option<&'static ClassTable> {
    classes::STATIC_CLASS_TABLES
        .iter()
        .find(|t| t.class_id == class_id)
}

/// All fully-encoded class tables.
pub fn static_class_tables() -> &'static [ClassTable] {
    classes::STATIC_CLASS_TABLES
}

/// Typical advertised capability for a fully-encoded class.
///
/// Includes typical traits, each trait's required points, required class
/// points, and closed-loop `setpoint_c` when the class documents a typical
/// range. Optional catalog points are omitted unless a device advertises them.
pub fn typical_capability(class_id: ApplianceClassId) -> Option<CapabilityModel> {
    let table = class_table(class_id)?;
    let mut model = CapabilityModel::new(class_id);
    model.class_version = DEFAULT_CLASS_VERSION;
    model.catalog_version = CATALOG_VERSION;

    let zones: Option<Vec<String>> = if table.typical_zones.is_empty() {
        None
    } else {
        Some(
            table
                .typical_zones
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        )
    };

    for &trait_id in table.typical_traits {
        let Some(tt) = trait_table(trait_id) else {
            continue;
        };
        let mut points = Vec::new();
        for p in tt.points {
            let include = p.required || extra_typical_trait_point(table, trait_id, p);
            if !include {
                continue;
            }
            let mut cap =
                PointCapability::from_catalog(format!("trait.{}.{}", trait_id.as_str(), p.id), p);
            cap = specialize_trait_point(table, trait_id, p, cap);
            if p.zoned {
                if let Some(z) = &zones {
                    cap.zones = Some(z.clone());
                }
            }
            points.push(cap);
        }
        model.traits.push(TraitCapability {
            trait_id,
            trait_version: DEFAULT_TRAIT_VERSION,
            points,
        });
    }

    for p in table.class_points {
        if !p.required && !extra_typical_class_point(table, p) {
            continue;
        }
        let mut cap =
            PointCapability::from_catalog(format!("class.{}.{}", class_id.as_str(), p.id), p);
        if p.zoned {
            if let Some(z) = &zones {
                cap.zones = Some(z.clone());
            }
        }
        model.class_points.push(cap);
    }

    Some(model)
}

fn extra_typical_trait_point(table: &ClassTable, trait_id: TraitId, point: &CatalogPoint) -> bool {
    if trait_id == TraitId::Temperature
        && point.id == "setpoint_c"
        && table.typical_setpoint_c.is_some()
    {
        return true;
    }
    // Oven / range: meat probe + preheat complete are typical cavity telemetry.
    if matches!(
        table.class_id,
        ApplianceClassId::Oven | ApplianceClassId::Range
    ) && trait_id == TraitId::Temperature
        && matches!(
            point.id,
            "probe_c" | "probe_target_c" | "probe_connected" | "preheat_complete"
        )
    {
        return true;
    }
    // Wine cooler: humidity target is typical when the cabinet actively humidifies.
    if table.class_id == ApplianceClassId::WineCooler
        && trait_id == TraitId::Humidity
        && point.id == "setpoint_rh"
    {
        return true;
    }
    // Ice maker: bin level/full + filter life are typical demo telemetry.
    if table.class_id == ApplianceClassId::IceMaker {
        if trait_id == TraitId::Ice && matches!(point.id, "bin_full" | "bin_percent") {
            return true;
        }
        if trait_id == TraitId::Filter && point.id == "life_percent" {
            return true;
        }
    }
    // Sous-vide: cycle remaining is typical cook-timer telemetry.
    if table.class_id == ApplianceClassId::SousVide
        && trait_id == TraitId::Cycle
        && point.id == "remaining_s"
    {
        return true;
    }
    // Multi-cooker: cycle remaining is typical cook-timer telemetry.
    if table.class_id == ApplianceClassId::MultiCooker
        && trait_id == TraitId::Cycle
        && point.id == "remaining_s"
    {
        return true;
    }
    // Toaster oven: cycle remaining is typical toast/bake timer telemetry.
    if table.class_id == ApplianceClassId::ToasterOven
        && trait_id == TraitId::Cycle
        && point.id == "remaining_s"
    {
        return true;
    }
    // Dehumidifier: humidity setpoint + fan speed are typical controls.
    if table.class_id == ApplianceClassId::Dehumidifier {
        if trait_id == TraitId::Humidity && point.id == "setpoint_rh" {
            return true;
        }
        if trait_id == TraitId::Fan && point.id == "fan_speed" {
            return true;
        }
    }
    // Humidifier: humidity setpoint is a typical comfort control.
    if table.class_id == ApplianceClassId::Humidifier
        && trait_id == TraitId::Humidity
        && point.id == "setpoint_rh"
    {
        return true;
    }
    // Range hood: fan speed + light dimming + grease-filter life are typical.
    if table.class_id == ApplianceClassId::RangeHood {
        if trait_id == TraitId::Fan && point.id == "fan_speed" {
            return true;
        }
        if trait_id == TraitId::Lighting && point.id == "light_percent" {
            return true;
        }
        if trait_id == TraitId::Filter && point.id == "life_percent" {
            return true;
        }
    }
    // Water dispenser: filter life is typical demo telemetry (child_lock already required).
    if table.class_id == ApplianceClassId::WaterDispenser
        && trait_id == TraitId::Filter
        && point.id == "life_percent"
    {
        return true;
    }
    // Water softener: hardness input + resin/filter life are typical.
    if table.class_id == ApplianceClassId::WaterSoftener {
        if trait_id == TraitId::Water && point.id == "hardness_ppm" {
            return true;
        }
        if trait_id == TraitId::Filter && point.id == "life_percent" {
            return true;
        }
    }
    // Water filter: filter life + measured flow are typical.
    if table.class_id == ApplianceClassId::WaterFilter {
        if trait_id == TraitId::Filter && point.id == "life_percent" {
            return true;
        }
        if trait_id == TraitId::Water && point.id == "flow_l_min" {
            return true;
        }
    }
    // Washer / dryer / washer_dryer / dishwasher: delay start is typical
    // laundry/wash scheduling (reuse TimeSchedule).
    if matches!(
        table.class_id,
        ApplianceClassId::Washer
            | ApplianceClassId::Dryer
            | ApplianceClassId::WasherDryer
            | ApplianceClassId::Dishwasher
    ) && trait_id == TraitId::TimeSchedule
        && point.id == "delay_start_s"
    {
        return true;
    }
    // Steam oven: cycle remaining + water hardness are typical.
    if table.class_id == ApplianceClassId::SteamOven {
        if trait_id == TraitId::Cycle && point.id == "remaining_s" {
            return true;
        }
        if trait_id == TraitId::Water && point.id == "hardness_ppm" {
            return true;
        }
    }
    false
}

/// Optional class points that the typical model still advertises for demos.
fn extra_typical_class_point(table: &ClassTable, point: &CatalogPoint) -> bool {
    // Stream 7 undepened Tier-A deepen: air_fryer optional telemetry/settings.
    // Advertise thin-table shake_enable/shake_due/preheat/basket_present/sync_finish;
    // add depth sabbath/eco/heater_on/fan_on/high_temp_alarm/door_ajar/timer_s.
    // Do not duplicate required cook_s (already required). Heater/Fan/DoorLid
    // traits already typical — class heater_on/fan_on/door_ajar are compact RE
    // telemetry (dehydrator / oven template), not trait replacements.
    if table.class_id == ApplianceClassId::AirFryer {
        return matches!(
            point.id,
            "shake_enable"
                | "shake_due"
                | "preheat"
                | "basket_present"
                | "sync_finish"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "fan_on"
                | "high_temp_alarm"
                | "door_ajar"
                | "timer_s"
        );
    }
    // Stream 7 undepened Tier-A deepen: oven optional telemetry/settings.
    // Advertise thin-table broil/convection/steam/cook/door_locked_clean/elements;
    // add OVEN_DEPTH sabbath/eco/heater_on/high_temp_alarm/door_ajar/timer_s.
    // Depth stays off OVEN_BASE so range/steam_oven/toaster_oven composition
    // is unchanged. Self-clean via program tokens + door_locked_clean (no
    // parallel self_clean class bool). Meat probe via Temperature trait.
    if table.class_id == ApplianceClassId::Oven {
        return matches!(
            point.id,
            "broil_level"
                | "convection_fan"
                | "steam_percent"
                | "cook_s"
                | "door_locked_clean"
                | "element_bake"
                | "element_broil"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "door_ajar"
                | "timer_s"
        );
    }
    // Stream 7 undepened Tier-A deepen: range combo optional telemetry/settings.
    // Advertise cooktop depth already on COOKTOP_POINTS + OVEN_BASE thin cavity
    // surface; add RANGE_EXTRA sabbath/eco/heater_on/high_temp_alarm/door_ajar
    // (not OVEN_DEPTH — timer_s would collide with cooktop zoned timer_s).
    // Required `surface` / `level` / `residual_heat` already typical via required.
    if table.class_id == ApplianceClassId::Range {
        return matches!(
            point.id,
            "boost"
                | "timer_s"
                | "bridge"
                | "flame_out"
                | "ignition_fail"
                | "power_limit_w"
                | "keep_warm"
                | "hotspot_alert"
                | "timer_active"
                | "paused"
                | "surface_c"
                | "element_fault"
                | "pan_detect"
                | "flame_on"
                | "broil_level"
                | "convection_fan"
                | "steam_percent"
                | "cook_s"
                | "door_locked_clean"
                | "element_bake"
                | "element_broil"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "door_ajar"
        );
    }
    // Stream 7 undepened Tier-A deepen: induction_hob optional telemetry/settings.
    // Advertise cooktop depth already on COOKTOP_POINTS + thin INDUCTION_HOB_EXTRA
    // (pan_size/power_w/limiter/cookware/temp_mode/flex); add EXTRA sabbath/eco/
    // power_share/auto_boost/overtemp. Required level/residual_heat/pan_present
    // already typical. ChildLock already on HOB_TRAITS. Do not redeclare cooktop
    // timer_s / pan_detect / residual_heat on EXTRA (id collisions).
    if table.class_id == ApplianceClassId::InductionHob {
        return matches!(
            point.id,
            "boost"
                | "timer_s"
                | "bridge"
                | "flame_out"
                | "ignition_fail"
                | "power_limit_w"
                | "keep_warm"
                | "hotspot_alert"
                | "timer_active"
                | "paused"
                | "surface_c"
                | "element_fault"
                | "pan_detect"
                | "flame_on"
                | "pan_size"
                | "power_w"
                | "limiter_active"
                | "cookware_ok"
                | "temp_mode"
                | "flex_group"
                | "sabbath_mode"
                | "eco_mode"
                | "power_share"
                | "auto_boost"
                | "overtemp_alarm"
        );
    }
    // Stream 7 undepened Tier-A deepen: microwave optional telemetry/settings.
    // Advertise thin-table power_w/defrost_g/turntable/inverter; add depth
    // sabbath/eco/door_ajar/magnetron_on/high_temp_alarm/timer_s. Do not
    // duplicate required cook_s / power_level_percent (already required).
    // ChildLock trait already on typical traits.
    if table.class_id == ApplianceClassId::Microwave {
        return matches!(
            point.id,
            "power_w"
                | "defrost_g"
                | "turntable"
                | "inverter"
                | "sabbath_mode"
                | "eco_mode"
                | "door_ajar"
                | "magnetron_on"
                | "high_temp_alarm"
                | "timer_s"
        );
    }
    // Stream 7 undepened Tier-A deepen: dishwasher optional telemetry/settings.
    // Advertise thin-table rinse_aid_level/salt_level + wash_temp_c; add depth
    // sabbath/eco/door/alarms/timer. thermal_port_* stay via Stream 5 match below
    // (do not return false here).
    if table.class_id == ApplianceClassId::Dishwasher
        && matches!(
            point.id,
            "wash_temp_c"
                | "rinse_aid_level"
                | "salt_level"
                | "sabbath_mode"
                | "eco_mode"
                | "door_ajar"
                | "door_locked"
                | "rinse_aid_low"
                | "salt_low"
                | "overflow_alarm"
                | "timer_s"
        )
    {
        return true;
    }
    // Stream 7 undepened Tier-A deepen: washer optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::Washer {
        return matches!(
            point.id,
            "detergent_level_percent"
                | "unbalance"
                | "sabbath_mode"
                | "eco_mode"
                | "door_ajar"
                | "door_locked"
                | "water_temp_alarm"
                | "overflow_alarm"
                | "detergent_low"
                | "timer_s"
        );
    }
    // Stream 7 undepened Tier-A deepen: dryer optional telemetry/settings in typical sim.
    // Reuse thin-table anti_crease / dryness_percent / vent_blocked / drain_tank;
    // dryer-only depth lives in DRYER_DEPTH (not DRYER_BASE) to avoid washer_dryer dups.
    // Do not early-return false — dryer also advertises thermal_port_* via the
    // shared Stream 5 fall-through below.
    if table.class_id == ApplianceClassId::Dryer
        && matches!(
            point.id,
            "anti_crease"
                | "dryness_percent"
                | "vent_blocked"
                | "drain_tank"
                | "sabbath_mode"
                | "eco_mode"
                | "door_ajar"
                | "door_locked"
                | "high_temp_alarm"
                | "lint_full"
                | "timer_s"
        )
    {
        return true;
    }
    // Stream 7 undepened Tier-A deepen: washer_dryer combo optional telemetry/settings.
    // Washer depth (sabbath/eco/door/alarms/detergent/timer) already lives on
    // WASHER_POINTS; dryer thin-table anti_crease/dryness/vent/drain on DRYER_BASE;
    // dryer-only high_temp_alarm/lint_full live on WASHER_DRYER_EXTRA (not DRYER_DEPTH).
    if table.class_id == ApplianceClassId::WasherDryer {
        return matches!(
            point.id,
            "detergent_level_percent"
                | "unbalance"
                | "sabbath_mode"
                | "eco_mode"
                | "door_ajar"
                | "door_locked"
                | "water_temp_alarm"
                | "overflow_alarm"
                | "detergent_low"
                | "timer_s"
                | "anti_crease"
                | "dryness_percent"
                | "vent_blocked"
                | "drain_tank"
                | "high_temp_alarm"
                | "lint_full"
                | "dry_after_wash"
                | "max_dry_s"
        );
    }
    // Stream 3: coffee brew procedure waits on boiler telemetry.
    if table.class_id == ApplianceClassId::CoffeeMachine
        && matches!(point.id, "boiler_c" | "brew_pressure_bar")
    {
        return true;
    }
    // Stream 7 catalog depth: wine cooler optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::WineCooler {
        return matches!(
            point.id,
            "vibration_reduce"
                | "uv_protect"
                | "sabbath_mode"
                | "compressor_on"
                | "high_temp_alarm"
                | "low_temp_alarm"
                | "vibration_alert"
                | "bottle_count"
        );
    }
    // Stream 7 catalog depth: ice maker optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::IceMaker {
        return matches!(
            point.id,
            "clean_cycle_needed"
                | "water_temp_c"
                | "water_low"
                | "scoop_light"
                | "max_ice_mode"
                | "harvest_fail"
                | "scale_alert"
                | "delayed_start_s"
        );
    }
    // Stream 7 catalog depth: sous-vide optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::SousVide {
        return matches!(
            point.id,
            "circulating"
                | "cook_s"
                | "water_level_ok"
                | "lid_closed"
                | "timer_remaining_s"
                | "target_done"
                | "overtemp_alarm"
                | "delayed_start_s"
                | "alarm_offset_c"
        );
    }
    // Stream 7 catalog depth: multi-cooker optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::MultiCooker {
        return matches!(
            point.id,
            "pressure_band"
                | "pressure_kpa"
                | "float_valve"
                | "remote_vent_enabled"
                | "burn_detected"
                | "pot_detect"
                | "cook_s"
                | "delayed_start_s"
                | "keep_warm"
                | "keep_warm_s"
                | "saute_level"
                | "overpressure_alarm"
                | "lid_mismatch"
        );
    }
    // Stream 7 catalog depth: toaster-oven optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::ToasterOven {
        return matches!(
            point.id,
            "toast_shade"
                | "crumb_tray"
                | "door_open"
                | "timer_remaining_s"
                | "delayed_start_s"
                | "rack_position"
                | "bagel"
                | "preheating"
                | "slices"
                | "toast_done"
                | "convection_fan"
                | "broil_level"
                | "cook_s"
                | "element_bake"
                | "element_broil"
        );
    }
    // Stream 7 catalog depth: dehumidifier optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::Dehumidifier {
        return matches!(
            point.id,
            "tank_full"
                | "pump_mode"
                | "defrost"
                | "compressor_on"
                | "high_rh_alarm"
                | "low_rh_alarm"
                | "continuous_mode"
                | "quiet_mode"
                | "bucket_removed"
                | "filter_dirty"
                | "delayed_start_s"
        );
    }
    // Stream 7 catalog depth: range hood optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::RangeHood {
        return matches!(
            point.id,
            "auto_mode"
                | "delay_off_s"
                | "voc_index"
                | "grease_filter"
                | "charcoal_filter"
                | "filter_dirty"
                | "boost"
                | "boost_remaining_s"
                | "light_level"
                | "grease_sensor"
                | "hob_linked"
                | "overtemp"
                | "charcoal_filter_life_percent"
        );
    }
    // Stream 7 catalog depth: cooktop optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::Cooktop {
        return matches!(
            point.id,
            "boost"
                | "timer_s"
                | "bridge"
                | "flame_out"
                | "ignition_fail"
                | "power_limit_w"
                | "keep_warm"
                | "hotspot_alert"
                | "timer_active"
                | "paused"
                | "surface_c"
                | "element_fault"
                | "pan_detect"
                | "flame_on"
        );
    }
    // Stream 7 catalog depth: humidifier optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::Humidifier {
        return matches!(
            point.id,
            "output_level"
                | "mist_type"
                | "wick_state"
                | "warm_mist"
                | "auto_humidity"
                | "mineral_filter"
                | "uv_clean"
                | "scale_alert"
                | "tank_removed"
                | "misting"
                | "night_mode"
        );
    }
    // Stream 7 undepened Tier-A deepen: fridge optional telemetry/settings.
    // Advertise shared cold-cabinet thin-table points + fridge-only door/low-temp;
    // thermal_port_* stay via the Stream 5 match below (do not return false here).
    if table.class_id == ApplianceClassId::Fridge
        && matches!(
            point.id,
            "vacation_mode"
                | "sabbath_mode"
                | "eco_mode"
                | "defrost_active"
                | "compressor_on"
                | "high_temp_alarm"
                | "power_fail_ms"
                | "door_ajar"
                | "low_temp_alarm"
        )
    {
        return true;
    }
    // Stream 7 catalog depth: freezer optional telemetry/settings in typical sim.
    // Includes shared cold-cabinet points + freezer-only extras.
    if table.class_id == ApplianceClassId::Freezer {
        return matches!(
            point.id,
            "vacation_mode"
                | "sabbath_mode"
                | "eco_mode"
                | "defrost_active"
                | "compressor_on"
                | "high_temp_alarm"
                | "power_fail_ms"
                | "fast_freeze"
                | "door_ajar"
                | "ice_buildup"
                | "low_temp_alarm"
                | "anti_sweat"
                | "fast_freeze_remaining_s"
                | "frost_clean_needed"
        );
    }
    // Stream 7 catalog depth: fridge_freezer dual-zone optional depth in typical sim.
    // Shared cold-cabinet + combo extras (per-side door/alarms, fast_freeze, ice_buildup,
    // convertible_zone_mode). No thermal-port surface (fridge owns condenser ports).
    if table.class_id == ApplianceClassId::FridgeFreezer {
        return matches!(
            point.id,
            "vacation_mode"
                | "sabbath_mode"
                | "eco_mode"
                | "defrost_active"
                | "compressor_on"
                | "high_temp_alarm"
                | "power_fail_ms"
                | "door_ajar_fridge"
                | "door_ajar_freezer"
                | "fast_freeze"
                | "ice_buildup"
                | "high_temp_alarm_fridge"
                | "high_temp_alarm_freezer"
                | "convertible_zone_mode"
        );
    }
    // Stream 7 catalog depth: beverage_cooler optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::BeverageCooler {
        return matches!(
            point.id,
            "sabbath_mode"
                | "eco_mode"
                | "compressor_on"
                | "high_temp_alarm"
                | "low_temp_alarm"
                | "door_ajar"
                | "can_capacity"
        );
    }
    // Stream 7 catalog depth: kegerator optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::Kegerator {
        return matches!(
            point.id,
            "co2_kpa"
                | "keg_percent"
                | "keg_empty"
                | "sabbath_mode"
                | "eco_mode"
                | "compressor_on"
                | "high_temp_alarm"
                | "low_temp_alarm"
                | "door_ajar"
        );
    }
    // Stream 7 catalog depth: warming_drawer optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::WarmingDrawer {
        return matches!(
            point.id,
            "level"
                | "moist"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "door_ajar"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: pizza_oven optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::PizzaOven {
        return matches!(
            point.id,
            "stone_c"
                | "dome_c"
                | "top_bottom_balance"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "door_ajar"
                | "timer_s"
                | "steam_inject"
        );
    }
    // Stream 7 catalog depth: electric_grill optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::ElectricGrill {
        return matches!(
            point.id,
            "plate_top_c"
                | "plate_bottom_c"
                | "sear"
                | "grease_tray"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "lid_open"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: electric_smoker optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::ElectricSmoker {
        return matches!(
            point.id,
            "chamber_c"
                | "smoke_on"
                | "fuel_percent"
                | "water_pan"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "door_ajar"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: espresso_machine optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::EspressoMachine {
        return matches!(
            point.id,
            "brew_pressure_bar"
                | "shot_ml"
                | "pump_on"
                | "steam_wand_on"
                | "sabbath_mode"
                | "eco_mode"
                | "boiler_ready"
                | "high_temp_alarm"
                | "water_tank_empty"
                | "descaling_needed"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: drip_coffee_maker optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::DripCoffeeMaker {
        return matches!(
            point.id,
            "cups"
                | "strength"
                | "keep_warm_s"
                | "carafe_present"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "water_tank_empty"
                | "descaling_needed"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: coffee_grinder optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::CoffeeGrinder {
        return matches!(
            point.id,
            "grind_s"
                | "dose_g"
                | "hopper_present"
                | "sabbath_mode"
                | "eco_mode"
                | "motor_on"
                | "hopper_empty"
                | "bean_level_percent"
                | "timer_s"
                | "single_dose"
        );
    }
    // Stream 7 catalog depth: water_dispenser optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::WaterDispenser {
        return matches!(
            point.id,
            "hot_setpoint_c"
                | "cold_setpoint_c"
                | "bottle_empty"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "cooler_on"
                | "high_temp_alarm"
                | "low_temp_alarm"
                | "water_tank_empty"
        );
    }
    // Stream 7 catalog depth: toaster optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::Toaster {
        return matches!(
            point.id,
            "bagel"
                | "frozen"
                | "single_side"
                | "carriage"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "timer_s"
                | "crumb_tray_full"
                | "slots"
        );
    }
    // Stream 7 catalog depth: blender optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::Blender {
        return matches!(
            point.id,
            "form_factor"
                | "pulse"
                | "jar_present"
                | "lid_locked"
                | "heated"
                | "sabbath_mode"
                | "eco_mode"
                | "motor_on"
                | "overload_trip"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: food processor optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::FoodProcessor {
        return matches!(
            point.id,
            "pulse"
                | "bowl_present"
                | "lid_locked"
                | "attachment"
                | "sabbath_mode"
                | "eco_mode"
                | "motor_on"
                | "overload_trip"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: stand mixer optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::StandMixer {
        return matches!(
            point.id,
            "bowl_present"
                | "head_down"
                | "mass_g"
                | "attachment"
                | "sabbath_mode"
                | "eco_mode"
                | "motor_on"
                | "overload_trip"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: juicer optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::Juicer {
        return matches!(
            point.id,
            "reverse"
                | "pulp_full"
                | "jug_present"
                | "sabbath_mode"
                | "eco_mode"
                | "motor_on"
                | "overload_trip"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: rice_cooker optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::RiceCooker {
        return matches!(
            point.id,
            "texture"
                | "bowl_present"
                | "keep_warm"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "lid_open"
                | "timer_s"
                | "water_ratio"
        );
    }
    // Stream 7 catalog depth: slow_cooker optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::SlowCooker {
        return matches!(
            point.id,
            "pot_present"
                | "keep_warm"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "lid_open"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: bread_maker optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::BreadMaker {
        return matches!(
            point.id,
            "crust"
                | "loaf_size"
                | "pan_present"
                | "keep_warm"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "lid_open"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: dehydrator optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::Dehydrator {
        return matches!(
            point.id,
            "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "fan_on"
                | "high_temp_alarm"
                | "door_ajar"
                | "timer_s"
                | "tray_count"
        );
    }
    // Stream 7 catalog depth: vacuum_sealer optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::VacuumSealer {
        return matches!(
            point.id,
            "moist"
                | "vacuum_kpa"
                | "bag_detect"
                | "form_factor"
                | "sabbath_mode"
                | "eco_mode"
                | "pump_on"
                | "seal_heater_on"
                | "lid_locked"
                | "seal_fail"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: ice_cream_maker optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::IceCreamMaker {
        return matches!(
            point.id,
            "doneness"
                | "sabbath_mode"
                | "eco_mode"
                | "compressor_on"
                | "motor_on"
                | "bowl_present"
                | "lid_locked"
                | "low_temp_alarm"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: yogurt_maker optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::YogurtMaker {
        return matches!(
            point.id,
            "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "low_temp_alarm"
                | "lid_open"
                | "jar_present"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: waffle_maker optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::WaffleMaker {
        return matches!(
            point.id,
            "shade"
                | "ready"
                | "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "lid_open"
                | "batter_done"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: pasta_maker optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::PastaMaker {
        return matches!(
            point.id,
            "die"
                | "jam"
                | "sabbath_mode"
                | "eco_mode"
                | "motor_on"
                | "dough_ready"
                | "hopper_empty"
                | "die_present"
                | "overload_trip"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: steam_cooker optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::SteamCooker {
        return matches!(
            point.id,
            "sabbath_mode"
                | "eco_mode"
                | "heater_on"
                | "high_temp_alarm"
                | "lid_open"
                | "steam_ready"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: garbage_disposal optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::GarbageDisposal {
        return matches!(
            point.id,
            "run_s"
                | "jam"
                | "reset_needed"
                | "reverse"
                | "sabbath_mode"
                | "eco_mode"
                | "motor_on"
                | "overload_trip"
                | "air_switch"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: trash_compactor optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::TrashCompactor {
        return matches!(
            point.id,
            "ram_state"
                | "bin_full"
                | "sabbath_mode"
                | "eco_mode"
                | "motor_on"
                | "drawer_open"
                | "overload_trip"
                | "key_lock"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: boiler optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::Boiler {
        return matches!(
            point.id,
            "pressure_bar"
                | "burner_on"
                | "flame_out"
                | "low_pressure"
                | "sabbath_mode"
                | "eco_mode"
                | "high_temp_alarm"
                | "lockout"
                | "ignition_fail"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: water_softener optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::WaterSoftener {
        return matches!(
            point.id,
            "capacity_remaining"
                | "salt_level"
                | "bypass"
                | "treated_l"
                | "sabbath_mode"
                | "eco_mode"
                | "regenerating"
                | "salt_low"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: water_filter optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::WaterFilter {
        return matches!(
            point.id,
            "tds_in_ppm"
                | "tds_out_ppm"
                | "tank_full"
                | "sabbath_mode"
                | "eco_mode"
                | "bypass"
                | "filter_clogged"
                | "replace_needed"
                | "timer_s"
        );
    }
    // Stream 7 catalog depth: steam oven optional telemetry/settings in typical sim.
    if table.class_id == ApplianceClassId::SteamOven {
        return matches!(
            point.id,
            "humidity_set_percent"
                | "water_tank_level"
                | "descaling_needed"
                | "steam_generator_on"
                | "cavity_humidity"
                | "door_locked"
                | "drain_full"
                | "generator_fault"
                | "delayed_start_s"
                | "steam_percent"
                | "convection_fan"
                | "broil_level"
                | "cook_s"
                | "element_bake"
                | "element_broil"
                | "door_locked_clean"
        );
    }
    // Stream 5: device-facing thermal-port surface on Tier-A water_heater / fridge / hvac / dishwasher / dryer.
    matches!(
        table.class_id,
        ApplianceClassId::WaterHeater
            | ApplianceClassId::Fridge
            | ApplianceClassId::Hvac
            | ApplianceClassId::Dishwasher
            | ApplianceClassId::Dryer
    ) && point.id.starts_with("thermal_port_")
}

fn specialize_trait_point(
    table: &ClassTable,
    trait_id: TraitId,
    point: &CatalogPoint,
    mut cap: PointCapability,
) -> PointCapability {
    if trait_id == TraitId::Program
        && matches!(point.id, "program" | "available_programs")
        && !table.program_tokens.is_empty()
    {
        cap.range = Some(ValueRange::enum_tokens(
            table.program_tokens.iter().copied(),
        ));
        if point.id == "available_programs" {
            cap.range = Some(ValueRange::List {
                max_len: 64,
                item: Some(Box::new(ValueRange::enum_tokens(
                    table.program_tokens.iter().copied(),
                ))),
            });
        }
    }
    if trait_id == TraitId::Temperature && point.id == "setpoint_c" {
        if let Some((min, max)) = table.typical_setpoint_c {
            cap.range = Some(ValueRange::Numeric {
                min: min as f64,
                max: max as f64,
            });
            cap.access = AccessMode::RWE;
            cap.value_type = ValueType::F32;
        }
    }
    if trait_id == TraitId::Zone && point.id == "zones" && !table.typical_zones.is_empty() {
        cap.range = Some(ValueRange::List {
            max_len: 16,
            item: None,
        });
    }
    cap
}

/// Look up a catalog point by qualified id in encoded traits / class tables.
pub fn catalog_point(namespace: &PointNamespace, id: &str) -> Option<&'static CatalogPoint> {
    match namespace {
        PointNamespace::Trait(t) => trait_table(*t).and_then(|tt| tt.point(id)),
        PointNamespace::Class(c) => class_table(*c).and_then(|ct| ct.class_point(id)),
        PointNamespace::Vendor(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::is_snake_case_id;
    use crate::types::Value;
    use crate::ErrorCode;

    const INDEX: &[&str] = &[
        "washer",
        "dryer",
        "washer_dryer",
        "fridge",
        "freezer",
        "fridge_freezer",
        "wine_cooler",
        "beverage_cooler",
        "ice_maker",
        "kegerator",
        "dishwasher",
        "microwave",
        "oven",
        "steam_oven",
        "toaster_oven",
        "range",
        "cooktop",
        "induction_hob",
        "warming_drawer",
        "pizza_oven",
        "air_fryer",
        "electric_grill",
        "electric_smoker",
        "range_hood",
        "coffee_machine",
        "espresso_machine",
        "drip_coffee_maker",
        "coffee_grinder",
        "kettle",
        "water_dispenser",
        "toaster",
        "blender",
        "food_processor",
        "stand_mixer",
        "juicer",
        "rice_cooker",
        "slow_cooker",
        "multi_cooker",
        "sous_vide",
        "bread_maker",
        "dehydrator",
        "vacuum_sealer",
        "ice_cream_maker",
        "yogurt_maker",
        "waffle_maker",
        "pasta_maker",
        "steam_cooker",
        "garbage_disposal",
        "trash_compactor",
        "water_heater",
        "boiler",
        "water_softener",
        "water_filter",
        "hvac",
        "dehumidifier",
        "humidifier",
    ];

    #[test]
    fn list_all_class_ids_matches_appliances_index() {
        let ids: Vec<&str> = list_all_class_ids().iter().map(|c| c.as_str()).collect();
        assert_eq!(ids, INDEX);
        assert_eq!(ids.len(), 56);
    }

    #[test]
    fn catalog_groups_match_appliances_index() {
        assert_eq!(CATALOG_GROUP_ORDER.len(), 9);
        for id in ApplianceClassId::ALL {
            assert!(
                CATALOG_GROUP_ORDER.contains(&catalog_group(*id)),
                "{id} group {} is not an Index group",
                catalog_group(*id)
            );
        }
        assert_eq!(catalog_group(ApplianceClassId::Washer), "Laundry");
        assert_eq!(catalog_group(ApplianceClassId::WineCooler), "Cold");
        assert_eq!(catalog_group(ApplianceClassId::Dishwasher), "Wash");
        assert_eq!(catalog_group(ApplianceClassId::SteamOven), "Cooking");
        assert_eq!(catalog_group(ApplianceClassId::RangeHood), "Ventilation");
        assert_eq!(catalog_group(ApplianceClassId::Kettle), "Beverage");
        assert_eq!(catalog_group(ApplianceClassId::SousVide), "Countertop");
        assert_eq!(catalog_group(ApplianceClassId::WaterHeater), "Utility");
        assert_eq!(catalog_group(ApplianceClassId::Hvac), "Climate");
    }

    #[test]
    fn tier_a_ids_match_roadmap() {
        assert_eq!(TIER_A_CLASS_IDS.len(), 25);
        for id in TIER_A_CLASS_IDS {
            assert!(
                ApplianceClassId::ALL.contains(id),
                "{id} is not in the appliances index"
            );
        }
        let mut seen = std::collections::BTreeSet::new();
        for id in TIER_A_CLASS_IDS {
            assert!(seen.insert(*id), "duplicate Tier-A id {id}");
        }
    }

    #[test]
    fn tier_b_ids_partition_catalog() {
        assert_eq!(TIER_B_CLASS_IDS.len(), 31);
        assert_eq!(
            TIER_A_CLASS_IDS.len() + TIER_B_CLASS_IDS.len(),
            ApplianceClassId::ALL.len()
        );
        let a: std::collections::BTreeSet<_> = TIER_A_CLASS_IDS.iter().copied().collect();
        let b: std::collections::BTreeSet<_> = TIER_B_CLASS_IDS.iter().copied().collect();
        assert!(a.is_disjoint(&b), "Tier-A and Tier-B overlap");
        let all: std::collections::BTreeSet<_> = ApplianceClassId::ALL.iter().copied().collect();
        assert_eq!(&a | &b, all);
        let mut seen = std::collections::BTreeSet::new();
        for id in TIER_B_CLASS_IDS {
            assert!(seen.insert(*id), "duplicate Tier-B id {id}");
        }
    }

    #[test]
    fn static_tables_for_encoded_classes() {
        assert_eq!(STATIC_CLASS_IDS.len(), static_class_tables().len());
        assert_eq!(STATIC_CLASS_IDS, ApplianceClassId::ALL);
        assert_eq!(STATIC_CLASS_IDS.len(), 56);
        assert_eq!(
            STATIC_CLASS_IDS.len(),
            TIER_A_CLASS_IDS.len() + TIER_B_CLASS_IDS.len()
        );
        for id in STATIC_CLASS_IDS {
            let table = class_table(*id).expect("static table");
            assert_eq!(table.class_id, *id);
            assert!(
                table.typical_traits.contains(&TraitId::Identity),
                "{id} missing identity"
            );
            for t in table.typical_traits {
                assert!(
                    trait_table(*t).is_some(),
                    "{id} typical trait {t} has no table"
                );
            }
            let cap = typical_capability(*id).unwrap();
            assert_eq!(cap.class_id, *id);
            for t in table.typical_traits {
                let tc = cap
                    .trait_cap(*t)
                    .unwrap_or_else(|| panic!("{id} missing {t}"));
                for p in trait_table(*t).unwrap().required_points() {
                    assert!(
                        tc.points.iter().any(|pc| pc.base_id().ends_with(p.id)),
                        "{id} trait {t} missing required {}",
                        p.id
                    );
                }
            }
            for p in table.required_class_points() {
                assert!(
                    cap.class_points
                        .iter()
                        .any(|pc| pc.base_id().ends_with(p.id)),
                    "{id} missing required class point {}",
                    p.id
                );
                assert!(is_snake_case_id(p.id));
            }
        }
    }

    #[test]
    fn thermal_vocab_tokens_match_schema_enums() {
        use crate::{
            Media, PortDirection, THERMAL_PORT_DIRECTION_TOKENS, THERMAL_PORT_MEDIA_TOKENS,
        };
        assert_eq!(THERMAL_PORT_MEDIA_TOKENS.len(), Media::ALL.len());
        for (token, media) in THERMAL_PORT_MEDIA_TOKENS.iter().zip(Media::ALL) {
            assert_eq!(*token, media.as_str());
        }
        assert_eq!(
            THERMAL_PORT_DIRECTION_TOKENS.len(),
            PortDirection::ALL.len()
        );
        for (token, dir) in THERMAL_PORT_DIRECTION_TOKENS.iter().zip(PortDirection::ALL) {
            assert_eq!(*token, dir.as_str());
        }
    }

    #[test]
    fn thermal_port_classes_advertise_heat_port_specs() {
        use crate::{HeatPortSpec, Media, PortDirection};

        let expected: &[(ApplianceClassId, HeatPortSpec)] = &[
            (
                ApplianceClassId::WaterHeater,
                HeatPortSpec::new("preheat", PortDirection::Sink, Media::Water, 2_000, None),
            ),
            (
                ApplianceClassId::Fridge,
                HeatPortSpec::new("condenser", PortDirection::Source, Media::Water, 120, None),
            ),
            (
                ApplianceClassId::Hvac,
                HeatPortSpec::new("coil", PortDirection::Sink, Media::Water, 5_000, None),
            ),
            (
                ApplianceClassId::Dishwasher,
                HeatPortSpec::new(
                    "inlet_preheat",
                    PortDirection::Sink,
                    Media::Water,
                    1_800,
                    None,
                ),
            ),
            (
                ApplianceClassId::Dryer,
                HeatPortSpec::new("exhaust", PortDirection::Source, Media::Air, 2_000, None),
            ),
        ];

        for (class_id, want) in expected {
            let table = class_table(*class_id).expect("static table");
            assert!(
                !table.thermal_ports.is_empty(),
                "{class_id} should advertise HeatPortSpec"
            );
            assert_eq!(table.thermal_ports.len(), 1);
            let got = &table.thermal_ports[0];
            assert_eq!(got.port_id, want.port_id, "{class_id} port_id");
            assert_eq!(got.direction, want.direction, "{class_id} direction");
            assert_eq!(got.media, want.media, "{class_id} media");
            assert_eq!(got.max_power_w, want.max_power_w, "{class_id} max_power_w");
            // Spec port_id / direction / media align with catalog thermal_port_* surface.
            assert!(
                table.class_point("thermal_port_id").is_some(),
                "{class_id} keeps thermal_port_id point"
            );
            assert!(
                table.class_point("thermal_port_direction").is_some(),
                "{class_id} keeps thermal_port_direction point"
            );
            assert!(
                table.class_point("thermal_port_media").is_some(),
                "{class_id} keeps thermal_port_media point"
            );
            assert!(
                table.class_point("thermal_port_max_power_w").is_some(),
                "{class_id} keeps thermal_port_max_power_w point"
            );
        }

        // Other static classes default to empty specs.
        for table in static_class_tables() {
            let is_thermal = matches!(
                table.class_id,
                ApplianceClassId::WaterHeater
                    | ApplianceClassId::Fridge
                    | ApplianceClassId::Hvac
                    | ApplianceClassId::Dishwasher
                    | ApplianceClassId::Dryer
            );
            if !is_thermal {
                assert!(
                    table.thermal_ports.is_empty(),
                    "{} should have empty thermal_ports",
                    table.class_id
                );
            }
        }
    }

    #[test]
    fn ice_maker_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::IceMaker).unwrap();
        for id in [
            "class.ice_maker.clean_cycle_needed",
            "class.ice_maker.water_temp_c",
            "class.ice_maker.water_low",
            "class.ice_maker.scoop_light",
            "class.ice_maker.max_ice_mode",
            "class.ice_maker.harvest_fail",
            "class.ice_maker.scale_alert",
            "class.ice_maker.delayed_start_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical ice_maker"
            );
        }
        let ice = cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Ice)
            .unwrap();
        assert!(ice.points.iter().any(|p| p.id == "trait.ice.bin_full"));
        assert!(ice.points.iter().any(|p| p.id == "trait.ice.bin_percent"));
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Filter)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.filter.life_percent"));
        cap.validate_write("class.ice_maker.scoop_light", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.ice_maker.max_ice_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.ice_maker.delayed_start_s", &Value::DurationS(3600))
            .unwrap();
        let err = cap
            .validate_write("class.ice_maker.water_low", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("trait.ice.bin_percent", &Value::Percent(50.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn wine_cooler_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::WineCooler).unwrap();
        for id in [
            "class.wine_cooler.vibration_reduce",
            "class.wine_cooler.uv_protect",
            "class.wine_cooler.sabbath_mode",
            "class.wine_cooler.compressor_on",
            "class.wine_cooler.high_temp_alarm",
            "class.wine_cooler.low_temp_alarm",
            "class.wine_cooler.vibration_alert",
            "class.wine_cooler.bottle_count",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical wine_cooler"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Humidity)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.humidity.setpoint_rh"));
        cap.validate_write("class.wine_cooler.vibration_reduce", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.wine_cooler.uv_protect", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.wine_cooler.sabbath_mode", &Value::Bool(true))
            .unwrap();
        let err = cap
            .validate_write("class.wine_cooler.bottle_count", &Value::U16(12))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        cap.validate_write("trait.humidity.setpoint_rh", &Value::Percent(60.0))
            .unwrap();
    }

    #[test]
    fn multi_cooker_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::MultiCooker).unwrap();
        for id in [
            "class.multi_cooker.lid_locked",
            "class.multi_cooker.safe_to_open",
            "class.multi_cooker.pressure_band",
            "class.multi_cooker.pressure_kpa",
            "class.multi_cooker.float_valve",
            "class.multi_cooker.remote_vent_enabled",
            "class.multi_cooker.burn_detected",
            "class.multi_cooker.pot_detect",
            "class.multi_cooker.cook_s",
            "class.multi_cooker.delayed_start_s",
            "class.multi_cooker.keep_warm",
            "class.multi_cooker.keep_warm_s",
            "class.multi_cooker.saute_level",
            "class.multi_cooker.overpressure_alarm",
            "class.multi_cooker.lid_mismatch",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical multi_cooker"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Cycle)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.cycle.remaining_s"));
        cap.validate_write(
            "class.multi_cooker.pressure_band",
            &Value::Enum("high".into()),
        )
        .unwrap();
        cap.validate_write("class.multi_cooker.cook_s", &Value::DurationS(1800))
            .unwrap();
        cap.validate_write("class.multi_cooker.delayed_start_s", &Value::DurationS(600))
            .unwrap();
        cap.validate_write("class.multi_cooker.keep_warm", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.multi_cooker.keep_warm_s", &Value::DurationS(3600))
            .unwrap();
        cap.validate_write(
            "class.multi_cooker.saute_level",
            &Value::Enum("normal".into()),
        )
        .unwrap();
        cap.validate_write("class.multi_cooker.remote_vent_enabled", &Value::Bool(true))
            .unwrap();
        let err = cap
            .validate_write("class.multi_cooker.pot_detect", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.multi_cooker.overpressure_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.multi_cooker.lid_mismatch", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.multi_cooker.pressure_kpa", &Value::F32(50.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("trait.cycle.remaining_s", &Value::DurationS(100))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn toaster_oven_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::ToasterOven).unwrap();
        for id in [
            "class.toaster_oven.toast_shade",
            "class.toaster_oven.crumb_tray",
            "class.toaster_oven.door_open",
            "class.toaster_oven.timer_remaining_s",
            "class.toaster_oven.delayed_start_s",
            "class.toaster_oven.rack_position",
            "class.toaster_oven.bagel",
            "class.toaster_oven.preheating",
            "class.toaster_oven.slices",
            "class.toaster_oven.toast_done",
            "class.toaster_oven.convection_fan",
            "class.toaster_oven.broil_level",
            "class.toaster_oven.cook_s",
            "class.toaster_oven.element_bake",
            "class.toaster_oven.element_broil",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical toaster_oven"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Cycle)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.cycle.remaining_s"));
        cap.validate_write("class.toaster_oven.toast_shade", &Value::U8(4))
            .unwrap();
        cap.validate_write("class.toaster_oven.delayed_start_s", &Value::DurationS(300))
            .unwrap();
        cap.validate_write(
            "class.toaster_oven.rack_position",
            &Value::Enum("middle".into()),
        )
        .unwrap();
        cap.validate_write("class.toaster_oven.bagel", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.toaster_oven.slices", &Value::U8(2))
            .unwrap();
        cap.validate_write("class.toaster_oven.convection_fan", &Value::Bool(true))
            .unwrap();
        cap.validate_write(
            "class.toaster_oven.broil_level",
            &Value::Enum("high".into()),
        )
        .unwrap();
        cap.validate_write("class.toaster_oven.cook_s", &Value::DurationS(900))
            .unwrap();
        let err = cap
            .validate_write("class.toaster_oven.door_open", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.toaster_oven.timer_remaining_s",
                &Value::DurationS(30),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.toaster_oven.crumb_tray", &Value::Enum("ok".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.toaster_oven.preheating", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.toaster_oven.toast_done", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.toaster_oven.element_bake", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("trait.cycle.remaining_s", &Value::DurationS(100))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn dehumidifier_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Dehumidifier).unwrap();
        for id in [
            "class.dehumidifier.tank_full",
            "class.dehumidifier.pump_mode",
            "class.dehumidifier.defrost",
            "class.dehumidifier.compressor_on",
            "class.dehumidifier.high_rh_alarm",
            "class.dehumidifier.low_rh_alarm",
            "class.dehumidifier.continuous_mode",
            "class.dehumidifier.quiet_mode",
            "class.dehumidifier.bucket_removed",
            "class.dehumidifier.filter_dirty",
            "class.dehumidifier.delayed_start_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical dehumidifier"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Humidity)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.humidity.setpoint_rh"));
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Fan)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.fan.fan_speed"));
        cap.validate_write("class.dehumidifier.pump_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.dehumidifier.continuous_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.dehumidifier.quiet_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write(
            "class.dehumidifier.delayed_start_s",
            &Value::DurationS(3600),
        )
        .unwrap();
        cap.validate_write("trait.humidity.setpoint_rh", &Value::Percent(45.0))
            .unwrap();
        cap.validate_write("trait.fan.fan_speed", &Value::U8(2))
            .unwrap();
        let err = cap
            .validate_write("class.dehumidifier.tank_full", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dehumidifier.defrost", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dehumidifier.compressor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dehumidifier.high_rh_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dehumidifier.low_rh_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dehumidifier.bucket_removed", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dehumidifier.filter_dirty", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn range_hood_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::RangeHood).unwrap();
        for id in [
            "class.range_hood.auto_mode",
            "class.range_hood.delay_off_s",
            "class.range_hood.voc_index",
            "class.range_hood.grease_filter",
            "class.range_hood.charcoal_filter",
            "class.range_hood.filter_dirty",
            "class.range_hood.boost",
            "class.range_hood.boost_remaining_s",
            "class.range_hood.light_level",
            "class.range_hood.grease_sensor",
            "class.range_hood.hob_linked",
            "class.range_hood.overtemp",
            "class.range_hood.charcoal_filter_life_percent",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical range_hood"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Fan)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.fan.fan_speed"));
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Lighting)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.lighting.light_percent"));
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Filter)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.filter.life_percent"));
        cap.validate_write("class.range_hood.auto_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.range_hood.boost", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.range_hood.hob_linked", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.range_hood.delay_off_s", &Value::DurationS(300))
            .unwrap();
        cap.validate_write("class.range_hood.light_level", &Value::U8(3))
            .unwrap();
        cap.validate_write("trait.fan.fan_speed", &Value::U8(2))
            .unwrap();
        cap.validate_write("trait.lighting.light_percent", &Value::Percent(80.0))
            .unwrap();
        let err = cap
            .validate_write("class.range_hood.voc_index", &Value::U16(100))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.range_hood.grease_filter", &Value::Enum("ok".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.range_hood.charcoal_filter",
                &Value::Enum("ok".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.range_hood.filter_dirty", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.range_hood.boost_remaining_s", &Value::DurationS(60))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.range_hood.grease_sensor", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.range_hood.overtemp", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.range_hood.charcoal_filter_life_percent",
                &Value::Percent(50.0),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn steam_oven_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::SteamOven).unwrap();
        for id in [
            "class.steam_oven.steam_mode",
            "class.steam_oven.water_tank",
            "class.steam_oven.humidity_set_percent",
            "class.steam_oven.water_tank_level",
            "class.steam_oven.descaling_needed",
            "class.steam_oven.steam_generator_on",
            "class.steam_oven.cavity_humidity",
            "class.steam_oven.door_locked",
            "class.steam_oven.drain_full",
            "class.steam_oven.generator_fault",
            "class.steam_oven.delayed_start_s",
            "class.steam_oven.steam_percent",
            "class.steam_oven.convection_fan",
            "class.steam_oven.broil_level",
            "class.steam_oven.cook_s",
            "class.steam_oven.element_bake",
            "class.steam_oven.element_broil",
            "class.steam_oven.door_locked_clean",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical steam_oven"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Cycle)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.cycle.remaining_s"));
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Water)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.water.hardness_ppm"));
        cap.validate_write("class.steam_oven.steam_mode", &Value::Enum("combi".into()))
            .unwrap();
        cap.validate_write(
            "class.steam_oven.humidity_set_percent",
            &Value::Percent(70.0),
        )
        .unwrap();
        cap.validate_write("class.steam_oven.delayed_start_s", &Value::DurationS(600))
            .unwrap();
        cap.validate_write("class.steam_oven.steam_percent", &Value::Percent(50.0))
            .unwrap();
        cap.validate_write("class.steam_oven.convection_fan", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.steam_oven.cook_s", &Value::DurationS(1800))
            .unwrap();
        cap.validate_write("trait.water.hardness_ppm", &Value::U16(120))
            .unwrap();
        let err = cap
            .validate_write("class.steam_oven.water_tank", &Value::Enum("ok".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.steam_oven.water_tank_level", &Value::Percent(50.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.steam_oven.descaling_needed", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.steam_oven.steam_generator_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.steam_oven.cavity_humidity", &Value::Percent(40.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.steam_oven.door_locked", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.steam_oven.drain_full", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.steam_oven.generator_fault", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.steam_oven.element_bake", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.steam_oven.door_locked_clean", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn cooktop_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Cooktop).unwrap();
        for id in [
            "class.cooktop.level",
            "class.cooktop.residual_heat",
            "class.cooktop.boost",
            "class.cooktop.timer_s",
            "class.cooktop.bridge",
            "class.cooktop.flame_out",
            "class.cooktop.ignition_fail",
            "class.cooktop.power_limit_w",
            "class.cooktop.keep_warm",
            "class.cooktop.hotspot_alert",
            "class.cooktop.timer_active",
            "class.cooktop.paused",
            "class.cooktop.surface_c",
            "class.cooktop.element_fault",
            "class.cooktop.pan_detect",
            "class.cooktop.flame_on",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical cooktop"
            );
        }
        let keep_warm = cap
            .class_points
            .iter()
            .find(|p| p.id == "class.cooktop.keep_warm")
            .unwrap();
        assert_eq!(
            keep_warm.zones.as_ref().unwrap(),
            &["hob_1", "hob_2", "hob_3", "hob_4"].map(str::to_string)
        );
        let paused = cap
            .class_points
            .iter()
            .find(|p| p.id == "class.cooktop.paused")
            .unwrap();
        assert!(paused.zones.is_none());
        cap.validate_write("class.cooktop.boost#hob_1", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.cooktop.timer_s#hob_2", &Value::DurationS(600))
            .unwrap();
        cap.validate_write("class.cooktop.bridge#hob_1", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.cooktop.power_limit_w", &Value::U32(7200))
            .unwrap();
        cap.validate_write("class.cooktop.keep_warm#hob_1", &Value::Bool(true))
            .unwrap();
        let err = cap
            .validate_write("class.cooktop.hotspot_alert#hob_1", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.cooktop.timer_active#hob_1", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.cooktop.paused", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.cooktop.surface_c#hob_1", &Value::F32(120.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.cooktop.element_fault#hob_1", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.cooktop.pan_detect#hob_1", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.cooktop.flame_on#hob_1", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.cooktop.flame_out", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.cooktop.ignition_fail", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.cooktop.residual_heat#hob_1", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn humidifier_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Humidifier).unwrap();
        for id in [
            "class.humidifier.output_level",
            "class.humidifier.mist_type",
            "class.humidifier.water_empty",
            "class.humidifier.wick_state",
            "class.humidifier.warm_mist",
            "class.humidifier.auto_humidity",
            "class.humidifier.mineral_filter",
            "class.humidifier.uv_clean",
            "class.humidifier.scale_alert",
            "class.humidifier.tank_removed",
            "class.humidifier.misting",
            "class.humidifier.night_mode",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical humidifier"
            );
        }
        let humidity = cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Humidity)
            .unwrap();
        assert!(humidity
            .points
            .iter()
            .any(|p| p.id == "trait.humidity.setpoint_rh"));
        cap.validate_write("class.humidifier.output_level", &Value::U8(5))
            .unwrap();
        cap.validate_write("class.humidifier.warm_mist", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.humidifier.auto_humidity", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.humidifier.uv_clean", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.humidifier.night_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("trait.humidity.setpoint_rh", &Value::Percent(50.0))
            .unwrap();
        let err = cap
            .validate_write("class.humidifier.mist_type", &Value::Enum("cool".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.humidifier.water_empty", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.humidifier.wick_state", &Value::Enum("ok".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.humidifier.mineral_filter", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.humidifier.scale_alert", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.humidifier.tank_removed", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.humidifier.misting", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn fridge_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Fridge).unwrap();
        for id in [
            "class.fridge.vacation_mode",
            "class.fridge.sabbath_mode",
            "class.fridge.eco_mode",
            "class.fridge.defrost_active",
            "class.fridge.compressor_on",
            "class.fridge.high_temp_alarm",
            "class.fridge.power_fail_ms",
            "class.fridge.door_ajar",
            "class.fridge.low_temp_alarm",
            // Thermal ports remain advertised (Stream 5).
            "class.fridge.thermal_port_id",
            "class.fridge.thermal_port_direction",
            "class.fridge.thermal_port_media",
            "class.fridge.thermal_port_max_power_w",
            "class.fridge.thermal_port_attached_reservoir_id",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical fridge"
            );
        }
        // Writable cold-cabinet settings.
        cap.validate_write("class.fridge.vacation_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.fridge.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.fridge.eco_mode", &Value::Bool(true))
            .unwrap();
        // Read-only telemetry / alarms.
        let err = cap
            .validate_write("class.fridge.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.fridge.low_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.fridge.defrost_active", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.fridge.compressor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.fridge.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.fridge.power_fail_ms", &Value::TimestampMs(1))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        // Freezer / fridge_freezer extras stay off fridge; fridge owns thermal ports.
        assert!(!cap
            .class_points
            .iter()
            .any(|p| p.id == "class.fridge.fast_freeze"));
        assert!(!cap
            .class_points
            .iter()
            .any(|p| p.id == "class.fridge.anti_sweat"));
        assert!(!cap
            .class_points
            .iter()
            .any(|p| p.id == "class.fridge.door_ajar_fridge"));
        let freezer = typical_capability(ApplianceClassId::Freezer).unwrap();
        assert!(freezer
            .class_points
            .iter()
            .any(|p| p.id == "class.freezer.door_ajar"));
        assert!(!freezer
            .class_points
            .iter()
            .any(|p| p.id == "class.freezer.thermal_port_id"));
        let ff = typical_capability(ApplianceClassId::FridgeFreezer).unwrap();
        assert!(ff
            .class_points
            .iter()
            .any(|p| p.id == "class.fridge_freezer.door_ajar_fridge"));
        assert!(!ff
            .class_points
            .iter()
            .any(|p| p.id == "class.fridge_freezer.thermal_port_id"));
    }

    #[test]
    fn freezer_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Freezer).unwrap();
        for id in [
            "class.freezer.vacation_mode",
            "class.freezer.sabbath_mode",
            "class.freezer.eco_mode",
            "class.freezer.defrost_active",
            "class.freezer.compressor_on",
            "class.freezer.high_temp_alarm",
            "class.freezer.power_fail_ms",
            "class.freezer.fast_freeze",
            "class.freezer.door_ajar",
            "class.freezer.ice_buildup",
            "class.freezer.low_temp_alarm",
            "class.freezer.anti_sweat",
            "class.freezer.fast_freeze_remaining_s",
            "class.freezer.frost_clean_needed",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical freezer"
            );
        }
        // No thermal-port surface on freezer (fridge owns condenser ports).
        assert!(!cap
            .class_points
            .iter()
            .any(|p| p.id.contains("thermal_port_")));

        cap.validate_write("class.freezer.fast_freeze", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.freezer.anti_sweat", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.freezer.vacation_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.freezer.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.freezer.eco_mode", &Value::Bool(true))
            .unwrap();

        let err = cap
            .validate_write("class.freezer.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.freezer.ice_buildup", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.freezer.low_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.freezer.frost_clean_needed", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.freezer.fast_freeze_remaining_s",
                &Value::DurationS(60),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.freezer.defrost_active", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.freezer.compressor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.freezer.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);

        // fridge_freezer has dual-zone depth (not full FREEZER_EXTRA) and no thermal ports.
        let ff = typical_capability(ApplianceClassId::FridgeFreezer).unwrap();
        assert!(ff
            .class_points
            .iter()
            .any(|p| p.id == "class.fridge_freezer.door_ajar_fridge"));
        assert!(ff
            .class_points
            .iter()
            .any(|p| p.id == "class.fridge_freezer.fast_freeze"));
        assert!(!ff
            .class_points
            .iter()
            .any(|p| p.id == "class.fridge_freezer.anti_sweat"));
        assert!(!ff
            .class_points
            .iter()
            .any(|p| p.id.contains("thermal_port_")));
    }

    #[test]
    fn fridge_freezer_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::FridgeFreezer).unwrap();
        for id in [
            "class.fridge_freezer.vacation_mode",
            "class.fridge_freezer.sabbath_mode",
            "class.fridge_freezer.eco_mode",
            "class.fridge_freezer.defrost_active",
            "class.fridge_freezer.compressor_on",
            "class.fridge_freezer.high_temp_alarm",
            "class.fridge_freezer.power_fail_ms",
            "class.fridge_freezer.door_ajar_fridge",
            "class.fridge_freezer.door_ajar_freezer",
            "class.fridge_freezer.fast_freeze",
            "class.fridge_freezer.ice_buildup",
            "class.fridge_freezer.high_temp_alarm_fridge",
            "class.fridge_freezer.high_temp_alarm_freezer",
            "class.fridge_freezer.convertible_zone_mode",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical fridge_freezer"
            );
        }
        // No thermal-port surface on fridge_freezer (fridge owns condenser ports).
        assert!(!cap
            .class_points
            .iter()
            .any(|p| p.id.contains("thermal_port_")));
        // Freezer-only extras not copied wholesale.
        assert!(!cap
            .class_points
            .iter()
            .any(|p| p.id == "class.fridge_freezer.anti_sweat"));
        assert!(!cap
            .class_points
            .iter()
            .any(|p| p.id == "class.fridge_freezer.frost_clean_needed"));
        assert!(!cap
            .class_points
            .iter()
            .any(|p| p.id == "class.fridge_freezer.door_ajar"));

        cap.validate_write("class.fridge_freezer.fast_freeze", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.fridge_freezer.vacation_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.fridge_freezer.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.fridge_freezer.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write(
            "class.fridge_freezer.convertible_zone_mode",
            &Value::Enum("freezer".into()),
        )
        .unwrap();

        let err = cap
            .validate_write("class.fridge_freezer.door_ajar_fridge", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.fridge_freezer.door_ajar_freezer", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.fridge_freezer.ice_buildup", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.fridge_freezer.high_temp_alarm_fridge",
                &Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.fridge_freezer.high_temp_alarm_freezer",
                &Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.fridge_freezer.defrost_active", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.fridge_freezer.compressor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.fridge_freezer.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);

        // Fridge thermal ports and freezer-only extras remain on their classes.
        let fridge = typical_capability(ApplianceClassId::Fridge).unwrap();
        assert!(fridge
            .class_points
            .iter()
            .any(|p| p.id.contains("thermal_port_")));
        let freezer = typical_capability(ApplianceClassId::Freezer).unwrap();
        assert!(freezer
            .class_points
            .iter()
            .any(|p| p.id == "class.freezer.anti_sweat"));
    }

    #[test]
    fn beverage_cooler_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::BeverageCooler).unwrap();
        for id in [
            "class.beverage_cooler.sabbath_mode",
            "class.beverage_cooler.eco_mode",
            "class.beverage_cooler.compressor_on",
            "class.beverage_cooler.high_temp_alarm",
            "class.beverage_cooler.low_temp_alarm",
            "class.beverage_cooler.door_ajar",
            "class.beverage_cooler.can_capacity",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical beverage_cooler"
            );
        }
        cap.validate_write("class.beverage_cooler.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.beverage_cooler.eco_mode", &Value::Bool(true))
            .unwrap();
        let err = cap
            .validate_write("class.beverage_cooler.can_capacity", &Value::U16(120))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.beverage_cooler.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.beverage_cooler.compressor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.beverage_cooler.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.beverage_cooler.low_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn kegerator_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Kegerator).unwrap();
        for id in [
            "class.kegerator.co2_kpa",
            "class.kegerator.keg_percent",
            "class.kegerator.keg_empty",
            "class.kegerator.sabbath_mode",
            "class.kegerator.eco_mode",
            "class.kegerator.compressor_on",
            "class.kegerator.high_temp_alarm",
            "class.kegerator.low_temp_alarm",
            "class.kegerator.door_ajar",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical kegerator"
            );
        }
        cap.validate_write("class.kegerator.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.kegerator.eco_mode", &Value::Bool(true))
            .unwrap();
        let err = cap
            .validate_write("class.kegerator.co2_kpa", &Value::F32(120.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.kegerator.keg_percent", &Value::Percent(50.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.kegerator.keg_empty", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.kegerator.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.kegerator.compressor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.kegerator.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.kegerator.low_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn warming_drawer_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::WarmingDrawer).unwrap();
        for id in [
            "class.warming_drawer.level",
            "class.warming_drawer.moist",
            "class.warming_drawer.sabbath_mode",
            "class.warming_drawer.eco_mode",
            "class.warming_drawer.heater_on",
            "class.warming_drawer.high_temp_alarm",
            "class.warming_drawer.door_ajar",
            "class.warming_drawer.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical warming_drawer"
            );
        }
        cap.validate_write("class.warming_drawer.level", &Value::Enum("medium".into()))
            .unwrap();
        cap.validate_write("class.warming_drawer.moist", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.warming_drawer.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.warming_drawer.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.warming_drawer.timer_s", &Value::DurationS(1800))
            .unwrap();
        let err = cap
            .validate_write("class.warming_drawer.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.warming_drawer.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.warming_drawer.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn pizza_oven_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::PizzaOven).unwrap();
        for id in [
            "class.pizza_oven.stone_c",
            "class.pizza_oven.dome_c",
            "class.pizza_oven.top_bottom_balance",
            "class.pizza_oven.sabbath_mode",
            "class.pizza_oven.eco_mode",
            "class.pizza_oven.heater_on",
            "class.pizza_oven.high_temp_alarm",
            "class.pizza_oven.door_ajar",
            "class.pizza_oven.timer_s",
            "class.pizza_oven.steam_inject",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical pizza_oven"
            );
        }
        cap.validate_write("class.pizza_oven.top_bottom_balance", &Value::I16(25))
            .unwrap();
        cap.validate_write("class.pizza_oven.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.pizza_oven.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.pizza_oven.timer_s", &Value::DurationS(900))
            .unwrap();
        cap.validate_write("class.pizza_oven.steam_inject", &Value::Bool(true))
            .unwrap();
        let err = cap
            .validate_write("class.pizza_oven.stone_c", &Value::F32(300.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.pizza_oven.dome_c", &Value::F32(350.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.pizza_oven.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.pizza_oven.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.pizza_oven.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn electric_grill_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::ElectricGrill).unwrap();
        for id in [
            "class.electric_grill.plate_top_c",
            "class.electric_grill.plate_bottom_c",
            "class.electric_grill.sear",
            "class.electric_grill.grease_tray",
            "class.electric_grill.sabbath_mode",
            "class.electric_grill.eco_mode",
            "class.electric_grill.heater_on",
            "class.electric_grill.high_temp_alarm",
            "class.electric_grill.lid_open",
            "class.electric_grill.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical electric_grill"
            );
        }
        cap.validate_write("class.electric_grill.sear", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.electric_grill.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.electric_grill.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.electric_grill.timer_s", &Value::DurationS(900))
            .unwrap();
        let err = cap
            .validate_write("class.electric_grill.plate_top_c", &Value::F32(200.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.electric_grill.plate_bottom_c", &Value::F32(200.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.electric_grill.grease_tray",
                &Value::Enum("full".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.electric_grill.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.electric_grill.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.electric_grill.lid_open", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn electric_smoker_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::ElectricSmoker).unwrap();
        for id in [
            "class.electric_smoker.chamber_c",
            "class.electric_smoker.smoke_on",
            "class.electric_smoker.fuel_percent",
            "class.electric_smoker.water_pan",
            "class.electric_smoker.sabbath_mode",
            "class.electric_smoker.eco_mode",
            "class.electric_smoker.heater_on",
            "class.electric_smoker.high_temp_alarm",
            "class.electric_smoker.door_ajar",
            "class.electric_smoker.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical electric_smoker"
            );
        }
        cap.validate_write("class.electric_smoker.smoke_on", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.electric_smoker.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.electric_smoker.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.electric_smoker.timer_s", &Value::DurationS(900))
            .unwrap();
        let err = cap
            .validate_write("class.electric_smoker.chamber_c", &Value::F32(100.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.electric_smoker.fuel_percent", &Value::Percent(50.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.electric_smoker.water_pan",
                &Value::Enum("empty".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.electric_smoker.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.electric_smoker.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.electric_smoker.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn espresso_machine_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::EspressoMachine).unwrap();
        for id in [
            "class.espresso_machine.brew_pressure_bar",
            "class.espresso_machine.shot_ml",
            "class.espresso_machine.pump_on",
            "class.espresso_machine.steam_wand_on",
            "class.espresso_machine.sabbath_mode",
            "class.espresso_machine.eco_mode",
            "class.espresso_machine.boiler_ready",
            "class.espresso_machine.high_temp_alarm",
            "class.espresso_machine.water_tank_empty",
            "class.espresso_machine.descaling_needed",
            "class.espresso_machine.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical espresso_machine"
            );
        }
        cap.validate_write("class.espresso_machine.shot_ml", &Value::U16(36))
            .unwrap();
        cap.validate_write("class.espresso_machine.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.espresso_machine.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.espresso_machine.timer_s", &Value::DurationS(300))
            .unwrap();
        let err = cap
            .validate_write("class.espresso_machine.brew_pressure_bar", &Value::F32(9.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.espresso_machine.pump_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.espresso_machine.steam_wand_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.espresso_machine.boiler_ready", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.espresso_machine.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.espresso_machine.water_tank_empty",
                &Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.espresso_machine.descaling_needed",
                &Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn drip_coffee_maker_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::DripCoffeeMaker).unwrap();
        for id in [
            "class.drip_coffee_maker.cups",
            "class.drip_coffee_maker.strength",
            "class.drip_coffee_maker.keep_warm_s",
            "class.drip_coffee_maker.carafe_present",
            "class.drip_coffee_maker.sabbath_mode",
            "class.drip_coffee_maker.eco_mode",
            "class.drip_coffee_maker.heater_on",
            "class.drip_coffee_maker.high_temp_alarm",
            "class.drip_coffee_maker.water_tank_empty",
            "class.drip_coffee_maker.descaling_needed",
            "class.drip_coffee_maker.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical drip_coffee_maker"
            );
        }
        cap.validate_write("class.drip_coffee_maker.cups", &Value::U8(8))
            .unwrap();
        cap.validate_write(
            "class.drip_coffee_maker.strength",
            &Value::Enum("strong".into()),
        )
        .unwrap();
        cap.validate_write(
            "class.drip_coffee_maker.keep_warm_s",
            &Value::DurationS(1800),
        )
        .unwrap();
        cap.validate_write("class.drip_coffee_maker.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.drip_coffee_maker.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.drip_coffee_maker.timer_s", &Value::DurationS(600))
            .unwrap();
        let err = cap
            .validate_write(
                "class.drip_coffee_maker.carafe_present",
                &Value::Bool(false),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.drip_coffee_maker.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.drip_coffee_maker.high_temp_alarm",
                &Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.drip_coffee_maker.water_tank_empty",
                &Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.drip_coffee_maker.descaling_needed",
                &Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn coffee_grinder_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::CoffeeGrinder).unwrap();
        for id in [
            "class.coffee_grinder.grind_s",
            "class.coffee_grinder.dose_g",
            "class.coffee_grinder.hopper_present",
            "class.coffee_grinder.sabbath_mode",
            "class.coffee_grinder.eco_mode",
            "class.coffee_grinder.motor_on",
            "class.coffee_grinder.hopper_empty",
            "class.coffee_grinder.bean_level_percent",
            "class.coffee_grinder.timer_s",
            "class.coffee_grinder.single_dose",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical coffee_grinder"
            );
        }
        cap.validate_write("class.coffee_grinder.grind_s", &Value::DurationS(10))
            .unwrap();
        cap.validate_write("class.coffee_grinder.dose_g", &Value::F32(18.0))
            .unwrap();
        cap.validate_write("class.coffee_grinder.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.coffee_grinder.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.coffee_grinder.timer_s", &Value::DurationS(120))
            .unwrap();
        cap.validate_write("class.coffee_grinder.single_dose", &Value::Bool(true))
            .unwrap();
        let err = cap
            .validate_write("class.coffee_grinder.hopper_present", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.coffee_grinder.motor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.coffee_grinder.hopper_empty", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.coffee_grinder.bean_level_percent",
                &Value::Percent(50.0),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn water_dispenser_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::WaterDispenser).unwrap();
        for id in [
            "class.water_dispenser.hot_setpoint_c",
            "class.water_dispenser.cold_setpoint_c",
            "class.water_dispenser.bottle_empty",
            "class.water_dispenser.sabbath_mode",
            "class.water_dispenser.eco_mode",
            "class.water_dispenser.heater_on",
            "class.water_dispenser.cooler_on",
            "class.water_dispenser.high_temp_alarm",
            "class.water_dispenser.low_temp_alarm",
            "class.water_dispenser.water_tank_empty",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical water_dispenser"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Filter)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.filter.life_percent"));
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::ChildLock)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.child_lock.child_lock"));
        cap.validate_write("class.water_dispenser.hot_setpoint_c", &Value::F32(90.0))
            .unwrap();
        cap.validate_write("class.water_dispenser.cold_setpoint_c", &Value::F32(8.0))
            .unwrap();
        cap.validate_write("class.water_dispenser.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.water_dispenser.eco_mode", &Value::Bool(true))
            .unwrap();
        let err = cap
            .validate_write("class.water_dispenser.bottle_empty", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.water_dispenser.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.water_dispenser.cooler_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.water_dispenser.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.water_dispenser.low_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.water_dispenser.water_tank_empty", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn toaster_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Toaster).unwrap();
        for id in [
            "class.toaster.shade",
            "class.toaster.bagel",
            "class.toaster.frozen",
            "class.toaster.single_side",
            "class.toaster.carriage",
            "class.toaster.sabbath_mode",
            "class.toaster.eco_mode",
            "class.toaster.heater_on",
            "class.toaster.high_temp_alarm",
            "class.toaster.timer_s",
            "class.toaster.crumb_tray_full",
            "class.toaster.slots",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical toaster"
            );
        }
        cap.validate_write("class.toaster.shade", &Value::U8(4))
            .unwrap();
        cap.validate_write("class.toaster.bagel", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.toaster.frozen", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.toaster.single_side", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.toaster.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.toaster.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.toaster.timer_s", &Value::DurationS(180))
            .unwrap();
        let err = cap
            .validate_write("class.toaster.carriage", &Value::Enum("down".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.toaster.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.toaster.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.toaster.crumb_tray_full", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.toaster.slots", &Value::U8(4))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn blender_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Blender).unwrap();
        for id in [
            "class.blender.speed_level",
            "class.blender.form_factor",
            "class.blender.pulse",
            "class.blender.jar_present",
            "class.blender.lid_locked",
            "class.blender.heated",
            "class.blender.sabbath_mode",
            "class.blender.eco_mode",
            "class.blender.motor_on",
            "class.blender.overload_trip",
            "class.blender.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical blender"
            );
        }
        cap.validate_write("class.blender.speed_level", &Value::U8(6))
            .unwrap();
        cap.validate_write("class.blender.pulse", &Value::Void)
            .unwrap();
        cap.validate_write("class.blender.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.blender.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.blender.timer_s", &Value::DurationS(90))
            .unwrap();
        let err = cap
            .validate_write("class.blender.form_factor", &Value::Enum("jar".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.blender.jar_present", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.blender.lid_locked", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.blender.heated", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.blender.motor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.blender.overload_trip", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn food_processor_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::FoodProcessor).unwrap();
        for id in [
            "class.food_processor.speed_level",
            "class.food_processor.pulse",
            "class.food_processor.bowl_present",
            "class.food_processor.lid_locked",
            "class.food_processor.attachment",
            "class.food_processor.sabbath_mode",
            "class.food_processor.eco_mode",
            "class.food_processor.motor_on",
            "class.food_processor.overload_trip",
            "class.food_processor.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical food_processor"
            );
        }
        cap.validate_write("class.food_processor.speed_level", &Value::U8(6))
            .unwrap();
        cap.validate_write("class.food_processor.pulse", &Value::Void)
            .unwrap();
        cap.validate_write("class.food_processor.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.food_processor.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.food_processor.timer_s", &Value::DurationS(90))
            .unwrap();
        let err = cap
            .validate_write(
                "class.food_processor.attachment",
                &Value::Enum("blade".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.food_processor.bowl_present", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.food_processor.lid_locked", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.food_processor.motor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.food_processor.overload_trip", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn stand_mixer_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::StandMixer).unwrap();
        for id in [
            "class.stand_mixer.speed_level",
            "class.stand_mixer.bowl_present",
            "class.stand_mixer.head_down",
            "class.stand_mixer.mass_g",
            "class.stand_mixer.attachment",
            "class.stand_mixer.sabbath_mode",
            "class.stand_mixer.eco_mode",
            "class.stand_mixer.motor_on",
            "class.stand_mixer.overload_trip",
            "class.stand_mixer.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical stand_mixer"
            );
        }
        cap.validate_write("class.stand_mixer.speed_level", &Value::U8(6))
            .unwrap();
        cap.validate_write("class.stand_mixer.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.stand_mixer.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.stand_mixer.timer_s", &Value::DurationS(90))
            .unwrap();
        let err = cap
            .validate_write(
                "class.stand_mixer.attachment",
                &Value::Enum("beater".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.stand_mixer.bowl_present", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.stand_mixer.head_down", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.stand_mixer.mass_g", &Value::F32(100.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.stand_mixer.motor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.stand_mixer.overload_trip", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn juicer_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Juicer).unwrap();
        for id in [
            "class.juicer.speed_level",
            "class.juicer.reverse",
            "class.juicer.pulp_full",
            "class.juicer.jug_present",
            "class.juicer.sabbath_mode",
            "class.juicer.eco_mode",
            "class.juicer.motor_on",
            "class.juicer.overload_trip",
            "class.juicer.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical juicer"
            );
        }
        cap.validate_write("class.juicer.speed_level", &Value::U8(6))
            .unwrap();
        cap.validate_write("class.juicer.reverse", &Value::Void)
            .unwrap();
        cap.validate_write("class.juicer.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.juicer.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.juicer.timer_s", &Value::DurationS(90))
            .unwrap();
        let err = cap
            .validate_write("class.juicer.pulp_full", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.juicer.jug_present", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.juicer.motor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.juicer.overload_trip", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn rice_cooker_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::RiceCooker).unwrap();
        for id in [
            "class.rice_cooker.texture",
            "class.rice_cooker.bowl_present",
            "class.rice_cooker.keep_warm",
            "class.rice_cooker.sabbath_mode",
            "class.rice_cooker.eco_mode",
            "class.rice_cooker.heater_on",
            "class.rice_cooker.high_temp_alarm",
            "class.rice_cooker.lid_open",
            "class.rice_cooker.timer_s",
            "class.rice_cooker.water_ratio",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical rice_cooker"
            );
        }
        cap.validate_write("class.rice_cooker.texture", &Value::Enum("firm".into()))
            .unwrap();
        cap.validate_write("class.rice_cooker.keep_warm", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.rice_cooker.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.rice_cooker.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.rice_cooker.timer_s", &Value::DurationS(3600))
            .unwrap();
        cap.validate_write("class.rice_cooker.water_ratio", &Value::F32(1.5))
            .unwrap();
        let err = cap
            .validate_write("class.rice_cooker.bowl_present", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.rice_cooker.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.rice_cooker.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.rice_cooker.lid_open", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn slow_cooker_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::SlowCooker).unwrap();
        for id in [
            "class.slow_cooker.heat_level",
            "class.slow_cooker.cook_s",
            "class.slow_cooker.pot_present",
            "class.slow_cooker.keep_warm",
            "class.slow_cooker.sabbath_mode",
            "class.slow_cooker.eco_mode",
            "class.slow_cooker.heater_on",
            "class.slow_cooker.high_temp_alarm",
            "class.slow_cooker.lid_open",
            "class.slow_cooker.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical slow_cooker"
            );
        }
        cap.validate_write("class.slow_cooker.heat_level", &Value::Enum("high".into()))
            .unwrap();
        cap.validate_write("class.slow_cooker.cook_s", &Value::DurationS(14400))
            .unwrap();
        cap.validate_write("class.slow_cooker.keep_warm", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.slow_cooker.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.slow_cooker.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.slow_cooker.timer_s", &Value::DurationS(3600))
            .unwrap();
        let err = cap
            .validate_write("class.slow_cooker.pot_present", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.slow_cooker.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.slow_cooker.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.slow_cooker.lid_open", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn bread_maker_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::BreadMaker).unwrap();
        for id in [
            "class.bread_maker.crust",
            "class.bread_maker.loaf_size",
            "class.bread_maker.pan_present",
            "class.bread_maker.keep_warm",
            "class.bread_maker.sabbath_mode",
            "class.bread_maker.eco_mode",
            "class.bread_maker.heater_on",
            "class.bread_maker.high_temp_alarm",
            "class.bread_maker.lid_open",
            "class.bread_maker.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical bread_maker"
            );
        }
        cap.validate_write("class.bread_maker.crust", &Value::Enum("dark".into()))
            .unwrap();
        cap.validate_write("class.bread_maker.loaf_size", &Value::Enum("large".into()))
            .unwrap();
        cap.validate_write("class.bread_maker.keep_warm", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.bread_maker.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.bread_maker.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.bread_maker.timer_s", &Value::DurationS(3600))
            .unwrap();
        let err = cap
            .validate_write("class.bread_maker.pan_present", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.bread_maker.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.bread_maker.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.bread_maker.lid_open", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn dehydrator_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Dehydrator).unwrap();
        for id in [
            "class.dehydrator.cook_s",
            "class.dehydrator.sabbath_mode",
            "class.dehydrator.eco_mode",
            "class.dehydrator.heater_on",
            "class.dehydrator.fan_on",
            "class.dehydrator.high_temp_alarm",
            "class.dehydrator.door_ajar",
            "class.dehydrator.timer_s",
            "class.dehydrator.tray_count",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical dehydrator"
            );
        }
        cap.validate_write("class.dehydrator.cook_s", &Value::DurationS(28800))
            .unwrap();
        cap.validate_write("class.dehydrator.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.dehydrator.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.dehydrator.timer_s", &Value::DurationS(3600))
            .unwrap();
        let err = cap
            .validate_write("class.dehydrator.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dehydrator.fan_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dehydrator.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dehydrator.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dehydrator.tray_count", &Value::U8(8))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn vacuum_sealer_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::VacuumSealer).unwrap();
        for id in [
            "class.vacuum_sealer.mode",
            "class.vacuum_sealer.moist",
            "class.vacuum_sealer.vacuum_kpa",
            "class.vacuum_sealer.bag_detect",
            "class.vacuum_sealer.form_factor",
            "class.vacuum_sealer.sabbath_mode",
            "class.vacuum_sealer.eco_mode",
            "class.vacuum_sealer.pump_on",
            "class.vacuum_sealer.seal_heater_on",
            "class.vacuum_sealer.lid_locked",
            "class.vacuum_sealer.seal_fail",
            "class.vacuum_sealer.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical vacuum_sealer"
            );
        }
        cap.validate_write("class.vacuum_sealer.mode", &Value::Enum("seal_only".into()))
            .unwrap();
        cap.validate_write("class.vacuum_sealer.moist", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.vacuum_sealer.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.vacuum_sealer.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.vacuum_sealer.timer_s", &Value::DurationS(30))
            .unwrap();
        let err = cap
            .validate_write("class.vacuum_sealer.vacuum_kpa", &Value::F32(20.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.vacuum_sealer.bag_detect", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.vacuum_sealer.form_factor",
                &Value::Enum("chamber".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.vacuum_sealer.pump_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.vacuum_sealer.seal_heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.vacuum_sealer.lid_locked", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.vacuum_sealer.seal_fail", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn ice_cream_maker_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::IceCreamMaker).unwrap();
        for id in [
            "class.ice_cream_maker.doneness",
            "class.ice_cream_maker.sabbath_mode",
            "class.ice_cream_maker.eco_mode",
            "class.ice_cream_maker.compressor_on",
            "class.ice_cream_maker.motor_on",
            "class.ice_cream_maker.bowl_present",
            "class.ice_cream_maker.lid_locked",
            "class.ice_cream_maker.low_temp_alarm",
            "class.ice_cream_maker.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical ice_cream_maker"
            );
        }
        cap.validate_write("class.ice_cream_maker.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.ice_cream_maker.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.ice_cream_maker.timer_s", &Value::DurationS(1800))
            .unwrap();
        let err = cap
            .validate_write("class.ice_cream_maker.doneness", &Value::Percent(50.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.ice_cream_maker.compressor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.ice_cream_maker.motor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.ice_cream_maker.bowl_present", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.ice_cream_maker.lid_locked", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.ice_cream_maker.low_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn yogurt_maker_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::YogurtMaker).unwrap();
        for id in [
            "class.yogurt_maker.incubate_s",
            "class.yogurt_maker.sabbath_mode",
            "class.yogurt_maker.eco_mode",
            "class.yogurt_maker.heater_on",
            "class.yogurt_maker.high_temp_alarm",
            "class.yogurt_maker.low_temp_alarm",
            "class.yogurt_maker.lid_open",
            "class.yogurt_maker.jar_present",
            "class.yogurt_maker.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical yogurt_maker"
            );
        }
        cap.validate_write("class.yogurt_maker.incubate_s", &Value::DurationS(28800))
            .unwrap();
        cap.validate_write("class.yogurt_maker.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.yogurt_maker.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.yogurt_maker.timer_s", &Value::DurationS(3600))
            .unwrap();
        let err = cap
            .validate_write("class.yogurt_maker.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.yogurt_maker.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.yogurt_maker.low_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.yogurt_maker.lid_open", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.yogurt_maker.jar_present", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn waffle_maker_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::WaffleMaker).unwrap();
        for id in [
            "class.waffle_maker.shade",
            "class.waffle_maker.ready",
            "class.waffle_maker.sabbath_mode",
            "class.waffle_maker.eco_mode",
            "class.waffle_maker.heater_on",
            "class.waffle_maker.high_temp_alarm",
            "class.waffle_maker.lid_open",
            "class.waffle_maker.batter_done",
            "class.waffle_maker.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical waffle_maker"
            );
        }
        cap.validate_write("class.waffle_maker.shade", &Value::U8(4))
            .unwrap();
        cap.validate_write("class.waffle_maker.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.waffle_maker.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.waffle_maker.timer_s", &Value::DurationS(180))
            .unwrap();
        let err = cap
            .validate_write("class.waffle_maker.ready", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.waffle_maker.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.waffle_maker.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.waffle_maker.lid_open", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.waffle_maker.batter_done", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn pasta_maker_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::PastaMaker).unwrap();
        for id in [
            "class.pasta_maker.die",
            "class.pasta_maker.jam",
            "class.pasta_maker.sabbath_mode",
            "class.pasta_maker.eco_mode",
            "class.pasta_maker.motor_on",
            "class.pasta_maker.dough_ready",
            "class.pasta_maker.hopper_empty",
            "class.pasta_maker.die_present",
            "class.pasta_maker.overload_trip",
            "class.pasta_maker.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical pasta_maker"
            );
        }
        cap.validate_write("class.pasta_maker.die", &Value::Enum("fettuccine".into()))
            .unwrap();
        cap.validate_write("class.pasta_maker.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.pasta_maker.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.pasta_maker.timer_s", &Value::DurationS(300))
            .unwrap();
        let err = cap
            .validate_write("class.pasta_maker.jam", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.pasta_maker.motor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.pasta_maker.dough_ready", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.pasta_maker.hopper_empty", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.pasta_maker.die_present", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.pasta_maker.overload_trip", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn steam_cooker_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::SteamCooker).unwrap();
        for id in [
            "class.steam_cooker.cook_s",
            "class.steam_cooker.water_empty",
            "class.steam_cooker.sabbath_mode",
            "class.steam_cooker.eco_mode",
            "class.steam_cooker.heater_on",
            "class.steam_cooker.high_temp_alarm",
            "class.steam_cooker.lid_open",
            "class.steam_cooker.steam_ready",
            "class.steam_cooker.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical steam_cooker"
            );
        }
        cap.validate_write("class.steam_cooker.cook_s", &Value::DurationS(1200))
            .unwrap();
        cap.validate_write("class.steam_cooker.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.steam_cooker.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.steam_cooker.timer_s", &Value::DurationS(600))
            .unwrap();
        let err = cap
            .validate_write("class.steam_cooker.water_empty", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.steam_cooker.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.steam_cooker.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.steam_cooker.lid_open", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.steam_cooker.steam_ready", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn garbage_disposal_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::GarbageDisposal).unwrap();
        for id in [
            "class.garbage_disposal.run_s",
            "class.garbage_disposal.jam",
            "class.garbage_disposal.reset_needed",
            "class.garbage_disposal.reverse",
            "class.garbage_disposal.sabbath_mode",
            "class.garbage_disposal.eco_mode",
            "class.garbage_disposal.motor_on",
            "class.garbage_disposal.overload_trip",
            "class.garbage_disposal.air_switch",
            "class.garbage_disposal.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical garbage_disposal"
            );
        }
        cap.validate_write("class.garbage_disposal.run_s", &Value::DurationS(20))
            .unwrap();
        cap.validate_write("class.garbage_disposal.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.garbage_disposal.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.garbage_disposal.air_switch", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.garbage_disposal.timer_s", &Value::DurationS(30))
            .unwrap();
        let err = cap
            .validate_write("class.garbage_disposal.jam", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.garbage_disposal.reset_needed", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.garbage_disposal.motor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.garbage_disposal.overload_trip", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn trash_compactor_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::TrashCompactor).unwrap();
        for id in [
            "class.trash_compactor.ram_state",
            "class.trash_compactor.bin_full",
            "class.trash_compactor.sabbath_mode",
            "class.trash_compactor.eco_mode",
            "class.trash_compactor.motor_on",
            "class.trash_compactor.drawer_open",
            "class.trash_compactor.overload_trip",
            "class.trash_compactor.key_lock",
            "class.trash_compactor.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical trash_compactor"
            );
        }
        cap.validate_write("class.trash_compactor.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.trash_compactor.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.trash_compactor.key_lock", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.trash_compactor.timer_s", &Value::DurationS(30))
            .unwrap();
        let err = cap
            .validate_write("class.trash_compactor.ram_state", &Value::Enum("up".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.trash_compactor.bin_full", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.trash_compactor.motor_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.trash_compactor.drawer_open", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.trash_compactor.overload_trip", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn boiler_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Boiler).unwrap();
        for id in [
            "class.boiler.pressure_bar",
            "class.boiler.burner_on",
            "class.boiler.flame_out",
            "class.boiler.low_pressure",
            "class.boiler.sabbath_mode",
            "class.boiler.eco_mode",
            "class.boiler.high_temp_alarm",
            "class.boiler.lockout",
            "class.boiler.ignition_fail",
            "class.boiler.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical boiler"
            );
        }
        cap.validate_write("class.boiler.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.boiler.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.boiler.timer_s", &Value::DurationS(30))
            .unwrap();
        let err = cap
            .validate_write("class.boiler.pressure_bar", &Value::F32(2.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.boiler.burner_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.boiler.flame_out", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.boiler.low_pressure", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.boiler.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.boiler.lockout", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.boiler.ignition_fail", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn water_softener_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::WaterSoftener).unwrap();
        for id in [
            "class.water_softener.capacity_remaining",
            "class.water_softener.salt_level",
            "class.water_softener.bypass",
            "class.water_softener.treated_l",
            "class.water_softener.sabbath_mode",
            "class.water_softener.eco_mode",
            "class.water_softener.regenerating",
            "class.water_softener.salt_low",
            "class.water_softener.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical water_softener"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Water)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.water.hardness_ppm"));
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Filter)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.filter.life_percent"));
        cap.validate_write("class.water_softener.bypass", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.water_softener.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.water_softener.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.water_softener.timer_s", &Value::DurationS(30))
            .unwrap();
        cap.validate_write("trait.water.hardness_ppm", &Value::U16(180))
            .unwrap();
        let err = cap
            .validate_write(
                "class.water_softener.capacity_remaining",
                &Value::F32(1000.0),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.water_softener.salt_level", &Value::Enum("ok".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.water_softener.treated_l", &Value::F32(10.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.water_softener.regenerating", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.water_softener.salt_low", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("trait.filter.life_percent", &Value::Percent(50.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn water_filter_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::WaterFilter).unwrap();
        for id in [
            "class.water_filter.tds_in_ppm",
            "class.water_filter.tds_out_ppm",
            "class.water_filter.tank_full",
            "class.water_filter.sabbath_mode",
            "class.water_filter.eco_mode",
            "class.water_filter.bypass",
            "class.water_filter.filter_clogged",
            "class.water_filter.replace_needed",
            "class.water_filter.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical water_filter"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Filter)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.filter.life_percent"));
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Water)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.water.flow_l_min"));
        cap.validate_write("class.water_filter.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.water_filter.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.water_filter.bypass", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.water_filter.timer_s", &Value::DurationS(30))
            .unwrap();
        let err = cap
            .validate_write("class.water_filter.tds_in_ppm", &Value::U16(250))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.water_filter.tds_out_ppm", &Value::U16(20))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.water_filter.tank_full", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.water_filter.filter_clogged", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.water_filter.replace_needed", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("trait.filter.life_percent", &Value::Percent(50.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("trait.water.flow_l_min", &Value::F32(2.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn sous_vide_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::SousVide).unwrap();
        for id in [
            "class.sous_vide.low_water",
            "class.sous_vide.circulating",
            "class.sous_vide.cook_s",
            "class.sous_vide.water_level_ok",
            "class.sous_vide.lid_closed",
            "class.sous_vide.timer_remaining_s",
            "class.sous_vide.target_done",
            "class.sous_vide.overtemp_alarm",
            "class.sous_vide.delayed_start_s",
            "class.sous_vide.alarm_offset_c",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical sous_vide"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Cycle)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.cycle.remaining_s"));
        cap.validate_write("class.sous_vide.cook_s", &Value::DurationS(3600))
            .unwrap();
        cap.validate_write("class.sous_vide.delayed_start_s", &Value::DurationS(600))
            .unwrap();
        cap.validate_write("class.sous_vide.alarm_offset_c", &Value::F32(1.0))
            .unwrap();
        let err = cap
            .validate_write("class.sous_vide.water_level_ok", &Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.sous_vide.overtemp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("trait.cycle.remaining_s", &Value::DurationS(100))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn water_heater_fridge_hvac_dishwasher_and_dryer_thermal_port_points() {
        let wh = typical_capability(ApplianceClassId::WaterHeater).unwrap();
        assert!(wh
            .class_points
            .iter()
            .any(|p| p.id == "class.water_heater.thermal_port_attached_reservoir_id"));
        wh.validate_write(
            "class.water_heater.thermal_port_attached_reservoir_id",
            &Value::String("dhw-tank".into()),
        )
        .unwrap();
        let err = wh
            .validate_write(
                "class.water_heater.thermal_port_direction",
                &Value::Enum("sink".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);

        let fridge = typical_capability(ApplianceClassId::Fridge).unwrap();
        assert!(fridge
            .class_points
            .iter()
            .any(|p| p.id == "class.fridge.thermal_port_max_power_w"));
        fridge
            .validate_write(
                "class.fridge.thermal_port_attached_reservoir_id",
                &Value::String("dhw-tank".into()),
            )
            .unwrap();

        let hvac = typical_capability(ApplianceClassId::Hvac).unwrap();
        assert!(hvac
            .class_points
            .iter()
            .any(|p| p.id == "class.hvac.thermal_port_attached_reservoir_id"));
        hvac.validate_write(
            "class.hvac.thermal_port_attached_reservoir_id",
            &Value::String("chw-buffer".into()),
        )
        .unwrap();
        let err = hvac
            .validate_write(
                "class.hvac.thermal_port_direction",
                &Value::Enum("source".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);

        let dw = typical_capability(ApplianceClassId::Dishwasher).unwrap();
        assert!(dw
            .class_points
            .iter()
            .any(|p| p.id == "class.dishwasher.thermal_port_attached_reservoir_id"));
        assert!(dw
            .class_points
            .iter()
            .any(|p| p.id == "class.dishwasher.thermal_port_id"));
        dw.validate_write(
            "class.dishwasher.thermal_port_attached_reservoir_id",
            &Value::String("dhw-tank".into()),
        )
        .unwrap();
        let err = dw
            .validate_write(
                "class.dishwasher.thermal_port_direction",
                &Value::Enum("source".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);

        let dryer = typical_capability(ApplianceClassId::Dryer).unwrap();
        assert!(dryer
            .class_points
            .iter()
            .any(|p| p.id == "class.dryer.thermal_port_attached_reservoir_id"));
        assert!(dryer
            .class_points
            .iter()
            .any(|p| p.id == "class.dryer.thermal_port_id"));
        dryer
            .validate_write(
                "class.dryer.thermal_port_attached_reservoir_id",
                &Value::String("air-buffer".into()),
            )
            .unwrap();
        let err = dryer
            .validate_write(
                "class.dryer.thermal_port_direction",
                &Value::Enum("sink".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);

        // Freezer has cold-cabinet + freezer-only depth (no thermal port surface).
        let freezer = typical_capability(ApplianceClassId::Freezer).unwrap();
        assert!(!freezer
            .class_points
            .iter()
            .any(|p| p.id.contains("thermal_port_")));
        assert!(freezer
            .class_points
            .iter()
            .any(|p| p.id == "class.freezer.fast_freeze"));
    }

    #[test]
    fn freezer_and_washer_dryer_typical_writes() {
        let freezer = typical_capability(ApplianceClassId::Freezer).unwrap();
        freezer
            .validate_write("trait.temperature.setpoint_c#freezer", &Value::F32(-18.0))
            .unwrap();
        let err = freezer
            .validate_write("trait.temperature.setpoint_c#freezer", &Value::F32(0.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfRange);

        let wd = typical_capability(ApplianceClassId::WasherDryer).unwrap();
        wd.validate_write(
            "class.washer_dryer.combo_mode",
            &Value::Enum("wash_and_dry".into()),
        )
        .unwrap();
        wd.validate_write("class.washer_dryer.spin_rpm", &Value::U16(800))
            .unwrap();
        let err = wd
            .validate_write("class.washer_dryer.spin_rpm", &Value::U16(2000))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfRange);

        let hvac = typical_capability(ApplianceClassId::Hvac).unwrap();
        hvac.validate_write("class.hvac.hvac_mode", &Value::Enum("heat".into()))
            .unwrap();
    }

    #[test]
    fn cooking_tier_a_typical_writes() {
        let steam = typical_capability(ApplianceClassId::SteamOven).unwrap();
        steam
            .validate_write("class.steam_oven.steam_mode", &Value::Enum("combi".into()))
            .unwrap();
        steam
            .validate_write("trait.program.program", &Value::Enum("steam".into()))
            .unwrap();
        steam
            .validate_write("trait.temperature.setpoint_c", &Value::F32(180.0))
            .unwrap();

        let cooktop = typical_capability(ApplianceClassId::Cooktop).unwrap();
        cooktop
            .validate_write("class.cooktop.level#hob_1", &Value::U8(5))
            .unwrap();
        let err = cooktop
            .validate_write("class.cooktop.level#hob_1", &Value::U8(20))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfRange);

        let range = typical_capability(ApplianceClassId::Range).unwrap();
        range
            .validate_write("class.range.level#hob_2", &Value::U8(3))
            .unwrap();
        range
            .validate_write("trait.temperature.setpoint_c#oven", &Value::F32(200.0))
            .unwrap();

        let coffee = typical_capability(ApplianceClassId::CoffeeMachine).unwrap();
        coffee
            .validate_write("trait.program.program", &Value::Enum("espresso".into()))
            .unwrap();

        let sv = typical_capability(ApplianceClassId::SousVide).unwrap();
        sv.validate_write("trait.temperature.setpoint_c", &Value::F32(55.0))
            .unwrap();
        let err = sv
            .validate_write("trait.temperature.setpoint_c", &Value::F32(10.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfRange);

        let multi = typical_capability(ApplianceClassId::MultiCooker).unwrap();
        multi
            .validate_write("trait.program.program", &Value::Enum("pressure".into()))
            .unwrap();
    }

    #[test]
    fn catalog_ids_are_snake_case_and_catalog_backed() {
        for id in ApplianceClassId::ALL {
            assert!(is_snake_case_id(id.as_str()), "{}", id.as_str());
        }
        for id in TraitId::ALL {
            assert!(is_snake_case_id(id.as_str()), "{}", id.as_str());
        }
        for table in static_class_tables() {
            for p in table.class_points {
                assert!(is_snake_case_id(p.id), "{}.{}", table.class_id, p.id);
            }
        }
        for t in TraitId::ALL {
            for p in trait_table(*t).unwrap().points {
                assert!(is_snake_case_id(p.id), "trait.{}.{}", t, p.id);
            }
        }
    }

    #[test]
    fn dryer_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Dryer).unwrap();
        for id in [
            "class.dryer.anti_crease",
            "class.dryer.dryness_percent",
            "class.dryer.vent_blocked",
            "class.dryer.drain_tank",
            "class.dryer.sabbath_mode",
            "class.dryer.eco_mode",
            "class.dryer.door_ajar",
            "class.dryer.door_locked",
            "class.dryer.high_temp_alarm",
            "class.dryer.lint_full",
            "class.dryer.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical dryer"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::TimeSchedule)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.time_schedule.delay_start_s"));
        cap.validate_write("class.dryer.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.dryer.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.dryer.anti_crease", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.dryer.timer_s", &Value::DurationS(3600))
            .unwrap();
        cap.validate_write("trait.time_schedule.delay_start_s", &Value::DurationS(1800))
            .unwrap();
        let err = cap
            .validate_write("class.dryer.dryness_percent", &Value::Percent(50.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dryer.vent_blocked", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dryer.drain_tank", &Value::Enum("full".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dryer.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dryer.door_locked", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dryer.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dryer.lint_full", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn washer_dryer_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::WasherDryer).unwrap();
        for id in [
            "class.washer_dryer.detergent_level_percent",
            "class.washer_dryer.unbalance",
            "class.washer_dryer.sabbath_mode",
            "class.washer_dryer.eco_mode",
            "class.washer_dryer.door_ajar",
            "class.washer_dryer.door_locked",
            "class.washer_dryer.water_temp_alarm",
            "class.washer_dryer.overflow_alarm",
            "class.washer_dryer.detergent_low",
            "class.washer_dryer.timer_s",
            "class.washer_dryer.anti_crease",
            "class.washer_dryer.dryness_percent",
            "class.washer_dryer.vent_blocked",
            "class.washer_dryer.drain_tank",
            "class.washer_dryer.high_temp_alarm",
            "class.washer_dryer.lint_full",
            "class.washer_dryer.dry_after_wash",
            "class.washer_dryer.max_dry_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical washer_dryer"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::TimeSchedule)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.time_schedule.delay_start_s"));
        // Composition: washer sabbath/eco/door/timer + dryer-specific high_temp/lint
        // — not a second copy of DRYER_DEPTH laundry ids.
        cap.validate_write("class.washer_dryer.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.washer_dryer.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.washer_dryer.anti_crease", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.washer_dryer.timer_s", &Value::DurationS(3600))
            .unwrap();
        cap.validate_write("class.washer_dryer.dry_after_wash", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.washer_dryer.max_dry_s", &Value::DurationS(7200))
            .unwrap();
        cap.validate_write("trait.time_schedule.delay_start_s", &Value::DurationS(1800))
            .unwrap();
        let err = cap
            .validate_write(
                "class.washer_dryer.detergent_level_percent",
                &Value::Percent(50.0),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer_dryer.unbalance", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer_dryer.dryness_percent", &Value::Percent(50.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer_dryer.vent_blocked", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer_dryer.drain_tank", &Value::Enum("full".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer_dryer.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer_dryer.door_locked", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer_dryer.water_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer_dryer.overflow_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer_dryer.detergent_low", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer_dryer.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer_dryer.lint_full", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn washer_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Washer).unwrap();
        for id in [
            "class.washer.detergent_level_percent",
            "class.washer.unbalance",
            "class.washer.sabbath_mode",
            "class.washer.eco_mode",
            "class.washer.door_ajar",
            "class.washer.door_locked",
            "class.washer.water_temp_alarm",
            "class.washer.overflow_alarm",
            "class.washer.detergent_low",
            "class.washer.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical washer"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::TimeSchedule)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.time_schedule.delay_start_s"));
        cap.validate_write("class.washer.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.washer.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.washer.timer_s", &Value::DurationS(3600))
            .unwrap();
        cap.validate_write("trait.time_schedule.delay_start_s", &Value::DurationS(1800))
            .unwrap();
        let err = cap
            .validate_write(
                "class.washer.detergent_level_percent",
                &Value::Percent(50.0),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer.unbalance", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer.door_locked", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer.water_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer.overflow_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.washer.detergent_low", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn dishwasher_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Dishwasher).unwrap();
        for id in [
            "class.dishwasher.wash_temp_c",
            "class.dishwasher.rinse_aid_level",
            "class.dishwasher.salt_level",
            "class.dishwasher.sabbath_mode",
            "class.dishwasher.eco_mode",
            "class.dishwasher.door_ajar",
            "class.dishwasher.door_locked",
            "class.dishwasher.rinse_aid_low",
            "class.dishwasher.salt_low",
            "class.dishwasher.overflow_alarm",
            "class.dishwasher.timer_s",
            // Thermal ports remain advertised (Stream 5).
            "class.dishwasher.thermal_port_id",
            "class.dishwasher.thermal_port_direction",
            "class.dishwasher.thermal_port_media",
            "class.dishwasher.thermal_port_max_power_w",
            "class.dishwasher.thermal_port_attached_reservoir_id",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical dishwasher"
            );
        }
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::TimeSchedule)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.time_schedule.delay_start_s"));
        cap.validate_write("class.dishwasher.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.dishwasher.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.dishwasher.timer_s", &Value::DurationS(3600))
            .unwrap();
        cap.validate_write("class.dishwasher.wash_temp_c", &Value::F32(45.0))
            .unwrap();
        cap.validate_write("trait.time_schedule.delay_start_s", &Value::DurationS(1800))
            .unwrap();
        let err = cap
            .validate_write(
                "class.dishwasher.rinse_aid_level",
                &Value::Enum("ok".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dishwasher.salt_level", &Value::Enum("ok".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dishwasher.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dishwasher.door_locked", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dishwasher.rinse_aid_low", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dishwasher.salt_low", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.dishwasher.overflow_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn microwave_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Microwave).unwrap();
        for id in [
            // Required cook surface still present.
            "class.microwave.cook_s",
            "class.microwave.power_level_percent",
            // Thin-table optional advertised in typical.
            "class.microwave.power_w",
            "class.microwave.defrost_g",
            "class.microwave.turntable",
            "class.microwave.inverter",
            // Depth points.
            "class.microwave.sabbath_mode",
            "class.microwave.eco_mode",
            "class.microwave.door_ajar",
            "class.microwave.magnetron_on",
            "class.microwave.high_temp_alarm",
            "class.microwave.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical microwave"
            );
        }
        // ChildLock already typical.
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::ChildLock)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.child_lock.child_lock"));
        cap.validate_write("class.microwave.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.microwave.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.microwave.timer_s", &Value::DurationS(3600))
            .unwrap();
        cap.validate_write("class.microwave.turntable", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.microwave.power_w", &Value::U16(1000))
            .unwrap();
        cap.validate_write("class.microwave.defrost_g", &Value::U16(500))
            .unwrap();
        cap.validate_write("trait.child_lock.child_lock", &Value::Bool(true))
            .unwrap();
        let err = cap
            .validate_write("class.microwave.inverter", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.microwave.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.microwave.magnetron_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.microwave.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn oven_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Oven).unwrap();
        for id in [
            // Thin-table optional advertised in typical.
            "class.oven.broil_level",
            "class.oven.convection_fan",
            "class.oven.steam_percent",
            "class.oven.cook_s",
            "class.oven.door_locked_clean",
            "class.oven.element_bake",
            "class.oven.element_broil",
            // OVEN_DEPTH points.
            "class.oven.sabbath_mode",
            "class.oven.eco_mode",
            "class.oven.heater_on",
            "class.oven.high_temp_alarm",
            "class.oven.door_ajar",
            "class.oven.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical oven"
            );
        }
        // Meat probe + preheat via Temperature trait (not class probe_c).
        let temp = cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Temperature)
            .unwrap();
        for id in [
            "trait.temperature.probe_c",
            "trait.temperature.probe_target_c",
            "trait.temperature.probe_connected",
            "trait.temperature.preheat_complete",
        ] {
            assert!(
                temp.points.iter().any(|p| p.id == id),
                "missing {id} in typical oven temperature trait"
            );
        }
        // ChildLock already typical.
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::ChildLock)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.child_lock.child_lock"));
        cap.validate_write("class.oven.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.oven.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.oven.timer_s", &Value::DurationS(3600))
            .unwrap();
        cap.validate_write("class.oven.broil_level", &Value::Enum("high".into()))
            .unwrap();
        cap.validate_write("class.oven.convection_fan", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.oven.steam_percent", &Value::Percent(40.0))
            .unwrap();
        cap.validate_write("class.oven.cook_s", &Value::DurationS(1800))
            .unwrap();
        cap.validate_write("trait.temperature.probe_target_c", &Value::F32(65.0))
            .unwrap();
        cap.validate_write("trait.child_lock.child_lock", &Value::Bool(true))
            .unwrap();
        let err = cap
            .validate_write("class.oven.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.oven.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.oven.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.oven.door_locked_clean", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.oven.element_bake", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.oven.element_broil", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("trait.temperature.probe_c", &Value::F32(55.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("trait.temperature.probe_connected", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("trait.temperature.preheat_complete", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);

        // steam_oven / toaster_oven merge OVEN_BASE only — OVEN_DEPTH must not
        // appear on their static class tables (laundry-style isolation).
        // `range` adds cavity depth via RANGE_EXTRA (not OVEN_DEPTH) and keeps
        // a single cooktop `timer_s` (merging OVEN_DEPTH would duplicate it).
        for class_id in [ApplianceClassId::SteamOven, ApplianceClassId::ToasterOven] {
            let table = class_table(class_id).unwrap();
            let ids: Vec<&str> = table.class_points.iter().map(|p| p.id).collect();
            assert!(
                ids.contains(&"cook_s"),
                "{class_id:?} should still merge OVEN_BASE cook_s"
            );
            assert!(
                ids.contains(&"door_locked_clean"),
                "{class_id:?} should still merge OVEN_BASE door_locked_clean"
            );
            for depth in [
                "sabbath_mode",
                "eco_mode",
                "heater_on",
                "high_temp_alarm",
                "door_ajar",
                "timer_s",
            ] {
                assert!(
                    !ids.contains(&depth),
                    "{depth} must not leak from OVEN_DEPTH into {class_id:?} class table"
                );
            }
        }
        let range_table = class_table(ApplianceClassId::Range).unwrap();
        let range_ids: Vec<&str> = range_table.class_points.iter().map(|p| p.id).collect();
        assert!(
            range_ids.contains(&"cook_s"),
            "range should still merge OVEN_BASE cook_s"
        );
        assert!(
            range_ids.contains(&"door_locked_clean"),
            "range should still merge OVEN_BASE door_locked_clean"
        );
        assert!(
            range_ids.contains(&"sabbath_mode"),
            "range should advertise cavity depth via RANGE_EXTRA (not OVEN_DEPTH merge)"
        );
        assert_eq!(
            range_ids.iter().filter(|&&id| id == "timer_s").count(),
            1,
            "range must keep a single cooktop timer_s (no OVEN_DEPTH duplicate)"
        );
    }

    #[test]
    fn range_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::Range).unwrap();
        for id in [
            // Required composition surface / hob.
            "class.range.surface",
            "class.range.level",
            "class.range.residual_heat",
            // Cooktop depth (COOKTOP_POINTS composition).
            "class.range.boost",
            "class.range.timer_s",
            "class.range.bridge",
            "class.range.flame_out",
            "class.range.ignition_fail",
            "class.range.power_limit_w",
            "class.range.keep_warm",
            "class.range.hotspot_alert",
            "class.range.timer_active",
            "class.range.paused",
            "class.range.surface_c",
            "class.range.element_fault",
            "class.range.pan_detect",
            "class.range.flame_on",
            // OVEN_BASE thin cavity.
            "class.range.broil_level",
            "class.range.convection_fan",
            "class.range.steam_percent",
            "class.range.cook_s",
            "class.range.door_locked_clean",
            "class.range.element_bake",
            "class.range.element_broil",
            // RANGE_EXTRA cavity depth (no timer_s duplicate).
            "class.range.sabbath_mode",
            "class.range.eco_mode",
            "class.range.heater_on",
            "class.range.high_temp_alarm",
            "class.range.door_ajar",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical range"
            );
        }
        // Meat probe + preheat via Temperature trait (not class probe_c).
        let temp = cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Temperature)
            .unwrap();
        for id in [
            "trait.temperature.probe_c",
            "trait.temperature.probe_target_c",
            "trait.temperature.probe_connected",
            "trait.temperature.preheat_complete",
        ] {
            assert!(
                temp.points.iter().any(|p| p.id == id),
                "missing {id} in typical range temperature trait"
            );
        }
        let probe = temp
            .points
            .iter()
            .find(|p| p.id == "trait.temperature.probe_c")
            .unwrap();
        assert_eq!(
            probe.zones.as_ref().unwrap(),
            &["hob_1", "hob_2", "hob_3", "hob_4", "oven"].map(str::to_string)
        );
        // ChildLock already typical.
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::ChildLock)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.child_lock.child_lock"));
        // Single cooktop timer_s (zoned) — not a second oven kitchen timer.
        assert_eq!(
            cap.class_points
                .iter()
                .filter(|p| p.id == "class.range.timer_s")
                .count(),
            1
        );
        let timer = cap
            .class_points
            .iter()
            .find(|p| p.id == "class.range.timer_s")
            .unwrap();
        assert_eq!(
            timer.zones.as_ref().unwrap(),
            &["hob_1", "hob_2", "hob_3", "hob_4", "oven"].map(str::to_string)
        );

        cap.validate_write("class.range.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.range.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.range.boost#hob_1", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.range.timer_s#hob_2", &Value::DurationS(600))
            .unwrap();
        cap.validate_write("class.range.keep_warm#hob_1", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.range.power_limit_w", &Value::U32(4800))
            .unwrap();
        cap.validate_write("class.range.broil_level", &Value::Enum("high".into()))
            .unwrap();
        cap.validate_write("class.range.convection_fan", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.range.steam_percent", &Value::Percent(40.0))
            .unwrap();
        cap.validate_write("class.range.cook_s", &Value::DurationS(1800))
            .unwrap();
        cap.validate_write("trait.temperature.probe_target_c", &Value::F32(65.0))
            .unwrap();
        cap.validate_write("trait.child_lock.child_lock", &Value::Bool(true))
            .unwrap();
        let err = cap
            .validate_write("class.range.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.range.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.range.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.range.door_locked_clean", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.range.element_bake", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.range.element_broil", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.range.surface", &Value::Enum("gas".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.range.hotspot_alert#hob_1", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.range.paused", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("trait.temperature.probe_c#oven", &Value::F32(55.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("trait.temperature.probe_connected", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("trait.temperature.preheat_complete", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn air_fryer_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::AirFryer).unwrap();
        for id in [
            // Required cook surface still present.
            "class.air_fryer.cook_s",
            // Thin-table optional advertised in typical.
            "class.air_fryer.shake_enable",
            "class.air_fryer.shake_due",
            "class.air_fryer.preheat",
            "class.air_fryer.basket_present",
            "class.air_fryer.sync_finish",
            // Depth points.
            "class.air_fryer.sabbath_mode",
            "class.air_fryer.eco_mode",
            "class.air_fryer.heater_on",
            "class.air_fryer.fan_on",
            "class.air_fryer.high_temp_alarm",
            "class.air_fryer.door_ajar",
            "class.air_fryer.timer_s",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical air_fryer"
            );
        }
        // Heater / Fan / DoorLid traits already typical.
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Heater)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.heater.heater_state"));
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::Fan)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.fan.fan_state"));
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::DoorLid)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.door_lid.door_state"));
        // Single kitchen timer_s — distinct from required cook_s.
        assert_eq!(
            cap.class_points
                .iter()
                .filter(|p| p.id == "class.air_fryer.timer_s")
                .count(),
            1
        );
        assert_eq!(
            cap.class_points
                .iter()
                .filter(|p| p.id == "class.air_fryer.cook_s")
                .count(),
            1
        );

        cap.validate_write("class.air_fryer.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.air_fryer.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.air_fryer.timer_s", &Value::DurationS(1800))
            .unwrap();
        cap.validate_write("class.air_fryer.shake_enable", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.air_fryer.preheat", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.air_fryer.sync_finish", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.air_fryer.cook_s", &Value::DurationS(900))
            .unwrap();
        let err = cap
            .validate_write("class.air_fryer.shake_due", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.air_fryer.basket_present", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.air_fryer.heater_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.air_fryer.fan_on", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.air_fryer.high_temp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.air_fryer.door_ajar", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn induction_hob_optional_depth_points_in_typical() {
        let cap = typical_capability(ApplianceClassId::InductionHob).unwrap();
        for id in [
            // Required composition.
            "class.induction_hob.level",
            "class.induction_hob.residual_heat",
            "class.induction_hob.pan_present",
            // Cooktop depth (COOKTOP_POINTS composition).
            "class.induction_hob.boost",
            "class.induction_hob.timer_s",
            "class.induction_hob.bridge",
            "class.induction_hob.flame_out",
            "class.induction_hob.ignition_fail",
            "class.induction_hob.power_limit_w",
            "class.induction_hob.keep_warm",
            "class.induction_hob.hotspot_alert",
            "class.induction_hob.timer_active",
            "class.induction_hob.paused",
            "class.induction_hob.surface_c",
            "class.induction_hob.element_fault",
            "class.induction_hob.pan_detect",
            "class.induction_hob.flame_on",
            // Thin INDUCTION_HOB_EXTRA surface.
            "class.induction_hob.pan_size",
            "class.induction_hob.power_w",
            "class.induction_hob.limiter_active",
            "class.induction_hob.cookware_ok",
            "class.induction_hob.temp_mode",
            "class.induction_hob.flex_group",
            // New EXTRA depth.
            "class.induction_hob.sabbath_mode",
            "class.induction_hob.eco_mode",
            "class.induction_hob.power_share",
            "class.induction_hob.auto_boost",
            "class.induction_hob.overtemp_alarm",
        ] {
            assert!(
                cap.class_points.iter().any(|p| p.id == id),
                "missing {id} in typical induction_hob"
            );
        }
        // ChildLock already typical on HOB_TRAITS.
        assert!(cap
            .traits
            .iter()
            .find(|t| t.trait_id == TraitId::ChildLock)
            .unwrap()
            .points
            .iter()
            .any(|p| p.id == "trait.child_lock.child_lock"));
        // Single cooktop timer_s (zoned) — no duplicate induction timer id.
        assert_eq!(
            cap.class_points
                .iter()
                .filter(|p| p.id == "class.induction_hob.timer_s")
                .count(),
            1
        );
        let timer = cap
            .class_points
            .iter()
            .find(|p| p.id == "class.induction_hob.timer_s")
            .unwrap();
        assert_eq!(
            timer.zones.as_ref().unwrap(),
            &["hob_1", "hob_2", "hob_3", "hob_4"].map(str::to_string)
        );
        let pan_detect = cap
            .class_points
            .iter()
            .find(|p| p.id == "class.induction_hob.pan_detect")
            .unwrap();
        assert_eq!(
            pan_detect.zones.as_ref().unwrap(),
            &["hob_1", "hob_2", "hob_3", "hob_4"].map(str::to_string)
        );
        // pan_present (induction) stays distinct from cooktop pan_detect.
        assert!(cap
            .class_points
            .iter()
            .any(|p| p.id == "class.induction_hob.pan_present"));

        cap.validate_write("class.induction_hob.sabbath_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.induction_hob.eco_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.induction_hob.power_share", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.induction_hob.auto_boost", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.induction_hob.temp_mode", &Value::Bool(true))
            .unwrap();
        cap.validate_write(
            "class.induction_hob.flex_group",
            &Value::String("hob_2".into()),
        )
        .unwrap();
        cap.validate_write("class.induction_hob.boost#hob_1", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.induction_hob.timer_s#hob_2", &Value::DurationS(600))
            .unwrap();
        cap.validate_write("class.induction_hob.keep_warm#hob_1", &Value::Bool(true))
            .unwrap();
        cap.validate_write("class.induction_hob.power_limit_w", &Value::U32(4800))
            .unwrap();
        cap.validate_write("trait.child_lock.child_lock", &Value::Bool(true))
            .unwrap();
        let err = cap
            .validate_write("class.induction_hob.overtemp_alarm", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.induction_hob.limiter_active", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.induction_hob.cookware_ok", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.induction_hob.pan_present#hob_1", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.induction_hob.pan_detect#hob_1", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.induction_hob.residual_heat#hob_1",
                &Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.induction_hob.paused", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write("class.induction_hob.power_w#hob_1", &Value::U16(1200))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = cap
            .validate_write(
                "class.induction_hob.pan_size#hob_1",
                &Value::Enum("medium".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn washer_spin_rpm_range() {
        let cap = typical_capability(ApplianceClassId::Washer).unwrap();
        cap.validate_write("class.washer.spin_rpm", &Value::U16(800))
            .unwrap();
        let err = cap
            .validate_write("class.washer.spin_rpm", &Value::U16(2000))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfRange);

        cap.validate_write("trait.program.program", &Value::Enum("eco".into()))
            .unwrap();
        let err = cap
            .validate_write(
                "trait.program.program",
                &Value::Enum("not_a_program".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidEnum);

        let err = cap
            .validate_write("trait.cycle.cycle_state", &Value::Enum("running".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);

        let err = cap
            .validate_write("class.washer.steam", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::UnsupportedCapability);

        let err = cap
            .validate_write("class.washer.no_such_point", &Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::UnknownVariable);
    }

    #[test]
    fn fridge_setpoint_typical_range() {
        let cap = typical_capability(ApplianceClassId::Fridge).unwrap();
        cap.validate_write("trait.temperature.setpoint_c", &Value::F32(4.0))
            .unwrap();
        let err = cap
            .validate_write("trait.temperature.setpoint_c", &Value::F32(20.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfRange);
    }

    #[test]
    fn enum_subset_is_advertised_not_full_catalog() {
        let mut cap = typical_capability(ApplianceClassId::Washer).unwrap();
        let program = cap
            .traits
            .iter_mut()
            .find(|t| t.trait_id == TraitId::Program)
            .unwrap()
            .points
            .iter_mut()
            .find(|p| p.id == "trait.program.program")
            .unwrap();
        program.range = Some(ValueRange::enum_tokens(["cotton", "eco"]));
        cap.validate_write("trait.program.program", &Value::Enum("eco".into()))
            .unwrap();
        let err = cap
            .validate_write("trait.program.program", &Value::Enum("wool".into()))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidEnum);
    }
}
