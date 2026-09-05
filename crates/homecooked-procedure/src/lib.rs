//! Procedure documents, validation, and a sequential runner.
//!
//! First executable slice of the recipe / AI-protocol layer described in
//! [`docs/standard/procedures.md`](../../docs/standard/procedures.md).
//! Steps are ordinary HomeCooked reads, writes, and commands plus waits,
//! comparison guards, timeouts, and thin thermal plant bridges (`thermal_wait`
//! with optional continuous `requeue_offer`, `thermal_offer` with soft decline /
//! fallback / Counter `accept_counter`) when a thermal backend is attached. Parallel steps and a
//! general expression language are out of scope.

mod backend;
mod document;
mod error;
mod guard;
mod runner;
mod validate;

pub use backend::{DeviceBackend, SimulatorBackend};
pub use document::{
    ClassHint, DeviceRef, OnDecline, Procedure, Step, StepAction, StepTarget, ThermalCmp,
    AIR_FRYER_COOK_200_JSON, BUNDLED_EXAMPLE_PROCEDURES, COFFEE_BREW_ESPRESSO_JSON,
    DISHWASHER_DHW_PREHEAT_JSON, KETTLE_HEAT_80_JSON, OFFER_FRIDGE_DHW_COUNTER_JSON,
    OFFER_FRIDGE_DHW_JSON, OFFER_FRIDGE_DHW_SOFT_JSON, OVEN_BAKE_180_JSON,
    REHEAT_DOMINOS_MICROWAVE_JSON, WAIT_DHW_RESERVOIR_JSON, WAIT_DHW_WITH_REQUEUE_JSON,
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
