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
    trait_id == TraitId::Temperature
        && point.id == "setpoint_c"
        && table.typical_setpoint_c.is_some()
}

/// Optional class points that the typical model still advertises for demos.
fn extra_typical_class_point(table: &ClassTable, point: &CatalogPoint) -> bool {
    table.class_id == ApplianceClassId::Dishwasher && point.id == "wash_temp_c"
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
