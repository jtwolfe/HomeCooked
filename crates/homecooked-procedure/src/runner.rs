//! Sequential procedure execution.

use std::collections::HashMap;
use std::fmt;

use homecooked_schema::{ErrorCode, Value};

use crate::backend::DeviceBackend;
use crate::document::{Procedure, Step, StepAction, StepTarget};
use crate::error::Error;
use crate::guard::Guard;

/// Default simulated poll interval while waiting on a guard (1 s of sim time).
pub const DEFAULT_POLL_INTERVAL_MS: u64 = 1_000;

/// Role → device id map used when a step names a role instead of a device.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeviceBindings {
    by_role: HashMap<String, String>,
}

impl DeviceBindings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(mut self, role: impl Into<String>, device_id: impl Into<String>) -> Self {
        self.insert(role, device_id);
        self
    }

    pub fn insert(&mut self, role: impl Into<String>, device_id: impl Into<String>) {
        self.by_role.insert(role.into(), device_id.into());
    }

    pub fn get(&self, role: &str) -> Option<&str> {
        self.by_role.get(role).map(String::as_str)
    }

    pub fn device_ids(&self) -> impl Iterator<Item = &str> {
        self.by_role.values().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.by_role.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_role.is_empty()
    }
}

/// Wait polling configuration (simulated time, not wall clock).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunConfig {
    pub poll_interval_ms: u64,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: DEFAULT_POLL_INTERVAL_MS,
        }
    }
}

/// Why a run stopped on a step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailReason {
    Validation(String),
    GuardFailed(String),
    Timeout,
    UnboundDevice { role: Option<String> },
    Backend { code: ErrorCode, message: String },
}

impl fmt::Display for FailReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(msg) => write!(f, "validation: {msg}"),
            Self::GuardFailed(msg) => write!(f, "guard failed: {msg}"),
            Self::Timeout => write!(f, "timeout"),
            Self::UnboundDevice { role } => match role {
                Some(r) => write!(f, "unbound device role {r}"),
                None => write!(f, "unbound device"),
            },
            Self::Backend { code, message } => {
                write!(f, "{code}")?;
                if !message.is_empty() {
                    write!(f, ": {message}")?;
                }
                Ok(())
            }
        }
    }
}

/// Per-step record.
#[derive(Debug, Clone, PartialEq)]
pub struct StepOutcome {
    pub step_id: String,
    pub action: StepAction,
    pub ok: bool,
    pub read_value: Option<Value>,
    pub message: Option<String>,
}

/// Final status after sequential execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunStatus {
    Completed,
    Failed { step_id: String, reason: FailReason },
}

/// Structured result of [`run`] / [`run_with_config`].
#[derive(Debug, Clone, PartialEq)]
pub struct RunResult {
    pub status: RunStatus,
    pub outcomes: Vec<StepOutcome>,
}

impl RunResult {
    pub fn is_completed(&self) -> bool {
        matches!(self.status, RunStatus::Completed)
    }

    pub fn failed_step(&self) -> Option<&str> {
        match &self.status {
            RunStatus::Failed { step_id, .. } => Some(step_id.as_str()),
            RunStatus::Completed => None,
        }
    }

    pub fn fail_reason(&self) -> Option<&FailReason> {
        match &self.status {
            RunStatus::Failed { reason, .. } => Some(reason),
            RunStatus::Completed => None,
        }
    }
}

/// Execute `procedure` against `backend` using `bindings`.
pub fn run(
    procedure: &Procedure,
    backend: &mut impl DeviceBackend,
    bindings: &DeviceBindings,
) -> RunResult {
    run_with_config(procedure, backend, bindings, &RunConfig::default())
}

