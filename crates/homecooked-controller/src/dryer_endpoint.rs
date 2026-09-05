//! Thin HomeCooked **device-role** adapter over [`DryerController`] for lab TCP.
//!
//! Same pattern as [`crate::endpoint::ControllerEndpoint`]: advertise a small
//! dryer capability whose writes map onto MockHal channels (`dryer_rules`
//! interlocks). Denied heater commands surface as
//! [`ErrorCode::SafetyInterlock`].
//!
//! Also exposes catalog cycle points so a TCP client can **start dryer cotton**
//! (`trait.cycle.start` → [`DryerController::start_dry`]) and read
//! `trait.cycle.cycle_state` / `trait.cycle.cycle_phase`. Clients write
//! DryOptions knobs as adjacent catalog setpoints (`class.dryer.dryness`,
//! `class.dryer.heat_level`) **before** void `trait.cycle.start` — same order
//! as washer-dryer-io §7 (maps onto [`DryOptions`] humidity / temp targets).
//! Lab-only `class.dryer.sim_tick` advances one host sim tick (cancel / pause /
//! typical_capability remain follow-ups). No TLS / OAuth.

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

use crate::cycle::DryOptions;
use crate::dryer_controller::DryerController;
use crate::error::Error;
use crate::plant;

/// Stable lab device id for [`DryerControllerEndpoint::dryer_lab`].
pub const DRYER_CTRL_DEVICE_ID: &str = "dryer-ctrl-lab";

/// Lab point → HAL channel mapping for the dryer controller endpoint.
const HEATER_POINT: &str = "class.dryer.heater_enable";
const DOOR_LOCK_POINT: &str = "class.dryer.door_lock";
const BLOWER_POINT: &str = "class.dryer.blower";
const DOOR_LOCK_FB_POINT: &str = "class.dryer.door_lock_fb";
/// Catalog DryOptions setpoints (written before `trait.cycle.start`).
const DRYNESS_POINT: &str = "class.dryer.dryness";
const HEAT_LEVEL_POINT: &str = "class.dryer.heat_level";

const HEATER_CHANNEL: &str = "aout.heater_enable";
const DOOR_LOCK_CHANNEL: &str = "aout.door_lock";
const BLOWER_CHANNEL: &str = "aout.blower";
const DOOR_LOCK_FB_CHANNEL: &str = "din.door_lock_fb";

/// Catalog cycle points (host controller naming).
const CYCLE_START_POINT: &str = "trait.cycle.start";
const CYCLE_STATE_POINT: &str = "trait.cycle.cycle_state";
const CYCLE_PHASE_POINT: &str = "trait.cycle.cycle_phase";
/// Lab-only class point: one host [`DryerController::tick`] (not a catalog point).
const LAB_TICK_POINT: &str = "class.dryer.sim_tick";

/// Dryer controller exposed as a single HomeCooked device (lab / smoke).
#[derive(Debug)]
pub struct DryerControllerEndpoint {
    device_id: String,
    capability: CapabilityModel,
    controller: DryerController,
    /// Pending DryOptions applied on the next `trait.cycle.start`.
    dry_opts: DryOptions,
    /// Wire token for [`DRYNESS_POINT`] (maps onto [`DryOptions::target_humidity_rh`]).
    dryness: String,
    /// Wire token for [`HEAT_LEVEL_POINT`] (maps onto [`DryOptions::target_temp_c`]).
    heat_level: String,
}

impl DryerControllerEndpoint {
    /// Dryer cotton demo HAL + interlocks, advertised as [`DRYER_CTRL_DEVICE_ID`].
    pub fn dryer_lab() -> Result<Self, Error> {
        Self::dryer_named(DRYER_CTRL_DEVICE_ID)
    }

