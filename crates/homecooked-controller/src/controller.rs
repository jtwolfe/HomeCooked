//! Host controller: IoMap + MockHal + washer interlocks + cotton cycle.

use homecooked_hal::{bridge, ChannelId, Hal, HalValue, MockHal, MotorCommand};
use homecooked_io_map::{IoMap, WASHER_FRAGMENT_YAML};

use crate::cycle::{CottonOptions, CyclePhase, CycleState, WasherState};
use crate::error::Error;
use crate::plant::{self, WATER_PRESENT_PA};

/// Extra HAL channels needed for cotton that the washer fragment may omit.
const EXTRA_CHANNELS: &[&str] = &[
    "aout.drain_pump",
    "motor.enable",
    "motor.direction",
    "dout.buzzer",
];

/// Host-side stand-in for the universal control computer (washer role).
///
/// Loads an [`IoMap`], holds a [`MockHal`] gated by
/// [`homecooked_interlock::washer_rules`], and runs a tick-driven cotton
/// cycle. Protocol / TCP glue is intentionally out of scope — drive this
/// API from tests (or a follow-up transport crate).
#[derive(Debug)]
pub struct Controller {
    io_map: IoMap,
    hal: MockHal,
    state: WasherState,
    cycle_state: CycleState,
    opts: CottonOptions,
    /// Ticks spent in the current timed sub-state (tumble / spin).
    dwell: u32,
    /// Logical sim clock advanced each [`Self::tick`].
    tick: u64,
}

impl Controller {
    /// Washer demo: fragment IoMap + washer interlocks + idle cotton runtime.
    pub fn washer_cotton_demo() -> Result<Self, Error> {
        let map = IoMap::from_yaml_str(WASHER_FRAGMENT_YAML)?;
        Self::from_io_map(map)
    }

    /// Build from a validated IoMap; register channels and attach interlocks.
    pub fn from_io_map(map: IoMap) -> Result<Self, Error> {
        let mut hal = MockHal::new();
        for binding in &map.bindings {
            hal.register_str(&binding.channel)?;
        }
        for ch in EXTRA_CHANNELS {
            let _ = hal.register_str(ch);
        }
        hal.set_interlocks(Some(homecooked_interlock::washer_rules()));
        hal.set_tick_ms(Some(0));

        // Sensible ambient / door defaults for a parked machine.
        let door = ChannelId::new("din.door_closed")?;
        if hal.get(&door).is_ok() {
            hal.inject(&door, true)?;
        }
        let temp = ChannelId::new("ain.tub_temp_c")?;
        if hal.get(&temp).is_ok() {
            hal.inject(&temp, 20.0)?;
        }

        plant::refresh_derived(&mut hal)?;

        Ok(Self {
            io_map: map,
            hal,
            state: WasherState::Idle,
            cycle_state: CycleState::Idle,
            opts: CottonOptions::default(),
            dwell: 0,
            tick: 0,
        })
    }

    pub fn hal(&self) -> &MockHal {
        &self.hal
    }

    pub fn hal_mut(&mut self) -> &mut MockHal {
        &mut self.hal
    }

    pub fn io_map(&self) -> &IoMap {
        &self.io_map
    }

    pub fn washer_state(&self) -> WasherState {
        self.state
    }

    pub fn phase(&self) -> CyclePhase {
        self.state.catalog_phase()
    }

    pub fn cycle_state(&self) -> CycleState {
        self.cycle_state
    }

    pub fn options(&self) -> &CottonOptions {
        &self.opts
    }

    /// Start the cotton program. Requires door closed when that channel exists.
    pub fn start_cotton(&mut self, opts: CottonOptions) -> Result<(), Error> {
        if matches!(
            self.cycle_state,
            CycleState::Running | CycleState::Paused | CycleState::Canceling
        ) {
            return Err(Error::Cycle("cycle already running".into()));
        }
        if let Ok(closed) = bridge::read_channel(&self.hal, "din.door_closed") {
            if closed.as_bool() != Some(true) {
                return Err(Error::Cycle("door must be closed to start".into()));
            }
        }
        self.opts = opts;
        self.dwell = 0;
        self.state = WasherState::Lock;
        self.cycle_state = CycleState::Running;
        Ok(())
    }

