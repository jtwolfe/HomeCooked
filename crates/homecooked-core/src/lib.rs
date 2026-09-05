//! Device registry, capability validation, and protocol request handling.

mod error;
mod handler;
mod id;
mod registry;
mod state;
mod validate;

pub use error::CoreError;
pub use handler::DeviceHub;
pub use id::DeviceId;
pub use registry::{DeviceRegistry, RegisteredDevice};
pub use state::DeviceState;
pub use validate::{is_command_point, lookup_point, validate_read};

#[cfg(test)]
mod tests {
    use homecooked_protocol::{
        Envelope, Payload, ReadRequest, WriteOp, WriteRequest, PROTOCOL_VERSION,
    };
    use homecooked_schema::{
        typical_capability, ApplianceClassId, DeviceIdentity, ErrorCode, QualifiedPointId, SemVer,
        Value,
    };

    use super::*;

    fn washer() -> (
        DeviceIdentity,
        homecooked_schema::CapabilityModel,
        DeviceState,
    ) {
        let identity = DeviceIdentity::new(
            "washer-1",
            "Acme",
            "W100",
            "0.1.0",
            ApplianceClassId::Washer,
        );
        let cap = typical_capability(ApplianceClassId::Washer).unwrap();
        let mut state = DeviceState::new();
        state.insert("class.washer.spin_rpm", Value::U16(800));
        state.insert("class.washer.wash_temp_c", Value::F32(40.0));
        state.insert("trait.program.program", Value::Enum("cotton".into()));
        state.insert("trait.cycle.cycle_state", Value::Enum("idle".into()));
        state.insert("trait.power.power_state", Value::Enum("on".into()));
        (identity, cap, state)
    }

    fn hub_with_washer() -> (DeviceHub, DeviceId) {
        let mut hub = DeviceHub::new();
        let (identity, cap, state) = washer();
        let id = hub.registry.register(identity, cap, state).unwrap();
        (hub, id)
    }

    fn qid(s: &str) -> QualifiedPointId {
        QualifiedPointId::parse(s).unwrap()
    }

    #[test]
    fn register_list_get_unregister() {
        let mut hub = DeviceHub::new();
        let (identity, cap, state) = washer();
        let id = hub.registry.register(identity, cap, state).unwrap();
        assert_eq!(hub.registry.len(), 1);
        assert_eq!(hub.registry.list()[0].identity.device_id, "washer-1");
        assert!(hub.registry.get(&id).is_some());
        hub.registry.unregister(&id).unwrap();
        assert!(hub.registry.is_empty());
        let err = hub.registry.unregister(&id).unwrap_err();
        assert_eq!(err.code, ErrorCode::UnknownDevice);
    }

    #[test]
    fn successful_write_updates_state() {
        let (mut hub, id) = hub_with_washer();
        let req = WriteRequest {
            writes: vec![WriteOp {
                id: qid("class.washer.spin_rpm"),
                value: Value::U16(1200),
            }],
            dry_run: false,
            atomic: false,
        };
        let ok = hub.write_points(&id, &req).unwrap();
        assert_eq!(ok.accepted.len(), 1);
        let stored = hub
            .registry
            .get(&id)
            .unwrap()
            .state
            .get("class.washer.spin_rpm")
            .cloned();
        assert_eq!(stored, Some(Value::U16(1200)));
    }

    #[test]
    fn capability_enforcement_rejects_bad_writes() {
        let (mut hub, id) = hub_with_washer();

        let out_of_range = hub
            .write_points(
                &id,
                &WriteRequest {
                    writes: vec![WriteOp {
                        id: qid("class.washer.spin_rpm"),
                        value: Value::U16(2000),
                    }],
                    dry_run: false,
                    atomic: false,
                },
            )
            .unwrap_err();
        assert_eq!(out_of_range.code, ErrorCode::OutOfRange);
        assert_eq!(
            hub.registry
                .get(&id)
                .unwrap()
                .state
                .get("class.washer.spin_rpm"),
            Some(&Value::U16(800))
        );

        let not_writable = hub
            .write_points(
                &id,
                &WriteRequest {
                    writes: vec![WriteOp {
                        id: qid("trait.cycle.cycle_state"),
                        value: Value::Enum("running".into()),
                    }],
                    dry_run: false,
                    atomic: false,
                },
            )
            .unwrap_err();
        assert_eq!(not_writable.code, ErrorCode::NotWritable);

        let unsupported = hub
            .write_points(
                &id,
                &WriteRequest {
                    writes: vec![WriteOp {
                        id: qid("class.washer.steam"),
                        value: Value::Bool(true),
                    }],
                    dry_run: false,
                    atomic: false,
                },
            )
            .unwrap_err();
        assert_eq!(unsupported.code, ErrorCode::UnsupportedCapability);

        let unknown = hub
            .write_points(
                &id,
                &WriteRequest {
                    writes: vec![WriteOp {
                        id: qid("class.washer.no_such_point"),
                        value: Value::Bool(true),
                    }],
                    dry_run: false,
                    atomic: false,
                },
            )
            .unwrap_err();
        assert_eq!(unknown.code, ErrorCode::UnknownVariable);
    }

