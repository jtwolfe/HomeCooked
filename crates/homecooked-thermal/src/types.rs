//! Plant objects and the offer / accept / decline / counter dialogue.
//!
//! Shapes follow [`docs/standard/thermal-plant.md`](../../../docs/standard/thermal-plant.md)
//! §§3–6. Plant runtime types (`Reservoir`, `HeatPort`, transfer dialogue) stay
//! crate-local. Shared catalog vocabulary (`Media`, `PortDirection`,
//! `TempBandC`) is re-exported from `homecooked-schema`.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::Error;

// Catalog vocabulary — schema is the source of truth for these tokens.
pub use homecooked_schema::{Media, PortDirection, TempBandC};

/// Joules (W·s) in one kilowatt-hour.
pub const JOULES_PER_KWH: f32 = 3_600_000.0;

/// Shared thermal buffer role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReservoirRole {
    Hot,
    Cold,
    Dhw,
    Other,
}

impl ReservoirRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hot => "hot",
            Self::Cold => "cold",
            Self::Dhw => "dhw",
            Self::Other => "other",
        }
    }
}

impl fmt::Display for ReservoirRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Shared thermal buffer (plant object, not an appliance class).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reservoir {
    pub id: String,
    pub role: ReservoirRole,
    pub media: Media,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp_c: Option<f32>,
    pub usable_band_c: TempBandC,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity_kwh: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headroom_kw: Option<f32>,
}

impl Reservoir {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        role: ReservoirRole,
        media: Media,
        temp_c: Option<f32>,
        usable_band_c: TempBandC,
        capacity_kwh: Option<f32>,
        headroom_kw: Option<f32>,
    ) -> Result<Self, Error> {
        let id = require_id(id.into(), "reservoir id")?;
        if let Some(c) = capacity_kwh {
            if c <= 0.0 {
                return Err(Error::InvalidCapacity);
            }
        }
        if let Some(h) = headroom_kw {
            if h < 0.0 {
                return Err(Error::InvalidHeadroom);
            }
        }
        Ok(Self {
            id,
            role,
            media,
            temp_c,
            usable_band_c,
            capacity_kwh,
            headroom_kw,
        })
    }
}

/// Advertised heat attachment on a device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeatPort {
    pub device_id: String,
    pub port_id: String,
    pub direction: PortDirection,
    pub max_power_w: u32,
    pub usable_temp_c: TempBandC,
    pub priority: u8,
    pub media: Media,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attached_reservoir_id: Option<String>,
}

impl HeatPort {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device_id: impl Into<String>,
        port_id: impl Into<String>,
        direction: PortDirection,
        max_power_w: u32,
        usable_temp_c: TempBandC,
        priority: u8,
        media: Media,
        attached_reservoir_id: Option<String>,
    ) -> Result<Self, Error> {
        let device_id = require_id(device_id.into(), "device id")?;
        let port_id = require_id(port_id.into(), "port id")?;
        if max_power_w == 0 {
            return Err(Error::ZeroPower);
        }
        Ok(Self {
            device_id,
            port_id,
            direction,
            max_power_w,
            usable_temp_c,
            priority,
            media,
            attached_reservoir_id,
        })
    }

    pub fn as_ref(&self) -> PortRef {
        PortRef {
            device_id: self.device_id.clone(),
            port_id: self.port_id.clone(),
        }
    }
}

/// Device-local port identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortRef {
    pub device_id: String,
    pub port_id: String,
}

impl PortRef {
    pub fn new(device_id: impl Into<String>, port_id: impl Into<String>) -> Result<Self, Error> {
        Ok(Self {
            device_id: require_id(device_id.into(), "device id")?,
            port_id: require_id(port_id.into(), "port id")?,
        })
    }
}

impl fmt::Display for PortRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.device_id, self.port_id)
    }
}

/// Inclusive offered power band in watts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PowerBandW {
    pub min: u32,
    pub max: u32,
}

