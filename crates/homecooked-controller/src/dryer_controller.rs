//! Host controller: IoMap + MockHal + dryer interlocks + cotton dry cycle.

use homecooked_hal::{bridge, ChannelId, Hal, HalValue, MockHal, MotorCommand};
use homecooked_io_map::{IoMap, DRYER_FRAGMENT_YAML};

use crate::cycle::{CyclePhase, CycleState, DryOptions, DryerState};
use crate::error::Error;
use crate::plant::{self, DRYER_AMBIENT_C};

/// Extra HAL channels needed for dryer cotton that the fragment may omit.
const EXTRA_CHANNELS: &[&str] = &[
    "motor.enable",
    "motor.direction",
    "dout.buzzer",
    "dout.drum_light",
];

/// Host-side stand-in for the universal control computer (dryer role).
///
/// Loads a dryer [`IoMap`], holds a [`MockHal`] gated by
/// [`homecooked_interlock::dryer_rules`], and runs a tick-driven cotton dry
/// cycle: Idle → Lock → Dry/Heat → Cool → Done.
#[derive(Debug)]
pub struct DryerController {
    io_map: IoMap,
    hal: MockHal,
    state: DryerState,
    cycle_state: CycleState,
    opts: DryOptions,
    dwell: u32,
    tick: u64,
}

