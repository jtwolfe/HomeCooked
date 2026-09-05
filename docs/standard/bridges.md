# Bridges — integration with existing home automation

Version **0.1.3** — design extension plus first crate slice.

HomeCooked is an **appliance semantics layer** for heavy whitegoods and
kitchen / utility plant. It is **not** a replacement for Zigbee, Matter,
Thread, or whole-home mesh fabrics. Most households already have lights,
sensors, and locks on those fabrics; HomeCooked should **bridge** to them
rather than compete for every endpoint.

Related:

- [`overview.md`](./overview.md) — transport-agnostic protocol
- [`../catalog/variables-and-settings.md`](../catalog/variables-and-settings.md)
  — `trait.connectivity.transport` enum
- [`thermal-plant.md`](./thermal-plant.md), [`procedures.md`](./procedures.md)

---

## 1. Positioning

| Layer | Responsibility |
|-------|----------------|
| Zigbee / Matter / Thread / Wi-Fi mesh | Device pairing, mesh routing, low-power sensors, lamps, simple actuators |
| Modbus / BACnet | Building / plant buses, meters, AHUs, some commercial kitchen gear |
| Proprietary Wi-Fi (vendor apps) | Today's shipping whitegoods cloud or LAN APIs |
| **HomeCooked** | Rich appliance **classes, traits, cycles, zones, water, interlocks** |

Clients that already speak Matter can keep doing so for bulbs. A washer,
oven, or multi-zone fridge benefits from HomeCooked's catalog model even when
the physical radio is Wi-Fi, Thread, or a vendor TLS socket.

---

## 2. Bridge pattern

A **bridge** (adapter) maps an external fabric's objects into HomeCooked
**devices + capabilities + points**.

```
 External fabric          Bridge                     HomeCooked peers
 ┌─────────────┐     ┌──────────────────┐     ┌─────────────────────┐
 │ Zigbee      │     │ discover native  │     │ hello / describe    │
 │ Matter      │ ──► │ map clusters →   │ ──► │ read / write /      │
 │ Thread      │     │ points           │     │ subscribe / events  │
 │ Modbus      │     │ enforce caps     │     │                     │
 │ BACnet      │     │                 │     │                     │
 │ Vendor Wi-Fi│     │                 │     │                     │
 └─────────────┘     └──────────────────┘     └─────────────────────┘
```

One bridge process may present many HomeCooked devices. Each native endpoint
SHOULD map to one `device_id` with a primary `class_id` when the semantics
fit; otherwise use a narrow class plus `vendor.*` points.

---

## 3. Transports already enumerated

`trait.connectivity.transport` in the variables catalog already lists:

`ip` · `ble` · `thread` · `zigbee` · `matter` · `uart` · `unknown`

Bindings document how sessions ride on each. Bridges add **semantic** mapping
on top of whatever transport the peer uses. Additional plant buses (Modbus,
BACnet) are expected to appear as transport tokens or vendor connectivity
metadata in a later catalog pass; until then, bridges may report `ip` or
`uart` and carry bus details under `vendor.*`.

---

## 4. Adapter responsibilities

A conforming bridge **SHOULD**:

1. **Discover** native devices and emit HomeCooked hello / describe.
2. **Map** clusters, objects, or vendor properties to catalog points when a
   clear match exists (`OnOff` → a boolean power point only if that matches
   the appliance model; washers are not light bulbs).
3. **Advertise honest capabilities** — omit points the native device cannot
   support; do not invent ranges.
4. **Validate writes** with the same capability rules as a native device
   before forwarding.
5. **Translate events** (cycle complete, door open, fault) into HomeCooked
   subscriptions where possible.
6. **Surface safety flags** (`remote_start_supported`, etc.) conservatively
   (default false when unknown).

A bridge **MUST NOT** claim a catalog class if behavior is only a trivial
subset that would mislead clients (e.g. advertising `washer` when only power
and a single opaque "start" exist with no cycle state). Prefer a smaller
trait set or `vendor.*` until mapping quality is adequate.

---

## 5. Explicitly out of scope for HomeCooked bridges

HomeCooked does **not** specify:

