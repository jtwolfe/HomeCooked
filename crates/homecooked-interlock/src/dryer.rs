//! Dryer heater and tumble interlock examples.
//!
//! Aligns with `docs/standard/examples/washer-dryer-io.md` §3.2:
//! heater requires lock feedback match (and blower when the SKU requires it).

use crate::condition::Condition;
use crate::engine::{Action, Rule, RuleSet};

fn door_locked() -> Condition {
    Condition::or(vec![
        Condition::eq("door_locked", true),
        Condition::eq("din.door_lock_fb", true),
    ])
}

/// Heater on requires a locked door (dryer: no `water_present` gate).
pub fn heater_rule() -> Rule {
    Rule {
        id: Some("il.heater".into()),
        when: Condition::eq("aout.heater_enable", true),
        require: Some(Condition::and(vec![
            door_locked(),
            Condition::or(vec![
                Condition::eq("aout.blower", true),
                Condition::eq("blower_on", true),
            ]),
        ])),
        reason: Some("dryer heater requires door locked and blower on".into()),
        actions: vec![
            Action::deny_actuator("aout.heater_enable"),
            Action::force_safe("aout.heater_enable", false),
        ],
    }
}

/// Drum tumble speed command requires a locked door.
pub fn motor_rule() -> Rule {
    Rule {
        id: Some("il.motor".into()),
        when: Condition::gt("motor.speed_rpm_cmd", 0),
        require: Some(door_locked()),
        reason: Some("dryer motor requires door locked".into()),
        actions: vec![
            Action::deny_actuator("motor.speed_rpm_cmd"),
            Action::force_safe("motor.speed_rpm_cmd", 0),
        ],
    }
}

/// Dryer heater + motor rules used by unit tests and the controller sim.
pub fn rules() -> RuleSet {
    RuleSet::new(vec![heater_rule(), motor_rule()])
}
