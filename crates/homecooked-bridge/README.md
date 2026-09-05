# homecooked-bridge

Bridge slice: **Modbus** (mocked plant bus) and **Matter** (mocked fabric)
adapters, plus stubs for Zigbee and BACnet. Aligns with
[`docs/standard/bridges.md`](../../docs/standard/bridges.md) and
[`docs/ROADMAP.md`](../../docs/ROADMAP.md) Stream 6.

HomeCooked stays the appliance semantics layer. A bridge maps an external
fabric's objects into devices + qualified catalog points +
`homecooked-schema::Value`. This crate does **not** replace pairing UX,
mesh routing, a production Modbus stack, or a CHIP / Matter SDK.

## Layout

| Module | Role |
|--------|------|
| `Bridge` | Small trait: `read_point` / `write_point` / `read_foreign` / `write_foreign` |
| `PointRef` | `device_id` + qualified point id (`trait.temperature.setpoint_c`) |
| `ForeignRef` / `ForeignRaw` | Fabric address + payload (Modbus register/coil **or** Matter endpoint/cluster/attribute) |
| `PointBackend` / `MemoryBackend` | Apply HomeCooked updates (test store; core can be wired later) |
| `modbus` | YAML/JSON map, in-memory slave, `ModbusBridge` |
| `matter` | YAML/JSON map, in-memory attribute store, `MatterBridge` |
| `zigbee` / `bacnet` | Compile-time stubs → `Error::UnsupportedFabric` |

## Modbus (implemented)

No `tokio-modbus`, serial, or TCP dependency. An in-memory slave holds
register and coil values. A serde-loadable map translates them:

| Foreign | HomeCooked point | Encoding |
|---------|------------------|----------|
| Holding 0 | `trait.temperature.setpoint_c` | signed i16, tenths of °C (`scale: 0.1`) |
| Holding 1 | `trait.temperature.current_c` | same; HomeCooked writes rejected (`access: r`) |
| Coil 0 | `trait.power.power_state` | `true` → `on`, `false` → `off` |

Example fixture: [`examples/water_heater_map.yaml`](examples/water_heater_map.yaml)
(fake plant `water_heater`, device id `water-heater-plant`).

```bash
cargo test -p homecooked-bridge --test water_heater_roundtrip
```

## Matter (mock fabric)

No CHIP / Matter SDK, Thread, or commissioning. An in-memory attribute store
holds cluster attribute values. A serde-loadable map translates them:

| Foreign (illustrative lab constants) | HomeCooked point | Encoding |
|--------------------------------------|------------------|----------|
| ep1 / OnOff `0x0006` / attr `0x0000` | `trait.power.power_state` | bool → `on`/`off` |
| ep1 / TemperatureMeasurement `0x0402` / MeasuredValue `0x0000` | `trait.temperature.current_c` | int16 hundredths °C (`scale: 0.01`); HC writes rejected |
| ep1 / thermostat-like `0x0201` / attr `0x0012` | `trait.temperature.setpoint_c` | int16 hundredths °C |

**These cluster IDs are illustrative lab constants, not a certified Matter
product.** The fixture documents OnOff + TemperatureMeasurement-style shapes
mapped onto kettle points so CI stays free of external SDKs.

Example fixture: [`examples/kettle_matter_map.yaml`](examples/kettle_matter_map.yaml)
(fake `kettle`, device id `kettle-lab-1`).

```bash
cargo test -p homecooked-bridge --test kettle_matter_roundtrip
```

Foreign attribute write → HomeCooked backend update, and HomeCooked write →
attribute update, are both covered (same pattern as the Modbus water_heater
test).

## Stubs

`ZigbeeBridge` and `BacnetBridge` implement `Bridge` and return a clear
unsupported error pointing at
[`docs/standard/bridges.md`](../../docs/standard/bridges.md).

## Still follow-up

- Real serial / TCP Modbus
- Real CHIP / Matter SDK (or thin bindings) behind the same map shape
- Capability-enforced writes through `homecooked-core` `DeviceHub`
- Zigbee / BACnet adapters
- Discovery / hello / describe from a live fabric
