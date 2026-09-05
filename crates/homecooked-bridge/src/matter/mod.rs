//! Matter adapter: YAML/JSON cluster map, in-memory attribute store, HomeCooked
//! translation.
//!
//! v1 uses a **mocked** fabric only — no CHIP / Matter SDK, no Thread, no
//! commissioning. Cluster IDs in fixtures are illustrative lab constants.

mod adapter;
mod map;
mod store;

pub use adapter::MatterBridge;
pub use map::{AttrValueType, MatterEntry, MatterMap, KETTLE_MATTER_MAP_YAML};
pub use store::{MatterAttrKey, MatterAttrValue, MatterFabric};
