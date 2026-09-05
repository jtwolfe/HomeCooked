# HomeCooked roadmap — ~75% project completeness

Version **0.1.0**. Planning doc for a long flesh-out of the catalog, control
stack, and simulator. It does **not** freeze APIs; crate and YAML shapes may
evolve with the code that implements each stream.

Related: [`../README.md`](../README.md), [`catalog/`](./catalog/),
[`standard/`](./standard/), especially
[`standard/control-system.md`](./standard/control-system.md) and
[`standard/examples/washer-dryer-io.md`](./standard/examples/washer-dryer-io.md).

---

## 1. Current state (~30%)

What exists on `main` today:

| Area | Status |
|------|--------|
| Catalog docs | Appliance class index (~56 ids), traits, variables/settings in `docs/catalog/` |
| Standard docs | Overview, thermal-plant, procedures, bridges, control-system sketches; washer/dryer I/O example |
| `homecooked-schema` | Serde types, capability model, write validation; **25** Tier-A static class tables |
| `homecooked-protocol` | Envelope, request/response kinds, discovery, JSON, errors (v0.1.0) |
| `homecooked-core` | Device registry, capability-enforced read/write |
| `homecooked-sim` | In-memory devices for the 25 Tier-A static classes |
| `homecooked-wasm` + `apps/simulator-web` | wasm-bindgen JSON API; simulator-web grouped Tier-A picker (25 classes) + procedure runner panel |
| `homecooked-io-map` | Chassis I/O map serde + validate |
| `homecooked-interlock` | Declarative interlock rules (washer heater/spin) |
| `homecooked-hal` | Firmware HAL sketch + host `MockHal` |
| `homecooked-procedure` | Procedure documents + sequential runner |
| `homecooked-controller` | Host controller sim: IoMap + MockHal + interlocks + washer cotton cycle |
| `homecooked-thermal` | First executable thermal plant slice (types, registry, offer/accept, tick) |
| `homecooked-bridge` | First bridge slice: Modbus map + in-memory slave; Zigbee/Matter/BACnet stubs |
| `homecooked-transport` | Lab TCP: length-prefixed JSON envelopes; sim-backed server + client smoke |
| `homecooked-conformance` | Light Stream 7 smoke: Tier-A / cotton / kettle procedure / thermal / Modbus / TCP |
| CI | rustfmt, clippy (`-D warnings`), `cargo test --workspace`, wasm-pack |

**25 Tier-A classes are fully tabled** (see §4).

`list_all_class_ids` already covers the full appliances index; most classes are
ids-only (no static tables / sim yet). Thermal has `homecooked-thermal` (plant
slice; catalog/sim ports still open). Bridges have `homecooked-bridge`
(Modbus first slice + stubs; real serial/TCP and other fabrics still
open); control-system has HAL + controller-sim + io-map/interlock crates
(TCP lab smoke in `homecooked-transport`; auth still out of scope); procedures has `homecooked-procedure`.

Rough completeness: docs + thin protocol/sim spine ≈ **~30%** of the 75%
target below. Remaining work is depth (tables, I/O map, interlocks, HAL/sim
transport, one bridge, UI/conformance), not greenfield product definition.

---

## 2. Definition of ~75% done

### In scope for ~75%

- Roadmap, I/O map crate, and declarative interlock crate (Foundation).
- **Tier-A** catalog classes fully static-tabled and simulated (~20–25 classes;
  see §4).
- Procedure crate + simulator support for ordered HomeCooked steps.
- HAL sketch + controller-sim + TCP transport (software path from write →
  logical channel; not production firmware).
- Thermal ports represented in schema and exercised in sim.
- **One** real bridge implementation (Matter **or** Modbus) plus stubs for the
  others.
- WASM UI improvements and a conformance suite against catalog/protocol rules.

### Explicitly out of scope (even at 75%)

- **IEC / functional-safety certification** (IEC 60335, IEC 61508, SIL/PL
  claims). Local hardwired interlocks remain device responsibility.
- **Production PCB / MCU / relay BOM** — hardware stays directional.
- **Cloud OAuth**, global device CA, or cloud identity product work.
- Full Tier-B table depth for every remaining catalog id.
- Shipping a commercial appliance or certified Matter/Modbus product.

---

## 3. Workstreams and ordered milestones

Merge order preference: Foundation first, then Tier-A tables, then procedure /
HAL / thermal / bridge / UI as capacity allows. Later streams may land as
multiple small PRs.

### Stream 1 — Foundation

**Milestones**

1. This roadmap (`docs/ROADMAP.md`) linked from the README.
2. `homecooked-io-map` — serde types for chassis I/O maps (YAML/JSON), load +
   validate, example aligned with the washer fragment in
   `docs/standard/examples/washer-dryer-io.md`.
