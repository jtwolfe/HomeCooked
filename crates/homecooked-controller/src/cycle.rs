//! Washer cotton and dryer cotton cycle state machines (host sim).
//!
//! Aligns with `docs/standard/examples/washer-dryer-io.md` §6–§7. Washer heat
//! and tumble are internal sub-states of catalog phase `wash`. Dryer dry/heat
//! maps to catalog `heating` / `drying`.

use serde::{Deserialize, Serialize};

/// Catalog-facing `trait.cycle.cycle_state` tokens used by this sim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CycleState {
    Idle,
    Running,
    Paused,
    Complete,
    /// Abort in progress (drain / cool as device policy).
    Canceling,
    Error,
}

impl CycleState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Complete => "complete",
            Self::Canceling => "canceling",
            Self::Error => "error",
        }
    }
}

/// Catalog-facing `trait.cycle.cycle_phase` tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CyclePhase {
    /// No active phase (idle / pre-start).
    None,
    // Washer catalog phases
    Fill,
    Wash,
    Drain,
    Rinse,
    Spin,
    // Dryer catalog phases
    Heating,
    Drying,
    Cooling,
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
            Self::Heating => "heating",
            Self::Drying => "drying",
            Self::Cooling => "cooling",
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
    /// Cancel path: drain remaining water then unlock → Idle.
    CancelDrain,
    Done,
}

impl WasherState {
    pub fn catalog_phase(self) -> CyclePhase {
        match self {
            Self::Idle => CyclePhase::None,
            Self::Lock | Self::Fill => CyclePhase::Fill,
            Self::Heat | Self::WashTumble => CyclePhase::Wash,
            Self::Drain | Self::CancelDrain => CyclePhase::Drain,
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

/// Internal dryer cotton runtime states.
///
/// Wire outline: Idle → Dry/Heat → Cool → Done (lock is an internal prelude).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DryerState {
    Idle,
    Lock,
    /// Combined dry/heat: blower + heater + tumble until humidity/temp target.
    Dry,
    Cool,
    /// Cancel path: stop heat, cool/vent, then unlock → Idle.
    CancelCool,
    Done,
}

impl DryerState {
    pub fn catalog_phase(self) -> CyclePhase {
        match self {
            Self::Idle => CyclePhase::None,
            Self::Lock => CyclePhase::Heating,
            Self::Dry => CyclePhase::Drying,
            Self::Cool | Self::CancelCool => CyclePhase::Cooling,
            Self::Done => CyclePhase::Complete,
        }
    }
}

/// Setpoints for a dryer cotton program start.
#[derive(Debug, Clone, PartialEq)]
pub struct DryOptions {
    /// Exit Dry when drum temp reaches this (°C), or humidity target, whichever first.
    pub target_temp_c: f64,
    /// Exit Dry when `ain.humidity_rh` is at or below this.
    pub target_humidity_rh: f64,
    /// Exit Cool when drum temp is at or below this (°C).
    pub cool_temp_c: f64,
    /// Tumble rpm during Dry / Cool.
    pub tumble_rpm: f64,
    /// Safety cap on Dry ticks.
    pub max_dry_ticks: u32,
    /// Safety cap on Cool ticks.
    pub max_cool_ticks: u32,
}

impl Default for DryOptions {
    fn default() -> Self {
        Self {
            target_temp_c: 55.0,
            target_humidity_rh: 25.0,
            cool_temp_c: 30.0,
            tumble_rpm: 50.0,
            max_dry_ticks: 20,
            max_cool_ticks: 20,
        }
    }
}
