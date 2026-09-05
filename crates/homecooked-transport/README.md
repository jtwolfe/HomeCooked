# homecooked-transport

Lab **TCP transport** for HomeCooked protocol envelopes. Closes Stream 4
milestone 3 in [`docs/ROADMAP.md`](../../docs/ROADMAP.md) (smoke path).

Host server accepts TCP connections, decodes a framed request, dispatches via
`homecooked-sim` / `homecooked-core` (`Simulator` → `DeviceHub`), and encodes
the response. A small client helper sends Discover / Describe / Read / Write.

**Out of scope:** TLS, OAuth / device auth, production session policy.

## Framing

**Length-prefixed JSON** (not NDJSON):

```text
[u32 big-endian length][UTF-8 JSON Envelope]
```

- Length covers the JSON bytes only (max **64 KiB**, overview §6.1).
- Payload is compact JSON from `Envelope::to_json`.
- Chosen for binary-safe, unambiguous boundaries (overview §6.1). See
  [`src/frame.rs`](src/frame.rs) for the full rationale.

## Run the demo

```bash
cargo run -p homecooked-transport --example homecooked-tcp-demo
```

Spawns a sim kettle, binds `127.0.0.1:0`, then runs discover → describe →
read → write → read-back over TCP.

## Tests

```bash
cargo test -p homecooked-transport
```

Integration tests bind `127.0.0.1:0` and round-trip describe / read / write
against kettle and washer sims (including an out-of-range write denial).

## Library sketch

```rust
use homecooked_schema::{ApplianceClassId, Value};
use homecooked_sim::Simulator;
use homecooked_transport::{spawn_server, TcpClient};

let mut sim = Simulator::new();
sim.spawn_named("kettle-1", ApplianceClassId::Kettle).unwrap();
let (addr, _, _) = spawn_server("127.0.0.1:0", sim).unwrap();
let mut client = TcpClient::connect(addr).unwrap();
let _ = client.describe("kettle-1", vec![]).unwrap();
```
