//! Structural checks and optional capability validation.

use std::collections::{HashMap, HashSet};

use homecooked_schema::{CapabilityModel, Value};

use crate::document::{Procedure, Step, StepAction};
use crate::error::Error;

/// Optional capability lookup keyed by role and/or device id.
pub type CapabilityMap<'a> = HashMap<String, &'a CapabilityModel>;

impl Procedure {
    /// Unique step ids, required fields per action, positive timeouts.
    pub fn validate(&self) -> Result<(), Error> {
        self.validate_with_capabilities(None)
    }

    /// Structural checks plus write/command values against advertised caps.
    pub fn validate_with_capabilities(
        &self,
        caps: Option<&CapabilityMap<'_>>,
    ) -> Result<(), Error> {
        if self.id.trim().is_empty() {
            return Err(Error::invalid("procedure id must not be empty"));
        }
        if self.steps.is_empty() {
            return Err(Error::invalid("procedure must have at least one step"));
        }

        let mut seen = HashSet::new();
        for step in &self.steps {
            if step.id.trim().is_empty() {
                return Err(Error::invalid("step id must not be empty"));
            }
            if !seen.insert(step.id.as_str()) {
                return Err(Error::at_step(&step.id, "duplicate step id"));
            }
            validate_step(step)?;
            if let Some(caps) = caps {
                validate_step_caps(self, step, caps)?;
            }
        }
        Ok(())
    }
}

