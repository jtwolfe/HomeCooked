//! Catalog class and trait ids, plus qualified point ids.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Catalog naming rule: `snake_case`, starts with a letter, length ≤ 64.
pub fn is_snake_case_id(s: &str) -> bool {
    let len = s.len();
    if len == 0 || len > 64 {
        return false;
    }
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Failed parse of a catalog id or qualified point id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseIdError {
    pub kind: &'static str,
    pub value: String,
}

impl fmt::Display for ParseIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown {} id {:?}", self.kind, self.value)
    }
}

impl std::error::Error for ParseIdError {}

macro_rules! snake_ids {
    ($vis:vis enum $name:ident { $($variant:ident => $str:literal),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub enum $name {
            $(
                #[serde(rename = $str)]
                $variant,
            )*
        }

        impl $name {
            pub const ALL: &'static [Self] = &[ $(Self::$variant),* ];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $str,)*
                }
            }

            pub fn from_str_id(s: &str) -> Option<Self> {
                match s {
                    $($str => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = ParseIdError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::from_str_id(s).ok_or_else(|| ParseIdError {
                    kind: stringify!($name),
                    value: s.to_string(),
                })
            }
        }
    };
}

snake_ids! {
    pub enum ApplianceClassId {
        Washer => "washer",
        Dryer => "dryer",
        WasherDryer => "washer_dryer",
        Fridge => "fridge",
        Freezer => "freezer",
        FridgeFreezer => "fridge_freezer",
        WineCooler => "wine_cooler",
        BeverageCooler => "beverage_cooler",
        IceMaker => "ice_maker",
        Kegerator => "kegerator",
        Dishwasher => "dishwasher",
        Microwave => "microwave",
        Oven => "oven",
        SteamOven => "steam_oven",
        ToasterOven => "toaster_oven",
        Range => "range",
        Cooktop => "cooktop",
        InductionHob => "induction_hob",
        WarmingDrawer => "warming_drawer",
        PizzaOven => "pizza_oven",
        AirFryer => "air_fryer",
        ElectricGrill => "electric_grill",
        ElectricSmoker => "electric_smoker",
        RangeHood => "range_hood",
        CoffeeMachine => "coffee_machine",
        EspressoMachine => "espresso_machine",
        DripCoffeeMaker => "drip_coffee_maker",
        CoffeeGrinder => "coffee_grinder",
        Kettle => "kettle",
        WaterDispenser => "water_dispenser",
        Toaster => "toaster",
        Blender => "blender",
        FoodProcessor => "food_processor",
        StandMixer => "stand_mixer",
        Juicer => "juicer",
        RiceCooker => "rice_cooker",
        SlowCooker => "slow_cooker",
        MultiCooker => "multi_cooker",
        SousVide => "sous_vide",
        BreadMaker => "bread_maker",
        Dehydrator => "dehydrator",
        VacuumSealer => "vacuum_sealer",
        IceCreamMaker => "ice_cream_maker",
        YogurtMaker => "yogurt_maker",
        WaffleMaker => "waffle_maker",
        PastaMaker => "pasta_maker",
        SteamCooker => "steam_cooker",
        GarbageDisposal => "garbage_disposal",
        TrashCompactor => "trash_compactor",
        WaterHeater => "water_heater",
        Boiler => "boiler",
        WaterSoftener => "water_softener",
        WaterFilter => "water_filter",
        Hvac => "hvac",
        Dehumidifier => "dehumidifier",
        Humidifier => "humidifier",
    }
}

snake_ids! {
    pub enum TraitId {
        Identity => "identity",
        Power => "power",
        Connectivity => "connectivity",
        TimeSchedule => "time_schedule",
        DoorLid => "door_lid",
        ChildLock => "child_lock",
        Lighting => "lighting",
        Audio => "audio",
        Temperature => "temperature",
        Humidity => "humidity",
        Cycle => "cycle",
        Program => "program",
        Fault => "fault",
        Energy => "energy",
        Water => "water",
        Filter => "filter",
        Remote => "remote",
        Maintenance => "maintenance",
        Safety => "safety",
        Fan => "fan",
        Heater => "heater",
        Motor => "motor",
        Zone => "zone",
        Dispense => "dispense",
        Ice => "ice",
        Ota => "ota",
    }
}

/// Namespace of a qualified point id (`trait.*`, `class.*`, `vendor.*`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
pub enum PointNamespace {
    Trait(TraitId),
    Class(ApplianceClassId),
    Vendor(String),
}

impl PointNamespace {
    pub fn as_prefix(&self) -> &'static str {
        match self {
            Self::Trait(_) => "trait",
            Self::Class(_) => "class",
            Self::Vendor(_) => "vendor",
        }
    }
}

/// Qualified point id, optionally zoned (`trait.temperature.setpoint_c#freezer`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedPointId {
    pub namespace: PointNamespace,
    pub id: String,
    pub zone: Option<String>,
}

impl QualifiedPointId {
    pub fn new(namespace: PointNamespace, id: impl Into<String>) -> Self {
        Self {
            namespace,
            id: id.into(),
            zone: None,
        }
    }

    pub fn with_zone(mut self, zone: impl Into<String>) -> Self {
        self.zone = Some(zone.into());
        self
    }

    pub fn trait_point(trait_id: TraitId, id: impl Into<String>) -> Self {
        Self::new(PointNamespace::Trait(trait_id), id)
    }

