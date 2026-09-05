//! Host-side **controller simulator** for HomeCooked.
//!
//! Ties together:
//! - [`homecooked_io_map::IoMap`] (washer chassis bindings)
//! - [`homecooked_hal::MockHal`] (logical channels, no GPIO)
//! - [`homecooked_interlock::washer_rules`] (heater / spin gates)
//! - a tick-driven washer **cotton** cycle runtime
//!
//! Design sketches:
//! - [`docs/standard/control-system.md`](../../docs/standard/control-system.md)
//! - [`docs/standard/examples/washer-dryer-io.md`](../../docs/standard/examples/washer-dryer-io.md) §6
//!
//! # Protocol / TCP
//!
//! This crate exposes a direct [`Controller`] API for tests and host tools.
//! Advertising as a HomeCooked washer over the wire protocol (and TCP
//! transport) is a deliberate follow-up — Stream 4 milestone 3 in
//! `docs/ROADMAP.md`.
//!
//! # Example
//!
//! ```
//! use homecooked_controller::{Controller, CottonOptions};
//!
//! let mut ctrl = Controller::washer_cotton_demo().unwrap();
//! ctrl.start_cotton(CottonOptions::default()).unwrap();
//! ctrl.run_until_done(200).unwrap();
//! assert_eq!(ctrl.cycle_state().as_str(), "complete");
//! ```

#![allow(clippy::module_name_repetitions)]

mod controller;
mod cycle;
mod error;
mod plant;

#[cfg(test)]
mod tests;

pub use controller::{write_hal, Controller};
pub use cycle::{CottonOptions, CyclePhase, CycleState, WasherState};
pub use error::Error;
pub use plant::{DRAIN_RATE_PA, FILL_RATE_PA, WATER_PRESENT_PA};
