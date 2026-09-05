//! In-memory device registry.

use std::collections::HashMap;

use homecooked_protocol::HelloRecord;
use homecooked_schema::{CapabilityModel, DeviceIdentity};

use crate::error::CoreError;
use crate::id::DeviceId;
use crate::state::DeviceState;

/// A registered device: identity, advertised caps, and live point values.
#[derive(Debug, Clone)]
pub struct RegisteredDevice {
    pub identity: DeviceIdentity,
    pub capability: CapabilityModel,
    pub state: DeviceState,
}

impl RegisteredDevice {
    pub fn new(identity: DeviceIdentity, capability: CapabilityModel, state: DeviceState) -> Self {
        Self {
            identity,
            capability,
            state,
        }
    }

    pub fn id(&self) -> DeviceId {
        DeviceId::new(&self.identity.device_id)
    }

    pub fn hello(&self) -> HelloRecord {
        HelloRecord {
            device_id: self.identity.device_id.clone(),
            protocol_version: self.identity.protocol_version,
            catalog_version: self.identity.catalog_version,
            class_id: self.identity.class_id,
            trait_ids: self.capability.traits.iter().map(|t| t.trait_id).collect(),
            display_name: self.identity.display_name.clone(),
            endpoint: None,
        }
    }
}

/// Register / unregister / list / get devices by [`DeviceId`].
#[derive(Debug, Default)]
pub struct DeviceRegistry {
    devices: HashMap<DeviceId, RegisteredDevice>,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        identity: DeviceIdentity,
        capability: CapabilityModel,
        state: DeviceState,
    ) -> Result<DeviceId, CoreError> {
        identity
            .validate()
            .map_err(|e| CoreError::invalid_request(e.to_string()))?;
        let id = DeviceId::new(&identity.device_id);
        if self.devices.contains_key(&id) {
            return Err(CoreError::invalid_request(format!(
                "device {} already registered",
                identity.device_id
            )));
        }
        self.devices.insert(
            id.clone(),
            RegisteredDevice::new(identity, capability, state),
        );
        Ok(id)
    }

    pub fn unregister(&mut self, id: &DeviceId) -> Result<RegisteredDevice, CoreError> {
        self.devices
            .remove(id)
            .ok_or_else(|| CoreError::unknown_device(id.as_str()))
    }

    pub fn get(&self, id: &DeviceId) -> Option<&RegisteredDevice> {
        self.devices.get(id)
    }

    pub fn get_mut(&mut self, id: &DeviceId) -> Option<&mut RegisteredDevice> {
        self.devices.get_mut(id)
    }

    pub fn require(&self, id: &DeviceId) -> Result<&RegisteredDevice, CoreError> {
        self.get(id)
            .ok_or_else(|| CoreError::unknown_device(id.as_str()))
    }

    pub fn require_mut(&mut self, id: &DeviceId) -> Result<&mut RegisteredDevice, CoreError> {
        let key = id.as_str().to_string();
        self.get_mut(id)
            .ok_or_else(|| CoreError::unknown_device(&key))
    }

    pub fn list(&self) -> Vec<&RegisteredDevice> {
        let mut devices: Vec<_> = self.devices.values().collect();
        devices.sort_by(|a, b| a.identity.device_id.cmp(&b.identity.device_id));
        devices
    }

    pub fn ids(&self) -> Vec<DeviceId> {
        self.list().into_iter().map(|d| d.id()).collect()
    }

    pub fn len(&self) -> usize {
        self.devices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    pub fn contains(&self, id: &DeviceId) -> bool {
        self.devices.contains_key(id)
    }
}
