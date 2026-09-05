//! Modbus adapter: YAML/JSON map, in-memory slave, HomeCooked translation,
//! and a CI-friendly Modbus TCP lab path (localhost loopback).
//!
//! The core map/slave path stays dependency-free. The TCP lab uses only
//! `std::net` — no `tokio-modbus`, serial RTU, or TLS.

mod adapter;
mod map;
mod mock;
mod tcp;

pub use adapter::ModbusBridge;
pub use map::{
    ForeignBits, MapAccess, ModbusEntry, ModbusMap, RegisterKind, WATER_HEATER_MAP_YAML,
};
pub use mock::ModbusSlave;
pub use tcp::{
    shared_bridge, spawn_modbus_tcp_lab, ModbusTcpClient, SharedModbusBridge, SpawnedModbusTcp,
};
