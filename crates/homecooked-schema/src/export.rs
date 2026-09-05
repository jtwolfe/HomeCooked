//! Machine-readable catalog dump for tooling (not a full OpenAPI server).
//!
//! Serialize with `serde_json` via [`export_catalog_json`] / [`CatalogExport`].

use serde::{Deserialize, Serialize};

use crate::capability::CapabilityModel;
use crate::catalog::{catalog_group, list_all_class_ids, typical_capability};
use crate::ids::ApplianceClassId;
use crate::version::{CatalogVersion, SchemaVersion, CATALOG_VERSION, SCHEMA_VERSION};

/// Stable document type id for tooling consumers.
pub const CATALOG_EXPORT_FORMAT: &str = "homecooked.catalog_export";

/// Format version of this JSON document shape (independent of catalog version).
pub const CATALOG_EXPORT_FORMAT_VERSION: &str = "0.1.0";

/// Top-level catalog export: every class id plus its typical capability points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogExport {
    /// Document type (`homecooked.catalog_export`).
    pub format: String,
    /// Shape version of this export document.
    pub format_version: String,
    pub schema_version: SchemaVersion,
    pub catalog_version: CatalogVersion,
    pub class_count: usize,
    pub classes: Vec<CatalogClassExport>,
}

/// One appliance class and its typical advertised points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogClassExport {
    pub class_id: ApplianceClassId,
    /// Index group from `docs/catalog/appliances.md` (e.g. `Laundry`).
    pub group: String,
    /// Typical capability (traits + required class points) for tooling.
    pub typical: CapabilityModel,
}

/// Build the full catalog export from static tables (all 56 classes).
pub fn catalog_export() -> CatalogExport {
    let classes: Vec<CatalogClassExport> = list_all_class_ids()
        .iter()
        .copied()
        .map(|class_id| {
            let typical = typical_capability(class_id).unwrap_or_else(|| {
                panic!("missing typical capability for class {}", class_id.as_str())
            });
            CatalogClassExport {
                class_id,
                group: catalog_group(class_id).to_string(),
                typical,
            }
        })
        .collect();

    CatalogExport {
        format: CATALOG_EXPORT_FORMAT.to_string(),
        format_version: CATALOG_EXPORT_FORMAT_VERSION.to_string(),
        schema_version: SCHEMA_VERSION,
        catalog_version: CATALOG_VERSION,
        class_count: classes.len(),
        classes,
    }
}

/// Pretty-printed JSON for the catalog export (stdout / files / CI).
pub fn export_catalog_json() -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&catalog_export())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn export_is_valid_json_with_all_56_classes() {
        let json = export_catalog_json().expect("serialize catalog export");
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse export JSON");

        assert_eq!(
            value["format"].as_str(),
            Some(CATALOG_EXPORT_FORMAT),
            "format marker"
        );
        assert_eq!(value["class_count"].as_u64(), Some(56));

        let classes = value["classes"].as_array().expect("classes array");
        assert_eq!(classes.len(), 56, "export must include all 56 classes");

        let mut ids: BTreeSet<String> = BTreeSet::new();
        for entry in classes {
            let id = entry["class_id"]
                .as_str()
                .expect("class_id string")
                .to_string();
            assert!(
                entry["typical"].is_object(),
                "class {id} must include typical capability object"
            );
            assert!(
                entry["typical"]["class_points"].is_array()
                    || entry["typical"]["traits"].is_array(),
                "class {id} typical must have traits or class_points"
            );
            assert!(
                entry["group"].as_str().is_some_and(|g| !g.is_empty()),
                "class {id} must have a catalog group"
            );
            assert!(ids.insert(id.clone()), "duplicate class_id {id}");
        }

        let expected: BTreeSet<_> = list_all_class_ids()
            .iter()
            .map(|c| c.as_str().to_string())
            .collect();
        assert_eq!(ids, expected);

        let roundtrip: CatalogExport = serde_json::from_str(&json).expect("deserialize export");
        assert_eq!(roundtrip.class_count, 56);
        assert_eq!(roundtrip.classes.len(), 56);
        assert_eq!(roundtrip.format, CATALOG_EXPORT_FORMAT);
    }
}
