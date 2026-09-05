//! [`BacnetBridge`]: map + in-memory device + HomeCooked backend.

use homecooked_schema::Value;

use super::map::{BacnetEntry, BacnetMap};
use super::store::{BacnetDevice, BacnetPropKey, BacnetPropValue};
use crate::backend::{MemoryBackend, PointBackend};
use crate::bridge::{Bridge, ForeignRaw, ForeignRef, PointRef};
use crate::error::Error;

/// BACnet adapter with a mocked device (no BACnet/IP / MS/TP stack dependency).
#[derive(Debug, Clone)]
pub struct BacnetBridge<B> {
    map: BacnetMap,
    device: BacnetDevice,
    backend: B,
}

impl BacnetBridge<MemoryBackend> {
    pub fn with_memory(map: BacnetMap) -> Result<Self, Error> {
        Self::new(map, MemoryBackend::new())
    }

    pub fn kettle_example() -> Result<Self, Error> {
        Self::with_memory(BacnetMap::kettle_example()?)
    }
}

impl<B: PointBackend> BacnetBridge<B> {
    pub fn new(map: BacnetMap, mut backend: B) -> Result<Self, Error> {
        map.validate()?;
        let device = BacnetDevice::from_map(&map)?;
        for entry in &map.entries {
            let key = BacnetPropKey::new(entry.object_type, entry.object_instance, entry.property);
            let prop = device
                .get(key)
                .ok_or_else(|| Error::InvalidMap(format!("missing seed for {}", entry.point)))?;
            let value = entry.decode_prop(prop)?;
            let point = PointRef::new(&map.device_id, &entry.point)?;
            backend.set(&point, value)?;
        }
        Ok(Self {
            map,
            device,
            backend,
        })
    }

    pub fn map(&self) -> &BacnetMap {
        &self.map
    }

    /// In-memory mock property store (not the [`Bridge::fabric`] token).
    pub fn prop_store(&self) -> &BacnetDevice {
        &self.device
    }

    pub fn prop_store_mut(&mut self) -> &mut BacnetDevice {
        &mut self.device
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

    fn entry_for_point(&self, point: &PointRef) -> Result<&BacnetEntry, Error> {
        self.require_device(&point.device_id)?;
        self.map
            .entry_for_point(&point.point_id)
            .ok_or_else(|| Error::UnmappedPoint {
                device_id: point.device_id.clone(),
                point_id: point.point_id.clone(),
            })
    }

    fn entry_for_foreign(&self, foreign: &ForeignRef) -> Result<&BacnetEntry, Error> {
        self.require_device(&foreign.device_id)?;
        let (device_instance, object_type, object_instance, property) =
            foreign.as_bacnet().ok_or_else(|| Error::LocatorMismatch {
                expected: "bacnet",
                locator: foreign.locator.clone(),
            })?;
        if device_instance != self.map.device_instance {
            return Err(Error::DeviceMismatch {
                expected: format!("device_instance {}", self.map.device_instance),
                actual: format!("device_instance {device_instance}"),
            });
        }
        self.map
            .entry_for_prop(object_type, object_instance, property)
            .ok_or(Error::UnmappedBacnetProperty {
                object_type: object_type.to_string(),
                object_instance,
                property: property.to_string(),
            })
    }

    fn apply_prop(&mut self, entry: &BacnetEntry, prop: BacnetPropValue) -> Result<Value, Error> {
        self.device.write(
            entry.object_type,
            entry.object_instance,
            entry.property,
            prop,
        );
        let value = entry.decode_prop(prop)?;
        let point = PointRef::new(&self.map.device_id, &entry.point)?;
        self.backend.set(&point, value.clone())?;
        Ok(value)
    }

    fn raw_to_prop(entry: &BacnetEntry, raw: ForeignRaw) -> Result<BacnetPropValue, Error> {
        match raw {
            ForeignRaw::Bacnet(m) => entry.encode_raw(m),
            ForeignRaw::Register(_)
            | ForeignRaw::Coil(_)
            | ForeignRaw::Matter(_)
            | ForeignRaw::Zigbee(_) => Err(Error::InvalidRaw {
                detail: "non-bacnet raw is not valid for bacnet bridge".into(),
            }),
        }
    }
}

impl<B: PointBackend> Bridge for BacnetBridge<B> {
    fn fabric(&self) -> &'static str {
        "bacnet"
    }

    fn read_point(&self, point: &PointRef) -> Result<Value, Error> {
        let entry = self.entry_for_point(point)?;
        let prop = self
            .device
            .read(entry.object_type, entry.object_instance, entry.property)
            .ok_or(Error::UnmappedBacnetProperty {
                object_type: entry.object_type.to_string(),
                object_instance: entry.object_instance,
                property: entry.property.to_string(),
            })?;
        entry.decode_prop(prop)
    }

    fn write_point(&mut self, point: &PointRef, value: &Value) -> Result<(), Error> {
        let entry = self.entry_for_point(point)?.clone();
        if !entry.access.is_writable() {
            return Err(Error::NotWritable {
                point_id: entry.point.clone(),
            });
        }
        let prop = entry.encode_value(value)?;
        self.apply_prop(&entry, prop)?;
        Ok(())
    }