/// Execute with an explicit poll interval for Wait steps.
pub fn run_with_config(
    procedure: &Procedure,
    backend: &mut impl DeviceBackend,
    bindings: &DeviceBindings,
    config: &RunConfig,
) -> RunResult {
    if let Err(err) = procedure.validate() {
        return RunResult {
            status: RunStatus::Failed {
                step_id: err_step_id(&err),
                reason: FailReason::Validation(err.to_string()),
            },
            outcomes: Vec::new(),
        };
    }

    let mut outcomes = Vec::with_capacity(procedure.steps.len());
    for step in &procedure.steps {
        match execute_step(procedure, step, backend, bindings, config) {
            Ok(outcome) => outcomes.push(outcome),
            Err((outcome, reason)) => {
                let step_id = outcome.step_id.clone();
                outcomes.push(outcome);
                return RunResult {
                    status: RunStatus::Failed { step_id, reason },
                    outcomes,
                };
            }
        }
    }

    RunResult {
        status: RunStatus::Completed,
        outcomes,
    }
}

fn err_step_id(err: &Error) -> String {
    match err {
        Error::Invalid {
            step_id: Some(id), ..
        } => id.clone(),
        _ => "_validate".to_string(),
    }
}

fn execute_step(
    procedure: &Procedure,
    step: &Step,
    backend: &mut impl DeviceBackend,
    bindings: &DeviceBindings,
    config: &RunConfig,
) -> Result<StepOutcome, (StepOutcome, FailReason)> {
    let fail = |reason: FailReason, message: Option<String>| {
        (
            StepOutcome {
                step_id: step.id.clone(),
                action: step.action,
                ok: false,
                read_value: None,
                message,
            },
            reason,
        )
    };

    let device = match resolve_device(procedure, step.target.as_ref(), bindings) {
        Ok(Some(id)) => Some(id),
        Ok(None) => None,
        Err(reason) => {
            return Err(fail(reason, Some("could not bind device".into())));
        }
    };

    match step.action {
        StepAction::Read => {
            let device = device.ok_or_else(|| fail_unbound(step))?;
            let point = step.point().expect("validated");
            let value = backend
                .read(&device, point)
                .map_err(|e| map_backend(step, e))?;
            if let Err(reason) = eval_guards(step.guards(), backend, &device) {
                return Err(fail(reason, Some("read guard failed".into())));
            }
            Ok(ok_outcome(step, Some(value), None))
        }
        StepAction::Write | StepAction::Command => {
            let device = device.ok_or_else(|| fail_unbound(step))?;
            let point = step.point().expect("validated");
            let value = step.value.clone().unwrap_or(Value::Void);
            backend
                .write(&device, point, &value)
                .map_err(|e| map_backend(step, e))?;
            Ok(ok_outcome(step, None, None))
        }
        StepAction::Assert => {
            let device = device.ok_or_else(|| fail_unbound(step))?;
            if let Err(reason) = eval_guards(step.guards(), backend, &device) {
                return Err(fail(reason, Some("assert failed".into())));
            }
            Ok(ok_outcome(step, None, None))
        }
        StepAction::Wait => wait_step(step, backend, device.as_deref(), bindings, config),
    }
}

fn fail_unbound(step: &Step) -> (StepOutcome, FailReason) {
    let role = step.role().map(str::to_string);
    (
        StepOutcome {
            step_id: step.id.clone(),
            action: step.action,
            ok: false,
            read_value: None,
            message: Some("unbound device".into()),
        },
        FailReason::UnboundDevice { role },
    )
}

fn ok_outcome(step: &Step, read_value: Option<Value>, message: Option<String>) -> StepOutcome {
    StepOutcome {
        step_id: step.id.clone(),
        action: step.action,
        ok: true,
        read_value,
        message,
    }
}

fn map_backend(step: &Step, err: Error) -> (StepOutcome, FailReason) {
    let (code, message) = match &err {
        Error::Backend { code, message, .. } => (*code, message.clone()),
        Error::Capability(v) => (v.code, v.message.clone()),
        other => (ErrorCode::Internal, other.to_string()),
    };
    (
        StepOutcome {
            step_id: step.id.clone(),
            action: step.action,
            ok: false,
            read_value: None,
            message: Some(message.clone()),
        },
        FailReason::Backend { code, message },
    )
}