    /// Pause a running cycle. Motors / heater / inlet / drain stop; door stays
    /// locked if locked. Idempotent when already paused.
    pub fn pause(&mut self) -> Result<(), Error> {
        match self.cycle_state {
            CycleState::Running => {
                self.hold_actuators_safe()?;
                self.cycle_state = CycleState::Paused;
                Ok(())
            }
            CycleState::Paused => Ok(()),
            _ => Err(Error::Cycle("cycle not running".into())),
        }
    }

    /// Resume a paused cycle. Idempotent when already running is not offered —
    /// resume requires [`CycleState::Paused`].
    pub fn resume(&mut self) -> Result<(), Error> {
        match self.cycle_state {
            CycleState::Paused => {
                self.cycle_state = CycleState::Running;
                Ok(())
            }
            _ => Err(Error::Cycle("cycle not paused".into())),
        }
    }

    /// Abort the cycle: enter [`CycleState::Canceling`], drain, then unlock →
    /// [`CycleState::Idle`]. Idempotent while already canceling. Denied when
    /// idle / complete / error (no active cycle).
    pub fn cancel(&mut self) -> Result<(), Error> {
        match self.cycle_state {
            CycleState::Running | CycleState::Paused => {
                self.enter_cancel()?;
                Ok(())
            }
            CycleState::Canceling => Ok(()),
            _ => Err(Error::Cycle("no active cycle to cancel".into())),
        }
    }