impl DryerController {
    /// Dryer demo: fragment IoMap + dryer interlocks + idle cotton runtime.
    pub fn dryer_cotton_demo() -> Result<Self, Error> {
        let map = IoMap::from_yaml_str(DRYER_FRAGMENT_YAML)?;
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
        hal.set_interlocks(Some(homecooked_interlock::dryer_rules()));
        hal.set_tick_ms(Some(0));

        let door = ChannelId::new("din.door_closed")?;
        if hal.get(&door).is_ok() {
            hal.inject(&door, true)?;
        }
        let temp = ChannelId::new("ain.drum_temp_c")?;
        if hal.get(&temp).is_ok() {
            hal.inject(&temp, DRYER_AMBIENT_C)?;
        }
        let rh = ChannelId::new("ain.humidity_rh")?;
        if hal.get(&rh).is_ok() {
            hal.inject(&rh, 65.0)?;
        }
        // Lint filter present / ok for start.
        let lint = ChannelId::new("din.lint_present")?;
        if hal.get(&lint).is_ok() {
            hal.inject(&lint, true)?;
        }

        plant::refresh_dryer_derived(&mut hal)?;

        Ok(Self {
            io_map: map,
            hal,
            state: DryerState::Idle,
            cycle_state: CycleState::Idle,
            opts: DryOptions::default(),
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

    pub fn dryer_state(&self) -> DryerState {
        self.state
    }

    pub fn phase(&self) -> CyclePhase {
        self.state.catalog_phase()
    }

    pub fn cycle_state(&self) -> CycleState {
        self.cycle_state
    }

    pub fn options(&self) -> &DryOptions {
        &self.opts
    }

    /// Start the dryer cotton program. Requires door closed when that channel exists.
    pub fn start_dry(&mut self, opts: DryOptions) -> Result<(), Error> {
        if self.cycle_state == CycleState::Running {
            return Err(Error::Cycle("cycle already running".into()));
        }
        if let Ok(closed) = bridge::read_channel(&self.hal, "din.door_closed") {
            if closed.as_bool() != Some(true) {
                return Err(Error::Cycle("door must be closed to start".into()));
            }
        }
        self.opts = opts;
        self.dwell = 0;
        self.state = DryerState::Lock;
        self.cycle_state = CycleState::Running;
        Ok(())
    }

    /// One sim step: plant feedback → derived keys → cycle commands → plant again.
    pub fn tick(&mut self) -> Result<(), Error> {
        self.tick = self.tick.saturating_add(1);
        self.hal.set_tick_ms(Some(self.tick));

        plant::step_dryer_plant(&mut self.hal)?;
        plant::refresh_dryer_derived(&mut self.hal)?;

        if self.cycle_state == CycleState::Running {
            self.advance_cycle()?;
            plant::step_dryer_plant(&mut self.hal)?;
            plant::refresh_dryer_derived(&mut self.hal)?;
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
        plant::refresh_dryer_derived(&mut self.hal)?;
        bridge::write_channel(&mut self.hal, "aout.heater_enable", true)?;
        Ok(())
    }

    fn advance_cycle(&mut self) -> Result<(), Error> {
        match self.state {
            DryerState::Idle | DryerState::Done => Ok(()),
            DryerState::Lock => self.step_lock(),
            DryerState::Dry => self.step_dry(),
            DryerState::Cool => self.step_cool(),
        }
    }

    fn step_lock(&mut self) -> Result<(), Error> {
        bridge::write_channel(&mut self.hal, "aout.door_lock", true)?;
        let fb = bridge::read_channel(&self.hal, "din.door_lock_fb")?
            .as_bool()
            .unwrap_or(false);
        if fb {
            self.state = DryerState::Dry;
            self.dwell = 0;
        }
        Ok(())
    }

    fn step_dry(&mut self) -> Result<(), Error> {
        // Blower before heater (interlock requires airflow).
        bridge::write_channel(&mut self.hal, "aout.blower", true)?;
        plant::refresh_dryer_derived(&mut self.hal)?;
        bridge::write_channel(&mut self.hal, "aout.heater_enable", true)?;
        self.hal.apply_motor_command(
            "motor",
            MotorCommand {
                enable: Some(true),
                speed_rpm: Some(self.opts.tumble_rpm),
                direction: Some(1.0),
            },
        )?;

        self.dwell += 1;
        let temp = bridge::read_channel(&self.hal, "ain.drum_temp_c")?
            .as_number()
            .unwrap_or(0.0);
        let rh = bridge::read_channel(&self.hal, "ain.humidity_rh")?
            .as_number()
            .unwrap_or(100.0);

        let reached_temp = temp >= self.opts.target_temp_c;
        let reached_dry = rh <= self.opts.target_humidity_rh;
        let timed_out = self.dwell >= self.opts.max_dry_ticks;

        if reached_temp || reached_dry || timed_out {
            bridge::write_channel(&mut self.hal, "aout.heater_enable", false)?;
            self.state = DryerState::Cool;
            self.dwell = 0;
        }
        Ok(())
    }

    fn step_cool(&mut self) -> Result<(), Error> {
        let _ = bridge::write_channel(&mut self.hal, "aout.heater_enable", false);
        bridge::write_channel(&mut self.hal, "aout.blower", true)?;
        plant::refresh_dryer_derived(&mut self.hal)?;
        self.hal.apply_motor_command(
            "motor",
            MotorCommand {
                enable: Some(true),
                speed_rpm: Some(self.opts.tumble_rpm),
                direction: Some(-1.0),
            },
        )?;

        self.dwell += 1;
        let temp = bridge::read_channel(&self.hal, "ain.drum_temp_c")?
            .as_number()
            .unwrap_or(0.0);
        let cooled = temp <= self.opts.cool_temp_c;
        let timed_out = self.dwell >= self.opts.max_cool_ticks;
        if cooled || timed_out {
            self.finish()?;
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        self.motor_off()?;
        let _ = bridge::write_channel(&mut self.hal, "aout.heater_enable", false);
        let _ = bridge::write_channel(&mut self.hal, "aout.blower", false);
        bridge::write_channel(&mut self.hal, "aout.door_lock", false)?;
        let _ = bridge::write_channel(&mut self.hal, "dout.buzzer", true);
        self.state = DryerState::Done;
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

/// Convenience: write a bool/number to a channel string on the dryer HAL.
pub fn write_dryer_hal(
    ctrl: &mut DryerController,
    channel: &str,
    value: HalValue,
) -> Result<(), Error> {
    plant::refresh_dryer_derived(ctrl.hal_mut())?;
    bridge::write_channel(ctrl.hal_mut(), channel, value)?;
    Ok(())
}
