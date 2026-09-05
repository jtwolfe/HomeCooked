//! High-level protocol request handling (describe / read / write / discover).

use homecooked_protocol::{
    check_protocol_version, DescribeRequest, DescribeResponse, DiscoverRequest, DiscoverResponse,
    Envelope, ErrorBody, Payload, PongBody, ReadRequest, ReadResponse, SubscribeRequest,
    SubscribeResponse, UnsubscribeRequest, UnsubscribeResponse, WriteRequest, WriteResponse,
    MAX_READ_POINTS,
};
use homecooked_schema::{CapabilityModel, ErrorCode, PointCapability, Value};

use crate::error::CoreError;
use crate::id::DeviceId;
use crate::registry::{DeviceRegistry, RegisteredDevice};
use crate::validate::{is_command_point, lookup_point, validate_read};

/// Device hub: registry plus request dispatch.
#[derive(Debug, Default)]
pub struct DeviceHub {
    pub registry: DeviceRegistry,
    /// Logical clock used as `ts_ms` on read values (tests set this).
    pub clock_ms: u64,
}

impl DeviceHub {
    pub fn new() -> Self {
        Self {
            registry: DeviceRegistry::new(),
            clock_ms: 0,
        }
    }

    pub fn handle(&mut self, request: &Envelope) -> Envelope {
        if let Err(err) = check_protocol_version(request.protocol_version) {
            return Envelope::error_to(request, err.to_error_body());
        }

        match &request.payload {
            Payload::Discover(body) => self.handle_discover(request, body),
            Payload::Describe(body) => self.handle_describe(request, body),
            Payload::Read(body) => self.handle_read(request, body),
            Payload::Write(body) => self.handle_write(request, body),
            Payload::Subscribe(body) => self.handle_subscribe(request, body),
            Payload::Unsubscribe(body) => self.handle_unsubscribe(request, body),
            Payload::Ping(body) => Envelope::respond_to(
                request,
                Payload::Pong(PongBody {
                    echo: body.echo.clone(),
                }),
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
                request,
                ErrorBody::invalid_request(format!("{} is not a request", request.kind().as_str())),
            ),
        }
    }

    pub fn handle_json(
        &mut self,
        json: &str,
    ) -> Result<String, homecooked_protocol::ProtocolError> {
        let env = Envelope::from_json(json)?;
        Ok(self.handle(&env).to_json()?)
    }

    fn device_id_from(request: &Envelope) -> Result<DeviceId, CoreError> {
        match &request.device_id {
            Some(id) if !id.is_empty() => Ok(DeviceId::new(id)),
            _ => Err(CoreError::invalid_request("device_id is required")),
        }
    }

    fn handle_discover(&self, request: &Envelope, body: &DiscoverRequest) -> Envelope {
        let mut devices = Vec::new();
        if let Some(id) = request.device_id.as_deref() {
            if !id.is_empty() {
                match self.registry.get(&DeviceId::new(id)) {
                    Some(dev) if matches_discover(dev, body) => devices.push(dev.hello()),
                    Some(_) => {}
                    None => {
                        return Envelope::error_to(request, ErrorBody::unknown_device(id));
                    }
                }
                return Envelope::respond_to(
                    request,
                    Payload::DiscoverOk(DiscoverResponse { devices }),
                );
            }
        }
        for dev in self.registry.list() {
            if matches_discover(dev, body) {
                devices.push(dev.hello());
            }
        }
        Envelope::respond_to(request, Payload::DiscoverOk(DiscoverResponse { devices }))
    }

    fn handle_describe(&self, request: &Envelope, body: &DescribeRequest) -> Envelope {
        let id = match Self::device_id_from(request) {
            Ok(id) => id,
            Err(e) => return Envelope::error_to(request, e.into()),
        };
        let dev = match self.registry.get(&id) {
            Some(d) => d,
            None => return Envelope::error_to(request, ErrorBody::unknown_device(id.as_str())),
        };
        match filter_capability(&dev.capability, body) {
            Ok(capability) => Envelope::respond_to(
                request,
                Payload::DescribeOk(Box::new(DescribeResponse { capability })),
            ),
            Err(err) => Envelope::error_to(request, err.into()),
        }
    }

    fn handle_read(&self, request: &Envelope, body: &ReadRequest) -> Envelope {
        let id = match Self::device_id_from(request) {
            Ok(id) => id,
            Err(e) => return Envelope::error_to(request, e.into()),
        };
        match self.read_points(&id, body) {
            Ok(resp) => Envelope::respond_to(request, Payload::ReadOk(resp)),
            Err(err) => Envelope::error_to(request, err.into()),
        }
    }

    fn handle_write(&mut self, request: &Envelope, body: &WriteRequest) -> Envelope {
        let id = match Self::device_id_from(request) {
            Ok(id) => id,
            Err(e) => return Envelope::error_to(request, e.into()),
        };
        match self.write_points(&id, body) {
            Ok(resp) => Envelope::respond_to(request, Payload::WriteOk(resp)),
            Err(err) => Envelope::error_to(request, err.into()),
        }
    }

