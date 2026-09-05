//! Small [`Bridge`] trait and HomeCooked / foreign address types.

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

/// Foreign fabric address on a mapped device (Modbus register / coil, …).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForeignRef {
    pub device_id: String,
    pub kind: RegisterKind,
    pub address: u16,
}

impl ForeignRef {
    pub fn new(
        device_id: impl Into<String>,
        kind: RegisterKind,
        address: u16,
    ) -> Result<Self, Error> {
        let device_id = device_id.into();
        if device_id.is_empty() {
            return Err(Error::EmptyId("device_id"));
        }
        Ok(Self {
            device_id,
            kind,
            address,
        })
    }

    pub fn holding(device_id: impl Into<String>, address: u16) -> Result<Self, Error> {
        Self::new(device_id, RegisterKind::Holding, address)
    }

    pub fn coil(device_id: impl Into<String>, address: u16) -> Result<Self, Error> {
        Self::new(device_id, RegisterKind::Coil, address)
    }
}

/// Untyped foreign payload before HomeCooked [`Value`] translation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForeignRaw {
    Register(u16),
    Coil(bool),
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
