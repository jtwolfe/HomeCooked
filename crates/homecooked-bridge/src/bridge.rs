//! Small [`Bridge`] trait and HomeCooked / foreign address types.

use std::fmt;

use homecooked_schema::{catalog_point, PointNamespace, QualifiedPointId, Value};

use crate::error::Error;
use crate::modbus::RegisterKind;

/// HomeCooked device + qualified catalog point (`trait.temperature.setpoint_c`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PointRef {
    pub device_id: String,
    pub point_id: String,
}

impl PointRef {
    pub fn new(device_id: impl Into<String>, point_id: impl Into<String>) -> Result<Self, Error> {
        let device_id = device_id.into();
        let point_id = point_id.into();
        if device_id.is_empty() {
            return Err(Error::EmptyId("device_id"));
        }
        if point_id.is_empty() {
            return Err(Error::EmptyId("point_id"));
        }
        let qid = QualifiedPointId::parse(&point_id)?;
        if !matches!(qid.namespace, PointNamespace::Vendor(_))
            && catalog_point(&qid.namespace, &qid.id).is_none()
        {
            return Err(Error::UnknownCatalogPoint(point_id));
        }
        Ok(Self {
            device_id,
            point_id,
        })
    }
}

/// Fabric-specific address on a mapped device.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ForeignLocator {
    Modbus {
        kind: RegisterKind,
        address: u16,
    },
    Matter {
        endpoint: u16,
        cluster_id: u32,
        attribute_id: u32,
    },
    Zigbee {
        endpoint: u16,
        cluster_id: u32,
        attribute_id: u32,
    },
}

impl fmt::Display for ForeignLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Modbus { kind, address } => write!(f, "{kind}@{address}"),
            Self::Matter {
                endpoint,
                cluster_id,
                attribute_id,
            } => write!(
                f,
                "ep{endpoint}/cluster={cluster_id:#x}/attr={attribute_id:#x}"
            ),
            Self::Zigbee {
                endpoint,
                cluster_id,
                attribute_id,
            } => write!(
                f,
                "zb-ep{endpoint}/cluster={cluster_id:#x}/attr={attribute_id:#x}"
            ),
        }
    }
}

/// Foreign fabric address on a mapped device (Modbus register / coil, Matter /
/// Zigbee endpoint+cluster+attribute, …).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForeignRef {
    pub device_id: String,
    pub locator: ForeignLocator,
}

impl ForeignRef {
    pub fn new(device_id: impl Into<String>, locator: ForeignLocator) -> Result<Self, Error> {
        let device_id = device_id.into();
        if device_id.is_empty() {
            return Err(Error::EmptyId("device_id"));
        }
        Ok(Self { device_id, locator })
    }

    pub fn holding(device_id: impl Into<String>, address: u16) -> Result<Self, Error> {
        Self::new(
            device_id,
            ForeignLocator::Modbus {
                kind: RegisterKind::Holding,
                address,
            },
        )
    }

    pub fn coil(device_id: impl Into<String>, address: u16) -> Result<Self, Error> {
        Self::new(
            device_id,
            ForeignLocator::Modbus {
                kind: RegisterKind::Coil,
                address,
            },
        )
    }

    pub fn matter(
        device_id: impl Into<String>,
        endpoint: u16,
        cluster_id: u32,
        attribute_id: u32,
    ) -> Result<Self, Error> {
        Self::new(
            device_id,
            ForeignLocator::Matter {
                endpoint,
                cluster_id,
                attribute_id,
            },
        )
    }

    pub fn zigbee(
        device_id: impl Into<String>,
        endpoint: u16,
        cluster_id: u32,
        attribute_id: u32,
    ) -> Result<Self, Error> {
        Self::new(
            device_id,
            ForeignLocator::Zigbee {
                endpoint,
                cluster_id,
                attribute_id,
            },
        )
    }

    pub fn as_modbus(&self) -> Option<(RegisterKind, u16)> {
        match self.locator {
            ForeignLocator::Modbus { kind, address } => Some((kind, address)),
            ForeignLocator::Matter { .. } | ForeignLocator::Zigbee { .. } => None,
        }
    }

    pub fn as_matter(&self) -> Option<(u16, u32, u32)> {
        match self.locator {
            ForeignLocator::Matter {
                endpoint,
                cluster_id,
                attribute_id,
            } => Some((endpoint, cluster_id, attribute_id)),
            ForeignLocator::Modbus { .. } | ForeignLocator::Zigbee { .. } => None,
        }
    }

    pub fn as_zigbee(&self) -> Option<(u16, u32, u32)> {
        match self.locator {
            ForeignLocator::Zigbee {
                endpoint,
                cluster_id,
                attribute_id,
            } => Some((endpoint, cluster_id, attribute_id)),
            ForeignLocator::Modbus { .. } | ForeignLocator::Matter { .. } => None,
        }
    }
}

/// Untyped foreign payload before HomeCooked [`Value`] translation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForeignRaw {
    Register(u16),
    Coil(bool),
    /// Matter mock attribute payload.
    Matter(MatterRaw),
    /// Zigbee mock attribute payload.
    Zigbee(ZigbeeRaw),
}

/// Raw Matter attribute encoding used by the in-memory mock fabric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatterRaw {
    Bool(bool),
    Int16(i16),
    UInt16(u16),
}

/// Raw Zigbee attribute encoding used by the in-memory mock network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZigbeeRaw {
    Bool(bool),
    Int16(i16),
    UInt16(u16),
}

/// Maps foreign fabric reads/writes ↔ HomeCooked points.
///
/// Values on the HomeCooked side are always [`Value`]. A conforming adapter
/// translates register/coil/cluster encodings using its mapping table.
pub trait Bridge {
    /// Fabric token (`modbus`, `zigbee`, `matter`, `bacnet`).
    fn fabric(&self) -> &'static str;

    /// Read a HomeCooked point by translating from the foreign fabric.
    fn read_point(&self, point: &PointRef) -> Result<Value, Error>;

    /// Write a HomeCooked point by translating onto the foreign fabric.
    fn write_point(&mut self, point: &PointRef, value: &Value) -> Result<(), Error>;

    /// Read a foreign address and return the translated HomeCooked value.
    fn read_foreign(&self, foreign: &ForeignRef) -> Result<Value, Error>;

    /// Apply a foreign-side write, update the HomeCooked backend, and return
    /// the translated value.
    fn write_foreign(&mut self, foreign: &ForeignRef, raw: ForeignRaw) -> Result<Value, Error>;
}
