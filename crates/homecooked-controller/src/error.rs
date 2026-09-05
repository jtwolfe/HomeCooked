//! Controller-sim errors.

use core::fmt;

/// Failure constructing or stepping the host controller simulator.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    IoMap(String),
    Hal(homecooked_hal::Error),
    /// Cycle rejected a start or step (e.g. door open, already running).
    Cycle(String),
    /// `run_until_done` hit the tick budget before Complete.
    Timeout {
        ticks: u32,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoMap(msg) => write!(f, "io map: {msg}"),
            Self::Hal(e) => write!(f, "hal: {e}"),
            Self::Cycle(msg) => write!(f, "cycle: {msg}"),
            Self::Timeout { ticks } => {
                write!(f, "cycle did not complete within {ticks} ticks")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<homecooked_hal::Error> for Error {
    fn from(value: homecooked_hal::Error) -> Self {
        Self::Hal(value)
    }
}

impl From<homecooked_io_map::Error> for Error {
    fn from(value: homecooked_io_map::Error) -> Self {
        Self::IoMap(value.to_string())
    }
}
