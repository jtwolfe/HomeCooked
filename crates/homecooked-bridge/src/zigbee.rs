//! Zigbee cluster → HomeCooked point adapter (stub).
//!
//! Not implemented in this slice. See
//! [`docs/standard/bridges.md`](../../../docs/standard/bridges.md).

use homecooked_schema::Value;

use crate::bridge::{Bridge, ForeignRaw, ForeignRef, PointRef};
use crate::error::Error;

/// Placeholder Zigbee fabric adapter.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ZigbeeBridge;

impl ZigbeeBridge {
    pub fn new() -> Self {
        Self
    }
}

impl Bridge for ZigbeeBridge {
    fn fabric(&self) -> &'static str {
        "zigbee"
    }

    fn read_point(&self, _point: &PointRef) -> Result<Value, Error> {
        Err(Error::UnsupportedFabric { fabric: "zigbee" })
    }

    fn write_point(&mut self, _point: &PointRef, _value: &Value) -> Result<(), Error> {
        Err(Error::UnsupportedFabric { fabric: "zigbee" })
    }

    fn read_foreign(&self, _foreign: &ForeignRef) -> Result<Value, Error> {
        Err(Error::UnsupportedFabric { fabric: "zigbee" })
    }

    fn write_foreign(&mut self, _foreign: &ForeignRef, _raw: ForeignRaw) -> Result<Value, Error> {
        Err(Error::UnsupportedFabric { fabric: "zigbee" })
    }
}
