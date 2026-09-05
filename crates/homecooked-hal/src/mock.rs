//! In-memory [`Hal`] for host-side tests.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::channel::{ChannelId, ChannelKind};
use crate::error::Error;
use crate::hal::Hal;
use crate::value::HalValue;

/// Recorded actuator write for assertions.
#[derive(Debug, Clone, PartialEq)]
pub struct ActuatorCommand {
    pub channel: ChannelId,
    pub value: HalValue,
    pub tick_ms: u64,
}

/// Host mock: inject sensor samples, record actuator commands.
///
/// Register every channel you intend to use. Inputs start at a default
/// (false / 0.0); outputs start de-energized / zero until written.
#[derive(Debug, Default)]
pub struct MockHal {
    values: BTreeMap<ChannelId, HalValue>,
    kinds: BTreeMap<ChannelId, ChannelKind>,
    commands: Vec<ActuatorCommand>,
    tick_ms: Option<u64>,
    faults: BTreeMap<ChannelId, String>,
    #[cfg(feature = "interlock")]
    interlocks: Option<homecooked_interlock::RuleSet>,
    /// Extra keys for interlock snapshots that are not HAL channels
    /// (e.g. derived `water_present`, `door_locked`).
    #[cfg(feature = "interlock")]
    derived: BTreeMap<String, HalValue>,
}

impl MockHal {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a channel with a default value for its kind.
    pub fn register(&mut self, channel: ChannelId) -> &mut Self {
        let default = match channel.kind() {
            ChannelKind::DigitalIn | ChannelKind::DigitalOut | ChannelKind::Relay => {
                HalValue::Bool(false)
            }
            ChannelKind::AnalogIn | ChannelKind::AnalogOut | ChannelKind::Motor => {
                HalValue::Number(0.0)
            }
        };
        self.kinds.insert(channel.clone(), channel.kind());
        self.values.entry(channel).or_insert(default);
        self
    }

    pub fn register_str(&mut self, channel: &str) -> Result<&mut Self, Error> {
        Ok(self.register(ChannelId::new(channel)?))
    }

    /// Inject a sensor (or any registered channel) sample.
    pub fn inject(&mut self, channel: &ChannelId, value: impl Into<HalValue>) -> Result<(), Error> {
        self.ensure_known(channel)?;
        if let Some(fault) = self.faults.get(channel) {
            return Err(Error::Fault {
                channel: channel.as_str().into(),
                detail: fault.clone(),
            });
        }
        self.values.insert(channel.clone(), value.into());
        Ok(())
    }

    pub fn inject_str(&mut self, channel: &str, value: impl Into<HalValue>) -> Result<(), Error> {
        let id = ChannelId::new(channel)?;
        self.inject(&id, value)
    }

    /// Mark a channel as faulted until cleared.
    pub fn set_fault(&mut self, channel: &ChannelId, detail: impl Into<String>) {
        self.faults.insert(channel.clone(), detail.into());
    }

    pub fn clear_fault(&mut self, channel: &ChannelId) {
        self.faults.remove(channel);
    }

    /// Override monotonic tick (tests). `None` uses wall-clock millis.
    pub fn set_tick_ms(&mut self, tick: Option<u64>) {
        self.tick_ms = tick;
    }

    /// All recorded actuator writes, in order.
    pub fn commands(&self) -> &[ActuatorCommand] {
        &self.commands
    }

    /// Last recorded command for `channel`, if any.
    pub fn last_command(&self, channel: &str) -> Option<&ActuatorCommand> {
        self.commands
            .iter()
            .rev()
            .find(|c| c.channel.as_str() == channel)
    }

    /// Current stored value (input or last output).
    pub fn get(&self, channel: &ChannelId) -> Result<&HalValue, Error> {
        self.ensure_known(channel)?;
        self.values
            .get(channel)
            .ok_or_else(|| Error::UnknownChannel {
                channel: channel.as_str().into(),
            })
    }

    /// Attach declarative interlock rules evaluated before actuator writes.
    #[cfg(feature = "interlock")]
    pub fn set_interlocks(&mut self, rules: Option<homecooked_interlock::RuleSet>) {
        self.interlocks = rules;
    }

    /// Set a derived interlock key that is not a HAL channel id.
    #[cfg(feature = "interlock")]
    pub fn set_derived(&mut self, key: impl Into<String>, value: impl Into<HalValue>) {
        self.derived.insert(key.into(), value.into());
    }

    fn ensure_known(&self, channel: &ChannelId) -> Result<(), Error> {
        if self.kinds.contains_key(channel) {
            Ok(())
        } else {
            Err(Error::UnknownChannel {
                channel: channel.as_str().into(),
            })
        }
    }

    fn ensure_kind(&self, channel: &ChannelId, expected: ChannelKind) -> Result<(), Error> {
        self.ensure_known(channel)?;
        let kind = channel.kind();
        if kind != expected {
            return Err(Error::KindMismatch {
                channel: channel.as_str().into(),
                expected: expected.as_str().into(),
                detail: format!("channel is {}", kind.as_str()),
            });
        }
        Ok(())
    }

