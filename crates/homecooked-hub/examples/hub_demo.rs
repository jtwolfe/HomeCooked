//! Lab hub demo: spawn kettle + washer + fridge, serve TCP, run a client smoke.
//!
//! ```bash
//! cargo run -p homecooked-hub --example hub_demo
//! ```
//!
//! Binds `127.0.0.1:0` (OS-assigned port). Set `HOMECOOKED_TCP_PSK` to enable
//! optional lab PSK on both server and demo client.

use std::env;
use std::thread;
use std::time::Duration;

use homecooked_hub::{LabHub, DEFAULT_BIND, LAB_KETTLE_ID};
use homecooked_schema::{ApplianceClassId, QualifiedPointId};
use homecooked_transport::{psk_from_env, ServerConfig, TcpClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut hub = LabHub::new();
    let set = hub.spawn_lab_set()?;
    println!(
        "lab set: {} / {} / {}",
        set.kettle.as_str(),
        set.washer.as_str(),
        set.fridge.as_str()
    );

    let config = ServerConfig::from_env();
    let psk = psk_from_env();
    if psk.is_some() {
        println!("PSK enabled via HOMECOOKED_TCP_PSK");
    } else {
        println!("open lab (no PSK); set HOMECOOKED_TCP_PSK to require pairing");
    }

    let bind = env::var("HOMECOOKED_HUB_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_string());
    let spawned = hub.serve_with_config(bind.as_str(), config)?;
    println!(
        "hub listening on {} (length-prefixed JSON; Discover lists all)",
        spawned.addr()
    );
    thread::sleep(Duration::from_millis(30));

    let mut client = TcpClient::connect_with_psk(spawned.addr(), psk.as_deref())?;

    let discovered = client.discover(None, vec![])?;
    println!(
        "discover → {} device(s): {:?}",
        discovered.devices.len(),
        discovered
            .devices
            .iter()
            .map(|d| d.device_id.as_str())
            .collect::<Vec<_>>()
    );

    let desc = client.describe(LAB_KETTLE_ID, vec![])?;
    println!(
        "describe {} → class={} traits={}",
        LAB_KETTLE_ID,
        desc.capability.class_id.as_str(),
        desc.capability.traits.len()
    );
    assert_eq!(desc.capability.class_id, ApplianceClassId::Kettle);

    let current = QualifiedPointId::parse("trait.temperature.current_c")?;
    let read = client.read(LAB_KETTLE_ID, vec![current])?;
    println!(
        "read {} trait.temperature.current_c → {:?}",
        LAB_KETTLE_ID, read.values[0].value
    );

    println!("ok");
    Ok(())
}
