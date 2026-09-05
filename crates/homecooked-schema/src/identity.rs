//! Device identity matching trait `identity`.

use serde::{Deserialize, Serialize};

use crate::error::{ErrorCode, ValidationError};
use crate::ids::ApplianceClassId;
use crate::version::{CatalogVersion, SemVer, CATALOG_VERSION};

/// Fields from trait `identity` (who the device is).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub device_id: String,
    pub manufacturer: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hw_version: Option<String>,
    pub fw_version: String,
    pub class_id: ApplianceClassId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary_class_ids: Vec<ApplianceClassId>,
    pub catalog_version: CatalogVersion,
    pub protocol_version: SemVer,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room: Option<String>,
}

impl DeviceIdentity {
    pub fn new(
        device_id: impl Into<String>,
        manufacturer: impl Into<String>,
        model: impl Into<String>,
        fw_version: impl Into<String>,
        class_id: ApplianceClassId,
    ) -> Self {
        Self {
            device_id: device_id.into(),
            manufacturer: manufacturer.into(),
            model: model.into(),
            serial: None,
            hw_version: None,
            fw_version: fw_version.into(),
            class_id,
            secondary_class_ids: Vec::new(),
            catalog_version: CATALOG_VERSION,
            protocol_version: SemVer::V0_1_0,
            display_name: None,
            room: None,
        }
    }

    pub fn is_valid_device_id(s: &str) -> bool {
        let len = s.len();
        (1..=128).contains(&len)
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '-'))
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        fn len_ok(s: &str, min: usize, max: usize, field: &str) -> Result<(), ValidationError> {
            if s.len() < min || s.len() > max {
                return Err(ValidationError::new(
                    ErrorCode::OutOfRange,
                    format!("{field} length {} not in {min}–{max}", s.len()),
                ));
            }
            Ok(())
        }

        if !Self::is_valid_device_id(&self.device_id) {
            return Err(ValidationError::new(
                ErrorCode::InvalidRequest,
                "device_id must match [a-zA-Z0-9._:-]+ (1–128 chars)",
            ));
        }
        len_ok(&self.manufacturer, 1, 64, "manufacturer")?;
        len_ok(&self.model, 1, 64, "model")?;
        len_ok(&self.fw_version, 1, 32, "fw_version")?;
        if let Some(s) = &self.serial {
            len_ok(s, 0, 64, "serial")?;
        }
        if let Some(s) = &self.hw_version {
            len_ok(s, 0, 32, "hw_version")?;
        }
        if let Some(s) = &self.display_name {
            len_ok(s, 0, 64, "display_name")?;
        }
        if let Some(s) = &self.room {
            len_ok(s, 0, 64, "room")?;
        }
        if self.secondary_class_ids.len() > 8 {
            return Err(ValidationError::new(
                ErrorCode::OutOfRange,
                "secondary_class_ids max 8",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_roundtrip() {
        let id = DeviceIdentity::new(
            "washer-1",
            "Acme",
            "W100",
            "1.2.3",
            ApplianceClassId::Washer,
        );
        id.validate().unwrap();
        let json = serde_json::to_string(&id).unwrap();
        let back: DeviceIdentity = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
        assert_eq!(back.catalog_version, CATALOG_VERSION);
    }

    #[test]
    fn device_id_charset() {
        assert!(DeviceIdentity::is_valid_device_id("abc.DEF_01:x-y"));
        assert!(!DeviceIdentity::is_valid_device_id(""));
        assert!(!DeviceIdentity::is_valid_device_id("has space"));
    }
}
