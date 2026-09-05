//! Shared thermal **plant runtime dialogue** types (serde / public shapes).
//!
//! These are the registry + offer/accept/decline/counter message shapes other
//! crates need to speak plant **without** depending on `homecooked-thermal`
//! internals. The live plant engine (`ThermalPlant`, tick, negotiate) remains
//! in `homecooked-thermal`.
//!
//! Catalog vocabulary (`Media`, `PortDirection`, `TempBandC`) lives in
//! [`crate::thermal`]. See
//! [`docs/standard/thermal-plant.md`](../../../docs/standard/thermal-plant.md)
//! §§3–6.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::thermal::{Media, PortDirection, TempBandC};

/// Failed plant-type constructor (empty id, bad capacity / power band, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlantTypeError {
    EmptyId(&'static str),
    InvalidCapacity,
    InvalidHeadroom,
    InvalidPowerBand { min: u32, max: u32 },
    ZeroPower,
}

impl fmt::Display for PlantTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId(field) => write!(f, "{field} must be non-empty"),
            Self::InvalidCapacity => write!(f, "capacity_kwh must be > 0"),
            Self::InvalidHeadroom => write!(f, "headroom_kw must be >= 0"),
            Self::InvalidPowerBand { min, max } => {
                write!(f, "power band min {min} W > max {max} W")
            }
            Self::ZeroPower => write!(f, "transfer power must be > 0"),
        }
    }
}

impl std::error::Error for PlantTypeError {}

