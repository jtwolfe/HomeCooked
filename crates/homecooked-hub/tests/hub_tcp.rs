//! Integration: hub with ≥2 devices over TCP — discover, describe, read.

use std::thread;
use std::time::Duration;

use homecooked_hub::{LabHub, LAB_KETTLE_ID, LAB_WASHER_ID};
use homecooked_schema::{ApplianceClassId, QualifiedPointId, Value};
use homecooked_transport::{ServerConfig, TcpClient};

fn qid(s: &str) -> QualifiedPointId {
    QualifiedPointId::parse(s).unwrap()
}

#[test]
fn hub_discover_describe_read_multiple_devices() {
    let mut hub = LabHub::new();
    let set = hub.spawn_lab_set().unwrap();
    assert_eq!(hub.list().len(), 3);

    let spawned = hub.serve("127.0.0.1:0").unwrap();
    thread::sleep(Duration::from_millis(20));

    let mut client = TcpClient::connect(spawned.addr()).unwrap();

    let discovered = client.discover(None, vec![]).unwrap();
    assert!(
        discovered.devices.len() >= 2,
        "expected ≥2 devices, got {}",
        discovered.devices.len()
    );
    let ids: Vec<&str> = discovered
        .devices
        .iter()
        .map(|d| d.device_id.as_str())
        .collect();
    assert!(ids.contains(&set.kettle.as_str()));
    assert!(ids.contains(&set.washer.as_str()));
    assert!(ids.contains(&set.fridge.as_str()));

    let desc = client.describe(LAB_KETTLE_ID, vec![]).unwrap();
    assert_eq!(desc.capability.class_id, ApplianceClassId::Kettle);

    let read = client
        .read(
            LAB_KETTLE_ID,
            vec![
                qid("trait.temperature.setpoint_c"),
                qid("trait.temperature.current_c"),
            ],
        )
        .unwrap();
    assert_eq!(read.values.len(), 2);
    assert_eq!(read.values[0].value, Some(Value::F32(100.0)));
    assert_eq!(read.values[1].value, Some(Value::F32(20.0)));

    // Route to a second device by id.
    let washer = client
        .read(LAB_WASHER_ID, vec![qid("class.washer.spin_rpm")])
        .unwrap();
    assert_eq!(washer.values[0].value, Some(Value::U16(800)));
}

#[test]
fn hub_optional_psk_via_server_config() {
    let mut hub = LabHub::new();
    hub.spawn_lab_set().unwrap();

    let spawned = hub
        .serve_with_config("127.0.0.1:0", ServerConfig::with_psk("hub-lab-secret"))
        .unwrap();
    thread::sleep(Duration::from_millis(20));

    let mut client = TcpClient::connect_with_psk(spawned.addr(), Some("hub-lab-secret")).unwrap();
    let discovered = client.discover(None, vec![]).unwrap();
    assert!(discovered.devices.len() >= 2);

    let desc = client.describe(LAB_KETTLE_ID, vec![]).unwrap();
    assert_eq!(desc.capability.class_id, ApplianceClassId::Kettle);
}
