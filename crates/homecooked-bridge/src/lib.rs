//! First **bridge** slice for HomeCooked: one real adapter (Modbus) plus
//! Zigbee / Matter / BACnet stubs.
//!
//! Aligns with [`docs/standard/bridges.md`](../../docs/standard/bridges.md)
//! and [`docs/ROADMAP.md`](../../docs/ROADMAP.md) Stream 6.
//!
//! The [`Bridge`] trait maps foreign reads/writes ↔ HomeCooked
//! [`PointRef`] values (`device_id` + qualified catalog point +
//! [`homecooked_schema::Value`]).
//!
//! [`modbus::ModbusBridge`] is the implemented path: a YAML/JSON register
//! map, an in-memory slave, and a [`PointBackend`] (tests use
//! [`MemoryBackend`]). There is **no** serial or TCP Modbus dependency.
//!
//! [`ZigbeeBridge`], [`MatterBridge`], and [`BacnetBridge`] compile and
//! return [`Error::UnsupportedFabric`].

#![allow(clippy::module_name_repetitions)]

mod backend;
mod bacnet;
mod bridge;
mod error;
mod matter;
pub mod modbus;
mod zigbee;

pub use backend::{MemoryBackend, PointBackend};
pub use bacnet::BacnetBridge;
pub use bridge::{Bridge, ForeignRaw, ForeignRef, PointRef};
pub use error::Error;
pub use matter::MatterBridge;
pub use zigbee::ZigbeeBridge;

pub use modbus::{
    ModbusBridge, ModbusEntry, ModbusMap, ModbusSlave, RegisterKind, WATER_HEATER_MAP_YAML,
};

#[cfg(test)]
mod stub_tests {
    use super::*;
    use homecooked_schema::Value;

    fn point() -> PointRef {
        PointRef::new("dev-1", "trait.temperature.setpoint_c").unwrap()
    }

    fn assert_unsupported<B: Bridge>(mut bridge: B, fabric: &'static str) {
        assert_eq!(bridge.fabric(), fabric);
        let err = bridge.read_point(&point()).unwrap_err();
        assert_eq!(err, Error::UnsupportedFabric { fabric });
        let err = bridge.write_point(&point(), &Value::F32(20.0)).unwrap_err();
        assert_eq!(err, Error::UnsupportedFabric { fabric });
        let foreign = ForeignRef::holding("dev-1", 0).unwrap();
        assert_eq!(
            bridge.read_foreign(&foreign).unwrap_err(),
            Error::UnsupportedFabric { fabric }
        );
        assert_eq!(
            bridge
                .write_foreign(&foreign, ForeignRaw::Register(1))
                .unwrap_err(),
            Error::UnsupportedFabric { fabric }
        );
    }

    #[test]
    fn stubs_are_unsupported() {
        assert_unsupported(ZigbeeBridge::new(), "zigbee");
        assert_unsupported(MatterBridge::new(), "matter");
        assert_unsupported(BacnetBridge::new(), "bacnet");
    }

    #[test]
    fn point_ref_rejects_empty_and_invented_ids() {
        assert!(matches!(
            PointRef::new("", "trait.temperature.setpoint_c"),
            Err(Error::EmptyId("device_id"))
        ));
        assert!(PointRef::new("dev", "trait.power.on").is_err());
        assert!(PointRef::new("dev", "trait.temperature.setpoint_c").is_ok());
    }
}