    fn read_foreign(&self, foreign: &ForeignRef) -> Result<Value, Error> {
        let entry = self.entry_for_foreign(foreign)?;
        let prop = self
            .device
            .read(entry.object_type, entry.object_instance, entry.property)
            .ok_or(Error::UnmappedBacnetProperty {
                object_type: entry.object_type.to_string(),
                object_instance: entry.object_instance,
                property: entry.property.to_string(),
            })?;
        entry.decode_prop(prop)
    }

    fn write_foreign(&mut self, foreign: &ForeignRef, raw: ForeignRaw) -> Result<Value, Error> {
        let entry = self.entry_for_foreign(foreign)?.clone();
        let prop = Self::raw_to_prop(&entry, raw)?;
        self.apply_prop(&entry, prop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bacnet::{BacnetObjectType, BacnetProperty};
    use crate::bridge::BacnetRaw;

    fn kettle() -> BacnetBridge<MemoryBackend> {
        BacnetBridge::kettle_example().unwrap()
    }

    fn point(id: &str) -> PointRef {
        PointRef::new("kettle-lab-1", id).unwrap()
    }

    #[test]
    fn seeds_backend_from_initial_props() {
        let bridge = kettle();
        assert_eq!(Bridge::fabric(&bridge), "bacnet");
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
    fn foreign_prop_write_updates_homecooked_backend() {
        let mut bridge = kettle();
        let foreign = ForeignRef::bacnet(
            "kettle-lab-1",
            1,
            BacnetObjectType::AnalogValue,
            1,
            BacnetProperty::PresentValue,
        )
        .unwrap();
        let value = bridge
            .write_foreign(&foreign, ForeignRaw::Bacnet(BacnetRaw::Int16(6500)))
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
    fn foreign_binary_write_updates_power_state() {
        let mut bridge = kettle();
        let foreign = ForeignRef::bacnet(
            "kettle-lab-1",
            1,
            BacnetObjectType::BinaryValue,
            1,
            BacnetProperty::PresentValue,
        )
        .unwrap();
        let value = bridge
            .write_foreign(&foreign, ForeignRaw::Bacnet(BacnetRaw::Bool(false)))
            .unwrap();
        assert_eq!(value, Value::Enum("off".into()));
        assert_eq!(
            bridge
                .backend()
                .get_value("kettle-lab-1", "trait.power.power_state"),
            Some(&Value::Enum("off".into()))
        );
        assert_eq!(
            bridge.prop_store().read(
                BacnetObjectType::BinaryValue,
                1,
                BacnetProperty::PresentValue
            ),
            Some(BacnetPropValue::Bool(false))
        );
    }

    #[test]
    fn homecooked_write_updates_property() {
        let mut bridge = kettle();
        bridge
            .write_point(&point("trait.temperature.setpoint_c"), &Value::F32(90.0))
            .unwrap();
        assert_eq!(
            bridge.prop_store().read(
                BacnetObjectType::AnalogValue,
                1,
                BacnetProperty::PresentValue
            ),
            Some(BacnetPropValue::Int16(9000))
        );
        assert_eq!(
            bridge
                .backend()
                .get_value("kettle-lab-1", "trait.temperature.setpoint_c"),
            Some(&Value::F32(90.0))
        );
    }

    #[test]
    fn homecooked_write_updates_binary() {
        let mut bridge = kettle();
        bridge
            .write_point(
                &point("trait.power.power_state"),
                &Value::Enum("off".into()),
            )
            .unwrap();
        assert_eq!(
            bridge.prop_store().read(
                BacnetObjectType::BinaryValue,
                1,
                BacnetProperty::PresentValue
            ),
            Some(BacnetPropValue::Bool(false))
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

    #[test]
    fn current_temp_is_read_only_from_homecooked() {
        let mut bridge = kettle();
        let err = bridge
            .write_point(&point("trait.temperature.current_c"), &Value::F32(50.0))
            .unwrap_err();
        assert!(matches!(err, Error::NotWritable { .. }));
        assert_eq!(
            bridge.prop_store().read(
                BacnetObjectType::AnalogInput,
                1,
                BacnetProperty::PresentValue
            ),
            Some(BacnetPropValue::Int16(2500))
        );

        let foreign = ForeignRef::bacnet(
            "kettle-lab-1",
            1,
            BacnetObjectType::AnalogInput,
            1,
            BacnetProperty::PresentValue,
        )
        .unwrap();
        bridge
            .write_foreign(&foreign, ForeignRaw::Bacnet(BacnetRaw::Int16(5120)))
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
        let missing = ForeignRef::bacnet(
            "kettle-lab-1",
            1,
            BacnetObjectType::AnalogValue,
            99,
            BacnetProperty::PresentValue,
        )
        .unwrap();
        assert!(matches!(
            bridge.write_foreign(&missing, ForeignRaw::Bacnet(BacnetRaw::Bool(true))),
            Err(Error::UnmappedBacnetProperty { .. })
        ));
        let modbus = ForeignRef::holding("kettle-lab-1", 0).unwrap();
        assert!(matches!(
            bridge.read_foreign(&modbus),
            Err(Error::LocatorMismatch { .. })
        ));
    }
}