fn resolve_device(
    procedure: &Procedure,
    target: Option<&StepTarget>,
    bindings: &DeviceBindings,
) -> Result<Option<String>, FailReason> {
    if let Some(t) = target {
        if let Some(id) = t.device_id.as_deref() {
            if !id.is_empty() {
                return Ok(Some(id.to_string()));
            }
        }
        if let Some(role) = t.role.as_deref() {
            if let Some(id) = bindings.get(role) {
                return Ok(Some(id.to_string()));
            }
            if let Some(dev) = procedure.device_ref(role) {
                if let Some(id) = &dev.device_id {
                    return Ok(Some(id.clone()));
                }
                if dev.optional {
                    return Ok(None);
                }
            }
            return Err(FailReason::UnboundDevice {
                role: Some(role.to_string()),
            });
        }
    }

    if bindings.len() == 1 {
        return Ok(bindings.device_ids().next().map(str::to_string));
    }
    if target.is_none() && bindings.is_empty() {
        return Ok(None);
    }
    Ok(None)
}

fn eval_guards(
    guards: &[Guard],
    backend: &mut impl DeviceBackend,
    device_id: &str,
) -> Result<(), FailReason> {
    for guard in guards {
        let got = backend
            .read(device_id, &guard.point)
            .map_err(|e| FailReason::Backend {
                code: match &e {
                    Error::Backend { code, .. } => *code,
                    _ => ErrorCode::Internal,
                },
                message: e.to_string(),
            })?;
        if !guard.eval(&got) {
            return Err(FailReason::GuardFailed(format!(
                "{} {:?} (got {got:?})",
                guard.point, guard.op
            )));
        }
    }
    Ok(())
}

fn wait_step(
    step: &Step,
    backend: &mut impl DeviceBackend,
    device: Option<&str>,
    bindings: &DeviceBindings,
    config: &RunConfig,
) -> Result<StepOutcome, (StepOutcome, FailReason)> {
    let has_guard = !step.guards().is_empty();
    let timeout_ms = step.timeout_s.map(|s| u64::from(s) * 1_000);
    let poll = config.poll_interval_ms.max(1);
    let mut elapsed = 0_u64;

    loop {
        if has_guard {
            if let Some(id) = device {
                match eval_guards(step.guards(), backend, id) {
                    Ok(()) => {
                        return Ok(ok_outcome(
                            step,
                            None,
                            Some(format!("wait satisfied after {elapsed} ms")),
                        ));
                    }
                    Err(FailReason::GuardFailed(_)) => {}
                    Err(other) => {
                        return Err((
                            StepOutcome {
                                step_id: step.id.clone(),
                                action: step.action,
                                ok: false,
                                read_value: None,
                                message: Some(other.to_string()),
                            },
                            other,
                        ));
                    }
                }
            }
        }

        if let Some(limit) = timeout_ms {
            if elapsed >= limit {
                if has_guard {
                    return Err((
                        StepOutcome {
                            step_id: step.id.clone(),
                            action: step.action,
                            ok: false,
                            read_value: None,
                            message: Some(format!("wait timed out after {elapsed} ms")),
                        },
                        FailReason::Timeout,
                    ));
                }
                return Ok(ok_outcome(step, None, Some(format!("waited {elapsed} ms"))));
            }
        } else if !has_guard {
            return Ok(ok_outcome(step, None, None));
        }

        let remaining = timeout_ms.map(|limit| limit.saturating_sub(elapsed));
        let dt = remaining.map(|r| r.min(poll)).unwrap_or(poll);
        tick_devices(backend, device, bindings, dt).map_err(|e| map_backend(step, e))?;
        elapsed = elapsed.saturating_add(dt);
    }
}

fn tick_devices(
    backend: &mut impl DeviceBackend,
    device: Option<&str>,
    bindings: &DeviceBindings,
    dt_ms: u64,
) -> Result<(), Error> {
    if let Some(id) = device {
        return backend.tick(id, dt_ms);
    }
    let mut seen = std::collections::HashSet::new();
    for id in bindings.device_ids() {
        if seen.insert(id.to_string()) {
            backend.tick(id, dt_ms)?;
        }
    }
    Ok(())
}