fn require_id(id: String, field: &'static str) -> Result<String, PlantTypeError> {
    if id.is_empty() {
        Err(PlantTypeError::EmptyId(field))
    } else {
        Ok(id)
    }
}

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
    ) -> Result<Self, PlantTypeError> {
        let id = require_id(id.into(), "reservoir id")?;
        if let Some(c) = capacity_kwh {
            if c <= 0.0 {
                return Err(PlantTypeError::InvalidCapacity);
            }
        }
        if let Some(h) = headroom_kw {
            if h < 0.0 {
                return Err(PlantTypeError::InvalidHeadroom);
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
    ) -> Result<Self, PlantTypeError> {
        let device_id = require_id(device_id.into(), "device id")?;
        let port_id = require_id(port_id.into(), "port id")?;
        if max_power_w == 0 {
            return Err(PlantTypeError::ZeroPower);
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
    pub fn new(
        device_id: impl Into<String>,
        port_id: impl Into<String>,
    ) -> Result<Self, PlantTypeError> {
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
    pub fn new(min: u32, max: u32) -> Result<Self, PlantTypeError> {
        if min > max {
            return Err(PlantTypeError::InvalidPowerBand { min, max });
        }
        if max == 0 {
            return Err(PlantTypeError::ZeroPower);
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
    pub fn port(
        device_id: impl Into<String>,
        port_id: impl Into<String>,
    ) -> Result<Self, PlantTypeError> {
        Ok(Self::Port {
            device_id: require_id(device_id.into(), "device id")?,
            port_id: require_id(port_id.into(), "port id")?,
        })
    }

    pub fn reservoir(reservoir_id: impl Into<String>) -> Result<Self, PlantTypeError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thermal::{Media, PortDirection, TempBandC};

    #[test]
    fn reservoir_role_serde_and_display() {
        for (role, token) in [
            (ReservoirRole::Hot, "hot"),
            (ReservoirRole::Cold, "cold"),
            (ReservoirRole::Dhw, "dhw"),
            (ReservoirRole::Other, "other"),
        ] {
            assert_eq!(role.as_str(), token);
            assert_eq!(role.to_string(), token);
            let json = serde_json::to_string(&role).unwrap();
            assert_eq!(json, format!("\"{token}\""));
            let back: ReservoirRole = serde_json::from_str(&json).unwrap();
            assert_eq!(back, role);
        }
    }

    #[test]
    fn reservoir_serde_roundtrip() {
        let band = TempBandC::new(20.0, 60.0).unwrap();
        let r = Reservoir::new(
            "dhw-tank",
            ReservoirRole::Dhw,
            Media::Water,
            Some(35.0),
            band,
            Some(4.0),
            Some(2.0),
        )
        .unwrap();
        let json = serde_json::to_string(&r).unwrap();
        let back: Reservoir = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
        assert!(
            Reservoir::new("", ReservoirRole::Hot, Media::Water, None, band, None, None).is_err()
        );
        assert!(Reservoir::new(
            "x",
            ReservoirRole::Hot,
            Media::Water,
            None,
            band,
            Some(0.0),
            None
        )
        .is_err());
        assert!(Reservoir::new(
            "x",
            ReservoirRole::Hot,
            Media::Water,
            None,
            band,
            None,
            Some(-1.0)
        )
        .is_err());
    }

    #[test]
    fn heat_port_and_port_ref_serde_roundtrip() {
        let band = TempBandC::new(35.0, 55.0).unwrap();
        let port = HeatPort::new(
            "fridge-kitchen",
            "condenser",
            PortDirection::Source,
            120,
            band,
            1,
            Media::Water,
            Some("dhw-tank".into()),
        )
        .unwrap();
        let json = serde_json::to_string(&port).unwrap();
        let back: HeatPort = serde_json::from_str(&json).unwrap();
        assert_eq!(back, port);
        assert_eq!(port.as_ref().to_string(), "fridge-kitchen/condenser");

        let pref = PortRef::new("fridge-kitchen", "condenser").unwrap();
        let json = serde_json::to_string(&pref).unwrap();
        let back: PortRef = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pref);
        assert!(PortRef::new("", "x").is_err());
        assert!(HeatPort::new(
            "d",
            "p",
            PortDirection::Source,
            0,
            band,
            0,
            Media::Water,
            None
        )
        .is_err());
    }

    #[test]
    fn power_band_and_transfer_target_serde() {
        let band = PowerBandW::new(80, 120).unwrap();
        let json = serde_json::to_string(&band).unwrap();
        let back: PowerBandW = serde_json::from_str(&json).unwrap();
        assert_eq!(back, band);
        assert!(PowerBandW::new(10, 5).is_err());
        assert!(PowerBandW::new(0, 0).is_err());

        let port = TransferTarget::port("water-heater-plant", "preheat").unwrap();
        let json = serde_json::to_string(&port).unwrap();
        assert!(json.contains("\"kind\":\"port\""));
        let back: TransferTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(back, port);
        assert_eq!(
            back.as_port_ref().unwrap().to_string(),
            "water-heater-plant/preheat"
        );

        let res = TransferTarget::reservoir("dhw-tank").unwrap();
        let json = serde_json::to_string(&res).unwrap();
        assert!(json.contains("\"kind\":\"reservoir\""));
        let back: TransferTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(back.reservoir_id(), Some("dhw-tank"));
    }

    #[test]
    fn transfer_dialogue_serde_roundtrip() {
        let offer = TransferOffer::new(
            PortRef::new("fridge-kitchen", "condenser").unwrap(),
            TransferTarget::port("water-heater-plant", "preheat").unwrap(),
            PowerBandW::new(80, 120).unwrap(),
            Some(3600),
            1,
        );
        let json = serde_json::to_string(&offer).unwrap();
        let back: TransferOffer = serde_json::from_str(&json).unwrap();
        assert_eq!(back, offer);

        let accept = TransferAccept {
            from_port: offer.from_port.clone(),
            to: offer.to.clone(),
            accepted_power_w: 120,
            duration_s: Some(3600),
            priority: 1,
        };
        let reply = TransferReply::Accept(accept.clone());
        let json = serde_json::to_string(&reply).unwrap();
        assert!(json.contains("\"kind\":\"accept\""));
        let back: TransferReply = serde_json::from_str(&json).unwrap();
        assert!(back.is_accept());
        assert_eq!(back, TransferReply::Accept(accept));

        let decline = TransferReply::Decline(TransferDecline::new("no headroom"));
        let json = serde_json::to_string(&decline).unwrap();
        assert!(json.contains("\"kind\":\"decline\""));
        let back: TransferReply = serde_json::from_str(&json).unwrap();
        assert!(back.is_decline());

        let counter = TransferReply::Counter(TransferCounter::new(
            PowerBandW::new(100, 100).unwrap(),
            "available below min",
        ));
        let json = serde_json::to_string(&counter).unwrap();
        assert!(json.contains("\"kind\":\"counter\""));
        let back: TransferReply = serde_json::from_str(&json).unwrap();
        assert!(back.is_counter());

        let result = TransferResult {
            from_port: offer.from_port,
            to: offer.to,
            power_w: 120,
            energy_kwh: 0.12,
            heated_reservoir_id: Some("dhw-tank".into()),
            delta_temp_c: 1.2,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: TransferResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back, result);
    }
}