    /// One sim step: plant feedback → derived keys → cycle commands → plant again.
    pub fn tick(&mut self) -> Result<(), Error> {
        self.tick = self.tick.saturating_add(1);
        self.hal.set_tick_ms(Some(self.tick));

        // Plant from prior actuator state, then refresh interlock keys.
        plant::step_plant(&mut self.hal)?;
        plant::refresh_derived(&mut self.hal)?;

        match self.cycle_state {
            CycleState::Running => {
                self.advance_cycle()?;
                // Apply plant once more so sensors reflect this tick's commands
                // (lock_fb, level) before the caller inspects state.
                plant::step_plant(&mut self.hal)?;
                plant::refresh_derived(&mut self.hal)?;
            }
            CycleState::Canceling => {
                self.advance_cancel()?;
                plant::step_plant(&mut self.hal)?;
                plant::refresh_derived(&mut self.hal)?;
            }
            CycleState::Paused => {
                // Keep a safe hold; do not advance phase.
                self.hold_actuators_safe()?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Tick until [`CycleState::Complete`] or `max_ticks`.
    pub fn run_until_done(&mut self, max_ticks: u32) -> Result<(), Error> {
        for _ in 0..max_ticks {
            if self.cycle_state == CycleState::Complete {
                return Ok(());
            }
            if self.cycle_state == CycleState::Error {
                return Err(Error::Cycle("cycle entered error".into()));
            }
            self.tick()?;
        }
        if self.cycle_state == CycleState::Complete {
            Ok(())
        } else {
            Err(Error::Timeout { ticks: max_ticks })
        }
    }

    /// Attempt heater enable via HAL (for interlock tests).
    pub fn try_heater_on(&mut self) -> Result<(), Error> {
        plant::refresh_derived(&mut self.hal)?;
        bridge::write_channel(&mut self.hal, "aout.heater_enable", true)?;
        Ok(())
    }

    /// Attempt spin-band motor command via HAL (for interlock tests).
    pub fn try_spin_rpm(&mut self, rpm: f64) -> Result<(), Error> {
        plant::refresh_derived(&mut self.hal)?;
        self.hal.apply_motor_command(
            "motor",
            MotorCommand {
                enable: Some(true),
                speed_rpm: Some(rpm),
                direction: None,
            },
        )?;
        Ok(())
    }

    fn advance_cycle(&mut self) -> Result<(), Error> {
        match self.state {
            WasherState::Idle | WasherState::Done | WasherState::CancelDrain => Ok(()),
            WasherState::Lock => self.step_lock(),
            WasherState::Fill => self.step_fill(),
            WasherState::Heat => self.step_heat(),
            WasherState::WashTumble => self.step_wash_tumble(),
            WasherState::Drain => self.step_drain(WasherState::RinseFill),
            WasherState::RinseFill => self.step_rinse_fill(),
            WasherState::RinseTumble => self.step_rinse_tumble(),
            WasherState::RinseDrain => self.step_drain(WasherState::Spin),
            WasherState::Spin => self.step_spin(),
        }
    }

    fn enter_cancel(&mut self) -> Result<(), Error> {
        self.hold_actuators_safe()?;
        self.state = WasherState::CancelDrain;
        self.cycle_state = CycleState::Canceling;
        self.dwell = 0;
        Ok(())
    }

    fn advance_cancel(&mut self) -> Result<(), Error> {
        // Drain remaining water (interlocks still apply), then unlock → idle.
        let _ = bridge::write_channel(&mut self.hal, "aout.heater_enable", false);
        let _ = bridge::write_channel(&mut self.hal, "aout.cold_inlet", false);
        self.motor_off()?;
        bridge::write_channel(&mut self.hal, "aout.drain_pump", true)?;

        let level = bridge::read_channel(&self.hal, "ain.water_level_pa")?
            .as_number()
            .unwrap_or(0.0);
        if level < WATER_PRESENT_PA {
            bridge::write_channel(&mut self.hal, "aout.drain_pump", false)?;
            bridge::write_channel(&mut self.hal, "aout.door_lock", false)?;
            self.state = WasherState::Idle;
            self.cycle_state = CycleState::Idle;
            self.dwell = 0;
        }
        Ok(())
    }

    /// Safe hold for pause / cancel entry: heaters, motors, inlet, drain off.
    /// Door lock is left as-is (stays locked mid-cycle).
    fn hold_actuators_safe(&mut self) -> Result<(), Error> {
        let _ = bridge::write_channel(&mut self.hal, "aout.heater_enable", false);
        let _ = bridge::write_channel(&mut self.hal, "aout.cold_inlet", false);
        let _ = bridge::write_channel(&mut self.hal, "aout.drain_pump", false);
        self.motor_off()?;
        Ok(())
    }

    fn step_lock(&mut self) -> Result<(), Error> {
        bridge::write_channel(&mut self.hal, "aout.door_lock", true)?;
        let fb = bridge::read_channel(&self.hal, "din.door_lock_fb")?
            .as_bool()
            .unwrap_or(false);
        // After write, plant in tick() will set fb; if already true from a
        // prior plant pass, advance. Otherwise wait one more tick.
        if fb {
            self.state = WasherState::Fill;
            self.dwell = 0;
        }
        Ok(())
    }

    fn step_fill(&mut self) -> Result<(), Error> {
        // Ensure drain off / heater off while filling.
        let _ = bridge::write_channel(&mut self.hal, "aout.drain_pump", false);
        let _ = bridge::write_channel(&mut self.hal, "aout.heater_enable", false);
        bridge::write_channel(&mut self.hal, "aout.cold_inlet", true)?;

        let level = bridge::read_channel(&self.hal, "ain.water_level_pa")?
            .as_number()
            .unwrap_or(0.0);
        if level >= self.opts.target_fill_pa {
            bridge::write_channel(&mut self.hal, "aout.cold_inlet", false)?;
            if self.opts.wash_temp_c <= 0.0 {
                self.state = WasherState::WashTumble;
            } else {
                self.state = WasherState::Heat;
            }
            self.dwell = 0;
        }
        Ok(())
    }

    fn step_heat(&mut self) -> Result<(), Error> {
        bridge::write_channel(&mut self.hal, "aout.cold_inlet", false)?;
        // Interlock requires water_present + door locked — refresh first.
        plant::refresh_derived(&mut self.hal)?;
        bridge::write_channel(&mut self.hal, "aout.heater_enable", true)?;

        let temp = bridge::read_channel(&self.hal, "ain.tub_temp_c")?
            .as_number()
            .unwrap_or(0.0);
        if temp >= self.opts.wash_temp_c {
            bridge::write_channel(&mut self.hal, "aout.heater_enable", false)?;
            self.state = WasherState::WashTumble;
            self.dwell = 0;
        }
        Ok(())
    }

    fn step_wash_tumble(&mut self) -> Result<(), Error> {
        let _ = bridge::write_channel(&mut self.hal, "aout.heater_enable", false);
        self.hal.apply_motor_command(
            "motor",
            MotorCommand {
                enable: Some(true),
                speed_rpm: Some(50.0),
                direction: Some(1.0),
            },
        )?;
        self.dwell += 1;
        if self.dwell >= self.opts.wash_tumble_ticks {
            self.motor_off()?;
            self.state = WasherState::Drain;
            self.dwell = 0;
        }
        Ok(())
    }

    fn step_drain(&mut self, next: WasherState) -> Result<(), Error> {
        let _ = bridge::write_channel(&mut self.hal, "aout.heater_enable", false);
        let _ = bridge::write_channel(&mut self.hal, "aout.cold_inlet", false);
        self.motor_off()?;
        bridge::write_channel(&mut self.hal, "aout.drain_pump", true)?;

        let level = bridge::read_channel(&self.hal, "ain.water_level_pa")?
            .as_number()
            .unwrap_or(0.0);
        if level < WATER_PRESENT_PA {
            bridge::write_channel(&mut self.hal, "aout.drain_pump", false)?;
            self.state = next;
            self.dwell = 0;
        }
        Ok(())
    }

    fn step_rinse_fill(&mut self) -> Result<(), Error> {
        let _ = bridge::write_channel(&mut self.hal, "aout.drain_pump", false);
        bridge::write_channel(&mut self.hal, "aout.cold_inlet", true)?;
        let level = bridge::read_channel(&self.hal, "ain.water_level_pa")?
            .as_number()
            .unwrap_or(0.0);
        // Rinse fill to a lower target for speed.
        let target = (self.opts.target_fill_pa * 0.6).max(WATER_PRESENT_PA + 100.0);
        if level >= target {
            bridge::write_channel(&mut self.hal, "aout.cold_inlet", false)?;
            self.state = WasherState::RinseTumble;
            self.dwell = 0;
        }
        Ok(())
    }

    fn step_rinse_tumble(&mut self) -> Result<(), Error> {
        self.hal.apply_motor_command(
            "motor",
            MotorCommand {
                enable: Some(true),
                speed_rpm: Some(50.0),
                direction: Some(-1.0),
            },
        )?;
        self.dwell += 1;
        if self.dwell >= self.opts.rinse_tumble_ticks {
            self.motor_off()?;
            self.state = WasherState::RinseDrain;
            self.dwell = 0;
        }
        Ok(())
    }

    fn step_spin(&mut self) -> Result<(), Error> {
        // Spin only when drained / spin-safe water.
        let level = bridge::read_channel(&self.hal, "ain.water_level_pa")?
            .as_number()
            .unwrap_or(0.0);
        if level >= WATER_PRESENT_PA {
            // Keep draining if somehow wet.
            bridge::write_channel(&mut self.hal, "aout.drain_pump", true)?;
            return Ok(());
        }
        let _ = bridge::write_channel(&mut self.hal, "aout.drain_pump", false);
        plant::refresh_derived(&mut self.hal)?;
        self.hal.apply_motor_command(
            "motor",
            MotorCommand {
                enable: Some(true),
                speed_rpm: Some(self.opts.spin_rpm),
                direction: Some(1.0),
            },
        )?;
        self.dwell += 1;
        if self.dwell >= self.opts.spin_ticks {
            self.finish()?;
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        self.motor_off()?;
        let _ = bridge::write_channel(&mut self.hal, "aout.heater_enable", false);
        let _ = bridge::write_channel(&mut self.hal, "aout.cold_inlet", false);
        let _ = bridge::write_channel(&mut self.hal, "aout.drain_pump", false);
        bridge::write_channel(&mut self.hal, "aout.door_lock", false)?;
        let _ = bridge::write_channel(&mut self.hal, "dout.buzzer", true);
        self.state = WasherState::Done;
        self.cycle_state = CycleState::Complete;
        self.dwell = 0;
        Ok(())
    }

    fn motor_off(&mut self) -> Result<(), Error> {
        self.hal.apply_motor_command(
            "motor",
            MotorCommand {
                enable: Some(false),
                speed_rpm: Some(0.0),
                direction: None,
            },
        )?;
        Ok(())
    }
}

/// Convenience: write a bool/number to a channel string on the controller HAL.
pub fn write_hal(ctrl: &mut Controller, channel: &str, value: HalValue) -> Result<(), Error> {
    plant::refresh_derived(ctrl.hal_mut())?;
    bridge::write_channel(ctrl.hal_mut(), channel, value)?;
    Ok(())
}
