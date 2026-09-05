//! Advertised capability object (overview §4.3).

use serde::{Deserialize, Serialize};

use crate::access::AccessMode;
use crate::ids::{ApplianceClassId, TraitId};
use crate::spec::{CatalogPoint, PointSpec};
use crate::types::{Unit, ValueRange, ValueType};
use crate::version::{CatalogVersion, SemVer, CATALOG_VERSION, DEFAULT_CLASS_VERSION};

/// Device-advertised intersection of the catalog and this firmware.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityModel {
    pub class_id: ApplianceClassId,
    pub class_version: SemVer,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary_class_ids: Vec<ApplianceClassId>,
    pub traits: Vec<TraitCapability>,
    /// Class-namespaced points (`class.<class_id>.*`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub class_points: Vec<PointCapability>,
    #[serde(default)]
    pub safety: SafetyFlags,
    pub catalog_version: CatalogVersion,
}

/// One advertised trait and its points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitCapability {
    pub trait_id: TraitId,
    pub trait_version: SemVer,
    pub points: Vec<PointCapability>,
}

/// Advertised point. `range` may be tighter than the catalog typical range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointCapability {
    /// Qualified id; may include `#zone`. Unzoned id + `zones` is preferred.
    pub id: String,
    #[serde(rename = "type")]
    pub value_type: ValueType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<Unit>,
    pub access: AccessMode,
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<ValueRange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zones: Option<Vec<String>>,
}

/// Default-deny remote actuator flags.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SafetyFlags {
    #[serde(default)]
    pub remote_start_supported: bool,
    #[serde(default)]
    pub gas_remote_ignite: bool,
    #[serde(default)]
    pub rf_remote_start: bool,
    #[serde(default)]
    pub remote_vent: bool,
}

impl PointCapability {
    pub fn from_spec(spec: &PointSpec) -> Self {
        Self {
            id: spec.qualified_id.clone(),
            value_type: spec.value_type,
            unit: spec.unit,
            access: spec.access,
            required: spec.required,
            range: spec.range.clone(),
            resolution: None,
            zones: None,
        }
    }

    pub fn from_catalog(qualified_id: impl Into<String>, point: &CatalogPoint) -> Self {
        Self {
            id: qualified_id.into(),
            value_type: point.value_type,
            unit: point.unit,
            access: point.access,
            required: point.required,
            range: point.range.map(|r| r.to_value_range()),
            resolution: None,
            zones: None,
        }
    }

    pub fn with_range(mut self, range: ValueRange) -> Self {
        self.range = Some(range);
        self
    }

    pub fn with_zones<I, S>(mut self, zones: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.zones = Some(zones.into_iter().map(Into::into).collect());
        self
    }

    pub fn base_id(&self) -> &str {
        self.id.split('#').next().unwrap_or(&self.id)
    }
}

impl CapabilityModel {
    pub fn new(class_id: ApplianceClassId) -> Self {
        Self {
            class_id,
            class_version: DEFAULT_CLASS_VERSION,
            secondary_class_ids: Vec::new(),
            traits: Vec::new(),
            class_points: Vec::new(),
            safety: SafetyFlags::default(),
            catalog_version: CATALOG_VERSION,
        }
    }

    pub fn trait_cap(&self, trait_id: TraitId) -> Option<&TraitCapability> {
        self.traits.iter().find(|t| t.trait_id == trait_id)
    }

    pub fn advertises_trait(&self, trait_id: TraitId) -> bool {
        self.trait_cap(trait_id).is_some()
    }

    pub fn advertises_class(&self, class_id: ApplianceClassId) -> bool {
        self.class_id == class_id || self.secondary_class_ids.contains(&class_id)
    }

    pub fn iter_points(&self) -> impl Iterator<Item = &PointCapability> {
        self.traits
            .iter()
            .flat_map(|t| t.points.iter())
            .chain(self.class_points.iter())
    }

    pub fn point(&self, qualified_id: &str) -> Option<&PointCapability> {
        let base = qualified_id.split('#').next().unwrap_or(qualified_id);
        self.iter_points()
            .find(|p| p.id == qualified_id || p.base_id() == base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::ApplianceClassId;

    #[test]
    fn capability_roundtrip() {
        let cap = CapabilityModel::new(ApplianceClassId::Washer);
        let json = serde_json::to_string(&cap).unwrap();
        let back: CapabilityModel = serde_json::from_str(&json).unwrap();
        assert_eq!(back.class_id, ApplianceClassId::Washer);
        assert_eq!(back.catalog_version, CATALOG_VERSION);
        assert!(!back.safety.remote_start_supported);
    }
}
