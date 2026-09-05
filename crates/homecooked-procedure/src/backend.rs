//! Device I/O used by the sequential runner.

use homecooked_core::DeviceId;
use homecooked_schema::{ErrorCode, Value};
use homecooked_sim::Simulator;
use homecooked_thermal::{ThermalPlant, TransferAccept, TransferOffer, TransferReply};

use crate::error::Error;

/// Read / write / simulated-time advance against a bound device.
///
/// Optional thermal plant hooks let [`crate::document::StepAction::ThermalWait`]
/// poll reservoir temperatures and [`crate::document::StepAction::ThermalOffer`]
/// submit transfer offers without inventing parallel appliance classes.
/// Default implementations report [`ErrorCode::UnsupportedOperation`].
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

    /// Read a thermal plant reservoir temperature (°C).
    fn thermal_read_reservoir_temp(&mut self, reservoir_id: &str) -> Result<f64, Error> {
        Err(Error::Backend {
            code: ErrorCode::UnsupportedOperation,
            message: format!(
                "thermal_read_reservoir_temp not supported (reservoir {reservoir_id})"
            ),
            point_id: None,
        })
    }

    /// Advance the attached thermal plant by `dt_ms` of simulated time.
    ///
    /// Default is a no-op so device-only backends ignore thermal waits' ticks.
    fn thermal_tick(&mut self, dt_ms: u64) -> Result<(), Error> {
        let _ = dt_ms;
        Ok(())
    }

    /// Validate a [`TransferOffer`] without changing plant state.
    fn thermal_offer(&mut self, offer: &TransferOffer) -> Result<(), Error> {
        let _ = offer;
        Err(Error::Backend {
            code: ErrorCode::UnsupportedOperation,
            message: "thermal_offer not supported".into(),
            point_id: None,
        })
    }

    /// Queue an accepted transfer at `accepted_power_w` (energy applied on tick).
    fn thermal_accept(
        &mut self,
        offer: TransferOffer,
        accepted_power_w: u32,
    ) -> Result<TransferAccept, Error> {
        let _ = (offer, accepted_power_w);
        Err(Error::Backend {
            code: ErrorCode::UnsupportedOperation,
            message: "thermal_accept not supported".into(),
            point_id: None,
        })
    }

    /// Immediate path: accept at max allowable power, Counter when 0 < max < min, or decline.
    ///
    /// Mirrors [`ThermalPlant::negotiate`]. Default is unsupported.
    fn thermal_negotiate(&mut self, offer: TransferOffer) -> Result<TransferReply, Error> {
        let _ = offer;
        Err(Error::Backend {
            code: ErrorCode::UnsupportedOperation,
            message: "thermal_negotiate not supported".into(),
            point_id: None,
        })
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
///
/// Optional [`ThermalPlant`] enables thermal procedure steps (`thermal_wait`,
/// `thermal_offer`).
#[derive(Debug)]
pub struct SimulatorBackend {
    pub sim: Simulator,
    pub plant: Option<ThermalPlant>,
}

impl SimulatorBackend {
    pub fn new(sim: Simulator) -> Self {
        Self { sim, plant: None }
    }

    pub fn with_plant(sim: Simulator, plant: ThermalPlant) -> Self {
        Self {
            sim,
            plant: Some(plant),
        }
    }

    pub fn inner(&self) -> &Simulator {
        &self.sim
    }

    pub fn inner_mut(&mut self) -> &mut Simulator {
        &mut self.sim
    }

    pub fn plant(&self) -> Option<&ThermalPlant> {
        self.plant.as_ref()
    }

    pub fn plant_mut(&mut self) -> Option<&mut ThermalPlant> {
        self.plant.as_mut()
    }

    pub fn set_plant(&mut self, plant: ThermalPlant) {
        self.plant = Some(plant);
    }

    pub fn take_plant(&mut self) -> Option<ThermalPlant> {
        self.plant.take()
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

    fn thermal_read_reservoir_temp(&mut self, reservoir_id: &str) -> Result<f64, Error> {
        let plant = self.plant.as_ref().ok_or_else(|| Error::Backend {
            code: ErrorCode::UnsupportedOperation,
            message: "no thermal plant attached to SimulatorBackend".into(),
            point_id: None,
        })?;
        let reservoir = plant
            .get_reservoir(reservoir_id)
            .ok_or_else(|| Error::Backend {
                code: ErrorCode::UnknownVariable,
                message: format!("unknown reservoir {reservoir_id}"),
                point_id: Some(reservoir_id.to_string()),
            })?;
        match reservoir.temp_c {
            Some(t) => Ok(f64::from(t)),
            None => Err(Error::Backend {
                code: ErrorCode::NotReadable,
                message: format!("reservoir {reservoir_id} has no temp_c"),
                point_id: Some(reservoir_id.to_string()),
            }),
        }
    }

    fn thermal_tick(&mut self, dt_ms: u64) -> Result<(), Error> {
        let Some(plant) = self.plant.as_mut() else {
            return Ok(());
        };
        let dt_s = dt_ms as f32 / 1_000.0;
        plant.step(dt_s).map_err(|e| Error::Backend {
            code: ErrorCode::Internal,
            message: e.to_string(),
            point_id: None,
        })?;
        Ok(())
    }

    fn thermal_offer(&mut self, offer: &TransferOffer) -> Result<(), Error> {
        let plant = self.plant.as_ref().ok_or_else(|| Error::Backend {
            code: ErrorCode::UnsupportedOperation,
            message: "no thermal plant attached to SimulatorBackend".into(),
            point_id: None,
        })?;
        plant.offer(offer).map_err(|e| Error::Backend {
            code: ErrorCode::InvalidRequest,
            message: e.to_string(),
            point_id: None,
        })
    }

    fn thermal_accept(
        &mut self,
        offer: TransferOffer,
        accepted_power_w: u32,
    ) -> Result<TransferAccept, Error> {
        let plant = self.plant.as_mut().ok_or_else(|| Error::Backend {
            code: ErrorCode::UnsupportedOperation,
            message: "no thermal plant attached to SimulatorBackend".into(),
            point_id: None,
        })?;
        plant
            .accept(offer, accepted_power_w)
            .map_err(|e| Error::Backend {
                code: ErrorCode::InvalidRequest,
                message: e.to_string(),
                point_id: None,
            })
    }

    fn thermal_negotiate(&mut self, offer: TransferOffer) -> Result<TransferReply, Error> {
        let plant = self.plant.as_mut().ok_or_else(|| Error::Backend {
            code: ErrorCode::UnsupportedOperation,
            message: "no thermal plant attached to SimulatorBackend".into(),
            point_id: None,
        })?;
        Ok(plant.negotiate(offer))
    }
}
