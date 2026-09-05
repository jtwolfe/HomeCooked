//! Plant registry and transfer errors.

use std::fmt;

use crate::types::{Media, PortDirection, TempBandC};

/// Failure while registering plant objects or negotiating a transfer.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    EmptyId(&'static str),
    InvalidBand {
        min: f32,
        max: f32,
    },
    InvalidCapacity,
    InvalidHeadroom,
    InvalidPowerBand {
        min: u32,
        max: u32,
    },
    DuplicateReservoir(String),
    UnknownReservoir(String),
    DuplicatePort {
        device_id: String,
        port_id: String,
    },
    UnknownPort {
        device_id: String,
        port_id: String,
    },
    WrongDirection {
        device_id: String,
        port_id: String,
        direction: PortDirection,
        needed: &'static str,
    },
    MediaMismatch {
        left: Media,
        right: Media,
    },
    TempBandMismatch {
        left: TempBandC,
        right: TempBandC,
    },
    TempOutOfBand {
        temp_c: f32,
        band: TempBandC,
    },
    PowerExceedsMax {
        requested: u32,
        max: u32,
    },
    ZeroPower,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId(field) => write!(f, "{field} must be non-empty"),
            Self::InvalidBand { min, max } => {
                write!(f, "temperature band min {min} > max {max}")
            }
            Self::InvalidCapacity => write!(f, "capacity_kwh must be > 0"),
            Self::InvalidHeadroom => write!(f, "headroom_kw must be >= 0"),
            Self::InvalidPowerBand { min, max } => {
                write!(f, "power band min {min} W > max {max} W")
            }
            Self::DuplicateReservoir(id) => write!(f, "duplicate reservoir {id}"),
            Self::UnknownReservoir(id) => write!(f, "unknown reservoir {id}"),
            Self::DuplicatePort { device_id, port_id } => {
                write!(f, "duplicate heat port {device_id}/{port_id}")
            }
            Self::UnknownPort { device_id, port_id } => {
                write!(f, "unknown heat port {device_id}/{port_id}")
            }
            Self::WrongDirection {
                device_id,
                port_id,
                direction,
                needed,
            } => write!(
                f,
                "port {device_id}/{port_id} is {direction}, needed {needed}"
            ),
            Self::MediaMismatch { left, right } => {
                write!(f, "media mismatch: {left} vs {right}")
            }
            Self::TempBandMismatch { left, right } => {
                write!(f, "temperature band mismatch: {left} vs {right}")
            }
            Self::TempOutOfBand { temp_c, band } => {
                write!(f, "temperature {temp_c} °C outside usable band {band}")
            }
            Self::PowerExceedsMax { requested, max } => {
                write!(f, "power {requested} W exceeds max {max} W")
            }
            Self::ZeroPower => write!(f, "transfer power must be > 0"),
        }
    }
}

impl std::error::Error for Error {}
