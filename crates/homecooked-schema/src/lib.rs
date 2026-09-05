//! Catalog-backed schema types for HomeCooked.
//!
//! Types, capability model, and write validation derived from
//! `docs/catalog` and `docs/standard/overview.md`. Schema and catalog
//! versions are both **0.1.0**. Shared thermal vocabulary (`Media`,
//! `PortDirection`, `TempBandC`, `HeatPortSpec`) aligns with catalog
//! `thermal_port_*` tokens; `ClassTable.thermal_ports` carries static
//! `HeatPortSpec` advertisement metadata. Full plant runtime stays in
//! `homecooked-thermal`.

mod access;
mod capability;
mod catalog;
mod error;
mod export;
mod identity;
mod ids;
mod spec;
mod thermal;
mod types;
mod validate;
mod version;

pub use access::AccessMode;
pub use capability::{CapabilityModel, PointCapability, SafetyFlags, TraitCapability};
pub use catalog::{
    catalog_group, catalog_point, class_table, list_all_class_ids, static_class_tables,
    trait_table, typical_capability, ClassTable, TraitTable, CATALOG_GROUP_ORDER, STATIC_CLASS_IDS,
    TIER_A_CLASS_IDS, TIER_B_CLASS_IDS,
};
pub use error::{ErrorCode, ValidationError};
pub use export::{
    catalog_export, export_catalog_json, CatalogClassExport, CatalogExport, CATALOG_EXPORT_FORMAT,
    CATALOG_EXPORT_FORMAT_VERSION,
};
pub use identity::DeviceIdentity;
pub use ids::{
    is_snake_case_id, ApplianceClassId, ParseIdError, PointNamespace, QualifiedPointId, TraitId,
};
pub use spec::{
    CatalogPoint, CatalogRange, CommandSpec, PointKind, PointSpec, SettingSpec, VariableSpec,
};
pub use thermal::{
    HeatPortSpec, InvalidTempBand, Media, PortDirection, TempBandC, THERMAL_PORT_DIRECTION_TOKENS,
    THERMAL_PORT_MEDIA_TOKENS,
};
pub use types::{CommandArg, ListItemType, Unit, Value, ValueRange, ValueType};
pub use validate::{is_write_validation_code, validate_against_spec};
pub use version::{
    CatalogVersion, SchemaVersion, SemVer, CATALOG_VERSION, DEFAULT_CLASS_VERSION,
    DEFAULT_TRAIT_VERSION, SCHEMA_VERSION,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_versions() {
        assert_eq!(SCHEMA_VERSION.to_string(), "0.1.0");
        assert_eq!(CATALOG_VERSION.to_string(), "0.1.0");
    }

    #[test]
    fn key_types_serde_roundtrip() {
        let class = ApplianceClassId::WasherDryer;
        let trait_id = TraitId::TimeSchedule;
        let access = AccessMode::RWE;
        let value = Value::U16(800);
        let identity = DeviceIdentity::new(
            "dev-1",
            "Acme",
            "WD200",
            "0.1.0",
            ApplianceClassId::WasherDryer,
        );
        let cap = typical_capability(ApplianceClassId::Oven).unwrap();

        for json in [
            serde_json::to_string(&class).unwrap(),
            serde_json::to_string(&trait_id).unwrap(),
            serde_json::to_string(&access).unwrap(),
            serde_json::to_string(&value).unwrap(),
            serde_json::to_string(&identity).unwrap(),
            serde_json::to_string(&cap).unwrap(),
            serde_json::to_string(&SCHEMA_VERSION).unwrap(),
            serde_json::to_string(&CATALOG_VERSION).unwrap(),
        ] {
            assert!(json.starts_with('"') || json.starts_with('{') || json.starts_with('['));
        }

        let cap2: CapabilityModel =
            serde_json::from_str(&serde_json::to_string(&cap).unwrap()).unwrap();
        assert_eq!(cap2.class_id, ApplianceClassId::Oven);
        assert!(cap2.advertises_trait(TraitId::Temperature));
    }

    #[test]
    fn command_and_setting_specs_roundtrip() {
        let table = class_table(ApplianceClassId::Microwave).unwrap();
        let cook = table.class_point("cook_s").unwrap();
        let setting =
            SettingSpec::from_catalog(PointNamespace::Class(ApplianceClassId::Microwave), cook);
        let json = serde_json::to_string(&setting).unwrap();
        let back: SettingSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back.spec.id, "cook_s");

        let start = trait_table(TraitId::Cycle).unwrap().point("start").unwrap();
        let cmd = CommandSpec::from_catalog(PointNamespace::Trait(TraitId::Cycle), start);
        let json = serde_json::to_string(&cmd).unwrap();
        let back: CommandSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back.spec.kind, PointKind::Command);
        assert_eq!(back.arg(), CommandArg::Void);
    }

    #[test]
    fn induction_level_zoned_write() {
        let cap = typical_capability(ApplianceClassId::InductionHob).unwrap();
        cap.validate_write("class.induction_hob.level#hob_1", &Value::U8(5))
            .unwrap();
        let err = cap
            .validate_write("class.induction_hob.level#hob_9", &Value::U8(5))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::UnknownVariable);
        let err = cap
            .validate_write("class.induction_hob.level#hob_1", &Value::U8(20))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfRange);
    }

    #[test]
    fn kettle_and_air_fryer_setpoints() {
        let kettle = typical_capability(ApplianceClassId::Kettle).unwrap();
        kettle
            .validate_write("trait.temperature.setpoint_c", &Value::F32(80.0))
            .unwrap();
        assert_eq!(
            kettle
                .validate_write("trait.temperature.setpoint_c", &Value::F32(20.0))
                .unwrap_err()
                .code,
            ErrorCode::OutOfRange
        );

        let fryer = typical_capability(ApplianceClassId::AirFryer).unwrap();
        fryer
            .validate_write("class.air_fryer.cook_s", &Value::DurationS(600))
            .unwrap();
        fryer
            .validate_write("trait.cycle.start", &Value::Void)
            .unwrap();
    }
}
