//! Matter cluster → HomeCooked point adapter (stub).
//!
//! Not implemented in this slice. See
//! [`docs/standard/bridges.md`](../../../docs/standard/bridges.md).
//! Stream 6 chose Modbus for the first real adapter so CI stays free of
//! Matter SDKs.

use homecooked_schema::Value;

use crate::bridge::{Bridge, ForeignRaw, ForeignRef, PointRef};
use crate::error::Error;

/// Placeholder Matter fabric adapter.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MatterBridge;

impl MatterBridge {
    pub fn new() -> Self {
        Self
    }
}

impl Bridge for MatterBridge {
    fn fabric(&self) -> &'static str {
        "matter"
    }

    fn read_point(&self, _point: &PointRef) -> Result<Value, Error> {
        Err(Error::UnsupportedFabric { fabric: "matter" })
    }

    fn write_point(&mut self, _point: &PointRef, _value: &Value) -> Result<(), Error> {
        Err(Error::UnsupportedFabric { fabric: "matter" })
    }

    fn read_foreign(&self, _foreign: &ForeignRef) -> Result<Value, Error> {
        Err(Error::UnsupportedFabric { fabric: "matter" })
    }

    fn write_foreign(&mut self, _foreign: &ForeignRef, _raw: ForeignRaw) -> Result<Value, Error> {
        Err(Error::UnsupportedFabric { fabric: "matter" })
    }
}