- Mesh routing, parent selection, or Thread border-router internals
- Pairing UX, QR commissioning flows, or Install Codes
- Zigbee binding tables or Matter ACL administration UIs
- Replacement of BACnet object browsers or Modbus register maps as such

Those remain concerns of the native fabric tools. The bridge consumes an
already-commissioned device or an upstream SDK.

---

## 6. Why whitegoods need a richer model than typical clusters

Common Zigbee / Matter clusters excel at:

- On / off, level, color temperature
- Simple thermostat setpoints
- Binary sensors and door contacts

Heavy whitegoods routinely need:

| Concern | HomeCooked angle |
|---------|------------------|
| Cycles & phases | `trait.cycle` state machine, phase enums, remaining time |
| Programs | Catalog / SKU program tokens with per-program parameter envelopes |
| Water | Fill levels, softener, leak, inlet temperature, drain |
| Multi-zone | `#fridge` / `#freezer` / hob zones with independent setpoints |
| Safety interlocks | Door, child lock, remote-start gates, gas / RF / vent flags |
| Composition | `washer_dryer`, `fridge_freezer`, `range` as combo patterns |

Flattening a dishwasher into `OnOff` + one mode byte loses the semantics
automations and procedures ([`procedures.md`](./procedures.md)) rely on.
Bridges should preserve cycle richness even if the upstream API is awkward.

---

## 7. Plant buses (Modbus / BACnet)

HVAC plant, boilers, and meters often speak Modbus or BACnet. HomeCooked
[`hvac`](../catalog/appliances.md), `boiler`, and `water_heater` classes exist
so those assets can appear beside kitchen devices in one client. Mapping tips:

- Expose stable points (setpoints, enable, fault) first.
- Do not pretend BACnet is HomeCooked; the bridge is the boundary
  ([`appliances.md`](../catalog/appliances.md) already notes HVAC is not a full BMS).
- Thermal reservoirs ([`thermal-plant.md`](./thermal-plant.md)) may be fed from
  BACnet analog inputs without forcing every analog into an appliance class.

---

## 8. First implementation slice

`crates/homecooked-bridge` is the first executable adapter:

- [`Bridge`](../../crates/homecooked-bridge/src/bridge.rs) trait maps foreign
  reads/writes ↔ HomeCooked `device_id` + qualified point + `Value`.
- **Modbus** is implemented with a YAML/JSON register map and an in-memory
  slave (no serial/TCP). Example:
  [`water_heater_map.yaml`](../../crates/homecooked-bridge/examples/water_heater_map.yaml)
  (`trait.temperature.setpoint_c`, `trait.temperature.current_c`,
  `trait.power.power_state`).
- **Matter** is implemented as a **thin mock fabric** (no CHIP / Matter SDK):
  YAML/JSON endpoint + cluster + attribute → point map and an in-memory
  attribute store. Example:
  [`kettle_matter_map.yaml`](../../crates/homecooked-bridge/examples/kettle_matter_map.yaml)
  (OnOff + TemperatureMeasurement-style attributes mapped to kettle
  `trait.power.power_state` / `trait.temperature.*`). Cluster IDs in that
  fixture are **illustrative lab constants**, not a certified Matter product.
- **Zigbee** is implemented as a **thin mock network** (no zigbee2mqtt / ZCL
  SDK): YAML/JSON endpoint + cluster + attribute → point map and an in-memory
  attribute store. Example:
  [`kettle_zigbee_map.yaml`](../../crates/homecooked-bridge/examples/kettle_zigbee_map.yaml).
  Cluster IDs are **illustrative lab constants**.
- **BACnet** compiles as a stub that returns a clear unsupported error.

See the crate [`README`](../../crates/homecooked-bridge/README.md). Real
plant buses, pairing, mesh administration, and production Matter / Zigbee
stacks stay out of scope (§5).

---

## 9. Document history

| Version | Notes |
|---------|--------|
| 0.1.0 | Initial bridges / home-automation integration sketch |
| 0.1.1 | First crate slice: Modbus mock adapter + Zigbee/Matter/BACnet stubs |
| 0.1.2 | Matter mock bridge (in-memory attributes + kettle map); Zigbee/BACnet remain stubs |
| 0.1.3 | Zigbee mock bridge (in-memory attributes + kettle map); BACnet remains stub |
