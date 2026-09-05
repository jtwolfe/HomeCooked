//! Firmware HAL surface sketch for HomeCooked.
//!
//! This is **not** production firmware. It defines the logical channel API
//! controller firmware will implement, plus a host-side [`MockHal`] for tests.
//!
//! Design: [`docs/standard/control-system.md`](../../docs/standard/control-system.md)
//! §4.3 (HAL) and [`docs/ROADMAP.md`](../../docs/ROADMAP.md) Stream 4.
//!
//! # Layers
//!
//! ```text
//! interlock / cycle runtime / device role
//!         │
//!         ▼
//!   I/O map  (homecooked-io-map)  — binds points ↔ HAL channel ids
//!         │
//!         ▼
//!   HAL      (this crate)         — din/ain/dout/aout/relay/motor
//!         │
//!         ▼
//!   boards / MockHal
//! ```
//!
//! The I/O map sits **above** the HAL. See [`bridge`] for the thin string
//! helpers and the layering note.
//!
//! # Features
//!
//! - `std` (default) — enables [`MockHal`] and host clocks. Core types
//!   (`ChannelId`, `Hal`, …) are still small and auditable; a firmware
//!   target can reimplement `Hal` without this crate's mock.
//! - `interlock` (default) — optional gate of mock writes through
//!   `homecooked-interlock`.
//!
//! No `embedded-hal` / GPIO dependency is required for CI on host.

#![allow(clippy::module_name_repetitions)]

pub mod bridge;
mod channel;
mod error;
mod hal;
#[cfg(feature = "std")]
mod mock;
mod value;

pub use channel::{channel_prefix, ChannelId, ChannelKind};
pub use error::Error;
pub use hal::Hal;
#[cfg(feature = "std")]
pub use mock::{ActuatorCommand, MockHal};
pub use value::{HalValue, MotorCommand};

#[cfg(all(test, feature = "std"))]
mod washer_tests {
    use super::*;

    /// Sample washer channels from `docs/standard/examples/washer-dryer-io.md`.
    fn washer_mock() -> MockHal {
        let mut hal = MockHal::new();
        for ch in [
            "din.door_closed",
            "din.door_lock_fb",
            "din.leak",
            "ain.water_level_pa",
            "ain.tub_temp_c",
            "ain.drum_rpm",
            "aout.door_lock",
            "aout.cold_inlet",
            "aout.heater_enable",
            "motor.enable",
            "motor.speed_rpm_cmd",
            "motor.direction",
            "dout.buzzer",
        ] {
            hal.register_str(ch).unwrap();
        }
        hal.set_tick_ms(Some(1_000));
        hal
    }

    #[test]
    fn washer_heater_smoke_records_command() {
        let mut hal = washer_mock();

        let lock_fb = ChannelId::new("din.door_lock_fb").unwrap();
        let level = ChannelId::new("ain.water_level_pa").unwrap();
        hal.inject(&lock_fb, true).unwrap();
        hal.inject(&level, 2500.0).unwrap();
        assert!(hal.read_di(&lock_fb).unwrap());
        assert_eq!(hal.read_ai(&level).unwrap(), 2500.0);

        #[cfg(feature = "interlock")]
        {
            hal.set_derived("water_present", true);
            hal.set_derived("door_locked", true);
            hal.set_interlocks(Some(homecooked_interlock::washer_rules()));
        }

        let heater = ChannelId::new("aout.heater_enable").unwrap();
        hal.write_aout(&heater, HalValue::Bool(true)).unwrap();

        let last = hal.last_command("aout.heater_enable").expect("recorded");
        assert_eq!(last.value, HalValue::Bool(true));
        assert_eq!(last.tick_ms, 1_000);
        assert_eq!(hal.get(&heater).unwrap(), &HalValue::Bool(true));
    }

    #[test]
    #[cfg(feature = "interlock")]
    fn washer_heater_denied_without_water() {
        let mut hal = washer_mock();
        hal.set_derived("water_present", false);
        hal.set_derived("door_locked", true);
        hal.set_interlocks(Some(homecooked_interlock::washer_rules()));

        let heater = ChannelId::new("aout.heater_enable").unwrap();
        let err = hal.write_aout(&heater, HalValue::Bool(true)).unwrap_err();
        match err {
            Error::InterlockDenied { channel, reason } => {
                assert_eq!(channel, "aout.heater_enable");
                assert!(reason.contains("water_present"), "{reason}");
            }
            other => panic!("expected InterlockDenied, got {other:?}"),
        }
        assert!(hal.last_command("aout.heater_enable").is_none());
    }

    #[test]
    fn unknown_channel_and_kind_mismatch() {
        let mut hal = washer_mock();
        let missing = ChannelId::new("din.missing_sensor").unwrap();
        assert!(matches!(
            hal.read_di(&missing),
            Err(Error::UnknownChannel { .. })
        ));
        let heater = ChannelId::new("aout.heater_enable").unwrap();
        assert!(matches!(
            hal.read_di(&heater),
            Err(Error::KindMismatch { .. })
        ));
        let door = ChannelId::new("din.door_closed").unwrap();
        assert!(matches!(
            hal.write_do(&door, true),
            Err(Error::KindMismatch { .. })
        ));
    }

    #[test]
    fn bridge_helpers_and_motor_command() {
        let mut hal = washer_mock();
        bridge::write_channel(&mut hal, "dout.buzzer", true).unwrap();
        let buzzer = ChannelId::new("dout.buzzer").unwrap();
        assert_eq!(hal.get(&buzzer).unwrap(), &HalValue::Bool(true));

        bridge::write_channel(&mut hal, "aout.cold_inlet", true).unwrap();
        assert_eq!(
            hal.last_command("aout.cold_inlet").unwrap().value,
            HalValue::Bool(true)
        );

        let door = ChannelId::new("din.door_closed").unwrap();
        hal.inject(&door, true).unwrap();
        assert_eq!(
            bridge::read_channel(&hal, "din.door_closed").unwrap(),
            HalValue::Bool(true)
        );

        hal.apply_motor_command(
            "motor",
            MotorCommand {
                enable: Some(true),
                speed_rpm: Some(50.0),
                direction: None,
            },
        )
        .unwrap();
        assert_eq!(
            hal.last_command("motor.speed_rpm_cmd").unwrap().value,
            HalValue::Number(50.0)
        );
        assert_eq!(
            hal.last_command("motor.enable").unwrap().value,
            HalValue::Bool(true)
        );
    }

    #[test]
    fn fault_blocks_read_write() {
        let mut hal = washer_mock();
        let door = ChannelId::new("din.door_closed").unwrap();
        hal.set_fault(&door, "open circuit");
        assert!(matches!(hal.read_di(&door), Err(Error::Fault { .. })));
        hal.clear_fault(&door);
        hal.inject(&door, true).unwrap();
        assert!(hal.read_di(&door).unwrap());
    }
}
