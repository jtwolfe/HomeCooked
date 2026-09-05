//! Integration: Modbus TCP lab on 127.0.0.1:0 against the water_heater map.
//!
//! Hardware-free CI path: minimal MBAP framing (FC01/FC03/FC05/FC06), no
//! serial RTU / tokio-modbus / TLS.

use std::net::SocketAddr;

use homecooked_bridge::{
    shared_bridge, spawn_modbus_tcp_lab, Bridge, ModbusBridge, ModbusTcpClient, PointRef,
};
use homecooked_schema::Value;

fn point(id: &str) -> PointRef {
    PointRef::new("water-heater-plant", id).unwrap()
}

fn spawn_lab() -> (SocketAddr, homecooked_bridge::SharedModbusBridge) {
    let bridge = ModbusBridge::water_heater_example().unwrap();
    let shared = shared_bridge(bridge);
    let (addr, shared, _join) = spawn_modbus_tcp_lab("127.0.0.1:0", shared).unwrap();
    (addr, shared)
}

#[test]
fn tcp_read_seeded_setpoint_temp_and_power() {
    let (addr, _shared) = spawn_lab();
    let mut client = ModbusTcpClient::connect(addr, 1).unwrap();

    let regs = client.read_holding_registers(0, 2).unwrap();
    assert_eq!(regs, vec![550, 480]); // 55.0 °C, 48.0 °C

    let coils = client.read_coils(0, 1).unwrap();
    assert_eq!(coils, vec![true]);
}

#[test]
fn tcp_write_setpoint_updates_homecooked_backend() {
    let (addr, shared) = spawn_lab();
    let mut client = ModbusTcpClient::connect(addr, 1).unwrap();

    // 62.0 °C as 620 tenths
    client.write_single_register(0, 620).unwrap();

    let regs = client.read_holding_registers(0, 1).unwrap();
    assert_eq!(regs, vec![620]);

    let bridge = shared.lock().unwrap();
    assert_eq!(
        bridge
            .backend()
            .get_value("water-heater-plant", "trait.temperature.setpoint_c"),
        Some(&Value::F32(62.0))
    );
    assert_eq!(
        bridge
            .read_point(&point("trait.temperature.setpoint_c"))
            .unwrap(),
        Value::F32(62.0)
    );
}

#[test]
fn tcp_write_coil_updates_power_state() {
    let (addr, shared) = spawn_lab();
    let mut client = ModbusTcpClient::connect(addr, 1).unwrap();

    client.write_single_coil(0, false).unwrap();
    assert_eq!(client.read_coils(0, 1).unwrap(), vec![false]);

    {
        let bridge = shared.lock().unwrap();
        assert_eq!(
            bridge
                .backend()
                .get_value("water-heater-plant", "trait.power.power_state"),
            Some(&Value::Enum("off".into()))
        );
    }

    client.write_single_coil(0, true).unwrap();
    assert_eq!(client.read_coils(0, 1).unwrap(), vec![true]);
}

#[test]
fn homecooked_write_visible_over_tcp() {
    let (addr, shared) = spawn_lab();

    {
        let mut bridge = shared.lock().unwrap();
        bridge
            .write_point(&point("trait.temperature.setpoint_c"), &Value::F32(44.5))
            .unwrap();
        assert_eq!(bridge.slave().get_holding(0), 445);
    }

    let mut client = ModbusTcpClient::connect(addr, 1).unwrap();
    assert_eq!(client.read_holding_registers(0, 1).unwrap(), vec![445]);
}