fn validate_step(step: &Step) -> Result<(), Error> {
    if let Some(t) = step.timeout_s {
        if t == 0 {
            return Err(Error::at_step(&step.id, "timeout_s must be positive"));
        }
    }

    match step.action {
        StepAction::Write => {
            require_point(step)?;
            if step.value.is_none() {
                return Err(Error::at_step(&step.id, "write requires value and point"));
            }
        }
        StepAction::Command => {
            require_point(step)?;
        }
        StepAction::Read => {
            require_point(step)?;
        }
        StepAction::Wait => {
            if step.timeout_s.is_none() && step.guards().is_empty() {
                return Err(Error::at_step(
                    &step.id,
                    "wait requires timeout_s and/or a guard",
                ));
            }
        }
        StepAction::Assert => {
            if step.guards().is_empty() {
                return Err(Error::at_step(&step.id, "assert requires a guard"));
            }
        }
        StepAction::ThermalWait => {
            match step.reservoir_id.as_deref() {
                Some(id) if !id.is_empty() => {}
                _ => {
                    return Err(Error::at_step(
                        &step.id,
                        "thermal_wait requires reservoir_id",
                    ));
                }
            }
            if step.cmp.is_none() {
                return Err(Error::at_step(&step.id, "thermal_wait requires cmp"));
            }
            if step.temp_c.is_none() {
                return Err(Error::at_step(&step.id, "thermal_wait requires temp_c"));
            }
            if step.timeout_s.is_none() {
                return Err(Error::at_step(&step.id, "thermal_wait requires timeout_s"));
            }
            if step.requeue_offer {
                if step.from_port.is_none() {
                    return Err(Error::at_step(
                        &step.id,
                        "thermal_wait requeue_offer requires from_port",
                    ));
                }
                let has_to_port = step.to_port.is_some();
                let has_to_res =
                    matches!(step.to_reservoir_id.as_deref(), Some(id) if !id.is_empty());
                match (has_to_port, has_to_res) {
                    (true, false) | (false, true) => {}
                    (true, true) => {
                        return Err(Error::at_step(
                            &step.id,
                            "thermal_wait requeue_offer requires exactly one of to_port or to_reservoir_id",
                        ));
                    }
                    (false, false) => {
                        return Err(Error::at_step(
                            &step.id,
                            "thermal_wait requeue_offer requires to_port or to_reservoir_id",
                        ));
                    }
                }
                if step.power_w.is_none() {
                    return Err(Error::at_step(
                        &step.id,
                        "thermal_wait requeue_offer requires power_w",
                    ));
                }
                if let Some(band) = step.power_w {
                    if band.min > band.max {
                        return Err(Error::at_step(
                            &step.id,
                            "thermal_wait requeue_offer power_w min must be <= max",
                        ));
                    }
                    if band.max == 0 {
                        return Err(Error::at_step(
                            &step.id,
                            "thermal_wait requeue_offer power_w max must be > 0",
                        ));
                    }
                }
            }
        }
        StepAction::ThermalOffer => {
            if step.from_port.is_none() {
                return Err(Error::at_step(&step.id, "thermal_offer requires from_port"));
            }
            let has_to_port = step.to_port.is_some();
            let has_to_res = matches!(step.to_reservoir_id.as_deref(), Some(id) if !id.is_empty());
            match (has_to_port, has_to_res) {
                (true, false) | (false, true) => {}
                (true, true) => {
                    return Err(Error::at_step(
                        &step.id,
                        "thermal_offer requires exactly one of to_port or to_reservoir_id",
                    ));
                }
                (false, false) => {
                    return Err(Error::at_step(
                        &step.id,
                        "thermal_offer requires to_port or to_reservoir_id",
                    ));
                }
            }
            if step.power_w.is_none() {
                return Err(Error::at_step(&step.id, "thermal_offer requires power_w"));
            }
            if let Some(0) = step.duration_s {
                return Err(Error::at_step(
                    &step.id,
                    "thermal_offer duration_s must be positive when set",
                ));
            }
            if let Some(band) = step.power_w {
                if band.min > band.max {
                    return Err(Error::at_step(
                        &step.id,
                        "thermal_offer power_w min must be <= max",
                    ));
                }
                if band.max == 0 {
                    return Err(Error::at_step(
                        &step.id,
                        "thermal_offer power_w max must be > 0",
                    ));
                }
            }
            if let Some(band) = step.fallback_power_w {
                if band.min > band.max {
                    return Err(Error::at_step(
                        &step.id,
                        "thermal_offer fallback_power_w min must be <= max",
                    ));
                }
                if band.max == 0 {
                    return Err(Error::at_step(
                        &step.id,
                        "thermal_offer fallback_power_w max must be > 0",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn require_point(step: &Step) -> Result<(), Error> {
    match step.point() {
        Some(p) if !p.is_empty() => Ok(()),
        _ => Err(Error::at_step(
            &step.id,
            format!("{} requires a target point", action_name(step.action)),
        )),
    }
}

fn action_name(action: StepAction) -> &'static str {
    match action {
        StepAction::Read => "read",
        StepAction::Write => "write",
        StepAction::Command => "command",
        StepAction::Wait => "wait",
        StepAction::Assert => "assert",
        StepAction::ThermalWait => "thermal_wait",
        StepAction::ThermalOffer => "thermal_offer",
    }
}

fn validate_step_caps(
    procedure: &Procedure,
    step: &Step,
    caps: &CapabilityMap<'_>,
) -> Result<(), Error> {
    if !matches!(step.action, StepAction::Write | StepAction::Command) {
        return Ok(());
    }
    let Some(point) = step.point() else {
        return Ok(());
    };
    let value = step.value.as_ref().unwrap_or(&Value::Void);
    if let Some(cap) = lookup_cap(procedure, step, caps) {
        cap.validate_write(point, value)?;
    }
    Ok(())
}

fn lookup_cap<'a>(
    procedure: &Procedure,
    step: &Step,
    caps: &CapabilityMap<'a>,
) -> Option<&'a CapabilityModel> {
    if let Some(t) = &step.target {
        if let Some(id) = t.device_id.as_deref() {
            if let Some(cap) = caps.get(id) {
                return Some(*cap);
            }
        }
        if let Some(role) = t.role.as_deref() {
            if let Some(cap) = caps.get(role) {
                return Some(*cap);
            }
            if let Some(dev) = procedure.device_ref(role) {
                if let Some(id) = &dev.device_id {
                    if let Some(cap) = caps.get(id.as_str()) {
                        return Some(*cap);
                    }
                }
            }
        }
    }
    None
}