    #[test]
    fn unknown_device_errors() {
        let mut hub = DeviceHub::new();
        let missing = DeviceId::new("nope");
        let err = hub
            .read_points(
                &missing,
                &ReadRequest {
                    points: vec![qid("trait.power.power_state")],
                    allow_partial: false,
                },
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::UnknownDevice);

        let env = Envelope::request(
            Some("nope".into()),
            Payload::Describe(homecooked_protocol::DescribeRequest { points: vec![] }),
        );
        let resp = hub.handle(&env);
        match resp.payload {
            Payload::Error(body) => assert_eq!(body.code, ErrorCode::UnknownDevice),
            other => panic!("expected error, got {other:?}"),
        }
    }

    #[test]
    fn handle_describe_read_write() {
        let (mut hub, id) = hub_with_washer();
        let describe = Envelope::request(
            Some(id.as_str().into()),
            Payload::Describe(homecooked_protocol::DescribeRequest { points: vec![] }),
        );
        let resp = hub.handle(&describe);
        match resp.payload {
            Payload::DescribeOk(body) => {
                assert_eq!(body.capability.class_id, ApplianceClassId::Washer);
            }
            other => panic!("expected describe_ok, got {other:?}"),
        }

        let read = Envelope::request(
            Some(id.as_str().into()),
            Payload::Read(ReadRequest {
                points: vec![qid("class.washer.spin_rpm")],
                allow_partial: false,
            }),
        );
        let resp = hub.handle(&read);
        match resp.payload {
            Payload::ReadOk(body) => {
                assert_eq!(body.values[0].value, Some(Value::U16(800)));
            }
            other => panic!("expected read_ok, got {other:?}"),
        }

        let write = Envelope::request(
            Some(id.as_str().into()),
            Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: qid("class.washer.wash_temp_c"),
                    value: Value::F32(60.0),
                }],
                dry_run: false,
                atomic: false,
            }),
        );
        let resp = hub.handle(&write);
        assert!(matches!(resp.payload, Payload::WriteOk(_)));
        assert_eq!(
            hub.registry
                .get(&id)
                .unwrap()
                .state
                .get("class.washer.wash_temp_c"),
            Some(&Value::F32(60.0))
        );
    }

    #[test]
    fn version_mismatch_is_error() {
        let (mut hub, id) = hub_with_washer();
        let mut env = Envelope::request(
            Some(id.as_str().into()),
            Payload::Ping(homecooked_protocol::PingBody { echo: None }),
        );
        env.protocol_version = SemVer::new(1, 0, 0);
        let resp = hub.handle(&env);
        match resp.payload {
            Payload::Error(body) => assert_eq!(body.code, ErrorCode::VersionMismatch),
            other => panic!("expected version_mismatch, got {other:?}"),
        }
        assert_eq!(PROTOCOL_VERSION.major, 0);
    }

    #[test]
    fn dry_run_does_not_mutate() {
        let (mut hub, id) = hub_with_washer();
        hub.write_points(
            &id,
            &WriteRequest {
                writes: vec![WriteOp {
                    id: qid("class.washer.spin_rpm"),
                    value: Value::U16(400),
                }],
                dry_run: true,
                atomic: false,
            },
        )
        .unwrap();
        assert_eq!(
            hub.registry
                .get(&id)
                .unwrap()
                .state
                .get("class.washer.spin_rpm"),
            Some(&Value::U16(800))
        );
    }

    #[test]
    fn commands_are_not_stored() {
        let (mut hub, id) = hub_with_washer();
        hub.write_points(
            &id,
            &WriteRequest {
                writes: vec![WriteOp {
                    id: qid("trait.cycle.start"),
                    value: Value::Void,
                }],
                dry_run: false,
                atomic: false,
            },
        )
        .unwrap();
        assert!(hub
            .registry
            .get(&id)
            .unwrap()
            .state
            .get("trait.cycle.start")
            .is_none());
    }

    #[test]
    fn empty_read_is_invalid_request() {
        let (hub, id) = hub_with_washer();
        let err = hub
            .read_points(
                &id,
                &ReadRequest {
                    points: vec![],
                    allow_partial: false,
                },
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidRequest);
    }
}
