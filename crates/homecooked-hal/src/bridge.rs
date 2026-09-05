//! Layering note: I/O map sits **above** the HAL.
//!
//! Per [`docs/standard/control-system.md`](../../../docs/standard/control-system.md)
//! §4.3–4.4:
//!
//! - The HAL exposes logical channel ids (`din.door_closed`, …).
//! - `homecooked-io-map` binds those channels to board pins and optional
//!   HomeCooked points. Higher layers (interlock, cycle runtime, device role)
//!   speak points / internal signals and resolve to HAL channels via the map.
//!
//! This crate does **not** depend on `homecooked-io-map`. Controllers should
//! load an `IoMap`, then call [`crate::Hal`] with the binding's `channel`
//! string parsed as [`crate::ChannelId`].
//!
//! The helpers below only parse channel strings — they do not load YAML.

use crate::channel::ChannelId;
use crate::error::Error;
use crate::hal::Hal;
use crate::value::HalValue;

/// Parse `channel` and [`Hal::read`] it.
pub fn read_channel(hal: &impl Hal, channel: &str) -> Result<HalValue, Error> {
    let id = ChannelId::new(channel)?;
    hal.read(&id)
}

/// Parse `channel` and [`Hal::write`] it.
pub fn write_channel(
    hal: &mut impl Hal,
    channel: &str,
    value: impl Into<HalValue>,
) -> Result<(), Error> {
    let id = ChannelId::new(channel)?;
    hal.write(&id, value.into())
}