    pub fn class_point(class_id: ApplianceClassId, id: impl Into<String>) -> Self {
        Self::new(PointNamespace::Class(class_id), id)
    }

    pub fn unzoned(&self) -> Self {
        Self {
            namespace: self.namespace.clone(),
            id: self.id.clone(),
            zone: None,
        }
    }

    pub fn base_string(&self) -> String {
        match &self.namespace {
            PointNamespace::Trait(t) => format!("trait.{}.{}", t.as_str(), self.id),
            PointNamespace::Class(c) => format!("class.{}.{}", c.as_str(), self.id),
            PointNamespace::Vendor(v) => format!("vendor.{}.{}", v, self.id),
        }
    }

    pub fn parse(s: &str) -> Result<Self, ParseIdError> {
        let (base, zone) = match s.split_once('#') {
            Some((b, z)) => {
                if z.is_empty() || !is_snake_case_id(z) || b.contains('#') {
                    return Err(ParseIdError {
                        kind: "qualified_point",
                        value: s.to_string(),
                    });
                }
                (b, Some(z.to_string()))
            }
            None => (s, None),
        };
        let mut parts = base.splitn(3, '.');
        let kind = parts.next().unwrap_or("");
        let mid = parts.next();
        let id = parts.next();
        match (kind, mid, id) {
            ("trait", Some(t), Some(id)) if !id.is_empty() => {
                let trait_id = TraitId::from_str_id(t).ok_or_else(|| ParseIdError {
                    kind: "qualified_point",
                    value: s.to_string(),
                })?;
                if !is_snake_case_id(id) {
                    return Err(ParseIdError {
                        kind: "qualified_point",
                        value: s.to_string(),
                    });
                }
                Ok(Self {
                    namespace: PointNamespace::Trait(trait_id),
                    id: id.to_string(),
                    zone,
                })
            }
            ("class", Some(c), Some(id)) if !id.is_empty() => {
                let class_id = ApplianceClassId::from_str_id(c).ok_or_else(|| ParseIdError {
                    kind: "qualified_point",
                    value: s.to_string(),
                })?;
                if !is_snake_case_id(id) {
                    return Err(ParseIdError {
                        kind: "qualified_point",
                        value: s.to_string(),
                    });
                }
                Ok(Self {
                    namespace: PointNamespace::Class(class_id),
                    id: id.to_string(),
                    zone,
                })
            }
            ("vendor", Some(v), Some(id)) if !id.is_empty() => {
                if !is_snake_case_id(v) || !is_snake_case_id(id) {
                    return Err(ParseIdError {
                        kind: "qualified_point",
                        value: s.to_string(),
                    });
                }
                Ok(Self {
                    namespace: PointNamespace::Vendor(v.to_string()),
                    id: id.to_string(),
                    zone,
                })
            }
            _ => Err(ParseIdError {
                kind: "qualified_point",
                value: s.to_string(),
            }),
        }
    }
}

impl fmt::Display for QualifiedPointId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let base = self.base_string();
        if let Some(zone) = &self.zone {
            write!(f, "{base}#{zone}")
        } else {
            f.write_str(&base)
        }
    }
}

impl FromStr for QualifiedPointId {
    type Err = ParseIdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for QualifiedPointId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for QualifiedPointId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case_rules() {
        assert!(is_snake_case_id("washer"));
        assert!(is_snake_case_id("induction_hob"));
        assert!(is_snake_case_id("a"));
        assert!(!is_snake_case_id(""));
        assert!(!is_snake_case_id("Washer"));
        assert!(!is_snake_case_id("1washer"));
        assert!(!is_snake_case_id("wash-er"));
        assert!(!is_snake_case_id(&"a".repeat(65)));
    }

    #[test]
    fn class_ids_roundtrip() {
        for id in ApplianceClassId::ALL {
            assert_eq!(ApplianceClassId::from_str_id(id.as_str()), Some(*id));
            let json = serde_json::to_string(id).unwrap();
            assert_eq!(json, format!("\"{}\"", id.as_str()));
            let back: ApplianceClassId = serde_json::from_str(&json).unwrap();
            assert_eq!(back, *id);
            assert!(is_snake_case_id(id.as_str()));
        }
    }

    #[test]
    fn trait_ids_roundtrip() {
        for id in TraitId::ALL {
            assert_eq!(TraitId::from_str_id(id.as_str()), Some(*id));
            let json = serde_json::to_string(id).unwrap();
            let back: TraitId = serde_json::from_str(&json).unwrap();
            assert_eq!(back, *id);
            assert!(is_snake_case_id(id.as_str()));
        }
    }

    #[test]
    fn qualified_id_parse() {
        let q = QualifiedPointId::parse("trait.temperature.setpoint_c#freezer").unwrap();
        assert_eq!(q.namespace, PointNamespace::Trait(TraitId::Temperature));
        assert_eq!(q.id, "setpoint_c");
        assert_eq!(q.zone.as_deref(), Some("freezer"));
        assert_eq!(q.to_string(), "trait.temperature.setpoint_c#freezer");

        let q = QualifiedPointId::parse("class.washer.spin_rpm").unwrap();
        assert_eq!(q.namespace, PointNamespace::Class(ApplianceClassId::Washer));
        assert_eq!(q.id, "spin_rpm");
        assert!(q.zone.is_none());

        assert!(QualifiedPointId::parse("foo.bar.baz").is_err());
        assert!(QualifiedPointId::parse("trait.not_a_trait.x").is_err());
    }
}
