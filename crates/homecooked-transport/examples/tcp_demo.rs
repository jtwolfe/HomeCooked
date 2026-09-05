//! Tiny lab demo: spawn a kettle on TCP, run discover → describe → read → write.
//!
//! ```bash
//! cargo run -p homecooked-transport --example homecooked-tcp-demo
//! ```

use std::thread;
use std::time::Duration;

use homecooked_protocol::WriteOp;
use homecooked_schema::{ApplianceClassId, QualifiedPointId, Value};
use homecooked_sim::Simulator;
use homecooked_transport::{spawn_server, TcpClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sim = Simulator::new();
    let id = sim.spawn_named("demo-kettle", ApplianceClassId::Kettle)?;
    println!("spawned sim device {}", id.as_str());

    let (addr, _shared, _join) = spawn_server("127.0.0.1:0", sim)?;
    println!("listening on {addr} (length-prefixed JSON envelopes)");
    thread::sleep(Duration::from_millis(30));

    let mut client = TcpClient::connect(addr)?;

    let discovered = client.discover(Some(ApplianceClassId::Kettle), vec![])?;
    println!(
        "discover → {} device(s): {:?}",
        discovered.devices.len(),
        discovered
            .devices
            .iter()
            .map(|d| d.device_id.as_str())
            .collect::<Vec<_>>()
    );

    let desc = client.describe("demo-kettle", vec![])?;
    println!(
        "describe → class={} traits={}",
        desc.capability.class_id.as_str(),
        desc.capability.traits.len()
    );

    let setpoint = QualifiedPointId::parse("trait.temperature.setpoint_c")?;
    let current = QualifiedPointId::parse("trait.temperature.current_c")?;
    let read = client.read("demo-kettle", vec![setpoint.clone(), current])?;
    for v in &read.values {
        println!("read {} → {:?}", v.id, v.value);
    }

    let write = client.write(
        "demo-kettle",
        vec![WriteOp {
            id: setpoint.clone(),
            value: Value::F32(85.0),
        }],
    )?;
    println!("write accepted: {:?}", write.accepted[0].value);

    let again = client.read("demo-kettle", vec![setpoint])?;
    println!("read-back setpoint → {:?}", again.values[0].value);

    println!("ok");
    Ok(())
}
