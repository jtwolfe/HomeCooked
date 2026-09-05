//! Bridge load, map, and translation errors.

use std::fmt;

use crate::bridge::ForeignLocator;
use crate::modbus::RegisterKind;

/// Failure while loading a map, translating a value, or talking to a backend.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Yaml(String),
    Json(String),
    Io(String),
    EmptyId(&'static str),
    InvalidMap(String),
    UnknownCatalogPoint(String),
    DuplicatePoint(String),
    DuplicateAddress {
        kind: RegisterKind,
        address: u16,
    },
    DuplicateMatterAttribute {
        endpoint: u16,
        cluster_id: u32,
        attribute_id: u32,
    },
    DuplicateZigbeeAttribute {
        endpoint: u16,
        cluster_id: u32,
        attribute_id: u32,
    },
    DuplicateBacnetProperty {
        object_type: String,
        object_instance: u32,
        property: String,
    },
    UnmappedPoint {
        device_id: String,
        point_id: String,
    },
    UnmappedAddress {
        kind: RegisterKind,
        address: u16,
    },
    UnmappedMatterAttribute {
        endpoint: u16,
        cluster_id: u32,
        attribute_id: u32,
    },
    UnmappedZigbeeAttribute {
        endpoint: u16,
        cluster_id: u32,
        attribute_id: u32,
    },
    UnmappedBacnetProperty {
        object_type: String,
        object_instance: u32,
        property: String,
    },
    DeviceMismatch {
        expected: String,
        actual: String,
    },
    NotWritable {
        point_id: String,
    },
    TypeMismatch {
        point_id: String,
        expected: String,
        detail: String,
    },
    ScaleOverflow {
        point_id: String,
        detail: String,
    },
    InvalidScale {
        point_id: String,
    },
    InvalidRaw {
        detail: String,
    },
    /// Foreign locator fabric does not match this bridge.
    LocatorMismatch {
        expected: &'static str,
        locator: ForeignLocator,
    },
    UnsupportedFabric {
        fabric: &'static str,
    },
    Backend(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yaml(msg) => write!(f, "yaml: {msg}"),
            Self::Json(msg) => write!(f, "json: {msg}"),
            Self::Io(msg) => write!(f, "io: {msg}"),
            Self::EmptyId(field) => write!(f, "{field} must be non-empty"),
            Self::InvalidMap(msg) => write!(f, "invalid map: {msg}"),
            Self::UnknownCatalogPoint(point) => {
                write!(f, "unknown catalog point {point}")
            }
            Self::DuplicatePoint(point) => write!(f, "duplicate point {point}"),
            Self::DuplicateAddress { kind, address } => {
                write!(f, "duplicate {kind} address {address}")
            }
            Self::DuplicateMatterAttribute {
                endpoint,
                cluster_id,
                attribute_id,
            } => write!(
                f,
                "duplicate Matter attribute ep{endpoint}/cluster={cluster_id:#x}/attr={attribute_id:#x}"
            ),
            Self::DuplicateZigbeeAttribute {
                endpoint,
                cluster_id,
                attribute_id,
            } => write!(
                f,
                "duplicate Zigbee attribute ep{endpoint}/cluster={cluster_id:#x}/attr={attribute_id:#x}"
            ),
            Self::DuplicateBacnetProperty {
                object_type,
                object_instance,
                property,
            } => write!(
                f,
                "duplicate BACnet property {object_type}/{object_instance}/{property}"
            ),
            Self::UnmappedPoint {
                device_id,
                point_id,
            } => write!(f, "unmapped point {device_id}/{point_id}"),
            Self::UnmappedAddress { kind, address } => {
                write!(f, "unmapped {kind} address {address}")
            }
            Self::UnmappedMatterAttribute {
                endpoint,
                cluster_id,
                attribute_id,
            } => write!(
                f,
                "unmapped Matter attribute ep{endpoint}/cluster={cluster_id:#x}/attr={attribute_id:#x}"
            ),
            Self::UnmappedZigbeeAttribute {
                endpoint,
                cluster_id,
                attribute_id,
            } => write!(
                f,
                "unmapped Zigbee attribute ep{endpoint}/cluster={cluster_id:#x}/attr={attribute_id:#x}"
            ),
            Self::UnmappedBacnetProperty {
                object_type,
                object_instance,
                property,
            } => write!(
                f,
                "unmapped BACnet property {object_type}/{object_instance}/{property}"
            ),
            Self::DeviceMismatch { expected, actual } => {
                write!(f, "device mismatch: expected {expected}, got {actual}")
            }
            Self::NotWritable { point_id } => write!(f, "point {point_id} is not writable"),
            Self::TypeMismatch {
                point_id,
                expected,
                detail,
            } => write!(
                f,
                "type mismatch on {point_id} (expected {expected}): {detail}"
            ),
            Self::ScaleOverflow { point_id, detail } => {
                write!(f, "scale overflow on {point_id}: {detail}")
            }
            Self::InvalidScale { point_id } => {
                write!(f, "scale must be finite and non-zero for {point_id}")
            }
            Self::InvalidRaw { detail } => write!(f, "invalid foreign raw: {detail}"),
            Self::LocatorMismatch { expected, locator } => {
                write!(f, "locator {locator} is not valid for {expected} bridge")
            }
            Self::UnsupportedFabric { fabric } => write!(
                f,
                "{fabric} bridge is not implemented; see docs/standard/bridges.md"
            ),
            Self::Backend(msg) => write!(f, "backend: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Yaml(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<homecooked_schema::ParseIdError> for Error {
    fn from(err: homecooked_schema::ParseIdError) -> Self {
        Self::UnknownCatalogPoint(err.value)
    }
}
