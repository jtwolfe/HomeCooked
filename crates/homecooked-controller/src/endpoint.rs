//! Thin HomeCooked **device-role** adapter over [`Controller`] for lab TCP.
//!
//! Advertises a small washer capability whose writes map onto MockHal channels
//! (I/O map + washer interlocks). Denied actuator commands surface as
//! [`ErrorCode::SafetyInterlock`] — the Stream 4 controller-sim-over-TCP smoke.
//!
//! Also exposes catalog cycle points so a TCP client can **start cotton**
//! (`trait.cycle.start` → [`Controller::start_cotton`]) and read
//! `trait.cycle.cycle_state` / `trait.cycle.cycle_phase`. Clients write
//! CottonOptions knobs as adjacent catalog setpoints (`class.washer.wash_temp_c`,
//! `class.washer.spin_rpm`) **before** void `trait.cycle.start` — same order as
//! washer-dryer-io §6. Lab-only `class.washer.sim_tick` advances one host sim
//! tick. Void writes to `trait.cycle.pause` / `resume` / `cancel` map onto host
//! pause/resume/cancel (typical_capability remains follow-up; dryer DryOptions
//! live on [`crate::DryerControllerEndpoint`]).
//! Dryer TCP: see [`crate::DryerControllerEndpoint`]. No TLS / OAuth.

use homecooked_hal::{bridge, ChannelId, HalValue};
use homecooked_protocol::{
    check_protocol_version, DescribeRequest, DescribeResponse, DiscoverRequest, DiscoverResponse,
    Envelope, ErrorBody, HelloRecord, Payload, PointValue, PongBody, ReadRequest, ReadResponse,
    WriteOp, WriteRequest, WriteResponse,
};
use homecooked_schema::{
    class_table, trait_table, AccessMode, ApplianceClassId, CapabilityModel, CommandArg, ErrorCode,
    PointCapability, TraitCapability, TraitId, Value, ValueRange, ValueType, CATALOG_VERSION,
    DEFAULT_CLASS_VERSION,
};
use homecooked_transport::RequestHandler;

use crate::controller::Controller;
use crate::cycle::CottonOptions;
use crate::error::Error;
use crate::plant;

/// Stable lab device id for [`ControllerEndpoint::washer_lab`].
pub const WASHER_CTRL_DEVICE_ID: &str = "washer-ctrl-lab";

/// Lab point → HAL channel mapping for the washer controller endpoint.
const HEATER_POINT: &str = "class.washer.heater_enable";
const DOOR_LOCK_POINT: &str = "class.washer.door_lock";
const WATER_LEVEL_POINT: &str = "class.washer.water_level_pa";
const DOOR_LOCK_FB_POINT: &str = "class.washer.door_lock_fb";
/// Catalog CottonOptions setpoints (written before `trait.cycle.start`).
const WASH_TEMP_POINT: &str = "class.washer.wash_temp_c";
const SPIN_RPM_POINT: &str = "class.washer.spin_rpm";

const HEATER_CHANNEL: &str = "aout.heater_enable";
const DOOR_LOCK_CHANNEL: &str = "aout.door_lock";
const WATER_LEVEL_CHANNEL: &str = "ain.water_level_pa";
const DOOR_LOCK_FB_CHANNEL: &str = "din.door_lock_fb";

/// Catalog cycle points (host controller naming).
const CYCLE_START_POINT: &str = "trait.cycle.start";
const CYCLE_PAUSE_POINT: &str = "trait.cycle.pause";
const CYCLE_RESUME_POINT: &str = "trait.cycle.resume";
const CYCLE_CANCEL_POINT: &str = "trait.cycle.cancel";
const CYCLE_STATE_POINT: &str = "trait.cycle.cycle_state";
const CYCLE_PHASE_POINT: &str = "trait.cycle.cycle_phase";
/// Lab-only class point: one host [`Controller::tick`] (not a catalog point).
const LAB_TICK_POINT: &str = "class.washer.sim_tick";

/// Washer controller exposed as a single HomeCooked device (lab / smoke).
#[derive(Debug)]
pub struct ControllerEndpoint {
    device_id: String,
    capability: CapabilityModel,
    controller: Controller,
    /// Pending CottonOptions applied on the next `trait.cycle.start`.
    cotton_opts: CottonOptions,
}