    /// Same as [`Self::dryer_lab`] with a custom device id.
    pub fn dryer_named(device_id: impl Into<String>) -> Result<Self, Error> {
        Ok(Self {
            device_id: device_id.into(),
            capability: lab_dryer_capability(),
            controller: DryerController::dryer_cotton_demo()?,
            dry_opts: DryOptions::default(),
            dryness: DEFAULT_DRYNESS.into(),
            heat_level: DEFAULT_HEAT_LEVEL.into(),
        })
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn controller(&self) -> &DryerController {
        &self.controller
    }

    pub fn controller_mut(&mut self) -> &mut DryerController {
        &mut self.controller
    }

    pub fn capability(&self) -> &CapabilityModel {
        &self.capability
    }

    /// Dispatch one protocol request (same contract as washer [`crate::ControllerEndpoint`]).
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
        let mut hello = HelloRecord::new(&self.device_id, ApplianceClassId::Dryer);
        hello.catalog_version = CATALOG_VERSION;
        hello.trait_ids = self.capability.traits.iter().map(|t| t.trait_id).collect();
        hello.display_name = Some("Dryer controller lab".into());
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
            Some(c) => c == ApplianceClassId::Dryer,
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
            DRYNESS_POINT => Ok(Value::Enum(self.dryness.clone())),
            HEAT_LEVEL_POINT => Ok(Value::Enum(self.heat_level.clone())),
            _ => {
                let channel = point_to_channel(point_id).ok_or_else(|| {
                    ErrorBody::new(
                        ErrorCode::UnknownVariable,
                        format!("unknown point {point_id}"),
                    )
                    .at_point(point_id)
                })?;
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
                hal_to_value(&raw)
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
                    .start_dry(self.dry_opts.clone())
                    .map_err(|e| cycle_error_body(&point_id, e))
            }
            DRYNESS_POINT => {
                let token = match &op.value {
                    Value::Enum(s) => s.as_str(),
                    _ => {
                        return Err(ErrorBody::new(ErrorCode::InvalidType, "expected enum")
                            .at_point(&point_id))
                    }
                };
                let rh = dryness_to_humidity_rh(token).ok_or_else(|| {
                    ErrorBody::new(ErrorCode::InvalidEnum, format!("unknown dryness {token}"))
                        .at_point(&point_id)
                })?;
                self.dryness = token.into();
                self.dry_opts.target_humidity_rh = rh;
                Ok(())
            }
            HEAT_LEVEL_POINT => {
                let token = match &op.value {
                    Value::Enum(s) => s.as_str(),
                    _ => {
                        return Err(ErrorBody::new(ErrorCode::InvalidType, "expected enum")
                            .at_point(&point_id))
                    }
                };
                let temp = heat_level_to_temp_c(token).ok_or_else(|| {
                    ErrorBody::new(
                        ErrorCode::InvalidEnum,
                        format!("unknown heat_level {token}"),
                    )
                    .at_point(&point_id)
                })?;
                self.heat_level = token.into();
                self.dry_opts.target_temp_c = temp;
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
            HEATER_POINT | DOOR_LOCK_POINT | BLOWER_POINT => {
                let channel = point_to_channel(&point_id).expect("mapped above");
                let hv = value_to_hal(&op.value).ok_or_else(|| {
                    ErrorBody::new(ErrorCode::InvalidType, "expected bool").at_point(&point_id)
                })?;
                plant::refresh_dryer_derived(self.controller.hal_mut()).map_err(|e| {
                    ErrorBody::new(ErrorCode::Internal, e.to_string()).at_point(&point_id)
                })?;
                match bridge::write_channel(self.controller.hal_mut(), channel, hv) {
                    Ok(()) => {
                        // Mirror lock / blower into plant feedback + derived
                        // keys so the next heater write sees a consistent
                        // door_locked / blower_on snapshot.
                        if channel == DOOR_LOCK_CHANNEL || channel == BLOWER_CHANNEL {
                            let _ = plant::step_dryer_plant(self.controller.hal_mut());
                            let _ = plant::refresh_dryer_derived(self.controller.hal_mut());
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
                format!("{other} is not writable on dryer controller lab endpoint"),
            )
            .at_point(other)),
        }
    }
}

impl RequestHandler for DryerControllerEndpoint {
    fn handle(&mut self, request: Envelope) -> Envelope {
        self.handle_request(request)
    }
}

/// Thin lab capability: HAL-backed dryer heater / door / blower points + DryOptions
/// setpoints + cycle start/read.
pub fn lab_dryer_capability() -> CapabilityModel {
    let mut cap = CapabilityModel::new(ApplianceClassId::Dryer);
    cap.class_version = DEFAULT_CLASS_VERSION;
    let dryer = class_table(ApplianceClassId::Dryer).expect("dryer class table");
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
            id: BLOWER_POINT.into(),
            value_type: ValueType::Bool,
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
        // Catalog DryOptions knobs (write before trait.cycle.start).
        PointCapability::from_catalog(
            DRYNESS_POINT,
            dryer.class_point("dryness").expect("dryer dryness"),
        ),
        PointCapability::from_catalog(
            HEAT_LEVEL_POINT,
            dryer.class_point("heat_level").expect("dryer heat_level"),
        ),
        // Lab-only tick (not catalog): advance host dryer cotton runtime one step.
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
    cap.traits.push(TraitCapability {
        trait_id: TraitId::DoorLid,
        trait_version: DEFAULT_CLASS_VERSION,
        points: Vec::new(),
    });
    // Catalog cycle points: start dryer cotton + observe state/phase over TCP.
    let cycle = trait_table(TraitId::Cycle).expect("cycle trait table");
    let mut cycle_points = Vec::new();
    for id in ["start", "cycle_state", "cycle_phase"] {
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

/// Defaults match [`DryOptions::default`] humidity / temp targets.
const DEFAULT_DRYNESS: &str = "cupboard";
const DEFAULT_HEAT_LEVEL: &str = "medium";

/// Catalog `class.dryer.dryness` → [`DryOptions::target_humidity_rh`].
fn dryness_to_humidity_rh(token: &str) -> Option<f64> {
    match token {
        "iron" => Some(40.0),
        "cupboard" => Some(25.0),
        "extra" => Some(15.0),
        "damp" => Some(50.0),
        _ => None,
    }
}

/// Catalog `class.dryer.heat_level` → [`DryOptions::target_temp_c`].
fn heat_level_to_temp_c(token: &str) -> Option<f64> {
    match token {
        "low" => Some(40.0),
        "medium" => Some(55.0),
        "high" => Some(70.0),
        "air" => Some(25.0),
        _ => None,
    }
}

fn cycle_error_body(point_id: &str, err: Error) -> ErrorBody {
    let msg = err.to_string();
    let code = match &err {
        Error::Cycle(m) if m.contains("already running") => ErrorCode::Busy,
        Error::Cycle(m) if m.contains("door") => ErrorCode::SafetyInterlock,
        _ => ErrorCode::Internal,
    };
    ErrorBody::new(code, msg).at_point(point_id)
}

fn point_to_channel(point_id: &str) -> Option<&'static str> {
    match point_id {
        HEATER_POINT => Some(HEATER_CHANNEL),
        DOOR_LOCK_POINT => Some(DOOR_LOCK_CHANNEL),
        BLOWER_POINT => Some(BLOWER_CHANNEL),
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

fn hal_to_value(raw: &HalValue) -> Result<Value, ErrorBody> {
    match raw {
        HalValue::Bool(b) => Ok(Value::Bool(*b)),
        HalValue::Number(n) => Ok(Value::Bool(*n != 0.0)),
    }
}

#[cfg(test)]
mod dryer_endpoint_tests {
    use super::*;
    use homecooked_protocol::{WriteOp, WriteRequest};
    use homecooked_schema::QualifiedPointId;

    fn qid(s: &str) -> QualifiedPointId {
        QualifiedPointId::parse(s).unwrap()
    }

    #[test]
    fn heater_denied_when_door_unlocked_via_handle() {
        let mut ep = DryerControllerEndpoint::dryer_lab().unwrap();
        // Blower on so the only failing require is the door lock.
        let blower = Envelope::request(
            Some(ep.device_id().into()),
            Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: qid(BLOWER_POINT),
                    value: Value::Bool(true),
                }],
                dry_run: false,
                atomic: false,
            }),
        );
        assert!(matches!(
            ep.handle_request(blower).payload,
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
                    body.message.contains("interlock") || body.message.contains("door"),
                    "{}",
                    body.message
                );
            }
            other => panic!("expected safety_interlock, got {other:?}"),
        }
    }

