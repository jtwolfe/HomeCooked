//! Integration: bind 127.0.0.1:0, describe + read + write against a sim kettle.

use std::thread;
use std::time::Duration;

use homecooked_protocol::{Payload, WriteOp};
use homecooked_schema::{ApplianceClassId, QualifiedPointId, Value};
use homecooked_sim::Simulator;
use homecooked_transport::{spawn_server, TcpClient};

fn qid(s: &str) -> QualifiedPointId {
    QualifiedPointId::parse(s).unwrap()
}

#[test]
fn tcp_describe_read_write_kettle() {
    let mut sim = Simulator::new();
    let kettle_id = sim
        .spawn_named("kettle-lab", ApplianceClassId::Kettle)
        .unwrap();
    assert_eq!(kettle_id.as_str(), "kettle-lab");

    let (addr, _shared, _server) = spawn_server("127.0.0.1:0", sim).unwrap();

    // Give the accept thread a moment to park on listen.
    thread::sleep(Duration::from_millis(20));

    let mut client = TcpClient::connect(addr).unwrap();

    // Discover
    let discovered = client
        .discover(Some(ApplianceClassId::Kettle), vec![])
        .unwrap();
    assert_eq!(discovered.devices.len(), 1);
    assert_eq!(discovered.devices[0].device_id, "kettle-lab");
    assert_eq!(discovered.devices[0].class_id, ApplianceClassId::Kettle);

    // Describe
    let desc = client.describe("kettle-lab", vec![]).unwrap();
    assert_eq!(desc.capability.class_id, ApplianceClassId::Kettle);
    assert!(desc
        .capability
        .traits
        .iter()
        .any(|t| t.trait_id.as_str() == "temperature"));

    // Read setpoint + current
    let read = client
        .read(
            "kettle-lab",
            vec![
                qid("trait.temperature.setpoint_c"),
                qid("trait.temperature.current_c"),
            ],
        )
        .unwrap();
    assert_eq!(read.values.len(), 2);
    assert_eq!(read.values[0].value, Some(Value::F32(100.0)));
    assert_eq!(read.values[1].value, Some(Value::F32(20.0)));

    // Write new setpoint
    let write = client
        .write(
            "kettle-lab",
            vec![WriteOp {
                id: qid("trait.temperature.setpoint_c"),
                value: Value::F32(80.0),
            }],
        )
        .unwrap();
    assert_eq!(write.accepted.len(), 1);
    assert_eq!(write.accepted[0].value, Value::F32(80.0));

    // Read back
    let read2 = client
        .read("kettle-lab", vec![qid("trait.temperature.setpoint_c")])
        .unwrap();
    assert_eq!(read2.values[0].value, Some(Value::F32(80.0)));
}

#[test]
fn tcp_washer_write_and_out_of_range() {
    let mut sim = Simulator::new();
    sim.spawn_named("washer-lab", ApplianceClassId::Washer)
        .unwrap();

    let (addr, _shared, _server) = spawn_server("127.0.0.1:0", sim).unwrap();
    thread::sleep(Duration::from_millis(20));

    let mut client = TcpClient::connect(addr).unwrap();

    client
        .write(
            "washer-lab",
            vec![WriteOp {
                id: qid("class.washer.spin_rpm"),
                value: Value::U16(1200),
            }],
        )
        .unwrap();

    let read = client
        .read("washer-lab", vec![qid("class.washer.spin_rpm")])
        .unwrap();
    assert_eq!(read.values[0].value, Some(Value::U16(1200)));

    let err = client
        .write(
            "washer-lab",
            vec![WriteOp {
                id: qid("class.washer.spin_rpm"),
                value: Value::U16(2000),
            }],
        )
        .unwrap_err();
    match err {
        homecooked_transport::TransportError::Remote(body) => {
            assert_eq!(body.code, homecooked_schema::ErrorCode::OutOfRange);
        }
        other => panic!("expected remote out_of_range, got {other}"),
    }
}

#[test]
fn tcp_raw_exchange_ping() {
    let sim = Simulator::new();
    let (addr, _shared, _server) = spawn_server("127.0.0.1:0", sim).unwrap();
    thread::sleep(Duration::from_millis(20));

    let mut client = TcpClient::connect(addr).unwrap();
    let req = homecooked_protocol::Envelope::new(Payload::Ping(homecooked_protocol::PingBody {
        echo: Some("tcp".into()),
    }));
    let resp = client.exchange(&req).unwrap();
    match resp.payload {
        Payload::Pong(p) => assert_eq!(p.echo.as_deref(), Some("tcp")),
        other => panic!("expected pong, got {other:?}"),
    }
}