impl ControllerEndpoint {
    /// Washer cotton demo HAL + interlocks, advertised as [`WASHER_CTRL_DEVICE_ID`].
    pub fn washer_lab() -> Result<Self, Error> {
        Self::washer_named(WASHER_CTRL_DEVICE_ID)
    }

    /// Same as [`Self::washer_lab`] with a custom device id.
    pub fn washer_named(device_id: impl Into<String>) -> Result<Self, Error> {
        Ok(Self {
            device_id: device_id.into(),
            capability: lab_washer_capability(),
            controller: Controller::washer_cotton_demo()?,
            cotton_opts: CottonOptions::default(),
        })
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn controller(&self) -> &Controller {
        &self.controller
    }

    pub fn controller_mut(&mut self) -> &mut Controller {
        &mut self.controller
    }

    pub fn capability(&self) -> &CapabilityModel {
        &self.capability
    }

    /// Dispatch one protocol request (same contract as [`homecooked_sim::Simulator::handle`]).
    pub fn handle_request(&mut self, request: Envelope) -> Envelope {
        if let Err(err) = check_protocol_version(request.protocol_version) {
            return Envelope::error_to(&request, err.to_error_body());
        }

        match &request.payload {
            Payload::Discover(body) => self.handle_discover(&request, body),
            Payload::Describe(body) => self.handle_describe(&request, body),
            Payload::Read(body) => self.handle_read(&request, body),
            Payload::Write(body) => self.handle_write(&request, body),
            Payload::Ping(body) => Envelope::respond_to(
                &request,
                Payload::Pong(PongBody {
                    echo: body.echo.clone(),
                }),
            ),
            Payload::Subscribe(_) | Payload::Unsubscribe(_) => Envelope::error_to(
                &request,
                ErrorBody::new(
                    ErrorCode::UnsupportedOperation,
                    "subscribe not supported on controller lab endpoint",
                ),
            ),
            Payload::Pong(_)
            | Payload::DiscoverOk(_)
            | Payload::DescribeOk(_)
            | Payload::ReadOk(_)
            | Payload::WriteOk(_)
            | Payload::SubscribeOk(_)
            | Payload::UnsubscribeOk(_)
            | Payload::Event(_)
            | Payload::Error(_)
            | Payload::CapsChanged(_) => Envelope::error_to(
                &request,
                ErrorBody::invalid_request(format!("{} is not a request", request.kind().as_str())),
            ),
        }
    }

    fn hello(&self) -> HelloRecord {
        let mut hello = HelloRecord::new(&self.device_id, ApplianceClassId::Washer);
        hello.catalog_version = CATALOG_VERSION;
        hello.trait_ids = self.capability.traits.iter().map(|t| t.trait_id).collect();
        hello.display_name = Some("Washer controller lab".into());
        hello
    }

    fn require_device(&self, request: &Envelope) -> Result<(), ErrorBody> {
        match &request.device_id {
            Some(id) if id == &self.device_id => Ok(()),
            Some(id) if !id.is_empty() => Err(ErrorBody::unknown_device(id)),
            _ => Err(ErrorBody::invalid_request("device_id is required")),
        }
    }

    fn handle_discover(&self, request: &Envelope, body: &DiscoverRequest) -> Envelope {
        if let Some(id) = request.device_id.as_deref() {
            if !id.is_empty() && id != self.device_id {
                return Envelope::error_to(request, ErrorBody::unknown_device(id));
            }
        }
        let mut devices = Vec::new();
        let class_ok = match body.class_id {
            None => true,
            Some(c) => c == ApplianceClassId::Washer,
        };
        if class_ok
            && body
                .trait_ids
                .iter()
                .all(|t| self.capability.advertises_trait(*t))
        {
            devices.push(self.hello());
        }
        Envelope::respond_to(request, Payload::DiscoverOk(DiscoverResponse { devices }))
    }

    fn handle_describe(&self, request: &Envelope, body: &DescribeRequest) -> Envelope {
        if let Err(err) = self.require_device(request) {
            return Envelope::error_to(request, err);
        }
        let capability = if body.points.is_empty() {
            self.capability.clone()
        } else {
            let mut filtered = self.capability.clone();
            let wanted: Vec<String> = body.points.iter().map(|p| p.to_string()).collect();
            for id in &wanted {
                if self.capability.point(id).is_none() {
                    return Envelope::error_to(
                        request,
                        ErrorBody::new(ErrorCode::UnknownVariable, format!("unknown point {id}"))
                            .at_point(id),
                    );
                }
            }
            filtered
                .class_points
                .retain(|p| wanted.iter().any(|w| w == &p.id || w == p.base_id()));
            for trait_cap in &mut filtered.traits {
                trait_cap
                    .points
                    .retain(|p| wanted.iter().any(|w| w == &p.id || w == p.base_id()));
            }
            filtered
        };
        Envelope::respond_to(
            request,
            Payload::DescribeOk(Box::new(DescribeResponse { capability })),
        )
    }

    fn handle_read(&self, request: &Envelope, body: &ReadRequest) -> Envelope {
        if let Err(err) = self.require_device(request) {
            return Envelope::error_to(request, err);
        }
        if body.points.is_empty() {
            return Envelope::error_to(
                request,
                ErrorBody::invalid_request("read points list must not be empty"),
            );
        }
        let mut values = Vec::new();
        for qid in &body.points {
            let point_id = qid.to_string();
            let Some(cap) = self.capability.point(&point_id) else {
                return Envelope::error_to(
                    request,
                    ErrorBody::new(
                        ErrorCode::UnknownVariable,
                        format!("unknown point {point_id}"),
                    )
                    .at_point(&point_id),
                );
            };
            if !cap.access.is_readable() {
                return Envelope::error_to(
                    request,
                    ErrorBody::new(
                        ErrorCode::NotReadable,
                        format!("{point_id} is not readable"),
                    )
                    .at_point(&point_id),
                );
            }
            match self.read_point(&point_id) {
                Ok(value) => values.push(PointValue::new(qid.clone(), value, 0)),
                Err(err) => {
                    return Envelope::error_to(request, err);
                }
            }
        }
        Envelope::respond_to(
            request,
            Payload::ReadOk(ReadResponse {
                values,
                errors: Vec::new(),
            }),
        )
    }

    fn handle_write(&mut self, request: &Envelope, body: &WriteRequest) -> Envelope {
        if let Err(err) = self.require_device(request) {
            return Envelope::error_to(request, err);
        }
        if body.writes.is_empty() {
            return Envelope::error_to(
                request,
                ErrorBody::invalid_request("write list must not be empty"),
            );
        }

        // Validate against advertised capability first (type / access).
        let mut accepted = Vec::new();
        for op in &body.writes {
            let point_id = op.id.to_string();
            if let Err(err) = self.capability.validate_write(&point_id, &op.value) {
                return Envelope::error_to(request, ErrorBody::from(err));
            }
            accepted.push(op.clone());
        }

        if body.dry_run {
            return Envelope::respond_to(request, Payload::WriteOk(WriteResponse { accepted }));
        }

        for op in &accepted {
            if let Err(err) = self.apply_write(op) {
                return Envelope::error_to(request, err);
            }
        }
        Envelope::respond_to(request, Payload::WriteOk(WriteResponse { accepted }))
    }

    fn read_point(&self, point_id: &str) -> Result<Value, ErrorBody> {
        match point_id {
            CYCLE_STATE_POINT => Ok(Value::Enum(self.controller.cycle_state().as_str().into())),
            CYCLE_PHASE_POINT => {
                let phase = self.controller.phase().as_str();
                // Catalog phase is empty when idle; advertise a stable idle token.
                Ok(Value::String(if phase.is_empty() {
                    "idle".into()
                } else {
                    phase.into()
                }))
            }
            WASH_TEMP_POINT => Ok(Value::F32(self.cotton_opts.wash_temp_c as f32)),
            SPIN_RPM_POINT => {
                let rpm = self
                    .cotton_opts
                    .spin_rpm
                    .round()
                    .clamp(0.0, u16::MAX as f64) as u16;
                Ok(Value::U16(rpm))
            }
            _ => {
                let channel = point_to_channel(point_id).ok_or_else(|| {
                    ErrorBody::new(
                        ErrorCode::UnknownVariable,
                        format!("unknown point {point_id}"),
                    )
                    .at_point(point_id)
                })?;
                // Prefer MockHal::get so actuator outputs (aout.*) are readable in lab.
                let id = ChannelId::new(channel).map_err(|e| {
                    ErrorBody::new(ErrorCode::Internal, e.to_string()).at_point(point_id)
                })?;
                let raw = self
                    .controller
                    .hal()
                    .get(&id)
                    .map_err(|e| {
                        ErrorBody::new(ErrorCode::Internal, format!("hal get {channel}: {e}"))
                            .at_point(point_id)
                    })?
                    .clone();
                hal_to_value(&raw, point_id)
            }
        }
    }

    fn apply_write(&mut self, op: &WriteOp) -> Result<(), ErrorBody> {
        let point_id = op.id.to_string();
        match point_id.as_str() {
            CYCLE_START_POINT => {
                if !matches!(op.value, Value::Void) {
                    return Err(
                        ErrorBody::new(ErrorCode::InvalidType, "expected void").at_point(&point_id)
                    );
                }
                self.controller
                    .start_cotton(self.cotton_opts.clone())
                    .map_err(|e| cycle_error_body(&point_id, e))
            }
            CYCLE_PAUSE_POINT => {
                if !matches!(op.value, Value::Void) {
                    return Err(
                        ErrorBody::new(ErrorCode::InvalidType, "expected void").at_point(&point_id)
                    );
                }
                self.controller
                    .pause()
                    .map_err(|e| cycle_error_body(&point_id, e))
            }
            CYCLE_RESUME_POINT => {
                if !matches!(op.value, Value::Void) {
                    return Err(
                        ErrorBody::new(ErrorCode::InvalidType, "expected void").at_point(&point_id)
                    );
                }
                self.controller
                    .resume()
                    .map_err(|e| cycle_error_body(&point_id, e))
            }
            CYCLE_CANCEL_POINT => {
                if !matches!(op.value, Value::Void) {
                    return Err(
                        ErrorBody::new(ErrorCode::InvalidType, "expected void").at_point(&point_id)
                    );
                }
                self.controller
                    .cancel()
                    .map_err(|e| cycle_error_body(&point_id, e))
            }
            WASH_TEMP_POINT => {
                let n = op.value.as_f64().ok_or_else(|| {
                    ErrorBody::new(ErrorCode::InvalidType, "expected number").at_point(&point_id)
                })?;
                self.cotton_opts.wash_temp_c = n;
                Ok(())
            }
            SPIN_RPM_POINT => {
                let n = op.value.as_f64().ok_or_else(|| {
                    ErrorBody::new(ErrorCode::InvalidType, "expected number").at_point(&point_id)
                })?;
                self.cotton_opts.spin_rpm = n;
                Ok(())
            }
            LAB_TICK_POINT => {
                if !matches!(op.value, Value::Void) {
                    return Err(
                        ErrorBody::new(ErrorCode::InvalidType, "expected void").at_point(&point_id)
                    );
                }
                self.controller
                    .tick()
                    .map_err(|e| cycle_error_body(&point_id, e))
            }
            WATER_LEVEL_POINT => {
                let n = op.value.as_f64().ok_or_else(|| {
                    ErrorBody::new(ErrorCode::InvalidType, "expected number").at_point(&point_id)
                })?;
                let id = ChannelId::new(WATER_LEVEL_CHANNEL).map_err(|e| {
                    ErrorBody::new(ErrorCode::Internal, e.to_string()).at_point(&point_id)
                })?;
                self.controller.hal_mut().inject(&id, n).map_err(|e| {
                    ErrorBody::new(ErrorCode::Internal, e.to_string()).at_point(&point_id)
                })?;
                plant::refresh_derived(self.controller.hal_mut()).map_err(|e| {
                    ErrorBody::new(ErrorCode::Internal, e.to_string()).at_point(&point_id)
                })?;
                Ok(())
            }
            HEATER_POINT | DOOR_LOCK_POINT => {
                let channel = point_to_channel(&point_id).expect("mapped above");
                let hv = value_to_hal(&op.value).ok_or_else(|| {
                    ErrorBody::new(ErrorCode::InvalidType, "expected bool").at_point(&point_id)
                })?;
                plant::refresh_derived(self.controller.hal_mut()).map_err(|e| {
                    ErrorBody::new(ErrorCode::Internal, e.to_string()).at_point(&point_id)
                })?;
                match bridge::write_channel(self.controller.hal_mut(), channel, hv) {
                    Ok(()) => {
                        // Mirror lock command into feedback so the next heater
                        // write sees a consistent door_locked snapshot.
                        if channel == DOOR_LOCK_CHANNEL {
                            let _ = plant::step_plant(self.controller.hal_mut());
                            let _ = plant::refresh_derived(self.controller.hal_mut());
                        }
                        Ok(())
                    }
                    Err(homecooked_hal::Error::InterlockDenied { channel, reason }) => {
                        Err(ErrorBody::new(
                            ErrorCode::SafetyInterlock,
                            format!("interlock denied {channel}: {reason}"),
                        )
                        .at_point(&point_id))
                    }
                    Err(e) => {
                        Err(ErrorBody::new(ErrorCode::Internal, e.to_string()).at_point(&point_id))
                    }
                }
            }
            other => Err(ErrorBody::new(
                ErrorCode::NotWritable,
                format!("{other} is not writable on controller lab endpoint"),
            )
            .at_point(other)),
        }
    }
}

impl RequestHandler for ControllerEndpoint {
    fn handle(&mut self, request: Envelope) -> Envelope {
        self.handle_request(request)
    }
}

/// Thin lab capability: HAL-backed washer heater / door / water points + CottonOptions
/// setpoints + cycle start/pause/resume/cancel + read.
pub fn lab_washer_capability() -> CapabilityModel {
    let mut cap = CapabilityModel::new(ApplianceClassId::Washer);
    cap.class_version = DEFAULT_CLASS_VERSION;
    let washer = class_table(ApplianceClassId::Washer).expect("washer class table");
    cap.class_points = vec![
        PointCapability {
            id: HEATER_POINT.into(),
            value_type: ValueType::Bool,
            unit: None,
            access: AccessMode::RW,
            required: true,
            range: None,
            resolution: None,
            zones: None,
        },
        PointCapability {
            id: DOOR_LOCK_POINT.into(),
            value_type: ValueType::Bool,
            unit: None,
            access: AccessMode::RW,
            required: true,
            range: None,
            resolution: None,
            zones: None,
        },
        PointCapability {
            id: WATER_LEVEL_POINT.into(),
            value_type: ValueType::F32,
            unit: None,
            access: AccessMode::RW,
            required: true,
            range: None,
            resolution: None,
            zones: None,
        },
        PointCapability {
            id: DOOR_LOCK_FB_POINT.into(),
            value_type: ValueType::Bool,
            unit: None,
            access: AccessMode::R,
            required: true,
            range: None,
            resolution: None,
            zones: None,
        },
        // Catalog CottonOptions knobs (write before trait.cycle.start).
        PointCapability::from_catalog(
            WASH_TEMP_POINT,
            washer
                .class_point("wash_temp_c")
                .expect("washer wash_temp_c"),
        ),
        PointCapability::from_catalog(
            SPIN_RPM_POINT,
            washer.class_point("spin_rpm").expect("washer spin_rpm"),
        ),
        // Lab-only tick (not catalog): advance host cotton runtime one step.
        PointCapability {
            id: LAB_TICK_POINT.into(),
            value_type: ValueType::Command,
            unit: None,
            access: AccessMode::W,
            required: false,
            range: Some(ValueRange::CommandArg {
                arg: CommandArg::Void,
            }),
            resolution: None,
            zones: None,
        },
    ];
    // Advertise door_lid so Discover trait filters can match laundry clients.
    cap.traits.push(TraitCapability {
        trait_id: TraitId::DoorLid,
        trait_version: DEFAULT_CLASS_VERSION,
        points: Vec::new(),
    });
    // Catalog cycle points: start/pause/resume/cancel + observe state/phase.
    let cycle = trait_table(TraitId::Cycle).expect("cycle trait table");
    let mut cycle_points = Vec::new();
    for id in [
        "start",
        "pause",
        "resume",
        "cancel",
        "cycle_state",
        "cycle_phase",
    ] {
        let p = cycle
            .point(id)
            .unwrap_or_else(|| panic!("missing trait.cycle.{id}"));
        cycle_points.push(PointCapability::from_catalog(
            format!("trait.cycle.{id}"),
            p,
        ));
    }
    cap.traits.push(TraitCapability {
        trait_id: TraitId::Cycle,
        trait_version: DEFAULT_CLASS_VERSION,
        points: cycle_points,
    });
    cap
}

fn cycle_error_body(point_id: &str, err: Error) -> ErrorBody {
    let msg = err.to_string();
    let code = match &err {
        Error::Cycle(m) if m.contains("already running") => ErrorCode::Busy,
        Error::Cycle(m) if m.contains("door") => ErrorCode::SafetyInterlock,
        Error::Cycle(m)
            if m.contains("no active") || m.contains("not running") || m.contains("not paused") =>
        {
            ErrorCode::InvalidRequest
        }
        _ => ErrorCode::Internal,
    };
    ErrorBody::new(code, msg).at_point(point_id)
}

fn point_to_channel(point_id: &str) -> Option<&'static str> {
    match point_id {
        HEATER_POINT => Some(HEATER_CHANNEL),
        DOOR_LOCK_POINT => Some(DOOR_LOCK_CHANNEL),
        WATER_LEVEL_POINT => Some(WATER_LEVEL_CHANNEL),
        DOOR_LOCK_FB_POINT => Some(DOOR_LOCK_FB_CHANNEL),
        _ => None,
    }
}

