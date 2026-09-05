//! Rules, actions, and evaluation against a snapshot + proposed command.

use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::error::Error;
use crate::value::{Command, Snapshot, Value};

/// Set a named channel to a safe value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForceSafe {
    pub channel: String,
    pub value: Value,
}

/// Deny an actuator command, or force named channels to a safe state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    DenyActuator {
        channel: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    ForceSafeState {
        channel: String,
        value: Value,
    },
}

impl Action {
    pub fn deny_actuator(channel: impl Into<String>) -> Self {
        Self::DenyActuator {
            channel: channel.into(),
            reason: None,
        }
    }

    pub fn force_safe(channel: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::ForceSafeState {
            channel: channel.into(),
            value: value.into(),
        }
    }
}

/// One declarative interlock rule.
///
/// A rule **fires** when [`Self::when`] is true on the snapshot (with the
/// proposed command overlaid) and [`Self::require`] is either absent or false.
/// Only then do actions run: matching [`Action::DenyActuator`] may deny the
/// current command; [`Action::ForceSafeState`] always applies on a fire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub when: Condition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require: Option<Condition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<Action>,
}

/// Ordered set of interlock rules.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RuleSet {
    #[serde(default)]
    pub rules: Vec<Rule>,
}

/// Result of evaluating a proposed actuator command.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Allow {
        force_safe: Vec<ForceSafe>,
    },
    Deny {
        reason: String,
        force_safe: Vec<ForceSafe>,
    },
}

impl Decision {
    pub fn is_allow(&self) -> bool {
        matches!(self, Self::Allow { .. })
    }

    pub fn is_deny(&self) -> bool {
        matches!(self, Self::Deny { .. })
    }

    pub fn force_safe(&self) -> &[ForceSafe] {
        match self {
            Self::Allow { force_safe } | Self::Deny { force_safe, .. } => force_safe,
        }
    }
}

impl RuleSet {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self { rules }
    }

    pub fn from_yaml_str(s: &str) -> Result<Self, Error> {
        Ok(serde_yaml::from_str(s)?)
    }

    pub fn from_json_str(s: &str) -> Result<Self, Error> {
        Ok(serde_json::from_str(s)?)
    }

    /// Evaluate `command` against `snapshot` before it is applied.
    ///
    /// Rules fire only when `when` holds and `require` fails (or is absent).
    /// An allowed command gets an empty `force_safe` list unless another
    /// firing rule forces channels (e.g. leak → force drain) without denying
    /// this command.
    pub fn evaluate(&self, snapshot: &Snapshot, command: &Command) -> Decision {
        let view = snapshot.with_command(command);
        let mut force_safe = Vec::new();
        let mut deny_reason: Option<String> = None;

        for rule in &self.rules {
            if !rule.when.eval(&view) {
                continue;
            }
            let require_ok = rule
                .require
                .as_ref()
                .map(|c| c.eval(&view))
                .unwrap_or(false);
            // Fire when require is missing (force-only / always-gate) or failed.
            if require_ok {
                continue;
            }

            for action in &rule.actions {
                match action {
                    Action::ForceSafeState { channel, value } => {
                        force_safe.push(ForceSafe {
                            channel: channel.clone(),
                            value: value.clone(),
                        });
                    }
                    Action::DenyActuator { channel, reason } => {
                        if channel == &command.channel && deny_reason.is_none() {
                            deny_reason =
                                Some(reason.clone().unwrap_or_else(|| rule_reason(rule, channel)));
                        }
                    }
                }
            }
        }

        match deny_reason {
            Some(reason) => Decision::Deny { reason, force_safe },
            None => Decision::Allow { force_safe },
        }
    }
}

fn rule_reason(rule: &Rule, channel: &str) -> String {
    if let Some(reason) = &rule.reason {
        return reason.clone();
    }
    match &rule.id {
        Some(id) => format!("{id} denied {channel}"),
        None => format!("interlock denied {channel}"),
    }
}
