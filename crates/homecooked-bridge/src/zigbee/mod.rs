//! Zigbee adapter: YAML/JSON cluster map, in-memory attribute store, HomeCooked
//! translation.
//!
//! v1 uses a **mocked** network only — no zigbee2mqtt, no ZCL SDK, no pairing.
//! Cluster IDs in fixtures are illustrative lab constants.

mod adapter;
mod map;
mod store;

pub use adapter::ZigbeeBridge;
pub use map::{AttrValueType, ZigbeeEntry, ZigbeeMap, KETTLE_ZIGBEE_MAP_YAML};
pub use store::{ZigbeeAttrKey, ZigbeeAttrValue, ZigbeeNetwork};
