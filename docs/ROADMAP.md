# HomeCooked roadmap — ~75% project completeness

Version **0.1.17**. Planning doc for a long flesh-out of the catalog, control
stack, and simulator. It does **not** freeze APIs; crate and YAML shapes may
evolve with the code that implements each stream.

Related: [`../README.md`](../README.md), [`catalog/`](./catalog/),
[`standard/`](./standard/), especially
[`standard/control-system.md`](./standard/control-system.md) and
[`standard/examples/washer-dryer-io.md`](./standard/examples/washer-dryer-io.md).

---

## 1. Current state (~55% toward the 75% target)

What exists on `main` today (Done highlights called out):

| Area | Status |
|------|--------|
| Catalog docs | Appliance class index (**56** ids), traits, variables/settings in `docs/catalog/` |
| Standard docs | Overview, thermal-plant, procedures, bridges, control-system sketches; washer/dryer I/O example |
| `homecooked-schema` | Serde types, capability model, write validation; **56** static class tables (**25 Tier-A + 31 Tier-B**); optional thermal-port class points on `water_heater` + `fridge` — **Done** |
| `homecooked-protocol` | Envelope, request/response kinds, discovery, JSON, errors (v0.1.0); **invalid Envelope JSON table tests** |
| `homecooked-core` | Device registry, capability-enforced read/write |
| `homecooked-sim` | In-memory devices for all 56 statically tabled classes; microwave cook ticks advance `elapsed_s`; water_heater/fridge thermal-port seeds + RW attach |
| `homecooked-wasm` + `apps/simulator-web` | wasm-bindgen JSON API; full-catalog picker (56) + procedure runner (kettle + Domino's + wash-then-dry `run_procedure` E2E) + thermal panel; **WASM fetch+blob load** (module cache defeat) — **Done** |
| `homecooked-io-map` | Chassis I/O map serde + validate (washer + dryer fragments) |
| `homecooked-interlock` | Declarative interlock rules (washer heater/spin; dryer heater/motor) |
| `homecooked-hal` | Firmware HAL sketch + host `MockHal` |
| `homecooked-procedure` | Procedure documents + sequential runner; Domino's microwave + wash-then-dry multi-device fixtures complete against sim |
| `homecooked-controller` | Host controller sim: IoMap + MockHal + interlocks + washer cotton / **dryer cycle** (Idle→Dry→Cool→Done) — **Done** |
| `homecooked-thermal` | First executable thermal plant slice (types, registry, offer/accept, tick); plant types still crate-local |
| `homecooked-bridge` | **Modbus + Matter + Zigbee + BACnet mocks** (no real serial/TCP/CHIP/z2m/BACnet stacks) — **Done** |
| `homecooked-transport` | Lab TCP JSON envelopes; **optional PSK pairing**; sim-backed server + client smoke; **malformed frame table tests** — **Done** |
| `homecooked-hub` | Optional multi-device lab TCP aggregator (**not required for devices**) — **Done** |
| `homecooked-conformance` | Stream 7 smoke: Tier-A / Tier-B / cotton / kettle + wash-then-dry procedures / thermal / `water_heater_thermal_ports` / Modbus / Matter / Zigbee / BACnet / TCP / TCP PSK / hub lab set |
| CI | rustfmt, clippy (`-D warnings`), `cargo test --workspace`, wasm-pack |

**Done (thin / lab depth):** Tier-A+B **56** static tables + sim; dryer controller cycle; bridge family mocks;
lab TCP + PSK; optional hub; simulator-web blob-load; Domino's microwave Run via
`run_procedure`.

**Still open toward 75%:** promote plant types into schema (beyond device port
points); real bridge SDKs; deeper conformance matrices; richer UI;
controller-sim-over-TCP; deeper Tier-B optional points.

Rough completeness: foundation + Tier-A/B tables + procedure/HAL/TCP/hub + bridge
mocks + UI/smoke ≈ **~55%** of the 75% target below (was ~30% at roadmap start).
Remaining work is depth, not greenfield product definition.

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
- Deeper Tier-B table depth (more optional points / programs) where devices need it.
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

1. ~~Expand static class tables (and sim devices) for all **Tier-A** ids (§4).~~
   **Done.**
2. ~~Keep Tier-B as catalog ids with thinner or absent tables until later.~~
   **Done (thin tables):** all 31 Tier-B ids have static `ClassTable`s + sim.
3. Document which points are required vs optional per class consistently with
   `docs/catalog/`.

**Definition of done**

- Each Tier-A class has a static `ClassTable`, typical capability, and a sim
  device that can describe / read / write within advertised ranges.
- Tests assert table presence and basic write validation for Tier-A.
- Tier-B ids have thin static tables + sim; deeper optional points can follow.

### Stream 3 — Procedure crate + sim

**Milestones**

1. ~~Crate for procedure / recipe documents as ordered HomeCooked steps
   (aligned with `docs/standard/procedures.md`).~~ **Done** —
   `homecooked-procedure` (serde + validate + sequential runner).
2. ~~Simulator can load and run a small library.~~ **Done** — bundled
   `kettle_heat_80` + `reheat_dominos_microwave` + `wash_then_dry`; wasm `run_procedure` E2E
   auto-spawns and completes both (microwave wait uses sim `elapsed_s` ticks).
3. ~~Failures surface as protocol / capability errors, never as interlock bypass.~~
   **Done** under tests (out-of-range write, guard fail, wait timeout).

**Definition of done**

- ~~Round-trip load of a procedure document; sim executes happy-path and a
  denied/aborted path under tests.~~ **Met** (`homecooked-procedure` +
  `homecooked-wasm` `run_procedure` API).

### Stream 4 — HAL sketch + controller-sim + TCP transport

**Milestones**

1. ~~Logical HAL channel kinds (`din` / `dout` / `ain` / `aout` / `relay` /
   `motor` / …) as types, not a real board driver.~~ **Done** —
   `homecooked-hal` + `MockHal`.
2. ~~Controller-sim: bind an I/O map + interlocks + washer cycle runtime.~~
   **Done (host API)** — `homecooked-controller` runs washer `cotton` and
   dryer Idle→Heat/Dry→Cool→Done on MockHal with class interlocks
   (`washer_rules` / `dryer_rules`); protocol device-role registration left
   thin / tests drive `Controller` directly.
3. ~~TCP transport for the existing protocol envelope (one peer = one sim
   controller).~~ **Done (lab smoke)** — `homecooked-transport`: length-prefixed
   JSON framing, sim-backed TCP server + client, integration tests for
   describe / read / write (kettle + washer). **Optional lab PSK pairing**
   (dedicated auth preamble; refuse anonymous clients when configured).
   **TLS / OAuth still out of scope.** Full controller-sim-over-TCP
   (interlock-gated actuator via wire) remains a thin follow-up; host
   controller unit tests already cover cotton and dryer cycles + interlock
   denies (incl. dryer heat blocked when door unlocked).

4. ~~Optional multi-device lab hub~~ **Done (thin)** — `homecooked-hub`
   wraps `Simulator` / `DeviceHub`, reuses `homecooked-transport` TCP + optional
   PSK, and provides a kettle+washer+fridge lab set + `hub_demo`. **The hub is
   an optional aggregator for labs; devices do not require it.** No cloud auth,
   TLS, or hub UI.

**Definition of done**

- ~~Integration test: client over TCP → describe / read / write against a sim
  device.~~ **Met** for protocol round-trip via `homecooked-transport` tests.
  Controller-sim + interlock path over TCP is optional follow-up (host API
  already tested in `homecooked-controller`).
- No claim of production firmware, TLS, OAuth, or certified safety path.
  Lab PSK is a shared-secret handshake only (cleartext over cleartext TCP).

### Stream 5 — Thermal ports in schema / sim

**Milestones**

1. Schema representation of thermal / hydraulic ports from
   `docs/standard/thermal-plant.md` (sketch → types). **Progressed** —
   first executable plant slice landed in `homecooked-thermal`
   (reservoirs, heat ports, offer/accept, tick transfer). Plant types remain
   crate-local; device-facing optional catalog points landed on
   `water_heater` / `fridge` (not a full schema promotion of plant objects).
2. Sim devices that advertise and update a minimal port set (e.g. water heater
   / HVAC heat interface). **Done (thin)** for `water_heater` + `fridge`:
   optional `thermal_port_*` class points; sim seeds match the plant demo;
   `thermal_port_attached_reservoir_id` is RW. HVAC / broader classes still
   open.
3. Docs note what remains vendor / experimental. **Progressed** — catalog +
   thermal-plant note that plant types stay in `homecooked-thermal`.

**Definition of done**

- ~~At least one Tier-A thermal-capable class exercises port read/write in tests.~~
  **Met** — `water_heater` (+ lighter `fridge`) in schema/sim tests and
  conformance scenario `water_heater_thermal_ports`. Plant object types remain
  crate-local in `homecooked-thermal` (not promoted into schema this slice).

### Stream 6 — One real bridge + stubs

**Milestones**

1. ~~Choose **Matter or Modbus** for the first non-stub bridge.~~ **Done** —
   Modbus (in-memory slave; no serial/TCP SDK, so CI stays hardware-free).
2. ~~Implement mapping for a small subset of Tier-A points.~~ **Done (first
   slice)** — `homecooked-bridge` maps a fake `water_heater` (setpoint,
   current temp, power state) through a YAML/JSON register map. Tests cover
   foreign → HomeCooked and HomeCooked → register.
3. ~~Stubs (compile + clear “unimplemented”) for the other bridge families.~~
   **Done** — Matter, Zigbee, and BACnet are no longer stubs: each has a mock
   map + in-memory store + kettle roundtrip (see below). Real fabric SDKs
   remain follow-up.

4. ~~Thin Matter mock (no CHIP SDK).~~ **Done** — `MatterBridge` with
   YAML/JSON endpoint/cluster/attribute map, in-memory attribute store, and
   kettle OnOff + TemperatureMeasurement-style roundtrip tests. Cluster IDs
   are illustrative lab constants.

5. ~~Thin Zigbee mock (no zigbee2mqtt).~~ **Done** — `ZigbeeBridge` with the
   same map/store pattern and kettle roundtrip tests. No zigbee2mqtt / ZCL
   SDK dependency.

6. ~~Thin BACnet mock (no BACnet stack).~~ **Done** — `BacnetBridge` with
   YAML/JSON device-instance + object type/instance + property map,
   in-memory property store, and kettle BinaryValue / Analog* roundtrip
   tests. No BACnet/IP or MS/TP dependency.

**Definition of done**

- One bridge crate or module with tests against a fake peer or recorded
  fixtures; stubs documented in README / bridges doc.
  **Met** for Modbus + Matter + Zigbee + BACnet mock — see
  `crates/homecooked-bridge`. Real serial/TCP Modbus, CHIP / Matter SDK,
  zigbee2mqtt, and BACnet stacks remain follow-up.

### Stream 7 — WASM UI + conformance suite

**Milestones**

1. Simulator-web UX sufficient to pick a Tier-A class, inspect capabilities,
   and exercise reads/writes.
   **Done (picker slice)** — `list_appliance_classes` / `create_device` cover
   all 56 statically tabled classes (`STATIC_CLASS_IDS` = Tier-A ∪ Tier-B).
   simulator-web shows the full catalog picker grouped with `<optgroup>` from the catalog
   Index (Laundry / Cold / Wash / Cooking / Ventilation / Beverage /
   Countertop / Utility / Climate). Class id + a few key telemetry chips
   (power / temperature / cycle when present) are shown in the device
   header. **Procedure UI slice is done** — `list_example_procedures` /
   `get_example_procedure` / `parse_procedure` / `run_procedure` expose the
   sequential runner; simulator-web has a picker + paste/run panel with
   step outcomes (kettle happy-path + Domino’s microwave fixture;
   both covered by wasm `run_procedure` E2E tests).
   **Thermal-port UI slice is done** — `create_thermal_demo` /
   `thermal_state` / `thermal_negotiate_demo` / `thermal_tick` /
   `thermal_demo_transfer` expose the fridge→DHW plant; simulator-web has a
   Load demo / Negotiate / Tick / Transfer panel showing reservoirs, ports,
   and last transfer results.
   **WASM module load:** simulator-web loads bindgen via **fetch + blob URL**
   (cache defeat after rebuilds) — **Done**.
   **Still open:** richer conformance-oriented screens.
2. Conformance suite: catalog id hygiene, capability advertisement rules,
   protocol major-version rejection, representative write denials.
   **Partial (smoke)** — `homecooked-conformance` runs named end-to-end
   scenarios (Tier-A catalog/sim/describe, washer cotton controller, kettle
   procedure, wash-then-dry, thermal fridge→DHW, thermal→dishwasher preheat
   dual-path, Modbus water_heater, Matter/Zigbee/BACnet
   kettle, TCP kettle, TCP PSK describe/ping, optional lab hub discover/describe).
   **Protocol/transport robustness (table-driven):** `homecooked-transport`
   malformed length-prefixed frames + `homecooked-protocol` invalid Envelope
   JSON (oversize length, truncated body/header, invalid UTF-8, unknown kind,
   truncated JSON). **`cargo fuzz` deferred** — thorough unit tests keep CI
   free of nightly/libFuzzer deps; optional fuzz targets can land later if
   needed. Deeper catalog hygiene / write-denial matrices remain follow-up.
3. CI runs the conformance suite (or a `cargo test` subset tagged as such).
   **Done (via workspace)** — `cargo test --workspace` includes
   `homecooked-conformance`; also `cargo test -p homecooked-conformance`.

**Definition of done**

- wasm-pack build remains in CI; UI documented in `apps/simulator-web`.
  *(Picker + procedure runner + thermal plant panel + list/spawn coverage is
  in; smoke suite is in; deeper matrices still open.)*
- Conformance failures are actionable (named assertions, not a single opaque
  binary).
  *(Smoke suite reports named scenario failures; deeper matrices still open.)*
- **Contributor tooling:** how to add a class is documented in
  [`docs/catalog/ADDING_A_CLASS.md`](catalog/ADDING_A_CLASS.md) (linked from
  [`CONTRIBUTING.md`](../CONTRIBUTING.md) and the root README). Keep that guide
  accurate when Tier-A / Tier-B / `STATIC_CLASS_IDS` layout changes.

---

## 4. Tier-A and Tier-B class sets

### Tier-A (fully static tables + sim) — proposed

Target **~20–25** classes. Fully tabled points, typical traits, and simulated
devices:

| Id | Notes |
|----|--------|
| `washer` | Already tabled; deepen with I/O / interlock examples |
| `dryer` | Controller-sim cycle + `DRYER_FRAGMENT_YAML` / `dryer_rules` (host); catalog/sim still tabled |
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

### Tier-B (thin static tables + sim) — done

All remaining ids in the appliances catalog index (**31** = 56 − 25 Tier-A).
Each has a thinner `ClassTable` (typical traits + catalog class points) and
sim spawn via `typical_capability`. `STATIC_CLASS_IDS` = Tier-A ∪ Tier-B =
`ApplianceClassId::ALL`. To extend the catalog, see
[`catalog/ADDING_A_CLASS.md`](catalog/ADDING_A_CLASS.md).

| Id | Notes |
|----|--------|
| `beverage_cooler` | Cold cabinet; setpoint 1–10 °C |
| `kegerator` | Cold + dispense; optional CO₂ / keg level |
| `warming_drawer` | Hold heat; level or °C |
| `pizza_oven` | High-temp; stone/dome telemetry |
| `electric_grill` | Contact grill plates |
| `electric_smoker` | Cabinet smoke + temp |
| `espresso_machine` | Brew/steam setpoints; shot commands |
| `drip_coffee_maker` | Batch brew + keep-warm |
| `coffee_grinder` | Burr dose / grind level |
| `water_dispenser` | Hot/cold dispense |
| `toaster` | Shade + carriage |
| `blender` | Speed + lid/jar interlocks |
| `food_processor` | Speed + bowl/lid |
| `stand_mixer` | Speed + head-down interlock |
| `juicer` | Speed / reverse |
| `rice_cooker` | Programs + keep-warm |
| `slow_cooker` | Heat level + cook_s |
| `bread_maker` | Programs + crust/loaf |
| `dehydrator` | Temp + cook_s |
| `vacuum_sealer` | Vacuum/seal modes |
| `ice_cream_maker` | Churn programs |
| `yogurt_maker` | Incubation |
| `waffle_maker` | Shade / ready |
| `pasta_maker` | Mix/extrude |
| `steam_cooker` | Cook time + water empty |
| `garbage_disposal` | Timed run pulse |
| `trash_compactor` | Compact cycle |
| `boiler` | CH/DHW plant |
| `water_softener` | Regen / salt |
| `water_filter` | TDS / flush |
| `humidifier` | Output + water empty |

Count: **31** Tier-B ids, all with thin static tables + sim.

---

## 5. Suggested sequencing (PRs)

| Order | Branch theme | Stream |
|------:|--------------|--------|
| A | `docs/roadmap-75` | 1 — this document |
| B | `feat/io-map-interlocks` | 1 — io_map + interlock crates |
| later | Tier-A table batches | 2 |
| later | procedure + sim | 3 |
| later | HAL + controller-sim + TCP | 4 — TCP lab smoke done (`homecooked-transport`) |
| later | thermal ports | 5 — **Done (thin)** water_heater+fridge catalog/sim ports; plant types still crate-local |
| later | `feat/bridges-modbus` | 6 — Modbus + stubs (first slice) |
| later | `feat/matter-mock-bridge` | 6 — Matter mock fabric + kettle map |
| later | `feat/simulator-tier-a-ui` | 7 — grouped Tier-A picker (first UI slice) |
| later | WASM UI + conformance suite | 7 — picker + procedure UI (incl. Domino's `run_procedure` E2E) + thermal UI + blob-load done; smoke suite done; deeper matrices remaining |
| later | Tier-B thin tables | 2 — **Done** (31 Tier-B → 56 total static + sim) |
| later | lab hub + PSK | 4 — **Done** (`homecooked-hub`, transport PSK) |
| later | bridge mocks (Matter/Zigbee/BACnet) | 6 — **Done** (thin mocks; real SDKs still open) |
| later | dryer controller cycle | 4 — **Done** |
| later | protocol/transport robustness tests | 7 — table-driven malformed frames + invalid Envelope JSON; `cargo fuzz` deferred |

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
| 0.1.4 | Stream 6 Matter mock bridge (`homecooked-bridge` kettle map; no CHIP SDK) |
| 0.1.5 | Stream 7 thermal plant UI (`homecooked-wasm` + simulator-web thermal panel) |
| 0.1.6 | Stream 6 Zigbee mock bridge + microwave sim cook-time advance |
| 0.1.7 | Stream 6 BACnet mock bridge (`homecooked-bridge` kettle map; no BACnet stack) |
| 0.1.8 | Stream 4 dryer cycle (`homecooked-controller` + dryer io_map/interlocks) |
| 0.1.9 | Stream 4 lab TCP PSK pairing (`homecooked-transport`); TLS/OAuth still out of scope |
| 0.1.10 | Optional lab hub (`homecooked-hub`): multi-device TCP aggregator; devices do not require it |
| 0.1.11 | Stream 7 conformance: optional lab hub smoke (`hub_lab_set_discover_describe`) |
| 0.1.12 | Stream 2 Tier-B thin ClassTables (31) → full catalog **56** static + sim |
| 0.1.13 | Stream 7 simulator-web WASM load via fetch+blob (module cache defeat) |
| 0.1.14 | Stream 3/7 Domino's microwave `run_procedure` E2E; roadmap Done-state refresh |
| 0.1.15 | Stream 7 tooling: protocol/transport malformed-frame + invalid Envelope JSON table tests; `cargo fuzz` deferred |
| 0.1.16 | Stream 7 tooling: contributor guide for adding a class (`docs/catalog/ADDING_A_CLASS.md` + `CONTRIBUTING.md`) |
| 0.1.17 | Stream 5: optional `thermal_port_*` catalog points on `water_heater` + `fridge`; sim RW + conformance `water_heater_thermal_ports`; plant types still crate-local |
