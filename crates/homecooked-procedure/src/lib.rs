//! Procedure documents, validation, and a sequential runner.
//!
//! First executable slice of the recipe / AI-protocol layer described in
//! [`docs/standard/procedures.md`](../../docs/standard/procedures.md).
//! Steps are ordinary HomeCooked reads, writes, and commands plus waits,
//! comparison guards, and timeouts. Parallel steps and a general expression
//! language are out of scope.

mod backend;
mod document;
mod error;
mod guard;
mod runner;
mod validate;

pub use backend::{DeviceBackend, SimulatorBackend};
pub use document::{
    ClassHint, DeviceRef, Procedure, Step, StepAction, StepTarget, BUNDLED_EXAMPLE_PROCEDURES,
    DISHWASHER_DHW_PREHEAT_JSON, KETTLE_HEAT_80_JSON, REHEAT_DOMINOS_MICROWAVE_JSON,
    WASH_THEN_DRY_JSON,
};
pub use error::Error;
pub use guard::{CmpOp, Guard, GuardSet};
pub use runner::{
    run, run_with_config, DeviceBindings, FailReason, RunConfig, RunResult, RunStatus, StepOutcome,
    DEFAULT_POLL_INTERVAL_MS,
};
pub use validate::CapabilityMap;

#[cfg(test)]
mod tests;
