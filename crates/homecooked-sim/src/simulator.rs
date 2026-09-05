//! In-memory simulated devices driven by catalog typical capabilities.

use homecooked_core::{CoreError, DeviceHub, DeviceId, DeviceState};
use homecooked_protocol::{Envelope, Payload, ReadRequest, WriteOp, WriteRequest, WriteResponse};
use homecooked_schema::{typical_capability, ApplianceClassId, Value, STATIC_CLASS_IDS};

use crate::behavior::{apply_writes, tick_device};
use crate::defaults::{seed_identity, seed_state, sim_capability};

/// Catalog-backed in-memory appliance simulator.
#[derive(Debug, Default)]
pub struct Simulator {
    hub: DeviceHub,
    next_seq: u64,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            hub: DeviceHub::new(),
            next_seq: 1,
        }
    }

    pub fn hub(&self) -> &DeviceHub {
        &self.hub
    }

    pub fn hub_mut(&mut self) -> &mut DeviceHub {
        &mut self.hub
    }

    /// Spawn one simulated device of `class_id` with a generated id.
    pub fn spawn(&mut self, class_id: ApplianceClassId) -> Result<DeviceId, CoreError> {
        let seq = self.next_seq;
        self.next_seq += 1;
        let device_id = format!("sim-{}-{seq}", class_id.as_str());
        self.spawn_named(device_id, class_id)
    }

    pub fn spawn_named(
        &mut self,
        device_id: impl Into<String>,
        class_id: ApplianceClassId,
    ) -> Result<DeviceId, CoreError> {
        if typical_capability(class_id).is_none() {
            return Err(CoreError::invalid_request(format!(
                "no static catalog table for class {}",
                class_id.as_str()
            )));
        }
        let identity = seed_identity(&device_id.into(), class_id);
        let cap = sim_capability(class_id).ok_or_else(|| {
            CoreError::invalid_request(format!("no sim capability for class {}", class_id.as_str()))
        })?;
        let state = seed_state(&identity, &cap);
        self.hub.registry.register(identity, cap, state)
    }

    /// Spawn one device for each of the nine static classes.
    pub fn spawn_static_kitchen(&mut self) -> Result<Vec<DeviceId>, CoreError> {
        STATIC_CLASS_IDS.iter().map(|id| self.spawn(*id)).collect()
    }

    pub fn list(&self) -> Vec<DeviceId> {
        self.hub.registry.ids()
    }

    pub fn handle(&mut self, request: Envelope) -> Envelope {
        let writes = match &request.payload {
            Payload::Write(body) if !body.dry_run => {
                Some((request.device_id.clone(), body.writes.clone()))
            }
            _ => None,
        };
        let response = self.hub.handle(&request);
        if matches!(response.payload, Payload::WriteOk(_)) {
            if let Some((Some(device_id), ops)) = writes {
                if let Some(dev) = self.hub.registry.get_mut(&DeviceId::new(device_id)) {
                    apply_writes(dev, &ops);
                }
            }
        }
        response
    }

    pub fn read(
        &self,
        id: &DeviceId,
        points: &[&str],
    ) -> Result<Vec<(String, Option<Value>)>, CoreError> {
        let req = ReadRequest {
            points: points
                .iter()
                .map(|p| p.parse())
                .collect::<Result<Vec<_>, homecooked_schema::ParseIdError>>()
                .map_err(|e| CoreError::invalid_request(e.to_string()))?,
            allow_partial: false,
        };
        let resp = self.hub.read_points(id, &req)?;
        Ok(resp
            .values
            .into_iter()
            .map(|v| (v.id.to_string(), v.value))
            .collect())
    }

    pub fn read_value(&self, id: &DeviceId, point_id: &str) -> Result<Value, CoreError> {
        let rows = self.read(id, &[point_id])?;
        rows.into_iter()
            .next()
            .and_then(|(_, v)| v)
            .ok_or_else(|| CoreError::invalid_request(format!("no value for {point_id}")))
    }

    pub fn write(
        &mut self,
        id: &DeviceId,
        point_id: &str,
        value: Value,
    ) -> Result<WriteResponse, CoreError> {
        self.write_many(id, &[(point_id, value)])
    }

    pub fn write_many(
        &mut self,
        id: &DeviceId,
        writes: &[(&str, Value)],
    ) -> Result<WriteResponse, CoreError> {
        let ops: Result<Vec<WriteOp>, CoreError> = writes
            .iter()
            .map(|(pid, value)| {
                Ok(WriteOp {
                    id: pid.parse().map_err(|e: homecooked_schema::ParseIdError| {
                        CoreError::invalid_request(e.to_string())
                    })?,
                    value: value.clone(),
                })
            })
            .collect();
        let env = Envelope::request(
            Some(id.as_str().to_string()),
            Payload::Write(WriteRequest {
                writes: ops?,
                dry_run: false,
                atomic: false,
            }),
        );
        match self.handle(env).payload {
            Payload::WriteOk(ok) => Ok(ok),
            Payload::Error(err) => Err(err.into()),
            other => Err(CoreError::new(
                homecooked_schema::ErrorCode::Internal,
                format!("unexpected write response {other:?}"),
            )),
        }
    }

    pub fn tick(&mut self, id: &DeviceId, dt_ms: u64) -> Result<(), CoreError> {
        self.hub.clock_ms = self.hub.clock_ms.saturating_add(dt_ms);
        let dev = self.hub.registry.require_mut(id)?;
        tick_device(dev, dt_ms);
        Ok(())
    }

    pub fn tick_all(&mut self, dt_ms: u64) {
        self.hub.clock_ms = self.hub.clock_ms.saturating_add(dt_ms);
        let ids = self.hub.registry.ids();
        for id in ids {
            if let Some(dev) = self.hub.registry.get_mut(&id) {
                tick_device(dev, dt_ms);
            }
        }
    }

    pub fn state(&self, id: &DeviceId) -> Option<&DeviceState> {
        self.hub.registry.get(id).map(|d| &d.state)
    }
}
