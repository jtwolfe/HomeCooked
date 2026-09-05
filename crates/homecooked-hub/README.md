# homecooked-hub

Optional **lab hub**: aggregate multiple HomeCooked sim devices behind one TCP
listener. Closes a thin Stream 4 follow-up — multi-device Discover on the same
lab port.

**Devices do not require a hub.** A single
[`homecooked-sim`](../homecooked-sim) +
[`homecooked-transport`](../homecooked-transport) peer is enough. This crate
only helps when you want several simulated appliances on one bind address.

**Out of scope:** cloud auth, TLS, hub UI.

## What it does

1. Holds a [`Simulator`](../homecooked-sim) / `DeviceHub` registry with multiple
   spawned devices.
2. Serves TCP via `homecooked-transport`: Discover lists all; Describe / Read /
   Write route by `device_id`.
3. Optional `spawn_lab_set()` — kettle + washer + fridge with stable ids
   (`lab-kettle`, `lab-washer`, `lab-fridge`).
4. Optional lab PSK via existing [`ServerConfig`](../homecooked-transport)
   (`ServerConfig::with_psk` / `HOMECOOKED_TCP_PSK`).

## Run the demo

```bash
cargo run -p homecooked-hub --example hub_demo
```

Binds `127.0.0.1:0` (OS-assigned port) unless `HOMECOOKED_HUB_BIND` is set.
Spawns the lab set, then runs discover → describe → read over TCP.

```bash
HOMECOOKED_TCP_PSK=lab-secret cargo run -p homecooked-hub --example hub_demo
```

## Tests

```bash
cargo test -p homecooked-hub
```

Integration tests bind `127.0.0.1:0`, discover ≥2 devices, describe/read one,
and cover optional PSK.

## Library sketch

```rust
use homecooked_hub::LabHub;
use homecooked_transport::TcpClient;

let mut hub = LabHub::new();
hub.spawn_lab_set().unwrap();
let spawned = hub.serve("127.0.0.1:0").unwrap();
let mut client = TcpClient::connect(spawned.addr()).unwrap();
let _ = client.discover(None, vec![]).unwrap();
```
