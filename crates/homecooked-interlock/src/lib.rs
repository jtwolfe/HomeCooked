//! Declarative interlock rules for HomeCooked.
//!
//! Conditions are boolean AND/OR and comparisons (`eq` / `neq` / `gt` / `gte`
//! / `lt` / `lte`) on string channel or point keys. Actions are
//! [`Action::DenyActuator`] and [`Action::ForceSafeState`].
//! [`RuleSet::evaluate`] overlays a proposed actuator command on a snapshot
//! and returns [`Decision::Allow`] or [`Decision::Deny`].

mod condition;
mod engine;
mod error;
mod value;
mod washer;

pub use condition::{CmpOp, Compare, Condition};
pub use engine::{Action, Decision, ForceSafe, Rule, RuleSet};
pub use error::Error;
pub use value::{Command, Snapshot, Value};
pub use washer::{heater_rule, rules as washer_rules, spin_rule, SPIN_RPM_THRESHOLD};

#[cfg(test)]
mod tests {
    use super::*;

    struct Case {
        name: &'static str,
        snapshot: Snapshot,
        command: Command,
        allow: bool,
        reason_contains: Option<&'static str>,
    }

    fn cases() -> Vec<Case> {
        vec![
            Case {
                name: "heater on with water and lock",
                snapshot: Snapshot::new()
                    .with("water_present", true)
                    .with("door_locked", true),
                command: Command::new("aout.heater_enable", true),
                allow: true,
                reason_contains: None,
            },
            Case {
                name: "heater on with lock feedback",
                snapshot: Snapshot::new()
                    .with("water_present", true)
                    .with("din.door_lock_fb", true),
                command: Command::new("aout.heater_enable", true),
                allow: true,
                reason_contains: None,
            },
            Case {
                name: "heater on without water",
                snapshot: Snapshot::new()
                    .with("water_present", false)
                    .with("door_locked", true),
                command: Command::new("aout.heater_enable", true),
                allow: false,
                reason_contains: Some("water_present"),
            },
            Case {
                name: "heater on without lock",
                snapshot: Snapshot::new()
                    .with("water_present", true)
                    .with("door_locked", false)
                    .with("din.door_lock_fb", false),
                command: Command::new("aout.heater_enable", true),
                allow: false,
                reason_contains: Some("door locked"),
            },
            Case {
                name: "heater off without water",
                snapshot: Snapshot::new().with("water_present", false),
                command: Command::new("aout.heater_enable", false),
                allow: true,
                reason_contains: None,
            },
            Case {
                name: "spin with door locked",
                snapshot: Snapshot::new().with("door_locked", true),
                command: Command::new("motor.speed_rpm_cmd", 800),
                allow: true,
                reason_contains: None,
            },
            Case {
                name: "spin with lock feedback",
                snapshot: Snapshot::new().with("din.door_lock_fb", true),
                command: Command::new("motor.speed_rpm_cmd", SPIN_RPM_THRESHOLD),
                allow: true,
                reason_contains: None,
            },
            Case {
                name: "spin without lock",
                snapshot: Snapshot::new()
                    .with("door_locked", false)
                    .with("din.door_lock_fb", false),
                command: Command::new("motor.speed_rpm_cmd", 800),
                allow: false,
                reason_contains: Some("door locked"),
            },
            Case {
                name: "tumble without lock is not spin",
                snapshot: Snapshot::new().with("door_locked", false),
                command: Command::new("motor.speed_rpm_cmd", 50),
                allow: true,
                reason_contains: None,
            },
        ]
    }

    #[test]
    fn washer_allow_deny_table() {
        let rules = washer_rules();
        for case in cases() {
            let decision = rules.evaluate(&case.snapshot, &case.command);
            assert_eq!(
                decision.is_allow(),
                case.allow,
                "{}: {decision:?}",
                case.name
            );
            if let Some(needle) = case.reason_contains {
                match &decision {
                    Decision::Deny { reason, .. } => {
                        assert!(
                            reason.contains(needle),
                            "{}: reason {reason:?} missing {needle:?}",
                            case.name
                        );
                    }
                    other => panic!("{}: expected deny, got {other:?}", case.name),
                }
            }
        }
    }

    #[test]
    fn heater_deny_forces_safe() {
        let decision = washer_rules().evaluate(
            &Snapshot::new().with("water_present", false),
            &Command::new("aout.heater_enable", true),
        );
        match decision {
            Decision::Deny { force_safe, .. } => {
                assert_eq!(force_safe.len(), 1);
                assert_eq!(force_safe[0].channel, "aout.heater_enable");
                assert_eq!(force_safe[0].value, Value::Bool(false));
            }
            other => panic!("expected deny, got {other:?}"),
        }
    }

    #[test]
    fn yaml_roundtrip_washer_rules() {
        let rules = washer_rules();
        let yaml = serde_yaml::to_string(&rules).unwrap();
        let back = RuleSet::from_yaml_str(&yaml).unwrap();
        assert_eq!(back.rules.len(), 2);
        let json = serde_json::to_string(&rules).unwrap();
        let from_json = RuleSet::from_json_str(&json).unwrap();
        assert_eq!(from_json, rules);

        let loaded = RuleSet::from_yaml_str(
            r#"
rules:
  - id: il.heater
    when: { channel: aout.heater_enable, eq: true }
    require:
      and:
        - { channel: water_present, eq: true }
        - { channel: door_locked, eq: true }
    reason: heater requires water_present and door locked
    actions:
      - action: deny_actuator
        channel: aout.heater_enable
"#,
        )
        .unwrap();
        let allow = loaded.evaluate(
            &Snapshot::new()
                .with("water_present", true)
                .with("door_locked", true),
            &Command::new("aout.heater_enable", true),
        );
        assert!(allow.is_allow());
        let deny = loaded.evaluate(
            &Snapshot::new()
                .with("water_present", false)
                .with("door_locked", true),
            &Command::new("aout.heater_enable", true),
        );
        assert!(deny.is_deny());
    }
}