impl PowerBandW {
    pub fn new(min: u32, max: u32) -> Result<Self, Error> {
        if min > max {
            return Err(Error::InvalidPowerBand { min, max });
        }
        if max == 0 {
            return Err(Error::ZeroPower);
        }
        Ok(Self { min, max })
    }
}

/// Offer destination: a peer port or a plant reservoir.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum TransferTarget {
    Port { device_id: String, port_id: String },
    Reservoir { reservoir_id: String },
}

impl TransferTarget {
    pub fn port(device_id: impl Into<String>, port_id: impl Into<String>) -> Result<Self, Error> {
        Ok(Self::Port {
            device_id: require_id(device_id.into(), "device id")?,
            port_id: require_id(port_id.into(), "port id")?,
        })
    }

    pub fn reservoir(reservoir_id: impl Into<String>) -> Result<Self, Error> {
        Ok(Self::Reservoir {
            reservoir_id: require_id(reservoir_id.into(), "reservoir id")?,
        })
    }

    pub fn as_port_ref(&self) -> Option<PortRef> {
        match self {
            Self::Port { device_id, port_id } => Some(PortRef {
                device_id: device_id.clone(),
                port_id: port_id.clone(),
            }),
            Self::Reservoir { .. } => None,
        }
    }

    pub fn reservoir_id(&self) -> Option<&str> {
        match self {
            Self::Reservoir { reservoir_id } => Some(reservoir_id),
            Self::Port { .. } => None,
        }
    }
}

/// Best-effort transfer proposal (no clearing price).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferOffer {
    pub from_port: PortRef,
    pub to: TransferTarget,
    pub power_w: PowerBandW,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<u32>,
    pub priority: u8,
}

impl TransferOffer {
    pub fn new(
        from_port: PortRef,
        to: TransferTarget,
        power_w: PowerBandW,
        duration_s: Option<u32>,
        priority: u8,
    ) -> Self {
        Self {
            from_port,
            to,
            power_w,
            duration_s,
            priority,
        }
    }
}

/// Accepted transfer; `accepted_power_w` may be a partial fill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferAccept {
    pub from_port: PortRef,
    pub to: TransferTarget,
    pub accepted_power_w: u32,
    pub duration_s: Option<u32>,
    pub priority: u8,
}

/// Explicit decline; plant state is unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferDecline {
    pub reason: String,
}

impl TransferDecline {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

/// Counter-offer when the plant can supply some power but below the offered min.
///
/// Suggested band is typically `{ min: available, max: available }` so a
/// follow-up accept / re-offer can fill without silent partial below the
/// original `power_w.min`. Plant state is unchanged until Accept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferCounter {
    pub suggested_power_w: PowerBandW,
    pub reason: String,
}

impl TransferCounter {
    pub fn new(suggested_power_w: PowerBandW, reason: impl Into<String>) -> Self {
        Self {
            suggested_power_w,
            reason: reason.into(),
        }
    }
}

/// Offer reply (accept may partial-fill; counter suggests a lower band).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum TransferReply {
    Accept(TransferAccept),
    Decline(TransferDecline),
    Counter(TransferCounter),
}

impl TransferReply {
    pub fn is_accept(&self) -> bool {
        matches!(self, Self::Accept(_))
    }

    pub fn is_decline(&self) -> bool {
        matches!(self, Self::Decline(_))
    }

    pub fn is_counter(&self) -> bool {
        matches!(self, Self::Counter(_))
    }
}

/// One applied tick transfer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferResult {
    pub from_port: PortRef,
    pub to: TransferTarget,
    pub power_w: u32,
    pub energy_kwh: f32,
    pub heated_reservoir_id: Option<String>,
    pub delta_temp_c: f32,
}

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

pub(crate) fn require_id(id: String, field: &'static str) -> Result<String, Error> {
    if id.is_empty() {
        Err(Error::EmptyId(field))
    } else {
        Ok(id)
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
