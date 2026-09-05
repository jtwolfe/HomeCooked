//! Read/write checks against an advertised [`CapabilityModel`].

use homecooked_schema::{
    catalog_point, CapabilityModel, ErrorCode, PointCapability, PointNamespace, QualifiedPointId,
    ValidationError, ValueType,
};

/// Resolve an advertised point, including `#zone` suffixes.
pub fn lookup_point<'a>(
    cap: &'a CapabilityModel,
    point_id: &str,
) -> Result<&'a PointCapability, ValidationError> {
    let parsed = match QualifiedPointId::parse(point_id) {
        Ok(p) => p,
        Err(_) => return Err(ValidationError::unknown_variable(point_id)),
    };

    if parsed.zone.is_some() {
        if let Some(point) = cap
            .point(point_id)
            .or_else(|| cap.point(&parsed.base_string()))
        {
            if let Some(zones) = &point.zones {
                let zone = parsed.zone.as_deref().unwrap();
                if !zones.iter().any(|z| z == zone) {
                    return Err(ValidationError::unknown_variable(point_id));
                }
            }
            return Ok(point);
        }
    } else if let Some(point) = cap.point(point_id) {
        return Ok(point);
    }

    Err(unadvertised_error(cap, &parsed, point_id))
}

fn unadvertised_error(
    cap: &CapabilityModel,
    parsed: &QualifiedPointId,
    point_id: &str,
) -> ValidationError {
    match &parsed.namespace {
        PointNamespace::Trait(trait_id) => {
            if !cap.advertises_trait(*trait_id) {
                return ValidationError::unsupported_capability(point_id);
            }
            if catalog_point(&parsed.namespace, &parsed.id).is_some() {
                ValidationError::unsupported_capability(point_id)
            } else {
                ValidationError::unknown_variable(point_id)
            }
        }
        PointNamespace::Class(class_id) => {
            if !cap.advertises_class(*class_id) {
                return ValidationError::unsupported_capability(point_id);
            }
            if catalog_point(&parsed.namespace, &parsed.id).is_some() {
                ValidationError::unsupported_capability(point_id)
            } else {
                ValidationError::unknown_variable(point_id)
            }
        }
        PointNamespace::Vendor(_) => ValidationError::unknown_variable(point_id),
    }
}

/// Validate a read against advertised access and presence.
pub fn validate_read(cap: &CapabilityModel, point_id: &str) -> Result<(), ValidationError> {
    let point = lookup_point(cap, point_id)?;
    if !point.access.is_readable() {
        return Err(ValidationError::new(
            ErrorCode::NotReadable,
            format!("point {point_id} is not readable"),
        )
        .at_point(point_id));
    }
    Ok(())
}

/// True when the advertised point is a command (action, not stored state).
pub fn is_command_point(cap: &CapabilityModel, point_id: &str) -> bool {
    lookup_point(cap, point_id)
        .map(|p| p.value_type == ValueType::Command)
        .unwrap_or(false)
}
