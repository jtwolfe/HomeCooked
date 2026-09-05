//! Lab PSK pairing: good secret works; bad/missing fails; open lab unchanged.

use std::thread;
use std::time::Duration;

use homecooked_protocol::{Envelope, Payload, PingBody};
use homecooked_schema::ApplianceClassId;
use homecooked_sim::Simulator;
use homecooked_transport::{
    spawn_server, spawn_server_with_config, ServerConfig, TcpClient, TransportError,
};

fn sleep_accept() {
    thread::sleep(Duration::from_millis(20));
}

#[test]
fn psk_good_client_works() {
    let mut sim = Simulator::new();
    sim.spawn_named("kettle-psk", ApplianceClassId::Kettle)
        .unwrap();

    let (addr, _shared, _server) =
        spawn_server_with_config("127.0.0.1:0", sim, ServerConfig::with_psk("lab-secret")).unwrap();
    sleep_accept();

    let mut client = TcpClient::connect_with_psk(addr, Some("lab-secret")).unwrap();
    let desc = client.describe("kettle-psk", vec![]).unwrap();
    assert_eq!(desc.capability.class_id, ApplianceClassId::Kettle);

    let req = Envelope::new(Payload::Ping(PingBody {
        echo: Some("psk".into()),
    }));
    let resp = client.exchange(&req).unwrap();
    match resp.payload {
        Payload::Pong(p) => assert_eq!(p.echo.as_deref(), Some("psk")),
        other => panic!("expected pong, got {other:?}"),
    }
}

#[test]
fn psk_bad_secret_fails() {
    let sim = Simulator::new();
    let (addr, _shared, _server) =
        spawn_server_with_config("127.0.0.1:0", sim, ServerConfig::with_psk("lab-secret")).unwrap();
    sleep_accept();

    let err = TcpClient::connect_with_psk(addr, Some("wrong-secret")).unwrap_err();
    match err {
        TransportError::Auth(msg) => {
            assert!(
                msg.contains("unauthorized") || msg.contains("PSK") || msg.contains("mismatch"),
                "unexpected auth message: {msg}"
            );
        }
        other => panic!("expected Auth error, got {other}"),
    }
}

#[test]
fn psk_missing_client_fails() {
    let sim = Simulator::new();
    let (addr, _shared, _server) =
        spawn_server_with_config("127.0.0.1:0", sim, ServerConfig::with_psk("lab-secret")).unwrap();
    sleep_accept();

    // Open client sends an envelope as the first frame → server rejects preamble.
    let err = TcpClient::connect(addr)
        .and_then(|mut c| {
            c.discover(None, vec![])?;
            Ok(())
        })
        .unwrap_err();
    // Either Auth on a later path, or I/O / UnexpectedEof / Protocol after server closes.
    match err {
        TransportError::Auth(_)
        | TransportError::UnexpectedEof
        | TransportError::Io(_)
        | TransportError::Protocol(_)
        | TransportError::Remote(_) => {}
        other => panic!("expected auth/io failure for missing PSK, got {other}"),
    }
}

#[test]
fn without_psk_open_lab_still_works() {
    let mut sim = Simulator::new();
    sim.spawn_named("kettle-open", ApplianceClassId::Kettle)
        .unwrap();

    let (addr, _shared, _server) = spawn_server("127.0.0.1:0", sim).unwrap();
    sleep_accept();

    let mut client = TcpClient::connect(addr).unwrap();
    let desc = client.describe("kettle-open", vec![]).unwrap();
    assert_eq!(desc.capability.class_id, ApplianceClassId::Kettle);
}
