//! Capability and range checks for writes.

use crate::capability::{CapabilityModel, PointCapability};
use crate::catalog::{class_table, trait_table};
use crate::error::{ErrorCode, ValidationError};
use crate::ids::{PointNamespace, QualifiedPointId};
use crate::spec::PointSpec;
use crate::types::{CommandArg, Value, ValueRange, ValueType};

impl CapabilityModel {
    /// Validate a write against advertised capabilities.
    ///
    /// Order matches the catalog: unknown → unsupported → not_writable →
    /// invalid_type / invalid_enum / out_of_range.
    #[must_use = "validation result should be checked"]
    pub fn validate_write(&self, point_id: &str, value: &Value) -> Result<(), ValidationError> {
        let parsed = match QualifiedPointId::parse(point_id) {
            Ok(p) => p,
            Err(_) => return Err(ValidationError::unknown_variable(point_id)),
        };

        if parsed.zone.is_some() {
            if let Some(cap) = self
                .point(point_id)
                .or_else(|| self.point(&parsed.base_string()))
            {
                if let Some(zones) = &cap.zones {
                    let zone = parsed.zone.as_deref().unwrap();
                    if !zones.iter().any(|z| z == zone) {
                        return Err(ValidationError::unknown_variable(point_id));
                    }
                }
                return validate_against_point(point_id, cap, value);
            }
        } else if let Some(cap) = self.point(point_id) {
            return validate_against_point(point_id, cap, value);
        }

        Err(self.unadvertised_error(&parsed, point_id))
    }

    fn unadvertised_error(&self, parsed: &QualifiedPointId, point_id: &str) -> ValidationError {
        match &parsed.namespace {
            PointNamespace::Trait(trait_id) => {
                if !self.advertises_trait(*trait_id) {
                    return ValidationError::unsupported_capability(point_id);
                }
                if trait_table(*trait_id)
                    .is_some_and(|t| t.points.iter().any(|p| p.id == parsed.id))
                {
                    ValidationError::unsupported_capability(point_id)
                } else {
                    ValidationError::unknown_variable(point_id)
                }
            }
            PointNamespace::Class(class_id) => {
                if !self.advertises_class(*class_id) {
                    return ValidationError::unsupported_capability(point_id);
                }
                if class_table(*class_id)
                    .is_some_and(|t| t.class_points.iter().any(|p| p.id == parsed.id))
                {
                    ValidationError::unsupported_capability(point_id)
                } else {
                    ValidationError::unknown_variable(point_id)
                }
            }
            PointNamespace::Vendor(_) => ValidationError::unknown_variable(point_id),
        }
    }
}

/// Validate a write against a [`PointSpec`] (catalog typical or advertised).
pub fn validate_against_spec(spec: &PointSpec, value: &Value) -> Result<(), ValidationError> {
    let cap = PointCapability::from_spec(spec);
    validate_against_point(&spec.qualified_id, &cap, value)
}

fn validate_against_point(
    point_id: &str,
    cap: &PointCapability,
    value: &Value,
) -> Result<(), ValidationError> {
    if !cap.access.is_writable() {
        return Err(ValidationError::not_writable(point_id));
    }
    if cap.value_type == ValueType::Command {
        return validate_command(point_id, cap, value);
    }
    if !value.matches_type(cap.value_type) {
        return Err(ValidationError::invalid_type(
            point_id,
            cap.value_type.as_str(),
        ));
    }
    validate_range(point_id, cap.value_type, cap.range.as_ref(), value)
}

