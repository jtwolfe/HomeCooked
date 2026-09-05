//! Point specifications: catalog rows and owned serde types.

use serde::{Deserialize, Serialize};

use crate::access::AccessMode;
use crate::ids::{ApplianceClassId, PointNamespace, QualifiedPointId, TraitId};
use crate::types::{CommandArg, Unit, ValueRange, ValueType};

/// Role of a catalog point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PointKind {
    Variable,
    Setting,
    Command,
}

/// Static range stored in catalog tables (const-friendly).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CatalogRange {
    Numeric {
        min: f64,
        max: f64,
    },
    Integer {
        min: i64,
        max: i64,
    },
    Enum(&'static [&'static str]),
    String {
        min_chars: u32,
        max_chars: u32,
    },
    List {
        max_len: u32,
    },
    CommandVoid,
    CommandTyped {
        value_type: ValueType,
        min: Option<f64>,
        max: Option<f64>,
        optional: bool,
    },
}

impl CatalogRange {
    pub fn to_value_range(self) -> ValueRange {
        match self {
            Self::Numeric { min, max } => ValueRange::Numeric { min, max },
            Self::Integer { min, max } => ValueRange::Integer { min, max },
            Self::Enum(tokens) => ValueRange::Enum {
                tokens: tokens.iter().map(|s| (*s).to_string()).collect(),
            },
            Self::String {
                min_chars,
                max_chars,
            } => ValueRange::String {
                min_chars,
                max_chars,
            },
            Self::List { max_len } => ValueRange::List {
                max_len,
                item: None,
            },
            Self::CommandVoid => ValueRange::CommandArg {
                arg: CommandArg::Void,
            },
            Self::CommandTyped {
                value_type,
                min,
                max,
                optional,
            } => {
                let inner = match (min, max) {
                    (Some(a), Some(b)) if value_type.is_numeric() => {
                        if matches!(value_type, ValueType::F32 | ValueType::Percent) {
                            Some(Box::new(ValueRange::Numeric { min: a, max: b }))
                        } else {
                            Some(Box::new(ValueRange::Integer {
                                min: a as i64,
                                max: b as i64,
                            }))
                        }
                    }
                    _ => None,
                };
                ValueRange::CommandArg {
                    arg: CommandArg::Typed {
                        value_type,
                        range: inner,
                        optional,
                    },
                }
            }
        }
    }
}

/// One row in a static trait or class table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CatalogPoint {
    pub id: &'static str,
    pub value_type: ValueType,
    pub unit: Option<Unit>,
    pub range: Option<CatalogRange>,
    pub access: AccessMode,
    pub required: bool,
    pub zoned: bool,
    pub kind: PointKind,
}

impl CatalogPoint {
    pub const fn variable(
        id: &'static str,
        value_type: ValueType,
        unit: Option<Unit>,
        range: Option<CatalogRange>,
        access: AccessMode,
        required: bool,
    ) -> Self {
        Self {
            id,
            value_type,
            unit,
            range,
            access,
            required,
            zoned: false,
            kind: PointKind::Variable,
        }
    }

    pub const fn setting(
        id: &'static str,
        value_type: ValueType,
        unit: Option<Unit>,
        range: Option<CatalogRange>,
        access: AccessMode,
        required: bool,
    ) -> Self {
        Self {
            id,
            value_type,
            unit,
            range,
            access,
            required,
            zoned: false,
            kind: PointKind::Setting,
        }
    }

    pub const fn command(id: &'static str, range: Option<CatalogRange>, required: bool) -> Self {
        Self {
            id,
            value_type: ValueType::Command,
            unit: None,
            range,
            access: AccessMode::W,
            required,
            zoned: false,
            kind: PointKind::Command,
        }
    }

    pub const fn zoned(mut self) -> Self {
        self.zoned = true;
        self
    }

    pub fn to_point_spec(&self, namespace: PointNamespace) -> PointSpec {
        let qualified = QualifiedPointId::new(namespace, self.id);
        PointSpec {
            id: self.id.to_string(),
            qualified_id: qualified.to_string(),
            kind: self.kind,
            value_type: self.value_type,
            unit: self.unit,
            range: self.range.map(CatalogRange::to_value_range),
            access: self.access,
            required: self.required,
            zoned: self.zoned,
        }
    }
}

/// Owned, serializable specification of a catalog or advertised point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointSpec {
    pub id: String,
    pub qualified_id: String,
    pub kind: PointKind,
    pub value_type: ValueType,
    pub unit: Option<Unit>,
    pub range: Option<ValueRange>,
    pub access: AccessMode,
    pub required: bool,
    pub zoned: bool,
}

impl PointSpec {
    pub fn trait_point(trait_id: TraitId, catalog: &CatalogPoint) -> Self {
        catalog.to_point_spec(PointNamespace::Trait(trait_id))
    }

    pub fn class_point(class_id: ApplianceClassId, catalog: &CatalogPoint) -> Self {
        catalog.to_point_spec(PointNamespace::Class(class_id))
    }

    pub fn as_variable(&self) -> Option<VariableSpec> {
        (self.kind == PointKind::Variable).then(|| VariableSpec { spec: self.clone() })
    }

    pub fn as_setting(&self) -> Option<SettingSpec> {
        (self.kind == PointKind::Setting).then(|| SettingSpec { spec: self.clone() })
    }

    pub fn as_command(&self) -> Option<CommandSpec> {
        (self.kind == PointKind::Command).then(|| CommandSpec { spec: self.clone() })
    }
}

/// Telemetry point (`r`, usually `e`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VariableSpec {
    pub spec: PointSpec,
}

impl VariableSpec {
    pub fn from_catalog(namespace: PointNamespace, point: &CatalogPoint) -> Self {
        Self {
            spec: point.to_point_spec(namespace),
        }
    }
}

/// Writable setting (`w`, usually also `r`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SettingSpec {
    pub spec: PointSpec,
}

impl SettingSpec {
    pub fn from_catalog(namespace: PointNamespace, point: &CatalogPoint) -> Self {
        Self {
            spec: point.to_point_spec(namespace),
        }
    }
}

/// Write-only action. Success means the action was accepted, not finished.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommandSpec {
    pub spec: PointSpec,
}

impl CommandSpec {
    pub fn from_catalog(namespace: PointNamespace, point: &CatalogPoint) -> Self {
        Self {
            spec: point.to_point_spec(namespace),
        }
    }

    pub fn arg(&self) -> CommandArg {
        match &self.spec.range {
            Some(ValueRange::CommandArg { arg }) => arg.clone(),
            _ => CommandArg::Void,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::TraitId;

    #[test]
    fn point_spec_roundtrip() {
        let cat = CatalogPoint::setting(
            "setpoint_c",
            ValueType::F32,
            Some(Unit::Celsius),
            Some(CatalogRange::Numeric { min: 1.0, max: 7.0 }),
            AccessMode::RWE,
            false,
        )
        .zoned();
        let spec = PointSpec::trait_point(TraitId::Temperature, &cat);
        assert_eq!(spec.qualified_id, "trait.temperature.setpoint_c");
        let json = serde_json::to_string(&spec).unwrap();
        let back: PointSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back, spec);
        assert!(spec.as_setting().is_some());
        assert!(spec.as_variable().is_none());
    }
}
