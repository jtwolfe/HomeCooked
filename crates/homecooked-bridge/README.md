# homecooked-bridge

Bridge slice: **Modbus** (mocked plant bus), **Matter** (mocked fabric),
**Zigbee** (mocked ZCL network), and **BACnet** (mocked device) adapters.
Aligns with [`docs/standard/bridges.md`](../../docs/standard/bridges.md) and
[`docs/ROADMAP.md`](../../docs/ROADMAP.md) Stream 6.

HomeCooked stays the appliance semantics layer. A bridge maps an external
fabric's objects into devices + qualified catalog points +
`homecooked-schema::Value`. This crate does **not** replace pairing UX,
mesh routing, a production Modbus / BACnet stack, or a CHIP / Matter SDK.

## Layout

| Module | Role |
|--------|------|
| `Bridge` | Small trait: `read_point` / `write_point` / `read_foreign` / `write_foreign` |
| `PointRef` | `device_id` + qualified point id (`trait.temperature.setpoint_c`) |
| `ForeignRef` / `ForeignRaw` | Fabric address + payload (Modbus register/coil, Matter/Zigbee endpoint/cluster/attribute, or BACnet object/property) |
| `PointBackend` / `MemoryBackend` | Apply HomeCooked updates (test store; core can be wired later) |
| `modbus` | YAML/JSON map, in-memory slave, `ModbusBridge`, localhost Modbus TCP lab |
| `matter` | YAML/JSON map, in-memory attribute store, `MatterBridge` |
| `zigbee` | YAML/JSON map, in-memory attribute store, `ZigbeeBridge` (no zigbee2mqtt) |
| `bacnet` | YAML/JSON map, in-memory property store, `BacnetBridge` (no BACnet stack) |

## Modbus (implemented)

An in-memory slave holds register and coil values. A serde-loadable map
translates them. There is **no** `tokio-modbus` crate and **no** serial RTU —
default builds stay dependency-light.

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

### Modbus TCP lab (CI / localhost)

`spawn_modbus_tcp_lab` exposes the same slave over **Modbus TCP** on
`127.0.0.1:0` using minimal MBAP framing (std only). Covered function codes
for the water_heater map:

| FC | Name | Map use |
|----|------|---------|
| 01 | Read Coils | power coil 0 |
| 03 | Read Holding Registers | setpoint + current temp |
| 05 | Write Single Coil | power on/off → HC backend |
| 06 | Write Single Register | setpoint → HC backend |

Hardware-free: loopback only. **Not** serial RTU, TLS, or a production stack.

```bash
cargo test -p homecooked-bridge --test water_heater_modbus_tcp
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

## Zigbee (mock network)

No zigbee2mqtt, ZCL SDK, or pairing. An in-memory attribute store holds
cluster attribute values. A serde-loadable map translates them (same shape as
the Matter mock — endpoint / cluster / attribute):

| Foreign (illustrative lab constants) | HomeCooked point | Encoding |
|--------------------------------------|------------------|----------|
| ep1 / OnOff `0x0006` / attr `0x0000` | `trait.power.power_state` | bool → `on`/`off` |
| ep1 / TemperatureMeasurement `0x0402` / MeasuredValue `0x0000` | `trait.temperature.current_c` | int16 hundredths °C (`scale: 0.01`); HC writes rejected |
| ep1 / thermostat-like `0x0201` / attr `0x0012` | `trait.temperature.setpoint_c` | int16 hundredths °C |

**These cluster IDs are illustrative lab constants, not a certified Zigbee
product.**

Example fixture: [`examples/kettle_zigbee_map.yaml`](examples/kettle_zigbee_map.yaml)
(fake `kettle`, device id `kettle-lab-1`).

```bash
cargo test -p homecooked-bridge --test kettle_zigbee_roundtrip
```

## BACnet (mock device)

No BACnet/IP, MS/TP, or ASHRAE stack. An in-memory property store holds
object present-values. A serde-loadable map translates them:

| Foreign (illustrative lab constants) | HomeCooked point | Encoding |
|--------------------------------------|------------------|----------|
| BinaryValue 1 / present_value | `trait.power.power_state` | bool → `on`/`off` |
| AnalogInput 1 / present_value | `trait.temperature.current_c` | int16 hundredths °C (`scale: 0.01`); HC writes rejected |
| AnalogValue 1 / present_value | `trait.temperature.setpoint_c` | int16 hundredths °C |

**These object types are illustrative lab constants, not a certified BACnet
product.**

Example fixture: [`examples/kettle_bacnet_map.yaml`](examples/kettle_bacnet_map.yaml)
(fake `kettle`, device id `kettle-lab-1`, device instance `1`).

```bash
cargo test -p homecooked-bridge --test kettle_bacnet_roundtrip
```

## Still follow-up

- Serial RTU Modbus (TCP lab path landed; RTU still deferred)
- Real CHIP / Matter SDK (or thin bindings) behind the same map shape
- Real zigbee2mqtt / ZCL bindings behind the same map shape
- Real BACnet/IP or MS/TP stack behind the same map shape
- Capability-enforced writes through `homecooked-core` `DeviceHub`
- Discovery / hello / describe from a live fabric
