# homecooked-bridge

First **bridge** slice: one real adapter (**Modbus**, mocked transport) plus
stubs for Zigbee, Matter, and BACnet. Aligns with
[`docs/standard/bridges.md`](../../docs/standard/bridges.md) and
[`docs/ROADMAP.md`](../../docs/ROADMAP.md) Stream 6.

HomeCooked stays the appliance semantics layer. A bridge maps an external
fabric's objects into devices + qualified catalog points +
`homecooked-schema::Value`. This crate does **not** replace pairing UX,
mesh routing, or a production Modbus stack.

## Layout

| Module | Role |
|--------|------|
| `Bridge` | Small trait: `read_point` / `write_point` / `read_foreign` / `write_foreign` |
| `PointRef` | `device_id` + qualified point id (`trait.temperature.setpoint_c`) |
| `PointBackend` / `MemoryBackend` | Apply HomeCooked updates (test store; core can be wired later) |
| `modbus` | YAML/JSON map, in-memory slave, `ModbusBridge` |
| `zigbee` / `matter` / `bacnet` | Compile-time stubs → `Error::UnsupportedFabric` |

## Modbus (implemented)

No `tokio-modbus`, serial, or TCP dependency. An in-memory slave holds
register and coil values. A serde-loadable map translates them:

| Foreign | HomeCooked point | Encoding |
|---------|------------------|----------|
| Holding 0 | `trait.temperature.setpoint_c` | signed i16, tenths of °C (`scale: 0.1`) |
| Holding 1 | `trait.temperature.current_c` | same; HomeCooked writes rejected (`access: r`) |
| Coil 0 | `trait.power.power_state` | `true` → `on`, `false` → `off` |

The catalog has no bool `trait.power.on`. Power is the `off|standby|on|fault`
enum on `trait.power.power_state` (see
`crates/homecooked-schema/src/catalog/traits.rs`). This slice maps the
on/off coil subset only.

Example fixture: [`examples/water_heater_map.yaml`](examples/water_heater_map.yaml)
(fake plant `water_heater`, device id `water-heater-plant`).

```bash
cargo test -p homecooked-bridge
cargo test -p homecooked-bridge --test water_heater_roundtrip
```

Foreign register write → HomeCooked backend update, and HomeCooked write →
register update, are both covered.

## Stubs

`ZigbeeBridge`, `MatterBridge`, and `BacnetBridge` implement `Bridge` and
return a clear unsupported error pointing at
[`docs/standard/bridges.md`](../../docs/standard/bridges.md). They exist so
the module tree shows the adapter pattern. Matter was deferred so CI stays
free of external SDKs.

## Still follow-up

- Real serial / TCP Modbus
- Capability-enforced writes through `homecooked-core` `DeviceHub`
- Zigbee / Matter / BACnet adapters
- Discovery / hello / describe from a live fabric