    fn handle_subscribe(&self, request: &Envelope, body: &SubscribeRequest) -> Envelope {
        let id = match Self::device_id_from(request) {
            Ok(id) => id,
            Err(e) => return Envelope::error_to(request, e.into()),
        };
        if self.registry.get(&id).is_none() {
            return Envelope::error_to(request, ErrorBody::unknown_device(id.as_str()));
        }
        if body.is_empty() {
            return Envelope::error_to(
                request,
                ErrorBody::invalid_request("subscribe requires points, traits, or all"),
            );
        }
        Envelope::respond_to(
            request,
            Payload::SubscribeOk(SubscribeResponse {
                points: body.points.clone(),
                traits: body.traits.clone(),
                all: body.all,
            }),
        )
    }

    fn handle_unsubscribe(&self, request: &Envelope, body: &UnsubscribeRequest) -> Envelope {
        let id = match Self::device_id_from(request) {
            Ok(id) => id,
            Err(e) => return Envelope::error_to(request, e.into()),
        };
        if self.registry.get(&id).is_none() {
            return Envelope::error_to(request, ErrorBody::unknown_device(id.as_str()));
        }
        Envelope::respond_to(
            request,
            Payload::UnsubscribeOk(UnsubscribeResponse {
                points: body.points.clone(),
            }),
        )
    }

    pub fn read_points(&self, id: &DeviceId, req: &ReadRequest) -> Result<ReadResponse, CoreError> {
        if req.points.is_empty() {
            return Err(CoreError::invalid_request(
                "read points list must not be empty",
            ));
        }
        if req.points.len() > MAX_READ_POINTS {
            return Err(CoreError::invalid_request(format!(
                "read supports at most {MAX_READ_POINTS} points"
            )));
        }
        let dev = self.registry.require(id)?;
        let mut values = Vec::new();
        let mut errors = Vec::new();
        for qid in &req.points {
            let point_id = qid.to_string();
            match validate_read(&dev.capability, &point_id) {
                Ok(()) => match dev.state.get(&point_id) {
                    Some(v) => values.push(homecooked_protocol::PointValue::new(
                        qid.clone(),
                        v.clone(),
                        self.clock_ms,
                    )),
                    None => {
                        let point = lookup_point(&dev.capability, &point_id).ok();
                        if point.is_some_and(|p| !p.required) {
                            values.push(homecooked_protocol::PointValue {
                                id: qid.clone(),
                                value: None,
                                ts_ms: self.clock_ms,
                            });
                        } else {
                            let err = ErrorBody::new(
                                ErrorCode::Internal,
                                format!("missing required point {point_id}"),
                            )
                            .at_point(&point_id);
                            if req.allow_partial {
                                errors.push(err);
                            } else {
                                return Err(err.into());
                            }
                        }
                    }
                },
                Err(e) => {
                    if req.allow_partial {
                        errors.push(e.into());
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }
        Ok(ReadResponse { values, errors })
    }

    pub fn write_points(
        &mut self,
        id: &DeviceId,
        req: &WriteRequest,
    ) -> Result<WriteResponse, CoreError> {
        if req.writes.is_empty() {
            return Err(CoreError::invalid_request("write list must not be empty"));
        }
        let dev = self.registry.require(id)?;

        let mut accepted = Vec::new();
        for op in &req.writes {
            let point_id = op.id.to_string();
            if let Err(err) = dev.capability.validate_write(&point_id, &op.value) {
                return Err(err.into());
            }
            accepted.push(op.clone());
        }

        if req.dry_run {
            return Ok(WriteResponse { accepted });
        }

        let dev = self.registry.require_mut(id)?;
        for op in &accepted {
            let point_id = op.id.to_string();
            if !is_command_point(&dev.capability, &point_id) {
                dev.state.insert(point_id, op.value.clone());
            }
        }
        Ok(WriteResponse { accepted })
    }

    /// Apply a single already-validated write (used by the simulator).
    pub fn apply_state(
        &mut self,
        id: &DeviceId,
        point_id: &str,
        value: Value,
    ) -> Result<(), CoreError> {
        let dev = self.registry.require_mut(id)?;
        dev.state.insert(point_id, value);
        Ok(())
    }
}

fn matches_discover(dev: &RegisteredDevice, req: &DiscoverRequest) -> bool {
    if let Some(class_id) = req.class_id {
        if !dev.capability.advertises_class(class_id) {
            return false;
        }
    }
    req.trait_ids
        .iter()
        .all(|t| dev.capability.advertises_trait(*t))
}

fn filter_capability(
    cap: &CapabilityModel,
    req: &DescribeRequest,
) -> Result<CapabilityModel, CoreError> {
    if req.points.is_empty() {
        return Ok(cap.clone());
    }
    let mut filtered = cap.clone();
    let wanted: Vec<String> = req.points.iter().map(|p| p.to_string()).collect();
    for id in &wanted {
        lookup_point(cap, id).map_err(CoreError::from)?;
    }
    for trait_cap in &mut filtered.traits {
        trait_cap.points.retain(|p| point_matches(p, &wanted));
    }
    filtered.class_points.retain(|p| point_matches(p, &wanted));
    Ok(filtered)
}

fn point_matches(point: &PointCapability, wanted: &[String]) -> bool {
    wanted.iter().any(|w| {
        w == &point.id
            || w.split('#').next() == Some(point.base_id())
            || point.id.split('#').next() == Some(w.as_str())
    })
}
