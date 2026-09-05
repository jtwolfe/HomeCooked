//! Shared thermal **vocabulary** types for catalog / docs alignment.
//!
//! These enums match tokens already used on device `thermal_port_media` and
//! `thermal_port_direction` catalog points. Plant **dialogue** shapes
//! (reservoirs, transfer offer/accept/…) live in [`crate::plant`]; the live
//! `ThermalPlant` engine / tick remains in `homecooked-thermal`.
//!
//! See [`docs/standard/thermal-plant.md`](../../../docs/standard/thermal-plant.md).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ids::ParseIdError;

/// Tokens for `thermal_port_media` / plant media fields.
pub const THERMAL_PORT_MEDIA_TOKENS: &[&str] =
    &["water", "air", "glycol", "refrigerant_proxy", "unknown"];

/// Tokens for `thermal_port_direction` / heat-port direction fields.
pub const THERMAL_PORT_DIRECTION_TOKENS: &[&str] = &["source", "sink", "bidirectional"];

/// Working fluid / air advertised on a port or reservoir.
///
/// Serde / `as_str` tokens match catalog `thermal_port_media`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Media {
    Water,
    Air,
    Glycol,
    RefrigerantProxy,
    Unknown,
}

impl Media {
    pub const ALL: &'static [Self] = &[
        Self::Water,
        Self::Air,
        Self::Glycol,
        Self::RefrigerantProxy,
        Self::Unknown,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Water => "water",
            Self::Air => "air",
            Self::Glycol => "glycol",
            Self::RefrigerantProxy => "refrigerant_proxy",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str_id(s: &str) -> Option<Self> {
        match s {
            "water" => Some(Self::Water),
            "air" => Some(Self::Air),
            "glycol" => Some(Self::Glycol),
            "refrigerant_proxy" => Some(Self::RefrigerantProxy),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    /// Equal medias match. [`Media::Unknown`] is compatible with anything
    /// (best-effort sketch; isolation metadata is still the caller's job).
    pub fn compatible_with(self, other: Self) -> bool {
        self == other || self == Self::Unknown || other == Self::Unknown
    }
}

impl fmt::Display for Media {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Media {
    type Err = ParseIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_id(s).ok_or_else(|| ParseIdError {
            kind: "Media",
            value: s.to_string(),
        })
    }
}

/// Direction from the appliance's point of view on a heat port.
///
/// Serde / `as_str` tokens match catalog `thermal_port_direction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
    Source,
    Sink,
    Bidirectional,
}

impl PortDirection {
    pub const ALL: &'static [Self] = &[Self::Source, Self::Sink, Self::Bidirectional];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Sink => "sink",
            Self::Bidirectional => "bidirectional",
        }
    }

    pub fn from_str_id(s: &str) -> Option<Self> {
        match s {
            "source" => Some(Self::Source),
            "sink" => Some(Self::Sink),
            "bidirectional" => Some(Self::Bidirectional),
            _ => None,
        }
    }

    pub const fn can_source(self) -> bool {
        matches!(self, Self::Source | Self::Bidirectional)
    }

    pub const fn can_sink(self) -> bool {
        matches!(self, Self::Sink | Self::Bidirectional)
    }
}

impl fmt::Display for PortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PortDirection {
    type Err = ParseIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_id(s).ok_or_else(|| ParseIdError {
            kind: "PortDirection",
            value: s.to_string(),
        })
    }
}

/// Failed [`TempBandC::new`] when `min > max`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidTempBand {
    pub min: f32,
    pub max: f32,
}

impl fmt::Display for InvalidTempBand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "temperature band min {} > max {}", self.min, self.max)
    }
}

impl std::error::Error for InvalidTempBand {}

/// Inclusive temperature band in °C (sketch; plant runtime may wrap validation).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TempBandC {
    pub min: f32,
    pub max: f32,
}

impl TempBandC {
    pub fn new(min: f32, max: f32) -> Result<Self, InvalidTempBand> {
        if min > max {
            return Err(InvalidTempBand { min, max });
        }
        Ok(Self { min, max })
    }

    pub fn contains(self, temp_c: f32) -> bool {
        temp_c >= self.min && temp_c <= self.max
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.min <= other.max && other.min <= self.max
    }

    /// `max - min`, used as the span in the thermal-mass proxy.
    pub fn span(self) -> f32 {
        self.max - self.min
    }
}

impl fmt::Display for TempBandC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}] °C", self.min, self.max)
    }
}

