//! In-memory BACnet property store (mock device — no BACnet stack).

use std::collections::HashMap;
use std::fmt;

use super::map::{BacnetMap, BacnetObjectType, BacnetProperty};
use crate::error::Error;

/// Key for one property on the mock BACnet device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BacnetPropKey {
    pub object_type: BacnetObjectType,
    pub object_instance: u32,
    pub property: BacnetProperty,
}

impl BacnetPropKey {
    pub const fn new(
        object_type: BacnetObjectType,
        object_instance: u32,
        property: BacnetProperty,
    ) -> Self {
        Self {
            object_type,
            object_instance,
            property,
        }
    }
}

impl fmt::Display for BacnetPropKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.object_type, self.object_instance, self.property
        )
    }
}

/// Stored property payload in the mock device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BacnetPropValue {
    Bool(bool),
    Int16(i16),
    UInt16(u16),
}

/// Mock BACnet device: device instance + property table (no MS/TP / IP stack).
#[derive(Debug, Clone, PartialEq)]
pub struct BacnetDevice {
    pub device_instance: u32,
    props: HashMap<BacnetPropKey, BacnetPropValue>,
}

impl BacnetDevice {
    pub fn new(device_instance: u32) -> Self {
        Self {
            device_instance,
            props: HashMap::new(),
        }
    }

    /// Allocate properties from a map and apply initial seeds.
    pub fn from_map(map: &BacnetMap) -> Result<Self, Error> {
        let mut device = Self::new(map.device_instance);
        for entry in &map.entries {
            let key = BacnetPropKey::new(entry.object_type, entry.object_instance, entry.property);
            device.props.insert(key, entry.seed_prop()?);
        }
        Ok(device)
    }

    pub fn get(&self, key: BacnetPropKey) -> Option<BacnetPropValue> {
        self.props.get(&key).copied()
    }

    pub fn get_or_default(&self, key: BacnetPropKey, default: BacnetPropValue) -> BacnetPropValue {
        self.props.get(&key).copied().unwrap_or(default)
    }

    pub fn set(&mut self, key: BacnetPropKey, value: BacnetPropValue) {
        self.props.insert(key, value);
    }

    pub fn read(
        &self,
        object_type: BacnetObjectType,
        object_instance: u32,
        property: BacnetProperty,
    ) -> Option<BacnetPropValue> {
        self.get(BacnetPropKey::new(object_type, object_instance, property))
    }

    pub fn write(
        &mut self,
        object_type: BacnetObjectType,
        object_instance: u32,
        property: BacnetProperty,
        value: BacnetPropValue,
    ) {
        self.set(
            BacnetPropKey::new(object_type, object_instance, property),
            value,
        );
    }

    pub fn len(&self) -> usize {
        self.props.len()
    }

    pub fn is_empty(&self) -> bool {
        self.props.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bacnet::BacnetMap;

    #[test]
    fn seeds_from_kettle_map() {
        let map = BacnetMap::kettle_example().unwrap();
        let device = BacnetDevice::from_map(&map).unwrap();
        assert_eq!(device.device_instance, 1);
        assert_eq!(
            device.read(
                BacnetObjectType::BinaryValue,
                1,
                BacnetProperty::PresentValue
            ),
            Some(BacnetPropValue::Bool(true))
        );
        assert_eq!(
            device.read(
                BacnetObjectType::AnalogInput,
                1,
                BacnetProperty::PresentValue
            ),
            Some(BacnetPropValue::Int16(2500))
        );
        assert_eq!(
            device.read(
                BacnetObjectType::AnalogValue,
                1,
                BacnetProperty::PresentValue
            ),
            Some(BacnetPropValue::Int16(8000))
        );
        assert_eq!(device.len(), 3);
    }
}
