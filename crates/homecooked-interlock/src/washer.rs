//! Washer heater and spin interlock examples.

use crate::condition::Condition;
use crate::engine::{Action, Rule, RuleSet};

/// Rpm at or above this is treated as a spin command.
pub const SPIN_RPM_THRESHOLD: f64 = 400.0;

fn door_locked() -> Condition {
    Condition::or(vec![
        Condition::eq("door_locked", true),
        Condition::eq("din.door_lock_fb", true),
    ])
}

/// Heater on requires `water_present` and a locked door.
pub fn heater_rule() -> Rule {
    Rule {
        id: Some("il.heater".into()),
        when: Condition::eq("aout.heater_enable", true),
        require: Some(Condition::and(vec![
            Condition::eq("water_present", true),
            door_locked(),
        ])),
        reason: Some("heater requires water_present and door locked".into()),
        actions: vec![
            Action::deny_actuator("aout.heater_enable"),
            Action::force_safe("aout.heater_enable", false),
        ],
    }
}

/// High rpm / spin requires a locked door.
pub fn spin_rule() -> Rule {
    Rule {
        id: Some("il.spin".into()),
        when: Condition::gte("motor.speed_rpm_cmd", SPIN_RPM_THRESHOLD),
        require: Some(door_locked()),
        reason: Some("spin requires door locked".into()),
        actions: vec![
            Action::deny_actuator("motor.speed_rpm_cmd"),
            Action::force_safe("motor.speed_rpm_cmd", 0),
        ],
    }
}

/// Washer heater + spin rules used by unit tests and as sample data.
pub fn rules() -> RuleSet {
    RuleSet::new(vec![heater_rule(), spin_rule()])
}
