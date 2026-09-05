use homecooked_hal::{bridge, ChannelId, HalValue};
use homecooked_io_map::{DRYER_FRAGMENT_YAML, WASHER_FRAGMENT_YAML};

use crate::{
    Controller, CottonOptions, CyclePhase, CycleState, DryOptions, DryerController, DryerState,
    WasherState,
};

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
fn loads_dryer_fragment_io_map() {
    let ctrl = DryerController::dryer_cotton_demo().expect("demo");
    assert_eq!(ctrl.io_map().class_id.as_deref(), Some("dryer"));
    assert!(ctrl
        .io_map()
        .bindings
        .iter()
        .any(|b| b.channel == "aout.blower"));
    let _ = homecooked_io_map::IoMap::from_yaml_str(DRYER_FRAGMENT_YAML).unwrap();
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
fn dryer_cycle_reaches_done() {
    let mut ctrl = DryerController::dryer_cotton_demo().unwrap();
    let door = ChannelId::new("din.door_closed").unwrap();
    ctrl.hal_mut().inject(&door, true).unwrap();

    let opts = DryOptions {
        target_temp_c: 45.0,
        target_humidity_rh: 30.0,
        cool_temp_c: 28.0,
        tumble_rpm: 50.0,
        max_dry_ticks: 15,
        max_cool_ticks: 15,
    };
    ctrl.start_dry(opts).unwrap();
    assert_eq!(ctrl.cycle_state(), CycleState::Running);
    assert_eq!(ctrl.dryer_state(), DryerState::Lock);

    ctrl.run_until_done(200)
        .expect("dryer cotton should complete");
    assert_eq!(ctrl.cycle_state(), CycleState::Complete);
    assert_eq!(ctrl.dryer_state(), DryerState::Done);
    assert_eq!(ctrl.phase(), CyclePhase::Complete);

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
    let heater = ctrl
        .hal()
        .get(&ChannelId::new("aout.heater_enable").unwrap())
        .unwrap();
    assert_eq!(heater, &HalValue::Bool(false));
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
fn dryer_heat_blocked_if_unlocked() {
    let mut ctrl = DryerController::dryer_cotton_demo().unwrap();
    // Blower on so the only failing require is the door lock.
    bridge::write_channel(ctrl.hal_mut(), "aout.blower", true).unwrap();
    bridge::write_channel(ctrl.hal_mut(), "aout.door_lock", false).unwrap();
    let lock_fb = ChannelId::new("din.door_lock_fb").unwrap();
    ctrl.hal_mut().inject(&lock_fb, false).unwrap();
    ctrl.hal_mut().set_derived("door_locked", false);
    ctrl.hal_mut().set_derived("blower_on", true);

    let err = ctrl
        .try_heater_on()
        .expect_err("heater must deny when unlocked");
    let msg = err.to_string();
    assert!(
        msg.contains("interlock") || msg.contains("door"),
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

#[test]
fn washer_pause_freezes_phase_resume_continues() {
    let mut ctrl = Controller::washer_cotton_demo().unwrap();
    ctrl.start_cotton(CottonOptions {
        wash_temp_c: 0.0,
        spin_rpm: 800.0,
        target_fill_pa: 2500.0,
        wash_tumble_ticks: 5,
        spin_ticks: 2,
        rinse_tumble_ticks: 2,
    })
    .unwrap();

    // Advance into fill / wash so phase is non-trivial.
    for _ in 0..8 {
        ctrl.tick().unwrap();
        if ctrl.washer_state() == WasherState::WashTumble {
            break;
        }
    }
    let phase_before = ctrl.phase();
    let state_before = ctrl.washer_state();
    assert_eq!(ctrl.cycle_state(), CycleState::Running);

    ctrl.pause().unwrap();
    assert_eq!(ctrl.cycle_state(), CycleState::Paused);
    // Idempotent pause.
    ctrl.pause().unwrap();
    assert_eq!(ctrl.cycle_state(), CycleState::Paused);

    for _ in 0..5 {
        ctrl.tick().unwrap();
    }
    assert_eq!(ctrl.cycle_state(), CycleState::Paused);
    assert_eq!(ctrl.washer_state(), state_before);
    assert_eq!(ctrl.phase(), phase_before);

    ctrl.resume().unwrap();
    assert_eq!(ctrl.cycle_state(), CycleState::Running);
    ctrl.run_until_done(200).expect("resume should complete");
    assert_eq!(ctrl.cycle_state(), CycleState::Complete);
}

#[test]
fn washer_cancel_mid_cycle_reaches_idle_unlocked() {
    let mut ctrl = Controller::washer_cotton_demo().unwrap();
    ctrl.start_cotton(CottonOptions {
        wash_temp_c: 0.0,
        spin_rpm: 800.0,
        target_fill_pa: 2500.0,
        wash_tumble_ticks: 5,
        spin_ticks: 3,
        rinse_tumble_ticks: 2,
    })
    .unwrap();
    for _ in 0..10 {
        ctrl.tick().unwrap();
    }
    assert_eq!(ctrl.cycle_state(), CycleState::Running);

    ctrl.cancel().unwrap();
    assert_eq!(ctrl.cycle_state(), CycleState::Canceling);
    // Idempotent cancel while canceling.
    ctrl.cancel().unwrap();

    for _ in 0..40 {
        if ctrl.cycle_state() == CycleState::Idle {
            break;
        }
        ctrl.tick().unwrap();
    }
    assert_eq!(ctrl.cycle_state(), CycleState::Idle);
    assert_eq!(ctrl.washer_state(), WasherState::Idle);
    let lock = ctrl
        .hal()
        .get(&ChannelId::new("aout.door_lock").unwrap())
        .unwrap();
    assert_eq!(lock, &HalValue::Bool(false));
}

#[test]
fn washer_cancel_while_idle_denied() {
    let mut ctrl = Controller::washer_cotton_demo().unwrap();
    let err = ctrl.cancel().expect_err("cancel idle");
    assert!(err.to_string().contains("no active"));
    let err = ctrl.pause().expect_err("pause idle");
    assert!(err.to_string().contains("not running"));
}

#[test]
fn dryer_pause_freezes_phase_resume_continues() {
    let mut ctrl = DryerController::dryer_cotton_demo().unwrap();
    ctrl.start_dry(DryOptions {
        target_temp_c: 55.0,
        target_humidity_rh: 25.0,
        cool_temp_c: 30.0,
        tumble_rpm: 50.0,
        max_dry_ticks: 20,
        max_cool_ticks: 20,
    })
    .unwrap();
    for _ in 0..5 {
        ctrl.tick().unwrap();
        if ctrl.dryer_state() == DryerState::Dry {
            break;
        }
    }
    let state_before = ctrl.dryer_state();
    let phase_before = ctrl.phase();
    assert_eq!(ctrl.cycle_state(), CycleState::Running);

    ctrl.pause().unwrap();
    assert_eq!(ctrl.cycle_state(), CycleState::Paused);
    ctrl.pause().unwrap();
    for _ in 0..5 {
        ctrl.tick().unwrap();
    }
    assert_eq!(ctrl.cycle_state(), CycleState::Paused);
    assert_eq!(ctrl.dryer_state(), state_before);
    assert_eq!(ctrl.phase(), phase_before);

    ctrl.resume().unwrap();
    assert_eq!(ctrl.cycle_state(), CycleState::Running);
    ctrl.run_until_done(200).expect("resume should complete");
    assert_eq!(ctrl.cycle_state(), CycleState::Complete);
}

#[test]
fn dryer_cancel_mid_cycle_reaches_idle_unlocked() {
    let mut ctrl = DryerController::dryer_cotton_demo().unwrap();
    ctrl.start_dry(DryOptions::default()).unwrap();
    for _ in 0..6 {
        ctrl.tick().unwrap();
    }
    assert_eq!(ctrl.cycle_state(), CycleState::Running);

    ctrl.cancel().unwrap();
    assert_eq!(ctrl.cycle_state(), CycleState::Canceling);

    for _ in 0..40 {
        if ctrl.cycle_state() == CycleState::Idle {
            break;
        }
        ctrl.tick().unwrap();
    }
    assert_eq!(ctrl.cycle_state(), CycleState::Idle);
    assert_eq!(ctrl.dryer_state(), DryerState::Idle);
    let lock = ctrl
        .hal()
        .get(&ChannelId::new("aout.door_lock").unwrap())
        .unwrap();
    assert_eq!(lock, &HalValue::Bool(false));
    let heater = ctrl
        .hal()
        .get(&ChannelId::new("aout.heater_enable").unwrap())
        .unwrap();
    assert_eq!(heater, &HalValue::Bool(false));
}

#[test]
fn dryer_cancel_while_idle_denied() {
    let mut ctrl = DryerController::dryer_cotton_demo().unwrap();
    let err = ctrl.cancel().expect_err("cancel idle");
    assert!(err.to_string().contains("no active"));
}
