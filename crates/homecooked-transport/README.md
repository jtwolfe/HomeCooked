# homecooked-transport

Lab **TCP transport** for HomeCooked protocol envelopes. Closes Stream 4
milestone 3 in [`docs/ROADMAP.md`](../../docs/ROADMAP.md) (smoke path).

Host server accepts TCP connections, decodes a framed request, dispatches via
`homecooked-sim` / `homecooked-core` (`Simulator` → `DeviceHub`), and encodes
the response. A small client helper sends Discover / Describe / Read / Write.

**Out of scope:** TLS, OAuth / device auth, production session policy.

**Optional lab PSK:** a cleartext shared-secret handshake so anonymous TCP
clients can be refused in lab setups (not a substitute for TLS).

## Framing

**Length-prefixed JSON** (not NDJSON):

```text
[u32 big-endian length][UTF-8 JSON Envelope]
```

- Length covers the JSON bytes only (max **64 KiB**, overview §6.1).
- Payload is compact JSON from `Envelope::to_json`.
- Chosen for binary-safe, unambiguous boundaries (overview §6.1). See
  [`src/frame.rs`](src/frame.rs) for the full rationale.

## Lab PSK pairing

**Choice: dedicated framing preamble** (not new protocol `auth` / `auth_ok`
kinds). Overview §9.1 treats `unauthorized` as binding-level auth; keeping the
handshake in the TCP binding avoids catalog churn and leaves open-lab servers
byte-compatible with today.

When the server has a PSK (`ServerConfig::with_psk` or `HOMECOOKED_TCP_PSK`):

```text
Client → Server: {"hc_tcp":"auth","v":1,"psk":"<shared-secret>"}
Server → Client: {"hc_tcp":"auth_ok","v":1}
  or on failure: {"hc_tcp":"auth_err","v":1,"code":"unauthorized","message":"..."}
                 then the server closes.
```

Same length-prefixed framing as envelopes. If **no** PSK is configured, there
is **no** preamble — first frame is an Envelope (open lab).

```rust
use homecooked_transport::{spawn_server_with_config, ServerConfig, TcpClient};

let (addr, _, _) = spawn_server_with_config(
    "127.0.0.1:0",
    sim,
    ServerConfig::with_psk("lab-secret"),
).unwrap();
let mut client = TcpClient::connect_with_psk(addr, Some("lab-secret")).unwrap();
```

Env helpers: `ServerConfig::from_env()`, `TcpClient::connect_from_env(addr)`
read `HOMECOOKED_TCP_PSK` when set.

## Run the demo

```bash
cargo run -p homecooked-transport --example homecooked-tcp-demo
```

Spawns a sim kettle, binds `127.0.0.1:0`, then runs discover → describe →
read → write → read-back over TCP (open lab, no PSK).

## Tests

```bash
cargo test -p homecooked-transport
```

Integration tests bind `127.0.0.1:0` and round-trip describe / read / write
against kettle and washer sims (including an out-of-range write denial), plus
PSK good / bad / missing / open-lab cases.

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
