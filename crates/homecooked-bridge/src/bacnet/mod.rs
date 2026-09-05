//! BACnet adapter: YAML/JSON object map, in-memory property store, HomeCooked
//! translation.
//!
//! v1 uses a **mocked** device only — no BACnet/IP, MS/TP, or ASHRAE stack.
//! Object types and property ids in fixtures are illustrative lab constants.

mod adapter;
mod map;
mod store;

pub use adapter::BacnetBridge;
pub use map::{
    AttrValueType, BacnetEntry, BacnetMap, BacnetObjectType, BacnetProperty, KETTLE_BACNET_MAP_YAML,
};
pub use store::{BacnetDevice, BacnetPropKey, BacnetPropValue};
