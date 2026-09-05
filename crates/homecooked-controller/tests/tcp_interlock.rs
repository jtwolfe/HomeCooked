//! Lab smoke: TCP client → ControllerEndpoint → MockHal interlocks.

use std::thread;
use std::time::Duration;

use homecooked_controller::{ControllerEndpoint, WASHER_CTRL_DEVICE_ID};
use homecooked_protocol::WriteOp;
use homecooked_schema::{ErrorCode, QualifiedPointId, Value};
use homecooked_transport::{spawn_handler_server, TcpClient, TransportError};

fn qid(s: &str) -> QualifiedPointId {
    QualifiedPointId::parse(s).unwrap()
}

#[test]
fn tcp_washer_heater_interlock_allow_and_deny() {
    let ep = ControllerEndpoint::washer_lab().unwrap();
    let (addr, _shared, _server) = spawn_handler_server("127.0.0.1:0", ep).unwrap();
    thread::sleep(Duration::from_millis(20));

    let mut client = TcpClient::connect(addr).unwrap();

    let desc = client.describe(WASHER_CTRL_DEVICE_ID, vec![]).unwrap();
    assert_eq!(
        desc.capability.class_id,
        homecooked_schema::ApplianceClassId::Washer
    );
    assert!(desc
        .capability
        .class_points
        .iter()
        .any(|p| p.id == "class.washer.heater_enable"));

    // Setup: door locked, water present → heater allow.
    client
        .write(
            WASHER_CTRL_DEVICE_ID,
            vec![WriteOp {
                id: qid("class.washer.door_lock"),
                value: Value::Bool(true),
            }],
        )
        .unwrap();
    client
        .write(
            WASHER_CTRL_DEVICE_ID,
            vec![WriteOp {
                id: qid("class.washer.water_level_pa"),
                value: Value::F32(2_000.0),
            }],
        )
        .unwrap();
    let ok = client
        .write(
            WASHER_CTRL_DEVICE_ID,
            vec![WriteOp {
                id: qid("class.washer.heater_enable"),
                value: Value::Bool(true),
            }],
        )
        .unwrap();
    assert_eq!(ok.accepted.len(), 1);

    let read = client
        .read(
            WASHER_CTRL_DEVICE_ID,
            vec![qid("class.washer.heater_enable")],
        )
        .unwrap();
    assert_eq!(read.values[0].value, Some(Value::Bool(true)));

    // Drain water → heater on denied by interlock.
    client
        .write(
            WASHER_CTRL_DEVICE_ID,
            vec![WriteOp {
                id: qid("class.washer.heater_enable"),
                value: Value::Bool(false),
            }],
        )
        .unwrap();
    client
        .write(
            WASHER_CTRL_DEVICE_ID,
            vec![WriteOp {
                id: qid("class.washer.water_level_pa"),
                value: Value::F32(0.0),
            }],
        )
        .unwrap();
    let err = client
        .write(
            WASHER_CTRL_DEVICE_ID,
            vec![WriteOp {
                id: qid("class.washer.heater_enable"),
                value: Value::Bool(true),
            }],
        )
        .unwrap_err();
    match err {
        TransportError::Remote(body) => {
            assert_eq!(body.code, ErrorCode::SafetyInterlock);
            assert!(
                body.message.contains("interlock") || body.message.contains("water"),
                "{}",
                body.message
            );
        }
        other => panic!("expected remote safety_interlock, got {other}"),
    }
}

#[test]
fn tcp_dryer_heater_interlock_allow_and_deny() {
    use homecooked_controller::{DryerControllerEndpoint, DRYER_CTRL_DEVICE_ID};

    let ep = DryerControllerEndpoint::dryer_lab().unwrap();
    let (addr, _shared, _server) = spawn_handler_server("127.0.0.1:0", ep).unwrap();
    thread::sleep(Duration::from_millis(20));

    let mut client = TcpClient::connect(addr).unwrap();

    let desc = client.describe(DRYER_CTRL_DEVICE_ID, vec![]).unwrap();
    assert_eq!(
        desc.capability.class_id,
        homecooked_schema::ApplianceClassId::Dryer
    );
    assert!(desc
        .capability
        .class_points
        .iter()
        .any(|p| p.id == "class.dryer.heater_enable"));

    // Setup: door locked + blower on → heater allow.
    client
        .write(
            DRYER_CTRL_DEVICE_ID,
            vec![WriteOp {
                id: qid("class.dryer.door_lock"),
                value: Value::Bool(true),
            }],
        )
        .unwrap();
    client
        .write(
            DRYER_CTRL_DEVICE_ID,
            vec![WriteOp {
                id: qid("class.dryer.blower"),
                value: Value::Bool(true),
            }],
        )
        .unwrap();
    let ok = client
        .write(
            DRYER_CTRL_DEVICE_ID,
            vec![WriteOp {
                id: qid("class.dryer.heater_enable"),
                value: Value::Bool(true),
            }],
        )
        .unwrap();
    assert_eq!(ok.accepted.len(), 1);

    let read = client
        .read(DRYER_CTRL_DEVICE_ID, vec![qid("class.dryer.heater_enable")])
        .unwrap();
    assert_eq!(read.values[0].value, Some(Value::Bool(true)));

    // Unlock door → heater on denied by interlock.
    client
        .write(
            DRYER_CTRL_DEVICE_ID,
            vec![WriteOp {
                id: qid("class.dryer.heater_enable"),
                value: Value::Bool(false),
            }],
        )
        .unwrap();
    client
        .write(
            DRYER_CTRL_DEVICE_ID,
            vec![WriteOp {
                id: qid("class.dryer.door_lock"),
                value: Value::Bool(false),
            }],
        )
        .unwrap();
    let err = client
        .write(
            DRYER_CTRL_DEVICE_ID,
            vec![WriteOp {
                id: qid("class.dryer.heater_enable"),
                value: Value::Bool(true),
            }],
        )
        .unwrap_err();
    match err {
        TransportError::Remote(body) => {
            assert_eq!(body.code, ErrorCode::SafetyInterlock);
            assert!(
                body.message.contains("interlock") || body.message.contains("door"),
                "{}",
                body.message
            );
        }
        other => panic!("expected remote safety_interlock, got {other}"),
    }
}
