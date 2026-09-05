//! [`Hal`] trait — the firmware surface above the pin mux.

use crate::channel::ChannelId;
use crate::error::Error;
use crate::value::{HalValue, MotorCommand};

/// Hardware abstraction: logical channels only, never GPIO numbers.
///
/// Firmware above this line (I/O map, interlocks, cycle runtime) talks in
/// channel ids from [`docs/standard/control-system.md`](../../../docs/standard/control-system.md)
/// §4.3. Production firmware implements this against boards; host tests use
/// [`crate::MockHal`].
pub trait Hal {
    /// Read a digital input (`din.*`).
    fn read_di(&self, channel: &ChannelId) -> Result<bool, Error>;

    /// Read an analog input (`ain.*`) in catalog / edge-converted units.
    fn read_ai(&self, channel: &ChannelId) -> Result<f64, Error>;

    /// Write a digital / LV output (`dout.*`).
    fn write_do(&mut self, channel: &ChannelId, value: bool) -> Result<(), Error>;

    /// Write an analog / gated actuator out (`aout.*`). Bool enables and
    /// numeric setpoints are both accepted as [`HalValue`].
    fn write_aout(&mut self, channel: &ChannelId, value: HalValue) -> Result<(), Error>;

    /// Energize or drop a relay coil (`relay.*`).
    fn write_relay(&mut self, channel: &ChannelId, energized: bool) -> Result<(), Error>;

    /// Write a single motor IF channel (`motor.enable`, `motor.speed_rpm_cmd`, …).
    fn write_motor(&mut self, channel: &ChannelId, value: HalValue) -> Result<(), Error>;

    /// Monotonic tick in milliseconds (host clock or firmware tick).
    fn tick_ms(&self) -> u64;

    /// Generic read dispatching on channel kind.
    fn read(&self, channel: &ChannelId) -> Result<HalValue, Error> {
        match channel.kind() {
            crate::ChannelKind::DigitalIn => self.read_di(channel).map(HalValue::Bool),
            crate::ChannelKind::AnalogIn => self.read_ai(channel).map(HalValue::Number),
            crate::ChannelKind::DigitalOut
            | crate::ChannelKind::AnalogOut
            | crate::ChannelKind::Relay
            | crate::ChannelKind::Motor => Err(Error::KindMismatch {
                channel: channel.as_str().into(),
                expected: "input".into(),
                detail: format!("cannot read {:?} via Hal::read", channel.kind()),
            }),
        }
    }

    /// Generic write dispatching on channel kind.
    fn write(&mut self, channel: &ChannelId, value: HalValue) -> Result<(), Error> {
        match channel.kind() {
            crate::ChannelKind::DigitalOut => {
                let b = value.as_bool().ok_or_else(|| Error::TypeMismatch {
                    channel: channel.as_str().into(),
                    detail: "dout expects bool".into(),
                })?;
                self.write_do(channel, b)
            }
            crate::ChannelKind::AnalogOut => self.write_aout(channel, value),
            crate::ChannelKind::Relay => {
                let b = value.as_bool().ok_or_else(|| Error::TypeMismatch {
                    channel: channel.as_str().into(),
                    detail: "relay expects bool".into(),
                })?;
                self.write_relay(channel, b)
            }
            crate::ChannelKind::Motor => self.write_motor(channel, value),
            crate::ChannelKind::DigitalIn | crate::ChannelKind::AnalogIn => {
                Err(Error::KindMismatch {
                    channel: channel.as_str().into(),
                    expected: "output".into(),
                    detail: format!("cannot write {:?} via Hal::write", channel.kind()),
                })
            }
        }
    }

    /// Apply a multi-field motor command to conventional channel names under
    /// the given motor IF prefix (default `motor`).
    fn apply_motor_command(&mut self, prefix: &str, cmd: MotorCommand) -> Result<(), Error> {
        if let Some(en) = cmd.enable {
            let id = ChannelId::new(format!("{prefix}.enable"))?;
            self.write_motor(&id, HalValue::Bool(en))?;
        }
        if let Some(rpm) = cmd.speed_rpm {
            let id = ChannelId::new(format!("{prefix}.speed_rpm_cmd"))?;
            self.write_motor(&id, HalValue::Number(rpm))?;
        }
        if let Some(dir) = cmd.direction {
            let id = ChannelId::new(format!("{prefix}.direction"))?;
            self.write_motor(&id, HalValue::Number(dir))?;
        }
        Ok(())
    }
}
