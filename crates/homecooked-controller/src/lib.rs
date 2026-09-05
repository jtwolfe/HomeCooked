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
//! [`ControllerEndpoint`] / [`DryerControllerEndpoint`] are thin lab
//! device-role adapters: protocol describe/read/write map onto MockHal
//! channels so interlock denies surface as `safety_interlock` over
//! [`homecooked_transport`] TCP (`spawn_handler_server`). Full catalog
//! device-role / cycle-over-TCP depth remains follow-up. Lab only — no TLS /
//! OAuth.
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
mod dryer_endpoint;
mod endpoint;
mod error;
mod plant;

#[cfg(test)]
mod tests;

pub use controller::{write_hal, Controller};
pub use cycle::{CottonOptions, CyclePhase, CycleState, DryOptions, DryerState, WasherState};
pub use dryer_controller::{write_dryer_hal, DryerController};
pub use dryer_endpoint::{lab_dryer_capability, DryerControllerEndpoint, DRYER_CTRL_DEVICE_ID};
pub use endpoint::{lab_washer_capability, ControllerEndpoint, WASHER_CTRL_DEVICE_ID};
pub use error::Error;
pub use plant::{DRAIN_RATE_PA, FILL_RATE_PA, WATER_PRESENT_PA};
