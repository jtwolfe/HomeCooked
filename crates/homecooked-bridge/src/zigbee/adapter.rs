//! [`ZigbeeBridge`]: map + in-memory network + HomeCooked backend.

use homecooked_schema::Value;

use super::map::{ZigbeeEntry, ZigbeeMap};
use super::store::{ZigbeeAttrKey, ZigbeeAttrValue, ZigbeeNetwork};
use crate::backend::{MemoryBackend, PointBackend};
use crate::bridge::{Bridge, ForeignRaw, ForeignRef, PointRef};
use crate::error::Error;

/// Zigbee adapter with a mocked fabric (no zigbee2mqtt / ZCL SDK dependency).
#[derive(Debug, Clone)]
pub struct ZigbeeBridge<B> {
    map: ZigbeeMap,
    fabric: ZigbeeNetwork,
    backend: B,
}

impl ZigbeeBridge<MemoryBackend> {
    pub fn with_memory(map: ZigbeeMap) -> Result<Self, Error> {
        Self::new(map, MemoryBackend::new())
    }

    pub fn kettle_example() -> Result<Self, Error> {
        Self::with_memory(ZigbeeMap::kettle_example()?)
    }
}

impl<B: PointBackend> ZigbeeBridge<B> {
    pub fn new(map: ZigbeeMap, mut backend: B) -> Result<Self, Error> {
        map.validate()?;
        let fabric = ZigbeeNetwork::from_map(&map)?;
        for entry in &map.entries {
            let key = ZigbeeAttrKey::new(entry.endpoint, entry.cluster_id, entry.attribute_id);
            let attr = fabric
                .get(key)
                .ok_or_else(|| Error::InvalidMap(format!("missing seed for {}", entry.point)))?;
            let value = entry.decode_attr(attr)?;
            let point = PointRef::new(&map.device_id, &entry.point)?;
            backend.set(&point, value)?;
        }
        Ok(Self {
            map,
            fabric,
            backend,
        })
    }

    pub fn map(&self) -> &ZigbeeMap {
        &self.map
    }

    /// In-memory mock attribute store (not the [`Bridge::fabric`] token).
    pub fn attr_store(&self) -> &ZigbeeNetwork {
        &self.fabric
    }

    pub fn attr_store_mut(&mut self) -> &mut ZigbeeNetwork {
        &mut self.fabric
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    fn require_device(&self, device_id: &str) -> Result<(), Error> {
        if device_id != self.map.device_id {
            return Err(Error::DeviceMismatch {
                expected: self.map.device_id.clone(),
                actual: device_id.to_string(),
            });
        }
        Ok(())
    }

    fn entry_for_point(&self, point: &PointRef) -> Result<&ZigbeeEntry, Error> {
        self.require_device(&point.device_id)?;
        self.map
            .entry_for_point(&point.point_id)
            .ok_or_else(|| Error::UnmappedPoint {
                device_id: point.device_id.clone(),
                point_id: point.point_id.clone(),
            })
    }

    fn entry_for_foreign(&self, foreign: &ForeignRef) -> Result<&ZigbeeEntry, Error> {
        self.require_device(&foreign.device_id)?;
        let (endpoint, cluster_id, attribute_id) =
            foreign.as_zigbee().ok_or_else(|| Error::LocatorMismatch {
                expected: "zigbee",
                locator: foreign.locator.clone(),
            })?;
        self.map
            .entry_for_attr(endpoint, cluster_id, attribute_id)
            .ok_or(Error::UnmappedZigbeeAttribute {
                endpoint,
                cluster_id,
                attribute_id,
            })
    }

    fn apply_attr(&mut self, entry: &ZigbeeEntry, attr: ZigbeeAttrValue) -> Result<Value, Error> {
        self.fabric
            .write(entry.endpoint, entry.cluster_id, entry.attribute_id, attr);
        let value = entry.decode_attr(attr)?;
        let point = PointRef::new(&self.map.device_id, &entry.point)?;
        self.backend.set(&point, value.clone())?;
        Ok(value)
    }

    fn raw_to_attr(entry: &ZigbeeEntry, raw: ForeignRaw) -> Result<ZigbeeAttrValue, Error> {
        match raw {
            ForeignRaw::Zigbee(m) => entry.encode_raw(m),
            ForeignRaw::Register(_) | ForeignRaw::Coil(_) | ForeignRaw::Matter(_) => {
                Err(Error::InvalidRaw {
                    detail: "non-zigbee raw is not valid for zigbee bridge".into(),
                })
            }
        }
    }
}

impl<B: PointBackend> Bridge for ZigbeeBridge<B> {
    fn fabric(&self) -> &'static str {
        "zigbee"
    }