3. `homecooked-interlock` — declarative rules (bool AND/OR, comparisons);
   deny actuator / force safe state; evaluate before applying actuator
   commands; washer heater/spin examples.

**Definition of done**

- Both crates in the workspace, auditable and small, with unit tests.
- `cargo test --workspace` and clippy (`-D warnings`) green.
- README workspace table lists the new crates.

### Stream 2 — Tier-A catalog tables

**Milestones**

1. Expand static class tables (and sim devices) for all **Tier-A** ids (§4).
2. Keep Tier-B as catalog ids with thinner or absent tables until later.
3. Document which points are required vs optional per class consistently with
   `docs/catalog/`.

**Definition of done**

- Each Tier-A class has a static `ClassTable`, typical capability, and a sim
  device that can describe / read / write within advertised ranges.
- Tests assert table presence and basic write validation for Tier-A.
- Tier-B ids remain in `list_all_class_ids` without blocking 75%.

### Stream 3 — Procedure crate + sim

**Milestones**

1. Crate for procedure / recipe documents as ordered HomeCooked steps
   (aligned with `docs/standard/procedures.md`). First slice:
   `homecooked-procedure` (serde + validate + sequential runner).
2. Simulator can load and run a small library (e.g. washer `cotton` outline
   at the *client* procedure level, not micro-stepping HV).
3. Failures surface as protocol / capability errors, never as interlock bypass.

**Definition of done**

- Round-trip load of a procedure document; sim executes happy-path and a
  denied/aborted path under tests.

### Stream 4 — HAL sketch + controller-sim + TCP transport

**Milestones**

1. ~~Logical HAL channel kinds (`din` / `dout` / `ain` / `aout` / `relay` /
   `motor` / …) as types, not a real board driver.~~ **Done** —
   `homecooked-hal` + `MockHal`.
2. ~~Controller-sim: bind an I/O map + interlocks + washer cycle runtime.~~
   **Done (host API)** — `homecooked-controller` runs washer `cotton` on
   MockHal with interlocks; protocol device-role registration left thin /
   tests drive `Controller` directly.
3. ~~TCP transport for the existing protocol envelope (one peer = one sim
   controller).~~ **Done (lab smoke)** — `homecooked-transport`: length-prefixed
   JSON framing, sim-backed TCP server + client, integration tests for
   describe / read / write (kettle + washer). **Auth / TLS still out of
   scope.** Full controller-sim-over-TCP (interlock-gated actuator via wire)
   remains a thin follow-up; host controller unit tests already cover cotton
   cycle + interlock denies.

**Definition of done**

- ~~Integration test: client over TCP → describe / read / write against a sim
  device.~~ **Met** for protocol round-trip via `homecooked-transport` tests.
  Controller-sim + interlock path over TCP is optional follow-up (host API
  already tested in `homecooked-controller`).
- No claim of production firmware, TLS, OAuth, or certified safety path.

### Stream 5 — Thermal ports in schema / sim

**Milestones**

1. Schema representation of thermal / hydraulic ports from
   `docs/standard/thermal-plant.md` (sketch → types). **Progressed** —
   first executable plant slice landed in `homecooked-thermal`
   (reservoirs, heat ports, offer/accept, tick transfer). Types are
   crate-local, not yet promoted into `homecooked-schema` / the catalog.
2. Sim devices that advertise and update a minimal port set (e.g. water heater
   / HVAC heat interface). **Still open** — Tier-A class port read/write
   in `homecooked-sim` is not in this slice.
3. Docs note what remains vendor / experimental.

**Definition of done**

- At least one Tier-A thermal-capable class exercises port read/write in tests.
  *(Plant-level fridge-condenser → DHW demo is in `homecooked-thermal`;
  catalog/sim port points remain follow-up.)*

### Stream 6 — One real bridge + stubs

**Milestones**

1. ~~Choose **Matter or Modbus** for the first non-stub bridge.~~ **Done** —
   Modbus (in-memory slave; no serial/TCP SDK, so CI stays hardware-free).
2. ~~Implement mapping for a small subset of Tier-A points.~~ **Done (first
   slice)** — `homecooked-bridge` maps a fake `water_heater` (setpoint,
   current temp, power state) through a YAML/JSON register map. Tests cover
   foreign → HomeCooked and HomeCooked → register.
3. ~~Stubs (compile + clear “unimplemented”) for the other bridge families.~~
   **Done** — `ZigbeeBridge`, `MatterBridge`, `BacnetBridge` return
   `Error::UnsupportedFabric` and point at
   [`standard/bridges.md`](./standard/bridges.md).

**Definition of done**

- One bridge crate or module with tests against a fake peer or recorded
  fixtures; stubs documented in README / bridges doc.
  **Met for the first slice** — see `crates/homecooked-bridge`. Real
  serial/TCP Modbus, Matter SDK, and Zigbee/BACnet adapters remain
  follow-up.

### Stream 7 — WASM UI + conformance suite

