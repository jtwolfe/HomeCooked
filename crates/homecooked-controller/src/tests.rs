use homecooked_hal::{bridge, ChannelId, HalValue};
use homecooked_io_map::WASHER_FRAGMENT_YAML;

use crate::{Controller, CottonOptions, CyclePhase, CycleState, WasherState};

#[test]
fn loads_washer_fragment_io_map() {
    let ctrl = Controller::washer_cotton_demo().expect("demo");
    assert_eq!(ctrl.io_map().class_id.as_deref(), Some("washer"));
    assert!(
        !ctrl.io_map().bindings.is_empty(),
        "expected bindings from fragment"
    );
    // Fragment YAML constant still parses standalone.
    let _ = homecooked_io_map::IoMap::from_yaml_str(WASHER_FRAGMENT_YAML).unwrap();
}

#[test]
fn cotton_cycle_reaches_done() {
    let mut ctrl = Controller::washer_cotton_demo().unwrap();
    // Door closed is default in demo; reinforce.
    let door = ChannelId::new("din.door_closed").unwrap();
    ctrl.hal_mut().inject(&door, true).unwrap();

    let opts = CottonOptions {
        wash_temp_c: 40.0,
        spin_rpm: 800.0,
        target_fill_pa: 2500.0,
        wash_tumble_ticks: 2,
        spin_ticks: 2,
        rinse_tumble_ticks: 1,
    };
    ctrl.start_cotton(opts).unwrap();
    assert_eq!(ctrl.cycle_state(), CycleState::Running);
    assert_eq!(ctrl.washer_state(), WasherState::Lock);

    ctrl.run_until_done(200).expect("cotton should complete");
    assert_eq!(ctrl.cycle_state(), CycleState::Complete);
    assert_eq!(ctrl.washer_state(), WasherState::Done);
    assert_eq!(ctrl.phase(), CyclePhase::Complete);

    // Motor should be off; door unlocked.
    let rpm = bridge::read_channel(ctrl.hal(), "ain.drum_rpm")
        .unwrap()
        .as_number()
        .unwrap();
    assert_eq!(rpm, 0.0);
    let lock = ctrl
        .hal()
        .get(&ChannelId::new("aout.door_lock").unwrap())
        .unwrap();
    assert_eq!(lock, &HalValue::Bool(false));
}

#[test]
fn heater_blocked_without_water() {
    let mut ctrl = Controller::washer_cotton_demo().unwrap();
    // Door locked path for heater require; water absent.
    ctrl.hal_mut().set_derived("water_present", false);
    ctrl.hal_mut().set_derived("door_locked", true);
    let lock_fb = ChannelId::new("din.door_lock_fb").unwrap();
    ctrl.hal_mut().inject(&lock_fb, true).unwrap();
    // Ensure level is below threshold.
    let level = ChannelId::new("ain.water_level_pa").unwrap();
    ctrl.hal_mut().inject(&level, 0.0).unwrap();

    let err = ctrl.try_heater_on().expect_err("heater must deny");
    let msg = err.to_string();
    assert!(
        msg.contains("interlock") || msg.contains("water_present"),
        "unexpected err: {msg}"
    );
    assert!(ctrl.hal().last_command("aout.heater_enable").is_none());
}

#[test]
fn spin_blocked_when_door_unlocked() {
    let mut ctrl = Controller::washer_cotton_demo().unwrap();
    ctrl.hal_mut().set_derived("door_locked", false);
    let lock_fb = ChannelId::new("din.door_lock_fb").unwrap();
    ctrl.hal_mut().inject(&lock_fb, false).unwrap();
    // Ensure lock command is off so derived refresh stays unlocked.
    bridge::write_channel(ctrl.hal_mut(), "aout.door_lock", false).unwrap();
    ctrl.hal_mut().set_derived("door_locked", false);

    let err = ctrl.try_spin_rpm(800.0).expect_err("spin must deny");
    let msg = err.to_string();
    assert!(
        msg.contains("interlock") || msg.contains("door"),
        "unexpected err: {msg}"
    );
    // No successful spin-band command recorded (deny happens before record).
    assert!(ctrl.hal().last_command("motor.speed_rpm_cmd").is_none());
}

#[test]
fn cold_wash_skips_heat() {
    let mut ctrl = Controller::washer_cotton_demo().unwrap();
    ctrl.start_cotton(CottonOptions {
        wash_temp_c: 0.0,
        spin_rpm: 800.0,
        target_fill_pa: 1600.0,
        wash_tumble_ticks: 1,
        spin_ticks: 1,
        rinse_tumble_ticks: 1,
    })
    .unwrap();
    ctrl.run_until_done(200).unwrap();
    assert_eq!(ctrl.phase(), CyclePhase::Complete);
    // Heater should never have been commanded on.
    if let Some(cmd) = ctrl.hal().last_command("aout.heater_enable") {
        assert_eq!(cmd.value, HalValue::Bool(false));
    }
}