    fn wall_tick() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    fn record_and_store(&mut self, channel: &ChannelId, value: HalValue) -> Result<(), Error> {
        if let Some(fault) = self.faults.get(channel) {
            return Err(Error::Fault {
                channel: channel.as_str().into(),
                detail: fault.clone(),
            });
        }

        #[cfg(feature = "interlock")]
        self.check_interlocks(channel, &value)?;

        let tick = self.tick_ms.unwrap_or_else(Self::wall_tick);
        self.commands.push(ActuatorCommand {
            channel: channel.clone(),
            value: value.clone(),
            tick_ms: tick,
        });
        self.values.insert(channel.clone(), value);
        Ok(())
    }

    #[cfg(feature = "interlock")]
    fn check_interlocks(&mut self, channel: &ChannelId, value: &HalValue) -> Result<(), Error> {
        let Some(rules) = self.interlocks.clone() else {
            return Ok(());
        };
        let snapshot = self.interlock_snapshot();
        let command = homecooked_interlock::Command::new(
            channel.as_str(),
            match value {
                HalValue::Bool(b) => homecooked_interlock::Value::Bool(*b),
                HalValue::Number(n) => homecooked_interlock::Value::Number(*n),
            },
        );
        let decision = rules.evaluate(&snapshot, &command);
        match decision {
            homecooked_interlock::Decision::Deny { reason, force_safe } => {
                for force in force_safe {
                    self.apply_force_safe(&force);
                }
                Err(Error::InterlockDenied {
                    channel: channel.as_str().into(),
                    reason,
                })
            }
            homecooked_interlock::Decision::Allow { force_safe } => {
                for force in force_safe {
                    self.apply_force_safe(&force);
                }
                Ok(())
            }
        }
    }

    #[cfg(feature = "interlock")]
    fn apply_force_safe(&mut self, force: &homecooked_interlock::ForceSafe) {
        if let Ok(id) = ChannelId::new(&force.channel) {
            if self.kinds.contains_key(&id) {
                let v = match &force.value {
                    homecooked_interlock::Value::Bool(b) => HalValue::Bool(*b),
                    homecooked_interlock::Value::Number(n) => HalValue::Number(*n),
                    homecooked_interlock::Value::String(_) => return,
                };
                self.values.insert(id, v);
            }
        }
    }

    #[cfg(feature = "interlock")]
    fn interlock_snapshot(&self) -> homecooked_interlock::Snapshot {
        let mut snap = homecooked_interlock::Snapshot::new();
        for (id, value) in &self.values {
            let v = match value {
                HalValue::Bool(b) => homecooked_interlock::Value::Bool(*b),
                HalValue::Number(n) => homecooked_interlock::Value::Number(*n),
            };
            snap.insert(id.as_str(), v);
        }
        for (key, value) in &self.derived {
            let v = match value {
                HalValue::Bool(b) => homecooked_interlock::Value::Bool(*b),
                HalValue::Number(n) => homecooked_interlock::Value::Number(*n),
            };
            snap.insert(key.clone(), v);
        }
        snap
    }
}

impl Hal for MockHal {
    fn read_di(&self, channel: &ChannelId) -> Result<bool, Error> {
        self.ensure_kind(channel, ChannelKind::DigitalIn)?;
        if let Some(fault) = self.faults.get(channel) {
            return Err(Error::Fault {
                channel: channel.as_str().into(),
                detail: fault.clone(),
            });
        }
        self.get(channel)?
            .as_bool()
            .ok_or_else(|| Error::TypeMismatch {
                channel: channel.as_str().into(),
                detail: "expected bool".into(),
            })
    }

    fn read_ai(&self, channel: &ChannelId) -> Result<f64, Error> {
        self.ensure_kind(channel, ChannelKind::AnalogIn)?;
        if let Some(fault) = self.faults.get(channel) {
            return Err(Error::Fault {
                channel: channel.as_str().into(),
                detail: fault.clone(),
            });
        }
        self.get(channel)?
            .as_number()
            .ok_or_else(|| Error::TypeMismatch {
                channel: channel.as_str().into(),
                detail: "expected number".into(),
            })
    }

    fn write_do(&mut self, channel: &ChannelId, value: bool) -> Result<(), Error> {
        self.ensure_kind(channel, ChannelKind::DigitalOut)?;
        self.record_and_store(channel, HalValue::Bool(value))
    }

    fn write_aout(&mut self, channel: &ChannelId, value: HalValue) -> Result<(), Error> {
        self.ensure_kind(channel, ChannelKind::AnalogOut)?;
        self.record_and_store(channel, value)
    }

    fn write_relay(&mut self, channel: &ChannelId, energized: bool) -> Result<(), Error> {
        self.ensure_kind(channel, ChannelKind::Relay)?;
        self.record_and_store(channel, HalValue::Bool(energized))
    }

    fn write_motor(&mut self, channel: &ChannelId, value: HalValue) -> Result<(), Error> {
        self.ensure_kind(channel, ChannelKind::Motor)?;
        self.record_and_store(channel, value)
    }

    fn tick_ms(&self) -> u64 {
        self.tick_ms.unwrap_or_else(Self::wall_tick)
    }
}
