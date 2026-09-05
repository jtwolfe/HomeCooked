//! In-memory Zigbee attribute store (mock network — no SDK).

use std::collections::HashMap;
use std::fmt;

use super::map::ZigbeeMap;
use crate::error::Error;

/// Key for one attribute on the mock network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZigbeeAttrKey {
    pub endpoint: u16,
    pub cluster_id: u32,
    pub attribute_id: u32,
}

impl ZigbeeAttrKey {
    pub const fn new(endpoint: u16, cluster_id: u32, attribute_id: u32) -> Self {
        Self {
            endpoint,
            cluster_id,
            attribute_id,
        }
    }
}

impl fmt::Display for ZigbeeAttrKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ep{}/cluster={:#x}/attr={:#x}",
            self.endpoint, self.cluster_id, self.attribute_id
        )
    }
}

/// Stored attribute payload in the mock network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZigbeeAttrValue {
    Bool(bool),
    Int16(i16),
    UInt16(u16),
}

/// Mock Zigbee network: node id + attribute table (no commissioning / ACL).
#[derive(Debug, Clone, PartialEq)]
pub struct ZigbeeNetwork {
    pub node_id: u64,
    attrs: HashMap<ZigbeeAttrKey, ZigbeeAttrValue>,
}

impl ZigbeeNetwork {
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            attrs: HashMap::new(),
        }
    }

    /// Allocate attributes from a map and apply initial seeds.
    pub fn from_map(map: &ZigbeeMap) -> Result<Self, Error> {
        let mut fabric = Self::new(map.node_id);
        for entry in &map.entries {
            let key = ZigbeeAttrKey::new(entry.endpoint, entry.cluster_id, entry.attribute_id);
            fabric.attrs.insert(key, entry.seed_attr()?);
        }
        Ok(fabric)
    }

    pub fn get(&self, key: ZigbeeAttrKey) -> Option<ZigbeeAttrValue> {
        self.attrs.get(&key).copied()
    }

    pub fn get_or_default(&self, key: ZigbeeAttrKey, default: ZigbeeAttrValue) -> ZigbeeAttrValue {
        self.attrs.get(&key).copied().unwrap_or(default)
    }

    pub fn set(&mut self, key: ZigbeeAttrKey, value: ZigbeeAttrValue) {
        self.attrs.insert(key, value);
    }

    pub fn read(
        &self,
        endpoint: u16,
        cluster_id: u32,
        attribute_id: u32,
    ) -> Option<ZigbeeAttrValue> {
        self.get(ZigbeeAttrKey::new(endpoint, cluster_id, attribute_id))
    }

    pub fn write(
        &mut self,
        endpoint: u16,
        cluster_id: u32,
        attribute_id: u32,
        value: ZigbeeAttrValue,
    ) {
        self.set(
            ZigbeeAttrKey::new(endpoint, cluster_id, attribute_id),
            value,
        );
    }

    pub fn len(&self) -> usize {
        self.attrs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.attrs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zigbee::ZigbeeMap;

    #[test]
    fn seeds_from_kettle_map() {
        let map = ZigbeeMap::kettle_example().unwrap();
        let fabric = ZigbeeNetwork::from_map(&map).unwrap();
        assert_eq!(fabric.node_id, 1);
        assert_eq!(
            fabric.read(1, 0x0006, 0x0000),
            Some(ZigbeeAttrValue::Bool(true))
        );
        assert_eq!(
            fabric.read(1, 0x0402, 0x0000),
            Some(ZigbeeAttrValue::Int16(2500))
        );
        assert_eq!(
            fabric.read(1, 0x0201, 0x0012),
            Some(ZigbeeAttrValue::Int16(8000))
        );
        assert_eq!(fabric.len(), 3);
    }
}
