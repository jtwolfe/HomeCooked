//! Integration: fake water_heater Modbus map, both translation directions.

use homecooked_bridge::{
    Bridge, ForeignRaw, ForeignRef, ModbusBridge, ModbusMap, PointRef, WATER_HEATER_MAP_YAML,
};
use homecooked_schema::Value;

fn point(id: &str) -> PointRef {
    PointRef::new("water-heater-plant", id).unwrap()
}

#[test]
fn example_yaml_is_the_published_fixture() {
    let map = ModbusMap::from_yaml_str(WATER_HEATER_MAP_YAML).unwrap();
    assert_eq!(map.class_id, "water_heater");
    assert_eq!(map.entries.len(), 3);
}

#[test]
fn foreign_register_write_updates_homecooked_point() {
    let mut bridge = ModbusBridge::water_heater_example().unwrap();
    let foreign = ForeignRef::holding("water-heater-plant", 0).unwrap();
    // 60.0 °C stored as 600 tenths
    let translated = bridge
        .write_foreign(&foreign, ForeignRaw::Register(600))
        .unwrap();
    assert_eq!(translated, Value::F32(60.0));
    assert_eq!(
        bridge
            .read_point(&point("trait.temperature.setpoint_c"))
            .unwrap(),
        Value::F32(60.0)
    );
    assert_eq!(
        bridge
            .backend()
            .get_value("water-heater-plant", "trait.temperature.setpoint_c"),
        Some(&Value::F32(60.0))
    );
}

#[test]
fn homecooked_write_updates_modbus_register() {
    let mut bridge = ModbusBridge::water_heater_example().unwrap();
    bridge
        .write_point(&point("trait.temperature.setpoint_c"), &Value::F32(42.0))
        .unwrap();
    assert_eq!(bridge.slave().get_holding(0), 420);
    assert_eq!(
        bridge
            .read_foreign(&ForeignRef::holding("water-heater-plant", 0).unwrap())
            .unwrap(),
        Value::F32(42.0)
    );
}

#[test]
fn power_coil_roundtrip_uses_catalog_enum() {
    let mut bridge = ModbusBridge::water_heater_example().unwrap();
    let coil = ForeignRef::coil("water-heater-plant", 0).unwrap();

    bridge
        .write_foreign(&coil, ForeignRaw::Coil(false))
        .unwrap();
    assert_eq!(
        bridge
            .backend()
            .get_value("water-heater-plant", "trait.power.power_state"),
        Some(&Value::Enum("off".into()))
    );

    bridge
        .write_point(&point("trait.power.power_state"), &Value::Enum("on".into()))
        .unwrap();
    assert!(bridge.slave().get_coil(0));
}