    #[test]
    fn heater_allowed_with_lock_and_blower() {
        let mut ep = DryerControllerEndpoint::dryer_lab().unwrap();
        for (point, value) in [
            (DOOR_LOCK_POINT, Value::Bool(true)),
            (BLOWER_POINT, Value::Bool(true)),
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
    fn dry_start_and_phase_via_handle() {
        let mut ep = DryerControllerEndpoint::dryer_lab().unwrap();
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
                        s == "heating" || s == "drying" || s == "cooling" || !s.is_empty(),
                        "unexpected phase {s}"
                    ),
                    other => panic!("expected string phase, got {other:?}"),
                }
            }
            other => panic!("expected ReadOk running, got {other:?}"),
        }

        // One lab tick should keep cycle running (phase may stay heating while locking).
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
    fn dry_options_over_wire_applied_on_start() {
        let mut ep = DryerControllerEndpoint::dryer_lab().unwrap();

        // Defaults advertised / readable before writes.
        let read_defaults = Envelope::request(
            Some(ep.device_id().into()),
            Payload::Read(homecooked_protocol::ReadRequest {
                points: vec![qid(DRYNESS_POINT), qid(HEAT_LEVEL_POINT)],
                allow_partial: false,
            }),
        );
        match ep.handle_request(read_defaults).payload {
            Payload::ReadOk(body) => {
                assert_eq!(body.values[0].value, Some(Value::Enum("cupboard".into())));
                assert_eq!(body.values[1].value, Some(Value::Enum("medium".into())));
            }
            other => panic!("expected ReadOk defaults, got {other:?}"),
        }

        for (point, value) in [
            (DRYNESS_POINT, Value::Enum("extra".into())),
            (HEAT_LEVEL_POINT, Value::Enum("high".into())),
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
        assert_eq!(opts.target_humidity_rh, 15.0);
        assert_eq!(opts.target_temp_c, 70.0);
        // Host-only tick knobs stay at DryOptions defaults.
        assert_eq!(opts.max_dry_ticks, DryOptions::default().max_dry_ticks);
        assert_eq!(ep.controller().cycle_state().as_str(), "running");
    }
}
