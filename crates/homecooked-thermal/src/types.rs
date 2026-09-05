//! Plant helpers and re-exports of schema-owned dialogue types.
//!
//! Shared catalog vocabulary and plant runtime dialogue shapes
//! (`Reservoir`, `HeatPort`, transfer offer/accept/decline/counter) are
//! defined in `homecooked-schema` and re-exported here for backward
//! compatibility. Tick energy helpers and media/band checks stay local
//! to this crate (engine support, not schema).
//!
//! See [`docs/standard/thermal-plant.md`](../../../docs/standard/thermal-plant.md)
//! §§3–6.

use crate::error::Error;

// Catalog vocabulary + plant dialogue — schema is the source of truth.
pub use homecooked_schema::{
    HeatPort, Media, PlantTypeError, PortDirection, PortRef, PowerBandW, Reservoir, ReservoirRole,
    TempBandC, TransferAccept, TransferCounter, TransferDecline, TransferOffer, TransferReply,
    TransferResult, TransferTarget,
};

/// Joules (W·s) in one kilowatt-hour.
pub const JOULES_PER_KWH: f32 = 3_600_000.0;

/// Energy (kWh) delivered at `power_w` over `dt_s` seconds.
pub fn energy_kwh(power_w: u32, dt_s: f32) -> f32 {
    power_w as f32 * dt_s / JOULES_PER_KWH
}

/// Coarse ΔT from energy using `capacity_kwh` as the energy to traverse
/// `usable_band_c`.
///
/// `ΔT_C = (E_kWh / capacity_kWh) × (T_max − T_min)`
///
/// Returns `0.0` when capacity is missing or the band span is zero.
pub fn delta_temp_c(energy_kwh: f32, capacity_kwh: Option<f32>, band: TempBandC) -> f32 {
    match capacity_kwh {
        Some(c) if c > 0.0 && band.span() > 0.0 => energy_kwh / c * band.span(),
        _ => 0.0,
    }
}

pub(crate) fn require_compatible(left: Media, right: Media) -> Result<(), Error> {
    if left.compatible_with(right) {
        Ok(())
    } else {
        Err(Error::MediaMismatch { left, right })
    }
}

pub(crate) fn require_overlap(left: TempBandC, right: TempBandC) -> Result<(), Error> {
    if left.overlaps(right) {
        Ok(())
    } else {
        Err(Error::TempBandMismatch { left, right })
    }
}

pub(crate) fn require_temp_in_band(temp_c: f32, band: TempBandC) -> Result<(), Error> {
    if band.contains(temp_c) {
        Ok(())
    } else {
        Err(Error::TempOutOfBand { temp_c, band })
    }
}
