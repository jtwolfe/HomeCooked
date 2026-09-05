//! Integration: fake kettle BACnet map, both translation directions.
//!
//! Object types in the fixture are illustrative lab constants — not a
//! certified BACnet product. No BACnet/IP stack dependency.

use homecooked_bridge::{
    BacnetBridge, BacnetMap, BacnetObjectType, BacnetPropValue, BacnetProperty, BacnetRaw, Bridge,
    ForeignRaw, ForeignRef, PointRef, KETTLE_BACNET_MAP_YAML,
};
use homecooked_schema::Value;

fn point(id: &str) -> PointRef {
    PointRef::new("kettle-lab-1", id).unwrap()
}

#[test]
fn example_yaml_is_the_published_fixture() {
    let map = BacnetMap::from_yaml_str(KETTLE_BACNET_MAP_YAML).unwrap();
    assert_eq!(map.class_id, "kettle");
    assert_eq!(map.device_instance, 1);
    assert_eq!(map.entries.len(), 3);
}

#[test]
fn foreign_prop_write_updates_homecooked_point() {
    let mut bridge = BacnetBridge::kettle_example().unwrap();
    let foreign = ForeignRef::bacnet(
        "kettle-lab-1",
        1,
        BacnetObjectType::AnalogValue,
        1,
        BacnetProperty::PresentValue,
    )
    .unwrap();
    // 60.0 °C stored as 6000 hundredths
    let translated = bridge
        .write_foreign(&foreign, ForeignRaw::Bacnet(BacnetRaw::Int16(6000)))
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
            .get_value("kettle-lab-1", "trait.temperature.setpoint_c"),
        Some(&Value::F32(60.0))
    );
}

#[test]
fn homecooked_write_updates_bacnet_property() {
    let mut bridge = BacnetBridge::kettle_example().unwrap();
    bridge
        .write_point(&point("trait.temperature.setpoint_c"), &Value::F32(42.0))
        .unwrap();
    assert_eq!(
        bridge.prop_store().read(
            BacnetObjectType::AnalogValue,
            1,
            BacnetProperty::PresentValue
        ),
        Some(BacnetPropValue::Int16(4200))
    );
    assert_eq!(
        bridge
            .read_foreign(
                &ForeignRef::bacnet(
                    "kettle-lab-1",
                    1,
                    BacnetObjectType::AnalogValue,
                    1,
                    BacnetProperty::PresentValue,
                )
                .unwrap()
            )
            .unwrap(),
        Value::F32(42.0)
    );
}

#[test]
fn binary_value_roundtrip_uses_catalog_enum() {
    let mut bridge = BacnetBridge::kettle_example().unwrap();
    let binary = ForeignRef::bacnet(
        "kettle-lab-1",
        1,
        BacnetObjectType::BinaryValue,
        1,
        BacnetProperty::PresentValue,
    )
    .unwrap();

    bridge
        .write_foreign(&binary, ForeignRaw::Bacnet(BacnetRaw::Bool(false)))
        .unwrap();
    assert_eq!(
        bridge
            .backend()
            .get_value("kettle-lab-1", "trait.power.power_state"),
        Some(&Value::Enum("off".into()))
    );

    bridge
        .write_point(&point("trait.power.power_state"), &Value::Enum("on".into()))
        .unwrap();
    assert_eq!(
        bridge.prop_store().read(
            BacnetObjectType::BinaryValue,
            1,
            BacnetProperty::PresentValue
        ),
        Some(BacnetPropValue::Bool(true))
    );
}
