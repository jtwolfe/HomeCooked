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
    false
}

/// Optional class points that the typical model still advertises for demos.
fn extra_typical_class_point(table: &ClassTable, point: &CatalogPoint) -> bool {
    if table.class_id == ApplianceClassId::Dishwasher && point.id == "wash_temp_c" {
        return true;
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

        // Freezer keeps cold-cabinet points only (no thermal port surface yet).
        let freezer = typical_capability(ApplianceClassId::Freezer).unwrap();
        assert!(!freezer
            .class_points
            .iter()
            .any(|p| p.id.contains("thermal_port_")));
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
