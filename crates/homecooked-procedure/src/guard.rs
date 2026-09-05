//! Simple comparison guards (AND of point vs value).

use serde::{Deserialize, Serialize};

use homecooked_schema::Value;

/// One comparison on a qualified point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Guard {
    pub point: String,
    #[serde(flatten)]
    pub op: CmpOp,
}

/// Equality and numeric comparisons.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CmpOp {
    #[serde(rename = "eq")]
    Eq(Value),
    #[serde(rename = "ne", alias = "neq")]
    Ne(Value),
    #[serde(rename = "gt")]
    Gt(Value),
    #[serde(rename = "gte")]
    Gte(Value),
    #[serde(rename = "lt")]
    Lt(Value),
    #[serde(rename = "lte")]
    Lte(Value),
}

/// Zero, one, or many guards (JSON `guard` object or `guards` array).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GuardSet {
    One(Guard),
    Many(Vec<Guard>),
}

impl Default for GuardSet {
    fn default() -> Self {
        Self::Many(Vec::new())
    }
}

impl GuardSet {
    pub fn as_slice(&self) -> &[Guard] {
        match self {
            Self::One(g) => std::slice::from_ref(g),
            Self::Many(v) => v,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Guard> {
        self.as_slice().iter()
    }
}

impl Guard {
    pub fn eq(point: impl Into<String>, value: Value) -> Self {
        Self {
            point: point.into(),
            op: CmpOp::Eq(value),
        }
    }

    pub fn ne(point: impl Into<String>, value: Value) -> Self {
        Self {
            point: point.into(),
            op: CmpOp::Ne(value),
        }
    }

    pub fn gt(point: impl Into<String>, value: Value) -> Self {
        Self {
            point: point.into(),
            op: CmpOp::Gt(value),
        }
    }

    pub fn gte(point: impl Into<String>, value: Value) -> Self {
        Self {
            point: point.into(),
            op: CmpOp::Gte(value),
        }
    }

    pub fn lt(point: impl Into<String>, value: Value) -> Self {
        Self {
            point: point.into(),
            op: CmpOp::Lt(value),
        }
    }

    pub fn lte(point: impl Into<String>, value: Value) -> Self {
        Self {
            point: point.into(),
            op: CmpOp::Lte(value),
        }
    }

    pub fn eval(&self, got: &Value) -> bool {
        self.op.eval(got)
    }
}

impl CmpOp {
    pub fn eval(&self, got: &Value) -> bool {
        match self {
            Self::Eq(expected) => values_eq(got, expected),
            Self::Ne(expected) => !values_eq(got, expected),
            Self::Gt(expected) => numeric(got, expected, |a, b| a > b),
            Self::Gte(expected) => numeric(got, expected, |a, b| a >= b),
            Self::Lt(expected) => numeric(got, expected, |a, b| a < b),
            Self::Lte(expected) => numeric(got, expected, |a, b| a <= b),
        }
    }
}

fn values_eq(got: &Value, expected: &Value) -> bool {
    if got == expected {
        return true;
    }
    match (got.as_f64(), expected.as_f64()) {
        (Some(a), Some(b)) => (a - b).abs() < 1e-6,
        _ => match (enum_or_str(got), enum_or_str(expected)) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        },
    }
}

fn enum_or_str(v: &Value) -> Option<&str> {
    match v {
        Value::Enum(s) | Value::String(s) => Some(s.as_str()),
        _ => None,
    }
}

fn numeric(got: &Value, expected: &Value, op: impl Fn(f64, f64) -> bool) -> bool {
    match (got.as_f64(), expected.as_f64()) {
        (Some(a), Some(b)) => op(a, b),
        _ => false,
    }
}
