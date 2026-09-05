//! Bridge slice for HomeCooked: Modbus + Matter + Zigbee + BACnet mock adapters.
//!
//! Aligns with [`docs/standard/bridges.md`](../../docs/standard/bridges.md)
//! and [`docs/ROADMAP.md`](../../docs/ROADMAP.md) Stream 6.
//!
//! The [`Bridge`] trait maps foreign reads/writes ↔ HomeCooked
//! [`PointRef`] values (`device_id` + qualified catalog point +
//! [`homecooked_schema::Value`]).
//!
//! [`modbus::ModbusBridge`] is the plant-bus path: a YAML/JSON register map,
//! an in-memory slave, and a [`PointBackend`] (tests use [`MemoryBackend`]).
//! There is **no** serial or TCP Modbus dependency.
//!
//! [`matter::MatterBridge`] is the fabric path: a YAML/JSON
//! endpoint/cluster/attribute map, an in-memory mock fabric, and the same
//! backend pattern. There is **no** CHIP / Matter SDK dependency; cluster IDs
//! in fixtures are illustrative lab constants, not certified product data.
//!
//! [`zigbee::ZigbeeBridge`] mirrors Matter with a YAML/JSON ZCL-style
//! endpoint/cluster/attribute map and an in-memory attribute store. There is
//! **no** zigbee2mqtt / MQTT / ZCL SDK dependency.
//!
//! [`bacnet::BacnetBridge`] mirrors the same pattern for plant-bus BACnet:
//! YAML/JSON device instance + object type/instance + property → point map
//! and an in-memory property store. There is **no** BACnet/IP / MS/TP stack
//! dependency; object types in fixtures are illustrative lab constants.

#![allow(clippy::module_name_repetitions)]

mod access;
mod backend;
pub mod bacnet;
mod bridge;
mod error;
pub mod matter;
pub mod modbus;
mod yaml_json;
pub mod zigbee;

pub use access::MapAccess;
pub use backend::{MemoryBackend, PointBackend};
pub use bacnet::{
    BacnetBridge, BacnetDevice, BacnetEntry, BacnetMap, BacnetObjectType, BacnetPropKey,
    BacnetPropValue, BacnetProperty, KETTLE_BACNET_MAP_YAML,
};
pub use bridge::{
    BacnetRaw, Bridge, ForeignLocator, ForeignRaw, ForeignRef, MatterRaw, PointRef, ZigbeeRaw,
};
pub use error::Error;
pub use matter::{
    AttrValueType, MatterAttrKey, MatterAttrValue, MatterBridge, MatterEntry, MatterFabric,
    MatterMap, KETTLE_MATTER_MAP_YAML,
};
pub use zigbee::{
    ZigbeeAttrKey, ZigbeeAttrValue, ZigbeeBridge, ZigbeeEntry, ZigbeeMap, ZigbeeNetwork,
    KETTLE_ZIGBEE_MAP_YAML,
};

pub use modbus::{
    ModbusBridge, ModbusEntry, ModbusMap, ModbusSlave, RegisterKind, WATER_HEATER_MAP_YAML,
};

#[cfg(test)]
mod stub_tests {
    use super::*;

    #[test]
    fn bacnet_is_no_longer_a_stub() {
        let bridge = BacnetBridge::kettle_example().unwrap();
        assert_eq!(bridge.fabric(), "bacnet");
        assert!(bridge
            .read_point(&PointRef::new("kettle-lab-1", "trait.temperature.setpoint_c").unwrap())
            .is_ok());
    }

    #[test]
    fn matter_is_no_longer_a_stub() {
        let bridge = MatterBridge::kettle_example().unwrap();
        assert_eq!(bridge.fabric(), "matter");
        assert!(bridge
            .read_point(&PointRef::new("kettle-lab-1", "trait.temperature.setpoint_c").unwrap())
            .is_ok());
    }

    #[test]
    fn zigbee_is_no_longer_a_stub() {
        let bridge = ZigbeeBridge::kettle_example().unwrap();
        assert_eq!(bridge.fabric(), "zigbee");
        assert!(bridge
            .read_point(&PointRef::new("kettle-lab-1", "trait.temperature.setpoint_c").unwrap())
            .is_ok());
    }

    #[test]
    fn point_ref_rejects_empty_and_invented_ids() {
        assert!(matches!(
            PointRef::new("", "trait.temperature.setpoint_c"),
            Err(Error::EmptyId("device_id"))
        ));
        assert!(matches!(
            PointRef::new("dev", "trait.power.on"),
            Err(Error::UnknownCatalogPoint(id)) if id == "trait.power.on"
        ));
        assert!(PointRef::new("dev", "trait.temperature.setpoint_c").is_ok());
    }
}
