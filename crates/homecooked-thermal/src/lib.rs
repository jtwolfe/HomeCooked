//! First executable **thermal plant** slice for HomeCooked.
//!
//! Reservoirs, heat ports, a best-effort offer / accept / decline dialogue,
//! and a coarse tick that moves `min(available, requested)` energy when a
//! source and sink agree. Aligns with
//! [`docs/standard/thermal-plant.md`](../../docs/standard/thermal-plant.md).
//!
//! This crate does **not** promote types into the catalog, drive
//! `homecooked-sim` class points, talk to bridges, or model CFD / plumbing.
//! Coordination remains best-effort and experimental.

#![allow(clippy::module_name_repetitions)]

mod error;
mod plant;
mod types;

pub use error::Error;
pub use plant::ThermalPlant;
pub use types::{
    delta_temp_c, energy_kwh, HeatPort, Media, PortDirection, PortRef, PowerBandW, Reservoir,
    ReservoirRole, TempBandC, TransferAccept, TransferDecline, TransferOffer, TransferReply,
    TransferResult, TransferTarget, JOULES_PER_KWH,
};

#[cfg(test)]
mod tests;
