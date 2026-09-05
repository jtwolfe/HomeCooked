//! Modbus adapter: YAML/JSON map, in-memory slave, HomeCooked translation.
//!
//! v1 uses a mocked transport only — no `tokio-modbus`, serial, or TCP.

mod adapter;
mod map;
mod mock;

pub use adapter::ModbusBridge;
pub use map::{
    ForeignBits, MapAccess, ModbusEntry, ModbusMap, RegisterKind, WATER_HEATER_MAP_YAML,
};
pub use mock::ModbusSlave;
