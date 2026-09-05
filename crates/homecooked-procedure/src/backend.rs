//! Device I/O used by the sequential runner.

use homecooked_core::DeviceId;
use homecooked_schema::Value;
use homecooked_sim::Simulator;

use crate::error::Error;

/// Read / write / simulated-time advance against a bound device.
pub trait DeviceBackend {
    fn read(&mut self, device_id: &str, point_id: &str) -> Result<Value, Error>;
    fn write(&mut self, device_id: &str, point_id: &str, value: &Value) -> Result<(), Error>;

    /// Advance device physics by `dt_ms` of simulated time.
    ///
    /// Default is a no-op so non-sim backends can ignore waits' ticks.
    fn tick(&mut self, device_id: &str, dt_ms: u64) -> Result<(), Error> {
        let _ = (device_id, dt_ms);
        Ok(())
    }
}

impl DeviceBackend for Simulator {
    fn read(&mut self, device_id: &str, point_id: &str) -> Result<Value, Error> {
        Ok(Simulator::read_value(
            self,
            &DeviceId::new(device_id),
            point_id,
        )?)
    }

    fn write(&mut self, device_id: &str, point_id: &str, value: &Value) -> Result<(), Error> {
        Simulator::write(self, &DeviceId::new(device_id), point_id, value.clone())?;
        Ok(())
    }

    fn tick(&mut self, device_id: &str, dt_ms: u64) -> Result<(), Error> {
        Simulator::tick(self, &DeviceId::new(device_id), dt_ms)?;
        Ok(())
    }
}

/// Owned wrapper so callers can name the adapter explicitly.
#[derive(Debug)]
pub struct SimulatorBackend {
    pub sim: Simulator,
}

impl SimulatorBackend {
    pub fn new(sim: Simulator) -> Self {
        Self { sim }
    }

    pub fn inner(&self) -> &Simulator {
        &self.sim
    }

    pub fn inner_mut(&mut self) -> &mut Simulator {
        &mut self.sim
    }
}

impl DeviceBackend for SimulatorBackend {
    fn read(&mut self, device_id: &str, point_id: &str) -> Result<Value, Error> {
        DeviceBackend::read(&mut self.sim, device_id, point_id)
    }

    fn write(&mut self, device_id: &str, point_id: &str, value: &Value) -> Result<(), Error> {
        DeviceBackend::write(&mut self.sim, device_id, point_id, value)
    }

    fn tick(&mut self, device_id: &str, dt_ms: u64) -> Result<(), Error> {
        DeviceBackend::tick(&mut self.sim, device_id, dt_ms)
    }
}
