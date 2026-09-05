//! Host-side **controller simulator** for HomeCooked.
//!
//! Ties together:
//! - [`homecooked_io_map::IoMap`] (washer / dryer chassis bindings)
//! - [`homecooked_hal::MockHal`] (logical channels, no GPIO)
//! - [`homecooked_interlock::washer_rules`] / [`homecooked_interlock::dryer_rules`]
//! - tick-driven washer **cotton** and dryer cycle runtimes
//!
//! Design sketches:
//! - [`docs/standard/control-system.md`](../../docs/standard/control-system.md)
//! - [`docs/standard/examples/washer-dryer-io.md`](../../docs/standard/examples/washer-dryer-io.md)
//!
//! # Protocol / TCP
//!
//! This crate exposes a direct [`Controller`] / [`DryerController`] API for
//! tests and host tools. Advertising these as HomeCooked devices over the wire
//! protocol is a deliberate follow-up. Lab TCP for protocol envelopes lives in
//! `homecooked-transport` (Stream 4 milestone 3 smoke).
//!
//! # Example
//!
//! ```
//! use homecooked_controller::{Controller, CottonOptions, DryOptions, DryerController};
//!
//! let mut washer = Controller::washer_cotton_demo().unwrap();
//! washer.start_cotton(CottonOptions::default()).unwrap();
//! washer.run_until_done(200).unwrap();
//! assert_eq!(washer.cycle_state().as_str(), "complete");
//!
//! let mut dryer = DryerController::dryer_cotton_demo().unwrap();
//! dryer.start_dry(DryOptions::default()).unwrap();
//! dryer.run_until_done(200).unwrap();
//! assert_eq!(dryer.cycle_state().as_str(), "complete");
//! ```

#![allow(clippy::module_name_repetitions)]

mod controller;
mod cycle;
mod dryer_controller;
mod error;
mod plant;

#[cfg(test)]
mod tests;

pub use controller::{write_hal, Controller};
pub use cycle::{CottonOptions, CyclePhase, CycleState, DryOptions, DryerState, WasherState};
pub use dryer_controller::{write_dryer_hal, DryerController};
pub use error::Error;
pub use plant::{DRAIN_RATE_PA, FILL_RATE_PA, WATER_PRESENT_PA};
