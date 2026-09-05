//! Washer cotton cycle state machine (host sim).
//!
//! Aligns with `docs/standard/examples/washer-dryer-io.md` §6. Heat and
//! tumble are internal sub-states of catalog phase `wash`.

use serde::{Deserialize, Serialize};

/// Catalog-facing `trait.cycle.cycle_state` tokens used by this sim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CycleState {
    Idle,
    Running,
    Complete,
    Error,
}

impl CycleState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Error => "error",
        }
    }
}

/// Catalog-facing `trait.cycle.cycle_phase` tokens (heat/tumble → `wash`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CyclePhase {
    /// No active phase (idle / pre-start).
    None,
    Fill,
    Wash,
    Drain,
    Rinse,
    Spin,
    Complete,
}

impl CyclePhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Fill => "fill",
            Self::Wash => "wash",
            Self::Drain => "drain",
            Self::Rinse => "rinse",
            Self::Spin => "spin",
            Self::Complete => "complete",
        }
    }
}

/// Internal cotton runtime states (finer than catalog phase).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasherState {
    Idle,
    Lock,
    Fill,
    Heat,
    WashTumble,
    Drain,
    /// Rinse composite: fill → short tumble → drain.
    RinseFill,
    RinseTumble,
    RinseDrain,
    Spin,
    Done,
}

impl WasherState {
    pub fn catalog_phase(self) -> CyclePhase {
        match self {
            Self::Idle => CyclePhase::None,
            Self::Lock | Self::Fill => CyclePhase::Fill,
            Self::Heat | Self::WashTumble => CyclePhase::Wash,
            Self::Drain => CyclePhase::Drain,
            Self::RinseFill | Self::RinseTumble | Self::RinseDrain => CyclePhase::Rinse,
            Self::Spin => CyclePhase::Spin,
            Self::Done => CyclePhase::Complete,
        }
    }
}

/// Setpoints for a cotton program start.
#[derive(Debug, Clone, PartialEq)]
pub struct CottonOptions {
    /// Target tub temperature (°C). `0` skips the Heat state.
    pub wash_temp_c: f64,
    /// Spin setpoint (rpm). Must clear the spin interlock threshold when ≥ 400.
    pub spin_rpm: f64,
    /// Fill target on `ain.water_level_pa`.
    pub target_fill_pa: f64,
    /// Ticks spent tumbling in wash.
    pub wash_tumble_ticks: u32,
    /// Ticks spent at spin rpm.
    pub spin_ticks: u32,
    /// Ticks for rinse tumble.
    pub rinse_tumble_ticks: u32,
}

impl Default for CottonOptions {
    fn default() -> Self {
        Self {
            wash_temp_c: 40.0,
            spin_rpm: 800.0,
            target_fill_pa: 2500.0,
            wash_tumble_ticks: 3,
            spin_ticks: 3,
            rinse_tumble_ticks: 2,
        }
    }
}
