//! HomeCooked wire protocol: framing, request/response, discovery, errors.
//!
//! Encoding is JSON. Field names are `snake_case`. Protocol version **0.1.0**
//! matches `docs/standard/overview.md`. Clients reject a peer only on
//! protocol **major** mismatch.

mod envelope;
mod error;
mod messages;

pub use envelope::{check_protocol_version, next_message_id, now_ms, Envelope};
pub use error::{is_retryable, ErrorBody, ProtocolError};
pub use messages::{
    CapsChangedBody, DescribeRequest, DescribeResponse, DiscoverRequest, DiscoverResponse,
    EventBody, EventReason, FaultSnapshot, HelloRecord, MessageKind, Payload, PingBody, PointValue,
    PongBody, ReadRequest, ReadResponse, SubscribeRequest, SubscribeResponse, UnsubscribeRequest,
    UnsubscribeResponse, WriteOp, WriteRequest, WriteResponse, MAX_READ_POINTS,
};

use homecooked_schema::SemVer;

/// Wire protocol version advertised in every envelope.
pub const PROTOCOL_VERSION: SemVer = SemVer::V0_1_0;

#[cfg(test)]
mod tests {
    use homecooked_schema::{
        typical_capability, ApplianceClassId, ErrorCode, QualifiedPointId, TraitId, Value,
    };

    use super::*;

    fn sample_read() -> Envelope {
        Envelope::request(
            Some("kettle-1".into()),
            Payload::Read(ReadRequest {
                points: vec![
                    QualifiedPointId::parse("trait.temperature.current_c").unwrap(),
                    QualifiedPointId::parse("trait.temperature.setpoint_c").unwrap(),
                ],
                allow_partial: false,
            }),
        )
        .with_correlation_id("corr-1")
        .with_ts_ms(1_700_000_000_000)
    }

    #[test]
    fn protocol_version_is_0_1_0() {
        assert_eq!(PROTOCOL_VERSION.to_string(), "0.1.0");
    }

    #[test]
    fn read_roundtrip() {
        let env = sample_read();
        let json = env.to_json().unwrap();
        assert!(json.contains("\"protocol_version\":\"0.1.0\""));
        assert!(json.contains("\"message_id\""));
        assert!(json.contains("\"correlation_id\":\"corr-1\""));
        assert!(json.contains("\"kind\":\"read\""));
        let back = Envelope::from_json(&json).unwrap();
        assert_eq!(back, env);
        assert_eq!(back.kind(), MessageKind::Read);
        back.check_protocol_version().unwrap();
    }

    #[test]
    fn proto_and_id_aliases_deserialize() {
        let json = r#"{
            "proto": "0.1.0",
            "id": "abc",
            "device_id": "dev-1",
            "kind": "ping",
            "body": { "echo": "hi" }
        }"#;
        let env = Envelope::from_json(json).unwrap();
        assert_eq!(env.protocol_version, PROTOCOL_VERSION);
        assert_eq!(env.message_id, "abc");
        assert_eq!(
            env.payload,
            Payload::Ping(PingBody {
                echo: Some("hi".into())
            })
        );
    }

    #[test]
    fn discover_describe_write_event_error_roundtrips() {
        let cap = typical_capability(ApplianceClassId::Kettle).unwrap();
        let cases = [
            Envelope::new(Payload::Discover(DiscoverRequest {
                class_id: Some(ApplianceClassId::Kettle),
                trait_ids: vec![TraitId::Temperature],
            })),
            Envelope::new(Payload::DiscoverOk(DiscoverResponse {
                devices: vec![HelloRecord::new("k1", ApplianceClassId::Kettle)],
            })),
            Envelope::new(Payload::Describe(DescribeRequest { points: vec![] })),
            Envelope::new(Payload::DescribeOk(Box::new(DescribeResponse {
                capability: cap.clone(),
            }))),
            Envelope::new(Payload::Write(WriteRequest {
                writes: vec![WriteOp {
                    id: QualifiedPointId::parse("trait.temperature.setpoint_c").unwrap(),
                    value: Value::F32(80.0),
                }],
                dry_run: false,
                atomic: false,
            })),
            Envelope::new(Payload::WriteOk(WriteResponse {
                accepted: vec![WriteOp {
                    id: QualifiedPointId::parse("trait.temperature.setpoint_c").unwrap(),
                    value: Value::F32(80.0),
                }],
            })),
            Envelope::new(Payload::Subscribe(SubscribeRequest {
                points: vec![QualifiedPointId::parse("trait.cycle.cycle_state").unwrap()],
                traits: vec![TraitId::Fault],
                all: false,
                min_period_ms: Some(1000),
                events: vec![EventReason::Value, EventReason::Cycle],
            })),
            Envelope::new(Payload::Event(EventBody {
                reason: EventReason::Cycle,
                values: vec![PointValue::new(
                    QualifiedPointId::parse("trait.cycle.cycle_state").unwrap(),
                    Value::Enum("running".into()),
                    1,
                )],
                cycle_state: Some("running".into()),
                fault: None,
            })),
            Envelope::new(Payload::Error(ErrorBody::new(
                ErrorCode::OutOfRange,
                "too hot",
            ))),
        ];

        for env in cases {
            let json = serde_json::to_string(&env).unwrap();
            let back: Envelope = serde_json::from_str(&json).unwrap();
            assert_eq!(back.payload.kind(), env.payload.kind());
            assert_eq!(back, env);
        }
    }

    #[test]
    fn version_mismatch_major() {
        let mut env = sample_read();
        env.protocol_version = SemVer::new(1, 0, 0);
        let err = env.check_protocol_version().unwrap_err();
        match err {
            ProtocolError::VersionMismatch {
                got,
                expected_major,
            } => {
                assert_eq!(got, SemVer::new(1, 0, 0));
                assert_eq!(expected_major, 0);
            }
            other => panic!("unexpected {other:?}"),
        }
        let json = env.to_json().unwrap();
        let decoded = Envelope::from_json(&json).unwrap();
        assert!(decoded.check_protocol_version().is_err());
        assert!(Envelope::decode(&json).is_err());
        assert_eq!(err.to_error_body().code, ErrorCode::VersionMismatch);
    }

    #[test]
    fn same_major_is_compatible() {
        let mut env = sample_read();
        env.protocol_version = SemVer::new(0, 9, 0);
        env.check_protocol_version().unwrap();
        assert!(check_protocol_version(SemVer::new(0, 1, 0)).is_ok());
        assert!(check_protocol_version(SemVer::new(2, 0, 0)).is_err());
    }

    #[test]
    fn error_codes_match_schema_tokens() {
        let body = ErrorBody::new(ErrorCode::UnknownDevice, "missing");
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"unknown_device\""));
        assert!(!body.retryable);
        assert!(is_retryable(ErrorCode::Busy));
        assert!(is_retryable(ErrorCode::Timeout));
        assert!(!is_retryable(ErrorCode::OutOfRange));
    }

    #[test]
    fn respond_to_echoes_correlation() {
        let req = sample_read();
        let resp = Envelope::respond_to(
            &req,
            Payload::ReadOk(ReadResponse {
                values: vec![PointValue::new(
                    QualifiedPointId::parse("trait.temperature.current_c").unwrap(),
                    Value::F32(20.0),
                    1,
                )],
                errors: vec![],
            }),
        );
        assert_eq!(resp.correlation_id.as_deref(), Some("corr-1"));
        assert_eq!(resp.device_id.as_deref(), Some("kettle-1"));
        assert_eq!(resp.kind(), MessageKind::ReadOk);
    }
}