fn value_to_hal(value: &Value) -> Option<HalValue> {
    match value {
        Value::Bool(b) => Some(HalValue::Bool(*b)),
        _ => None,
    }
}

fn hal_to_value(raw: &HalValue, point_id: &str) -> Result<Value, ErrorBody> {
    match (point_id, raw) {
        (WATER_LEVEL_POINT, HalValue::Number(n)) => Ok(Value::F32(*n as f32)),
        (WATER_LEVEL_POINT, _) => Err(ErrorBody::new(
            ErrorCode::Internal,
            "water_level_pa expected number",
        )
        .at_point(point_id)),
        (_, HalValue::Bool(b)) => Ok(Value::Bool(*b)),
        (_, HalValue::Number(n)) => Ok(Value::Bool(*n != 0.0)),
    }
}

#[cfg(test)]
mod endpoint_tests {
    use super::*;
    use homecooked_protocol::{WriteOp, WriteRequest};
    use homecooked_schema::QualifiedPointId;

    fn qid(s: &str) -> QualifiedPointId {
        QualifiedPointId::parse(s).unwrap()
    }

    #[test]
    fn heater_denied_without_water_via_handle() {
        let mut ep = ControllerEndpoint::washer_lab().unwrap();
        // Door locked, water empty.
        let lock = Envelope::request(
            Some(ep.device_id().into()),
            Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: qid(DOOR_LOCK_POINT),
                    value: Value::Bool(true),
                }],
                dry_run: false,
                atomic: false,
            }),
        );
        assert!(matches!(
            ep.handle_request(lock).payload,
            Payload::WriteOk(_)
        ));

        let heat = Envelope::request(
            Some(ep.device_id().into()),
            Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: qid(HEATER_POINT),
                    value: Value::Bool(true),
                }],
                dry_run: false,
                atomic: false,
            }),
        );
        match ep.handle_request(heat).payload {
            Payload::Error(body) => {
                assert_eq!(body.code, ErrorCode::SafetyInterlock);
                assert!(
                    body.message.contains("interlock") || body.message.contains("water"),
                    "{}",
                    body.message
                );
            }
            other => panic!("expected safety_interlock, got {other:?}"),
        }
    }

    #[test]
    fn heater_allowed_with_water_and_lock() {
        let mut ep = ControllerEndpoint::washer_lab().unwrap();
        for (point, value) in [
            (DOOR_LOCK_POINT, Value::Bool(true)),
            (WATER_LEVEL_POINT, Value::F32(2_000.0)),
        ] {
            let env = Envelope::request(
                Some(ep.device_id().into()),
                Payload::Write(WriteRequest {
                    writes: vec![WriteOp {
                        id: qid(point),
                        value,
                    }],
                    dry_run: false,
                    atomic: false,
                }),
            );
            assert!(
                matches!(ep.handle_request(env).payload, Payload::WriteOk(_)),
                "setup write {point}"
            );
        }

        let heat = Envelope::request(
            Some(ep.device_id().into()),
            Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: qid(HEATER_POINT),
                    value: Value::Bool(true),
                }],
                dry_run: false,
                atomic: false,
            }),
        );
        assert!(matches!(
            ep.handle_request(heat).payload,
            Payload::WriteOk(_)
        ));
        assert_eq!(
            ep.controller()
                .hal()
                .last_command(HEATER_CHANNEL)
                .map(|c| c.value.clone()),
            Some(HalValue::Bool(true))
        );
    }

    #[test]
    fn cotton_start_and_phase_via_handle() {
        let mut ep = ControllerEndpoint::washer_lab().unwrap();
        let idle = Envelope::request(
            Some(ep.device_id().into()),
            Payload::Read(homecooked_protocol::ReadRequest {
                points: vec![qid(CYCLE_STATE_POINT), qid(CYCLE_PHASE_POINT)],
                allow_partial: false,
            }),
        );
        match ep.handle_request(idle).payload {
            Payload::ReadOk(body) => {
                assert_eq!(body.values[0].value, Some(Value::Enum("idle".into())));
                assert_eq!(body.values[1].value, Some(Value::String("idle".into())));
            }
            other => panic!("expected ReadOk idle, got {other:?}"),
        }

        let start = Envelope::request(
            Some(ep.device_id().into()),
            Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: qid(CYCLE_START_POINT),
                    value: Value::Void,
                }],
                dry_run: false,
                atomic: false,
            }),
        );
        assert!(matches!(
            ep.handle_request(start).payload,
            Payload::WriteOk(_)
        ));
        assert_eq!(ep.controller().cycle_state().as_str(), "running");

        let running = Envelope::request(
            Some(ep.device_id().into()),
            Payload::Read(homecooked_protocol::ReadRequest {
                points: vec![qid(CYCLE_STATE_POINT), qid(CYCLE_PHASE_POINT)],
                allow_partial: false,
            }),
        );
        match ep.handle_request(running).payload {
            Payload::ReadOk(body) => {
                assert_eq!(body.values[0].value, Some(Value::Enum("running".into())));
                let phase = body.values[1].value.as_ref().unwrap();
                match phase {
                    Value::String(s) => assert!(
                        s == "fill" || s == "wash" || s == "drain" || !s.is_empty(),
                        "unexpected phase {s}"
                    ),
                    other => panic!("expected string phase, got {other:?}"),
                }
            }
            other => panic!("expected ReadOk running, got {other:?}"),
        }

        // One lab tick should keep cycle running (phase may stay fill while locking).
        let tick = Envelope::request(
            Some(ep.device_id().into()),
            Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: qid(LAB_TICK_POINT),
                    value: Value::Void,
                }],
                dry_run: false,
                atomic: false,
            }),
        );
        assert!(matches!(
            ep.handle_request(tick).payload,
            Payload::WriteOk(_)
        ));
        assert_eq!(ep.controller().cycle_state().as_str(), "running");
    }

    #[test]
    fn cotton_options_over_wire_applied_on_start() {
        let mut ep = ControllerEndpoint::washer_lab().unwrap();

        // Defaults advertised / readable before writes.
        let read_defaults = Envelope::request(
            Some(ep.device_id().into()),
            Payload::Read(homecooked_protocol::ReadRequest {
                points: vec![qid(WASH_TEMP_POINT), qid(SPIN_RPM_POINT)],
                allow_partial: false,
            }),
        );
        match ep.handle_request(read_defaults).payload {
            Payload::ReadOk(body) => {
                assert_eq!(body.values[0].value, Some(Value::F32(40.0)));
                assert_eq!(body.values[1].value, Some(Value::U16(800)));
            }
            other => panic!("expected ReadOk defaults, got {other:?}"),
        }

        for (point, value) in [
            (WASH_TEMP_POINT, Value::F32(0.0)),
            (SPIN_RPM_POINT, Value::U16(1_200)),
        ] {
            let env = Envelope::request(
                Some(ep.device_id().into()),
                Payload::Write(WriteRequest {
                    writes: vec![WriteOp {
                        id: qid(point),
                        value,
                    }],
                    dry_run: false,
                    atomic: false,
                }),
            );
            assert!(
                matches!(ep.handle_request(env).payload, Payload::WriteOk(_)),
                "write {point}"
            );
        }

        let start = Envelope::request(
            Some(ep.device_id().into()),
            Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: qid(CYCLE_START_POINT),
                    value: Value::Void,
                }],
                dry_run: false,
                atomic: false,
            }),
        );
        assert!(matches!(
            ep.handle_request(start).payload,
            Payload::WriteOk(_)
        ));

        let opts = ep.controller().options();
        assert_eq!(opts.wash_temp_c, 0.0);
        assert_eq!(opts.spin_rpm, 1_200.0);
        // Host-only tick knobs stay at CottonOptions defaults.
        assert_eq!(
            opts.wash_tumble_ticks,
            CottonOptions::default().wash_tumble_ticks
        );
        assert_eq!(ep.controller().cycle_state().as_str(), "running");
    }

    #[test]
    fn cycle_pause_resume_cancel_via_handle() {
        let mut ep = ControllerEndpoint::washer_lab().unwrap();
        let id = ep.device_id().to_string();

        // Cancel while idle → invalid_request.
        let cancel_idle = Envelope::request(
            Some(id.clone()),
            Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: qid(CYCLE_CANCEL_POINT),
                    value: Value::Void,
                }],
                dry_run: false,
                atomic: false,
            }),
        );
        match ep.handle_request(cancel_idle).payload {
            Payload::Error(body) => assert_eq!(body.code, ErrorCode::InvalidRequest),
            other => panic!("expected invalid_request cancel idle, got {other:?}"),
        }

        let start = Envelope::request(
            Some(id.clone()),
            Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: qid(CYCLE_START_POINT),
                    value: Value::Void,
                }],
                dry_run: false,
                atomic: false,
            }),
        );
        assert!(matches!(
            ep.handle_request(start).payload,
            Payload::WriteOk(_)
        ));

        // A few ticks so we're mid-cycle.
        for _ in 0..3 {
            let tick = Envelope::request(
                Some(id.clone()),
                Payload::Write(WriteRequest {
                    writes: vec![WriteOp {
                        id: qid(LAB_TICK_POINT),
                        value: Value::Void,
                    }],
                    dry_run: false,
                    atomic: false,
                }),
            );
            assert!(matches!(
                ep.handle_request(tick).payload,
                Payload::WriteOk(_)
            ));
        }

        let pause = Envelope::request(
            Some(id.clone()),
            Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: qid(CYCLE_PAUSE_POINT),
                    value: Value::Void,
                }],
                dry_run: false,
                atomic: false,
            }),
        );
        assert!(matches!(
            ep.handle_request(pause).payload,
            Payload::WriteOk(_)
        ));
        assert_eq!(ep.controller().cycle_state().as_str(), "paused");

        // Phase frozen across paused ticks.
        let phase_paused = ep.controller().phase().as_str().to_string();
        for _ in 0..3 {
            let tick = Envelope::request(
                Some(id.clone()),
                Payload::Write(WriteRequest {
                    writes: vec![WriteOp {
                        id: qid(LAB_TICK_POINT),
                        value: Value::Void,
                    }],
                    dry_run: false,
                    atomic: false,
                }),
            );
            assert!(matches!(
                ep.handle_request(tick).payload,
                Payload::WriteOk(_)
            ));
        }
        assert_eq!(ep.controller().cycle_state().as_str(), "paused");
        assert_eq!(ep.controller().phase().as_str(), phase_paused);

        let resume = Envelope::request(
            Some(id.clone()),
            Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: qid(CYCLE_RESUME_POINT),
                    value: Value::Void,
                }],
                dry_run: false,
                atomic: false,
            }),
        );
        assert!(matches!(
            ep.handle_request(resume).payload,
            Payload::WriteOk(_)
        ));
        assert_eq!(ep.controller().cycle_state().as_str(), "running");

        let cancel = Envelope::request(
            Some(id.clone()),
            Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: qid(CYCLE_CANCEL_POINT),
                    value: Value::Void,
                }],
                dry_run: false,
                atomic: false,
            }),
        );
        assert!(matches!(
            ep.handle_request(cancel).payload,
            Payload::WriteOk(_)
        ));
        assert_eq!(ep.controller().cycle_state().as_str(), "canceling");

        for _ in 0..40 {
            if ep.controller().cycle_state().as_str() == "idle" {
                break;
            }
            let tick = Envelope::request(
                Some(id.clone()),
                Payload::Write(WriteRequest {
                    writes: vec![WriteOp {
                        id: qid(LAB_TICK_POINT),
                        value: Value::Void,
                    }],
                    dry_run: false,
                    atomic: false,
                }),
            );
            assert!(matches!(
                ep.handle_request(tick).payload,
                Payload::WriteOk(_)
            ));
        }
        assert_eq!(ep.controller().cycle_state().as_str(), "idle");
    }
}