/// Static descriptor for a device heat port on [`crate::ClassTable`].
///
/// Advertisement metadata only — catalog `thermal_port_*` points remain the
/// device RW surface. Vocabulary aligns with those optional points / sim seeds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct HeatPortSpec {
    pub port_id: &'static str,
    pub direction: PortDirection,
    pub media: Media,
    pub max_power_w: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usable_temp_c: Option<TempBandC>,
}

impl HeatPortSpec {
    pub const fn new(
        port_id: &'static str,
        direction: PortDirection,
        media: Media,
        max_power_w: u32,
        usable_temp_c: Option<TempBandC>,
    ) -> Self {
        Self {
            port_id,
            direction,
            media,
            max_power_w,
            usable_temp_c,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_as_str_and_tokens_align() {
        assert_eq!(Media::ALL.len(), THERMAL_PORT_MEDIA_TOKENS.len());
        for (media, token) in Media::ALL.iter().zip(THERMAL_PORT_MEDIA_TOKENS) {
            assert_eq!(media.as_str(), *token);
            assert_eq!(Media::from_str_id(token), Some(*media));
            assert_eq!(token.parse::<Media>().unwrap(), *media);
        }
        assert!(Media::from_str_id("steam").is_none());
    }

    #[test]
    fn media_serde_roundtrip() {
        for media in Media::ALL {
            let json = serde_json::to_string(media).unwrap();
            assert_eq!(json, format!("\"{}\"", media.as_str()));
            let back: Media = serde_json::from_str(&json).unwrap();
            assert_eq!(back, *media);
        }
    }

    #[test]
    fn media_compatible_with() {
        assert!(Media::Water.compatible_with(Media::Water));
        assert!(!Media::Water.compatible_with(Media::Air));
        assert!(Media::Water.compatible_with(Media::Unknown));
        assert!(Media::Unknown.compatible_with(Media::Glycol));
        assert!(Media::Unknown.compatible_with(Media::Unknown));
    }

    #[test]
    fn port_direction_as_str_and_tokens_align() {
        assert_eq!(
            PortDirection::ALL.len(),
            THERMAL_PORT_DIRECTION_TOKENS.len()
        );
        for (dir, token) in PortDirection::ALL.iter().zip(THERMAL_PORT_DIRECTION_TOKENS) {
            assert_eq!(dir.as_str(), *token);
            assert_eq!(PortDirection::from_str_id(token), Some(*dir));
            assert_eq!(token.parse::<PortDirection>().unwrap(), *dir);
        }
        assert!(PortDirection::Source.can_source());
        assert!(!PortDirection::Source.can_sink());
        assert!(PortDirection::Sink.can_sink());
        assert!(!PortDirection::Sink.can_source());
        assert!(PortDirection::Bidirectional.can_source());
        assert!(PortDirection::Bidirectional.can_sink());
    }

    #[test]
    fn port_direction_serde_roundtrip() {
        for dir in PortDirection::ALL {
            let json = serde_json::to_string(dir).unwrap();
            assert_eq!(json, format!("\"{}\"", dir.as_str()));
            let back: PortDirection = serde_json::from_str(&json).unwrap();
            assert_eq!(back, *dir);
        }
    }

    #[test]
    fn temp_band_new_contains_overlaps_span() {
        let band = TempBandC::new(20.0, 60.0).unwrap();
        assert!(band.contains(20.0));
        assert!(band.contains(60.0));
        assert!(!band.contains(19.9));
        assert!(band.overlaps(TempBandC::new(50.0, 70.0).unwrap()));
        assert!(!band.overlaps(TempBandC::new(61.0, 70.0).unwrap()));
        assert_eq!(band.span(), 40.0);
        assert!(TempBandC::new(10.0, 5.0).is_err());
    }

    #[test]
    fn temp_band_and_heat_port_spec_serde_roundtrip() {
        let band = TempBandC::new(-5.0, 40.0).unwrap();
        let json = serde_json::to_string(&band).unwrap();
        let back: TempBandC = serde_json::from_str(&json).unwrap();
        assert_eq!(back, band);

        let spec = HeatPortSpec::new(
            "condenser",
            PortDirection::Source,
            Media::Water,
            120,
            Some(band),
        );
        let json = serde_json::to_string(&spec).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["port_id"], "condenser");
        assert_eq!(v["direction"], "source");
        assert_eq!(v["media"], "water");
        assert_eq!(v["max_power_w"], 120);
        assert_eq!(v["usable_temp_c"]["min"], -5.0);
        assert_eq!(v["usable_temp_c"]["max"], 40.0);
        assert_eq!(spec.port_id, "condenser");
    }
}
