//! Boolean and comparison conditions on snapshot values.

use serde::{Deserialize, Serialize};

use crate::value::{Snapshot, Value};

/// Predicate over channel / point values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Condition {
    And { and: Vec<Condition> },
    Or { or: Vec<Condition> },
    Cmp(Compare),
}

/// Comparison against one channel or point key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Compare {
    pub channel: String,
    #[serde(flatten)]
    pub op: CmpOp,
}

/// Equality and numeric comparisons.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CmpOp {
    Eq(Value),
    Neq(Value),
    Gt(Value),
    Gte(Value),
    Lt(Value),
    Lte(Value),
}

impl Condition {
    pub fn and(parts: Vec<Condition>) -> Self {
        Self::And { and: parts }
    }

    pub fn or(parts: Vec<Condition>) -> Self {
        Self::Or { or: parts }
    }

    pub fn eq(channel: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Cmp(Compare {
            channel: channel.into(),
            op: CmpOp::Eq(value.into()),
        })
    }

    pub fn neq(channel: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Cmp(Compare {
            channel: channel.into(),
            op: CmpOp::Neq(value.into()),
        })
    }

    pub fn gt(channel: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Cmp(Compare {
            channel: channel.into(),
            op: CmpOp::Gt(value.into()),
        })
    }

    pub fn gte(channel: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Cmp(Compare {
            channel: channel.into(),
            op: CmpOp::Gte(value.into()),
        })
    }

    pub fn lt(channel: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Cmp(Compare {
            channel: channel.into(),
            op: CmpOp::Lt(value.into()),
        })
    }

    pub fn lte(channel: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Cmp(Compare {
            channel: channel.into(),
            op: CmpOp::Lte(value.into()),
        })
    }

    pub fn eval(&self, snapshot: &Snapshot) -> bool {
        match self {
            Self::And { and } => and.iter().all(|c| c.eval(snapshot)),
            Self::Or { or } => or.iter().any(|c| c.eval(snapshot)),
            Self::Cmp(cmp) => cmp.eval(snapshot),
        }
    }
}

impl Compare {
    fn eval(&self, snapshot: &Snapshot) -> bool {
        let got = snapshot.get(&self.channel);
        match &self.op {
            CmpOp::Eq(expected) => got.map(|v| v.equal_to(expected)).unwrap_or(false),
            CmpOp::Neq(expected) => got.map(|v| !v.equal_to(expected)).unwrap_or(true),
            CmpOp::Gt(expected) => numeric(got, expected, |a, b| a > b),
            CmpOp::Gte(expected) => numeric(got, expected, |a, b| a >= b),
            CmpOp::Lt(expected) => numeric(got, expected, |a, b| a < b),
            CmpOp::Lte(expected) => numeric(got, expected, |a, b| a <= b),
        }
    }
}

fn numeric(got: Option<&Value>, expected: &Value, op: impl Fn(f64, f64) -> bool) -> bool {
    match (got.and_then(Value::as_number), expected.as_number()) {
        (Some(a), Some(b)) => op(a, b),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and_or_cmp() {
        let snap = Snapshot::new()
            .with("water_present", true)
            .with("door_locked", false)
            .with("motor.speed_rpm_cmd", 50);

        assert!(Condition::eq("water_present", true).eval(&snap));
        assert!(!Condition::eq("door_locked", true).eval(&snap));
        assert!(Condition::neq("door_locked", true).eval(&snap));
        assert!(Condition::lt("motor.speed_rpm_cmd", 400).eval(&snap));
        assert!(!Condition::gte("motor.speed_rpm_cmd", 400).eval(&snap));
        assert!(Condition::and(vec![
            Condition::eq("water_present", true),
            Condition::eq("door_locked", false),
        ])
        .eval(&snap));
        assert!(Condition::or(vec![
            Condition::eq("door_locked", true),
            Condition::eq("water_present", true),
        ])
        .eval(&snap));
        assert!(!Condition::and(vec![
            Condition::eq("water_present", true),
            Condition::eq("door_locked", true),
        ])
        .eval(&snap));
        assert!(!Condition::eq("missing", true).eval(&snap));
        assert!(Condition::neq("missing", true).eval(&snap));
    }
}
