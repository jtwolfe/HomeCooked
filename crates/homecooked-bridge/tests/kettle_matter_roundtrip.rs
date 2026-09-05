//! Integration: fake kettle Matter map, both translation directions.
//!
//! Cluster IDs in the fixture are illustrative lab constants — not a
//! certified Matter product.

use homecooked_bridge::{
    Bridge, ForeignRaw, ForeignRef, MatterAttrValue, MatterBridge, MatterMap, MatterRaw, PointRef,
    KETTLE_MATTER_MAP_YAML,
};
use homecooked_schema::Value;

fn point(id: &str) -> PointRef {
    PointRef::new("kettle-lab-1", id).unwrap()
}

#[test]
fn example_yaml_is_the_published_fixture() {
    let map = MatterMap::from_yaml_str(KETTLE_MATTER_MAP_YAML).unwrap();
    assert_eq!(map.class_id, "kettle");
    assert_eq!(map.entries.len(), 3);
}

#[test]
fn foreign_attr_write_updates_homecooked_point() {
    let mut bridge = MatterBridge::kettle_example().unwrap();
    let foreign = ForeignRef::matter("kettle-lab-1", 1, 0x0201, 0x0012).unwrap();
    // 60.0 °C stored as 6000 hundredths
    let translated = bridge
        .write_foreign(&foreign, ForeignRaw::Matter(MatterRaw::Int16(6000)))
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
fn homecooked_write_updates_matter_attribute() {
    let mut bridge = MatterBridge::kettle_example().unwrap();
    bridge
        .write_point(&point("trait.temperature.setpoint_c"), &Value::F32(42.0))
        .unwrap();
    assert_eq!(
        bridge.attr_store().read(1, 0x0201, 0x0012),
        Some(MatterAttrValue::Int16(4200))
    );
    assert_eq!(
        bridge
            .read_foreign(&ForeignRef::matter("kettle-lab-1", 1, 0x0201, 0x0012).unwrap())
            .unwrap(),
        Value::F32(42.0)
    );
}

#[test]
fn onoff_roundtrip_uses_catalog_enum() {
    let mut bridge = MatterBridge::kettle_example().unwrap();
    let onoff = ForeignRef::matter("kettle-lab-1", 1, 0x0006, 0x0000).unwrap();

    bridge
        .write_foreign(&onoff, ForeignRaw::Matter(MatterRaw::Bool(false)))
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
        bridge.attr_store().read(1, 0x0006, 0x0000),
        Some(MatterAttrValue::Bool(true))
    );
}