    fn read_point(&self, point: &PointRef) -> Result<Value, Error> {
        let entry = self.entry_for_point(point)?;
        let attr = self
            .fabric
            .read(entry.endpoint, entry.cluster_id, entry.attribute_id)
            .ok_or(Error::UnmappedZigbeeAttribute {
                endpoint: entry.endpoint,
                cluster_id: entry.cluster_id,
                attribute_id: entry.attribute_id,
            })?;
        entry.decode_attr(attr)
    }

    fn write_point(&mut self, point: &PointRef, value: &Value) -> Result<(), Error> {
        let entry = self.entry_for_point(point)?.clone();
        if !entry.access.is_writable() {
            return Err(Error::NotWritable {
                point_id: entry.point.clone(),
            });
        }
        let attr = entry.encode_value(value)?;
        self.apply_attr(&entry, attr)?;
        Ok(())
    }

    fn read_foreign(&self, foreign: &ForeignRef) -> Result<Value, Error> {
        let entry = self.entry_for_foreign(foreign)?;
        let attr = self
            .fabric
            .read(entry.endpoint, entry.cluster_id, entry.attribute_id)
            .ok_or(Error::UnmappedZigbeeAttribute {
                endpoint: entry.endpoint,
                cluster_id: entry.cluster_id,
                attribute_id: entry.attribute_id,
            })?;
        entry.decode_attr(attr)
    }

