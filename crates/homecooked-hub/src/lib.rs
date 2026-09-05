//! Optional lab **hub**: aggregate multiple HomeCooked sim devices behind one
//! TCP listener.
//!
//! Devices do **not** require a hub. A single [`homecooked_sim::Simulator`] plus
//! [`homecooked_transport`] is enough for one peer. This crate is a thin
//! convenience for multi-device labs: spawn several sims into one registry,
//! then reuse the existing TCP transport so Discover lists all devices and
//! Describe / Read / Write route by `device_id`.
//!
//! # Scope
//!
//! - Registry = [`Simulator`] / [`homecooked_core::DeviceHub`]
//! - Wire = [`homecooked_transport`] (optional lab PSK via [`ServerConfig`])
//! - **Not** cloud auth, TLS, or a hub UI

use std::net::{SocketAddr, ToSocketAddrs};
use std::thread::JoinHandle;

use homecooked_core::{CoreError, DeviceHub, DeviceId};
use homecooked_schema::ApplianceClassId;
use homecooked_sim::Simulator;
use homecooked_transport::{
    spawn_server_with_config, ServerConfig, SharedSim, SpawnedServer, TransportError,
};

/// Stable lab ids for [`LabHub::spawn_lab_set`].
pub const LAB_KETTLE_ID: &str = "lab-kettle";
/// Stable lab id for the washer in [`LabHub::spawn_lab_set`].
pub const LAB_WASHER_ID: &str = "lab-washer";
/// Stable lab id for the fridge in [`LabHub::spawn_lab_set`].
pub const LAB_FRIDGE_ID: &str = "lab-fridge";

/// Default bind for demos/tests (`127.0.0.1:0` → OS-assigned port).
pub const DEFAULT_BIND: &str = "127.0.0.1:0";

/// Device ids returned by [`LabHub::spawn_lab_set`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabSet {
    pub kettle: DeviceId,
    pub washer: DeviceId,
    pub fridge: DeviceId,
}

impl LabSet {
    /// All three lab device ids in spawn order.
    pub fn ids(&self) -> [&DeviceId; 3] {
        [&self.kettle, &self.washer, &self.fridge]
    }
}

/// Thin multi-device lab hub wrapping a [`Simulator`] registry.
#[derive(Debug, Default)]
pub struct LabHub {
    sim: Simulator,
}

impl LabHub {
    /// Empty hub (no devices yet).
    pub fn new() -> Self {
        Self {
            sim: Simulator::new(),
        }
    }

    /// Wrap an existing simulator (devices already registered stay registered).
    pub fn from_simulator(sim: Simulator) -> Self {
        Self { sim }
    }

    pub fn simulator(&self) -> &Simulator {
        &self.sim
    }

    pub fn simulator_mut(&mut self) -> &mut Simulator {
        &mut self.sim
    }

    /// Borrow the underlying [`DeviceHub`] registry.
    pub fn device_hub(&self) -> &DeviceHub {
        self.sim.hub()
    }

    pub fn into_simulator(self) -> Simulator {
        self.sim
    }

    /// Spawn one device with a generated id.
    pub fn spawn(&mut self, class_id: ApplianceClassId) -> Result<DeviceId, CoreError> {
        self.sim.spawn(class_id)
    }

    /// Spawn one device with a stable id (lab-friendly).
    pub fn spawn_named(
        &mut self,
        device_id: impl Into<String>,
        class_id: ApplianceClassId,
    ) -> Result<DeviceId, CoreError> {
        self.sim.spawn_named(device_id, class_id)
    }

    /// Spawn a small lab set: kettle + washer + fridge with stable ids.
    pub fn spawn_lab_set(&mut self) -> Result<LabSet, CoreError> {
        let kettle = self.spawn_named(LAB_KETTLE_ID, ApplianceClassId::Kettle)?;
        let washer = self.spawn_named(LAB_WASHER_ID, ApplianceClassId::Washer)?;
        let fridge = self.spawn_named(LAB_FRIDGE_ID, ApplianceClassId::Fridge)?;
        Ok(LabSet {
            kettle,
            washer,
            fridge,
        })
    }

    /// Registered device ids (Discover lists these).
    pub fn list(&self) -> Vec<DeviceId> {
        self.sim.list()
    }

    /// Bind TCP and serve Discover / Describe / Read / Write (open lab: no PSK).
    pub fn serve(self, addr: impl ToSocketAddrs) -> Result<SpawnedHub, TransportError> {
        self.serve_with_config(addr, ServerConfig::open())
    }

    /// Bind TCP with optional lab PSK ([`ServerConfig`]).
    pub fn serve_with_config(
        self,
        addr: impl ToSocketAddrs,
        config: ServerConfig,
    ) -> Result<SpawnedHub, TransportError> {
        let (local, shared, handle) = spawn_server_with_config(addr, self.sim, config)?;
        Ok(SpawnedHub {
            addr: local,
            sim: shared,
            join: handle,
        })
    }
}

/// Result of [`LabHub::serve`] / [`LabHub::serve_with_config`].
#[derive(Debug)]
pub struct SpawnedHub {
    pub addr: SocketAddr,
    pub sim: SharedSim,
    pub join: JoinHandle<Result<(), TransportError>>,
}

impl SpawnedHub {
    /// Local bind address (use when bound to `:0`).
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Shared simulator behind the accept loop.
    pub fn shared_sim(&self) -> &SharedSim {
        &self.sim
    }

    /// Decompose into the same tuple as [`homecooked_transport::spawn_server`].
    pub fn into_parts(self) -> SpawnedServer {
        (self.addr, self.sim, self.join)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lab_set_registers_three_devices() {
        let mut hub = LabHub::new();
        let set = hub.spawn_lab_set().unwrap();
        assert_eq!(set.kettle.as_str(), LAB_KETTLE_ID);
        assert_eq!(set.washer.as_str(), LAB_WASHER_ID);
        assert_eq!(set.fridge.as_str(), LAB_FRIDGE_ID);
        assert_eq!(hub.list().len(), 3);
        assert_eq!(hub.device_hub().registry.len(), 3);
    }
}