fn validate_command(
    point_id: &str,
    cap: &PointCapability,
    value: &Value,
) -> Result<(), ValidationError> {
    let arg = match &cap.range {
        Some(ValueRange::CommandArg { arg }) => arg,
        _ => &CommandArg::Void,
    };
    match arg {
        CommandArg::Void => {
            if matches!(value, Value::Void) {
                Ok(())
            } else {
                Err(ValidationError::invalid_type(point_id, "void"))
            }
        }
        CommandArg::Typed {
            value_type,
            range,
            optional,
        } => {
            if matches!(value, Value::Void) {
                if *optional {
                    return Ok(());
                }
                return Err(ValidationError::invalid_type(point_id, value_type.as_str()));
            }
            if !value.matches_type(*value_type) {
                return Err(ValidationError::invalid_type(point_id, value_type.as_str()));
            }
            validate_range(point_id, *value_type, range.as_deref(), value)
        }
    }
}

fn validate_range(
    point_id: &str,
    value_type: ValueType,
    range: Option<&ValueRange>,
    value: &Value,
) -> Result<(), ValidationError> {
    if value_type == ValueType::Percent {
        if let Some(v) = value.as_f64() {
            let (min, max) = match range {
                Some(ValueRange::Numeric { min, max }) => (*min, *max),
                Some(ValueRange::Integer { min, max }) => (*min as f64, *max as f64),
                _ => (0.0, 100.0),
            };
            if v < min || v > max {
                return Err(ValidationError::out_of_range(
                    point_id,
                    format!("{min}–{max}"),
                ));
            }
        }
        return Ok(());
    }

    let Some(range) = range else {
        return Ok(());
    };

    match (range, value) {
        (ValueRange::Enum { tokens }, Value::Enum(token)) => {
            if !tokens.iter().any(|t| t == token) {
                return Err(
                    ValidationError::invalid_enum(point_id, token).expected(tokens.join("|"))
                );
            }
        }
        (ValueRange::Numeric { min, max }, _) => {
            if let Some(v) = value.as_f64() {
                if v < *min || v > *max {
                    return Err(ValidationError::out_of_range(
                        point_id,
                        format!("{min}–{max}"),
                    ));
                }
            }
        }
        (ValueRange::Integer { min, max }, _) => {
            if let Some(v) = value.as_i64() {
                if v < *min || v > *max {
                    return Err(ValidationError::out_of_range(
                        point_id,
                        format!("{min}–{max}"),
                    ));
                }
            } else if let Some(v) = value.as_f64() {
                if !range.contains_f64(v) {
                    return Err(ValidationError::out_of_range(
                        point_id,
                        format!("{min}–{max}"),
                    ));
                }
            }
        }
        (
            ValueRange::String {
                min_chars,
                max_chars,
            },
            Value::String(s),
        ) => {
            let n = s.chars().count() as u32;
            if n < *min_chars || n > *max_chars {
                return Err(ValidationError::out_of_range(
                    point_id,
                    format!("{min_chars}–{max_chars} chars"),
                ));
            }
        }
        (ValueRange::List { max_len, item }, Value::List(values)) => {
            if values.len() as u32 > *max_len {
                return Err(ValidationError::out_of_range(
                    point_id,
                    format!("max {max_len} items"),
                ));
            }
            if let Some(item_range) = item {
                for v in values {
                    validate_range(
                        point_id,
                        v.value_type().unwrap_or(value_type),
                        Some(item_range),
                        v,
                    )?;
                }
            } else if let ValueType::List(inner) = value_type {
                if inner == crate::types::ListItemType::Enum {
                    for v in values {
                        validate_range(point_id, ValueType::Enum, Some(range), v)?;
                    }
                }
            }
        }
        (ValueRange::Enum { tokens }, Value::List(values)) => {
            for v in values {
                if let Value::Enum(token) = v {
                    if !tokens.iter().any(|t| t == token) {
                        return Err(ValidationError::invalid_enum(point_id, token)
                            .expected(tokens.join("|")));
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// True when `code` is one of the write-validation tokens used by this crate.
pub fn is_write_validation_code(code: ErrorCode) -> bool {
    matches!(
        code,
        ErrorCode::UnknownVariable
            | ErrorCode::UnsupportedCapability
            | ErrorCode::NotWritable
            | ErrorCode::InvalidType
            | ErrorCode::InvalidEnum
            | ErrorCode::OutOfRange
    )
}
