//! HomeCooked point store used by adapters after translation.

use std::collections::HashMap;

use homecooked_schema::Value;

use crate::bridge::PointRef;
use crate::error::Error;

/// Apply or read HomeCooked point updates after a foreign translation.
///
/// Tests use [`MemoryBackend`]. A later slice can wrap `homecooked-core`
/// `DeviceHub` writes behind the same trait.
pub trait PointBackend {
    fn get(&self, point: &PointRef) -> Result<Option<Value>, Error>;
    fn set(&mut self, point: &PointRef, value: Value) -> Result<(), Error>;
}

/// In-memory `(device_id, point_id) → Value` store for tests and fixtures.
#[derive(Debug, Default, Clone)]
pub struct MemoryBackend {
    points: HashMap<PointRef, Value>,
}

impl MemoryBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn get_value(&self, device_id: &str, point_id: &str) -> Option<&Value> {
        self.points.iter().find_map(|(k, v)| {
            if k.device_id == device_id && k.point_id == point_id {
                Some(v)
            } else {
                None
            }
        })
    }
}

impl PointBackend for MemoryBackend {
    fn get(&self, point: &PointRef) -> Result<Option<Value>, Error> {
        Ok(self.points.get(point).cloned())
    }

    fn set(&mut self, point: &PointRef, value: Value) -> Result<(), Error> {
        self.points.insert(point.clone(), value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_backend_roundtrip() {
        let mut store = MemoryBackend::new();
        let point = PointRef::new("dev-1", "trait.temperature.setpoint_c").unwrap();
        assert!(store.get(&point).unwrap().is_none());
        store.set(&point, Value::F32(55.0)).unwrap();
        assert_eq!(store.get(&point).unwrap(), Some(Value::F32(55.0)));
        assert_eq!(
            store.get_value("dev-1", "trait.temperature.setpoint_c"),
            Some(&Value::F32(55.0))
        );
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());
    }
}