**Milestones**

1. Simulator-web UX sufficient to pick a Tier-A class, inspect capabilities,
   and exercise reads/writes.
   **Done (picker slice)** — `list_appliance_classes` / `create_device` cover
   all 25 `TIER_A_CLASS_IDS` (same set as `STATIC_CLASS_IDS`). simulator-web
   shows the full Tier-A picker grouped with `<optgroup>` from the catalog
   Index (Laundry / Cold / Wash / Cooking / Ventilation / Beverage /
   Countertop / Utility / Climate). Class id + a few key telemetry chips
   (power / temperature / cycle when present) are shown in the device
   header. **Procedure UI slice is done** — `list_example_procedures` /
   `get_example_procedure` / `parse_procedure` / `run_procedure` expose the
   sequential runner; simulator-web has a picker + paste/run panel with
   step outcomes (kettle happy-path + Domino’s microwave fixture).
   **Still open:** thermal-port UI and richer conformance-oriented screens.
2. Conformance suite: catalog id hygiene, capability advertisement rules,
   protocol major-version rejection, representative write denials.
   **Partial (smoke)** — `homecooked-conformance` runs six named end-to-end
   scenarios (Tier-A catalog/sim/describe, washer cotton controller, kettle
   procedure, thermal fridge→DHW, Modbus water_heater, TCP kettle). Deeper
   catalog hygiene / major-version / write-denial matrices remain follow-up.
3. CI runs the conformance suite (or a `cargo test` subset tagged as such).
   **Done (via workspace)** — `cargo test --workspace` includes
   `homecooked-conformance`; also `cargo test -p homecooked-conformance`.

**Definition of done**

- wasm-pack build remains in CI; UI documented in `apps/simulator-web`.
  *(Picker + procedure runner panel + list/spawn coverage is in; smoke suite
  is in; deeper matrices / thermal-port UI still open.)*
- Conformance failures are actionable (named assertions, not a single opaque
  binary).
  *(Smoke suite reports named scenario failures; deeper matrices still open.)*

---

## 4. Tier-A and Tier-B class sets

### Tier-A (fully static tables + sim) — proposed

Target **~20–25** classes. Fully tabled points, typical traits, and simulated
devices:

| Id | Notes |
|----|--------|
| `washer` | Already tabled; deepen with I/O / interlock examples |
| `dryer` | Already tabled |
| `washer_dryer` | Composition of laundry traits |
| `fridge` | Already tabled |
| `freezer` | |
| `fridge_freezer` | Zoned cooling |
| `dishwasher` | Already tabled |
| `microwave` | Already tabled |
| `oven` | Already tabled |
| `steam_oven` | |
| `range` | |
| `cooktop` | |
| `induction_hob` | Already tabled |
| `air_fryer` | Already tabled |
| `kettle` | Already tabled |
| `coffee_machine` | |
| `water_heater` | Thermal-port candidate |
| `hvac` | Thermal-port candidate |
| `dehumidifier` | |
| `range_hood` | |
| `toaster_oven` | |
| `sous_vide` | |
| `multi_cooker` | |
| `ice_maker` | |
| `wine_cooler` | |

Count: **25** Tier-A ids, all with static tables + sim.

### Tier-B

All remaining ids in the appliances catalog index (today: 56 total − 25
Tier-A ≈ **31** Tier-B). Ids exist in schema/`list_all_class_ids`; thinner or
absent static tables are OK until after the 75% bar.

---

## 5. Suggested sequencing (PRs)

| Order | Branch theme | Stream |
|------:|--------------|--------|
| A | `docs/roadmap-75` | 1 — this document |
| B | `feat/io-map-interlocks` | 1 — io_map + interlock crates |
| later | Tier-A table batches | 2 |
| later | procedure + sim | 3 |
| later | HAL + controller-sim + TCP | 4 — TCP lab smoke done (`homecooked-transport`) |
| later | thermal ports | 5 |
| later | `feat/bridges-modbus` | 6 — Modbus + stubs (first slice) |
| later | `feat/simulator-tier-a-ui` | 7 — grouped Tier-A picker (first UI slice) |
| later | WASM UI + conformance suite | 7 — picker + procedure UI done; smoke suite done (`homecooked-conformance`); deeper matrices / thermal-port UI remaining |

One concern per PR when practical. Catalog/standard docs land before or with
the code that implements them.

---

## 6. Document history

| Version | Notes |
|---------|--------|
| 0.1.0 | Initial ~30% → ~75% roadmap; Tier-A list; seven workstreams |
| 0.1.1 | Stream 4 TCP lab smoke (`homecooked-transport`); auth/TLS still out of scope |
| 0.1.2 | Stream 7 conformance smoke crate (`homecooked-conformance`) |
| 0.1.3 | Stream 7 procedure UI slice (`homecooked-wasm` + simulator-web runner panel) |
