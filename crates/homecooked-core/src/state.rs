//! Per-device point value store.

use std::collections::HashMap;

use homecooked_schema::Value;

/// Map of qualified point id → current value.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeviceState {
    values: HashMap<String, Value>,
}

impl DeviceState {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get(&self, point_id: &str) -> Option<&Value> {
        self.values.get(point_id)
    }

    pub fn insert(&mut self, point_id: impl Into<String>, value: Value) -> Option<Value> {
        self.values.insert(point_id.into(), value)
    }

    pub fn remove(&mut self, point_id: &str) -> Option<Value> {
        self.values.remove(point_id)
    }

    pub fn contains(&self, point_id: &str) -> bool {
        self.values.contains_key(point_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.values.iter()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