    fn write_foreign(&mut self, foreign: &ForeignRef, raw: ForeignRaw) -> Result<Value, Error> {
        let entry = self.entry_for_foreign(foreign)?.clone();
        let attr = Self::raw_to_attr(&entry, raw)?;
        self.apply_attr(&entry, attr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::ZigbeeRaw;

    fn kettle() -> ZigbeeBridge<MemoryBackend> {
        ZigbeeBridge::kettle_example().unwrap()
    }

    fn point(id: &str) -> PointRef {
        PointRef::new("kettle-lab-1", id).unwrap()
    }

    #[test]
    fn seeds_backend_from_initial_attrs() {
        let bridge = kettle();
        assert_eq!(Bridge::fabric(&bridge), "zigbee");
        assert_eq!(
            bridge
                .backend()
                .get_value("kettle-lab-1", "trait.power.power_state"),
            Some(&Value::Enum("on".into()))
        );
        assert_eq!(
            bridge
                .backend()
                .get_value("kettle-lab-1", "trait.temperature.current_c"),
            Some(&Value::F32(25.0))
        );
        assert_eq!(
            bridge
                .backend()
                .get_value("kettle-lab-1", "trait.temperature.setpoint_c"),
            Some(&Value::F32(80.0))
        );
    }

    #[test]
    fn foreign_attr_write_updates_homecooked_backend() {
        let mut bridge = kettle();
        let foreign = ForeignRef::zigbee("kettle-lab-1", 1, 0x0201, 0x0012).unwrap();
        let value = bridge
            .write_foreign(&foreign, ForeignRaw::Zigbee(ZigbeeRaw::Int16(6500)))
            .unwrap();
        assert_eq!(value, Value::F32(65.0));
        assert_eq!(
            bridge
                .backend()
                .get_value("kettle-lab-1", "trait.temperature.setpoint_c"),
            Some(&Value::F32(65.0))
        );
        assert_eq!(
            bridge
                .read_point(&point("trait.temperature.setpoint_c"))
                .unwrap(),
            Value::F32(65.0)
        );
    }

    #[test]
    fn foreign_onoff_write_updates_power_state() {
        let mut bridge = kettle();
        let foreign = ForeignRef::zigbee("kettle-lab-1", 1, 0x0006, 0x0000).unwrap();
        let value = bridge
            .write_foreign(&foreign, ForeignRaw::Zigbee(ZigbeeRaw::Bool(false)))
            .unwrap();
        assert_eq!(value, Value::Enum("off".into()));
        assert_eq!(
            bridge
                .backend()
                .get_value("kettle-lab-1", "trait.power.power_state"),
            Some(&Value::Enum("off".into()))
        );
        assert_eq!(
            bridge.attr_store().read(1, 0x0006, 0x0000),
            Some(ZigbeeAttrValue::Bool(false))
        );
    }

    #[test]
    fn homecooked_write_updates_attribute() {
        let mut bridge = kettle();
        bridge
            .write_point(&point("trait.temperature.setpoint_c"), &Value::F32(90.0))
            .unwrap();
        assert_eq!(
            bridge.attr_store().read(1, 0x0201, 0x0012),
            Some(ZigbeeAttrValue::Int16(9000))
        );
        assert_eq!(
            bridge
                .backend()
                .get_value("kettle-lab-1", "trait.temperature.setpoint_c"),
            Some(&Value::F32(90.0))
        );
    }

    #[test]
    fn homecooked_write_updates_onoff() {
        let mut bridge = kettle();
        bridge
            .write_point(
                &point("trait.power.power_state"),
                &Value::Enum("off".into()),
            )
            .unwrap();
        assert_eq!(
            bridge.attr_store().read(1, 0x0006, 0x0000),
            Some(ZigbeeAttrValue::Bool(false))
        );
        bridge
            .write_point(&point("trait.power.power_state"), &Value::Enum("on".into()))
            .unwrap();
        assert_eq!(
            bridge.attr_store().read(1, 0x0006, 0x0000),
            Some(ZigbeeAttrValue::Bool(true))
        );
    }

    #[test]
    fn current_temp_is_read_only_from_homecooked() {
        let mut bridge = kettle();
        let err = bridge
            .write_point(&point("trait.temperature.current_c"), &Value::F32(50.0))
            .unwrap_err();
        assert!(matches!(err, Error::NotWritable { .. }));
        assert_eq!(
            bridge.attr_store().read(1, 0x0402, 0x0000),
            Some(ZigbeeAttrValue::Int16(2500))
        );

        let foreign = ForeignRef::zigbee("kettle-lab-1", 1, 0x0402, 0x0000).unwrap();
        bridge
            .write_foreign(&foreign, ForeignRaw::Zigbee(ZigbeeRaw::Int16(5120)))
            .unwrap();
        assert_eq!(
            bridge
                .read_point(&point("trait.temperature.current_c"))
                .unwrap(),
            Value::F32(51.2)
        );
    }

    #[test]
    fn rejects_unmapped_and_wrong_device() {
        let mut bridge = kettle();
        let other = PointRef::new("other-device", "trait.temperature.setpoint_c").unwrap();
        assert!(matches!(
            bridge.read_point(&other),
            Err(Error::DeviceMismatch { .. })
        ));
        let unknown = point("trait.energy.power_w");
        assert!(matches!(
            bridge.read_point(&unknown),
            Err(Error::UnmappedPoint { .. })
        ));
        let missing = ForeignRef::zigbee("kettle-lab-1", 1, 0x9999, 0).unwrap();
        assert!(matches!(
            bridge.write_foreign(&missing, ForeignRaw::Zigbee(ZigbeeRaw::Bool(true))),
            Err(Error::UnmappedZigbeeAttribute { .. })
        ));
        let modbus = ForeignRef::holding("kettle-lab-1", 0).unwrap();
        assert!(matches!(
            bridge.read_foreign(&modbus),
            Err(Error::LocatorMismatch { .. })
        ));
    }
}
