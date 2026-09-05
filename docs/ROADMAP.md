# HomeCooked roadmap — ~75% project completeness

Version **0.1.32**. Planning doc for a long flesh-out of the catalog, control
stack, and simulator. It does **not** freeze APIs; crate and YAML shapes may
evolve with the code that implements each stream.

Related: [`../README.md`](../README.md), [`catalog/`](./catalog/),
[`standard/`](./standard/), especially
[`standard/control-system.md`](./standard/control-system.md) and
[`standard/examples/washer-dryer-io.md`](./standard/examples/washer-dryer-io.md).

---

## 1. Current state (~70% toward the 75% target)

What exists on `main` today (Done highlights called out):

| Area | Status |
|------|--------|
| Catalog docs | Appliance class index (**56** ids), traits, variables/settings in `docs/catalog/` |
| Standard docs | Overview, thermal-plant, procedures, bridges, control-system sketches; washer/dryer I/O example |
| `homecooked-schema` | Serde types, capability model, write validation; **56** static class tables (**25 Tier-A + 31 Tier-B**); optional thermal-port class points on `water_heater` + `fridge` + `hvac` + `dishwasher` + `dryer`; `ClassTable.thermal_ports: &[HeatPortSpec]` (static advertisement matching sim seeds); shared thermal **vocabulary** (`Media` / `PortDirection` / `TempBandC` / `HeatPortSpec`) — **Done** |
| `homecooked-protocol` | Envelope, request/response kinds, discovery, JSON, errors (v0.1.0); **invalid Envelope JSON table tests** |
| `homecooked-core` | Device registry, capability-enforced read/write |
| `homecooked-sim` | In-memory devices for all 56 statically tabled classes; microwave cook ticks advance `elapsed_s`; water_heater/fridge/hvac/dishwasher/dryer thermal-port seeds + RW attach |
| `homecooked-wasm` + `apps/simulator-web` | wasm-bindgen JSON API; full-catalog picker (56) + procedure runner (kettle + Domino's + wash-then-dry + oven bake + coffee brew + air fryer cook `run_procedure` E2E) + thermal panel + device `thermal_port_*` UI (auto when `thermal_port_id` present; `water_heater`/`fridge`/`hvac`/`dishwasher`/`dryer`); **WASM fetch+blob load** (module cache defeat) — **Done** |
| `homecooked-io-map` | Chassis I/O map serde + validate (washer + dryer fragments) |
| `homecooked-interlock` | Declarative interlock rules (washer heater/spin; dryer heater/motor) |
| `homecooked-hal` | Firmware HAL sketch + host `MockHal` |
| `homecooked-procedure` | Procedure documents + sequential runner; Domino's microwave + wash-then-dry + oven bake + coffee brew + air fryer cook + thin `thermal_wait` / `wait_dhw_reservoir` fixtures |
| `homecooked-controller` | Host controller sim: IoMap + MockHal + interlocks + washer cotton / **dryer cycle**; **lab TCP endpoints** (`ControllerEndpoint` + `DryerControllerEndpoint` interlock deny) — **Done** |
| `homecooked-thermal` | First executable thermal plant slice (types, registry, offer/accept, tick); re-exports schema thermal vocabulary; plant **runtime** still crate-local (not promoted with ClassTable `HeatPortSpec`) |
| `homecooked-bridge` | **Modbus + Matter + Zigbee + BACnet mocks** (no real serial/TCP/CHIP/z2m/BACnet stacks) — **Done** |
| `homecooked-transport` | Lab TCP JSON envelopes; **optional PSK pairing**; sim-backed server + **pluggable `RequestHandler`**; malformed frame table tests — **Done** |
| `homecooked-hub` | Optional multi-device lab TCP aggregator (**not required for devices**) — **Done** |
| `homecooked-conformance` | Stream 7 smoke: Tier-A / Tier-B / `catalog_hygiene` / `write_denial_matrix` / cotton / kettle + oven bake + coffee brew + air fryer cook + wash-then-dry procedures / thermal / `procedure_thermal_wait_dhw` / `water_heater_thermal_ports` / Modbus / Matter / Zigbee / BACnet / TCP / TCP PSK / `controller_tcp_washer_interlock` / `controller_tcp_dryer_interlock` / hub lab set |
| CI | rustfmt, clippy (`-D warnings`), `cargo test --workspace`, wasm-pack |

**Done (thin / lab depth):** Tier-A+B **56** static tables + sim; dryer controller
cycle; bridge family mocks; lab TCP + PSK; optional hub (in conformance suite);
simulator-web blob-load; procedure library (kettle + Domino's + wash-then-dry +
`oven_bake_180` + `coffee_brew_espresso` + `air_fryer_cook_200`); controller-sim-over-TCP interlock smoke (washer + dryer);
catalog `thermal_port_*` on `water_heater` / `fridge` / `hvac` / `dishwasher` / `dryer` + sim UI chips;
`write_denial_matrix` + `catalog_hygiene` conformance.

**Still open toward 75%:** promote full plant **runtime** into schema (vocabulary
types landed; `ThermalPlant` / transfer dialogue still crate-local); **one real bridge SDK** (Modbus serial/TCP or Matter/CHIP — mocks only
today); richer UI / conformance matrices beyond write-denial; deeper Tier-B
optional points; procedure⇄thermal **thin-present** (`thermal_wait` + backend
hooks + `wait_dhw_reservoir`; offer/negotiate-as-steps and fuller wasm/UI
wiring still open); richer controller device-role over TCP (cycle start /
typical caps — washer+dryer TCP interlock smoke done); TLS (still out of scope
for lab transport).

Rough completeness: foundation + Tier-A/B tables + procedure library (kettle /
Domino's / wash-then-dry / oven / coffee / air fryer + thin `thermal_wait`) +
HAL / controller TCP (washer+dryer) + hub-in-suite + thermal-port surface
(5 classes + UI) + bridge mocks + write-denial matrix ≈ **~70%** of the 75%
target below (was ~30% at roadmap start; ~65% at the v0.1.24 refresh after
oven bake / early thermal ports / controller TCP / denial matrix). PRs since
then (coffee + air-fryer procedures, dishwasher+dryer ports, `thermal_wait`)
are real but thin — they deepen Streams 3/5 without clearing the large §2
gaps — so a band of **~68–72%** is honest; **~70%** is the midpoint, not a
precise metric. Remaining work is still depth (real bridge SDK, full plant runtime schema
promotion, richer UI / procedure⇄thermal steps) — not greenfield product
definition.

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

### Honest gaps vs this 75% definition (still open)

Even with Stream 3–7 thin DoDs met on `main`, the §2 bar is not fully cleared:

- **Real bridge SDK** — in-scope asks for *one* real Matter **or** Modbus
  implementation; `homecooked-bridge` has Modbus + Matter + Zigbee + BACnet
  **mocks** only (no serial/TCP Modbus, CHIP, z2m, or BACnet stack).
- **Plant runtime** — device `thermal_port_*` points exist on
  `water_heater` / `fridge` / `hvac` / `dishwasher` / `dryer`; shared vocabulary
  (`Media` / `PortDirection` / `TempBandC`) lives in `homecooked-schema`; plant
  object runtime (`ThermalPlant`, reservoirs, offer/accept, tick) remains
  crate-local in `homecooked-thermal` (not full schema promotion).
- **Richer UI** — picker + procedure runner + thermal panel + port chips are
  in; conformance-oriented / deeper screens remain.
- **Deeper Tier-B optional points** — thin tables cover all 31 ids; more
  optional points/programs where devices need them can follow.
- **Procedure⇄thermal depth** — thin `thermal_wait` on reservoir `temp_c` is
  present (`wait_dhw_reservoir` + conformance); offer/accept/negotiate as
  procedure steps and wasm/UI wiring remain open. Dual-path dishwasher demo
  still orchestrates transfer outside the procedure JSON.
- **TLS** — lab TCP stays cleartext (+ optional PSK); TLS/OAuth remain out of
  scope for the lab path.
- **Richer controller-over-TCP** — interlock smoke for washer+dryer is done;
  cycle start / typical_capability over TCP is optional follow-up.

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
   `kettle_heat_80` + `reheat_dominos_microwave` + `wash_then_dry` + `oven_bake_180` + `coffee_brew_espresso` + `air_fryer_cook_200` + thin `wait_dhw_reservoir` (`thermal_wait`); wasm `run_procedure` E2E
   auto-spawns and completes device fixtures (microwave wait uses sim `elapsed_s` ticks). Wasm export for `thermal_wait` deferred.
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
   (`washer_rules` / `dryer_rules`); thin lab device-role via
   `ControllerEndpoint` / `DryerControllerEndpoint` (TCP interlock smoke);
   fuller typical_capability / cycle-over-TCP still follow-up.
3. ~~TCP transport for the existing protocol envelope (one peer = one sim
   controller).~~ **Done (lab smoke)** — `homecooked-transport`: length-prefixed
   JSON framing, sim-backed TCP server + client, integration tests for
   describe / read / write (kettle + washer). **Optional lab PSK pairing**
   (dedicated auth preamble; refuse anonymous clients when configured).
   **TLS / OAuth still out of scope.**
   ~~Controller-sim-over-TCP~~ **Done (lab smoke)** —
   `ControllerEndpoint` / `DryerControllerEndpoint` + `spawn_handler_server`:
   TCP write of washer heater succeeds when water+lock (deny when dry);
   dryer heater succeeds when lock+blower (deny when door unlocked) as
   `safety_interlock`. Host unit tests still cover cotton/dryer cycles.
   Deeper device-role (typical_capability, cycle start over TCP) remains
   optional follow-up.

4. ~~Optional multi-device lab hub~~ **Done (thin)** — `homecooked-hub`
   wraps `Simulator` / `DeviceHub`, reuses `homecooked-transport` TCP + optional
   PSK, and provides a kettle+washer+fridge lab set + `hub_demo`. **The hub is
   an optional aggregator for labs; devices do not require it.** No cloud auth,
   TLS, or hub UI.

**Definition of done**

- ~~Integration test: client over TCP → describe / read / write against a sim
  device.~~ **Met** for protocol round-trip via `homecooked-transport` tests.
  ~~Controller-sim + interlock path over TCP~~ **Met (lab smoke)** —
  `homecooked-controller` `tcp_interlock` + conformance
  `controller_tcp_washer_interlock` / `controller_tcp_dryer_interlock`.
- No claim of production firmware, TLS, OAuth, or certified safety path.
  Lab PSK is a shared-secret handshake only (cleartext over cleartext TCP).

### Stream 5 — Thermal ports in schema / sim

**Milestones**

1. Schema representation of thermal / hydraulic ports from
   `docs/standard/thermal-plant.md` (sketch → types). **Progressed (device
   ports Done; vocabulary types in schema; `ClassTable` carries `HeatPortSpec`;
   plant runtime still crate-local)** —
   first executable plant slice in `homecooked-thermal` (reservoirs, heat ports,
   offer/accept, tick transfer); `Media` / `PortDirection` / `TempBandC` /
   `HeatPortSpec` shared with catalog tokens via `homecooked-schema`.
   Device-facing optional catalog points landed on
   `water_heater` / `fridge` / `hvac` / `dishwasher` / `dryer`; static
   `ClassTable.thermal_ports` specs match sim seeds (not a full schema promotion of plant runtime).
2. Sim devices that advertise and update a minimal port set (e.g. water heater
   / HVAC heat interface). **Done (thin)** for `water_heater` + `fridge` +
   `hvac` + `dishwasher` + `dryer`: optional `thermal_port_*` class points; sim seeds match plant /
   hydronic lab defaults; `thermal_port_attached_reservoir_id` is RW.
   simulator-web device panel auto-surfaces ports when `thermal_port_id` is
   present (no class-id hardcoding). Broader classes still open.
3. Docs note what remains vendor / experimental. **Progressed** — catalog +
   thermal-plant note that plant **runtime** stays in `homecooked-thermal`;
   vocabulary types are schema-owned.

**Definition of done**

- ~~At least one Tier-A thermal-capable class exercises port read/write in tests.~~
  **Met** — `water_heater` (+ lighter `fridge` + `hvac` + `dishwasher` + `dryer`) in schema/sim tests and
  conformance scenario `water_heater_thermal_ports`. Plant **runtime** types remain
  crate-local in `homecooked-thermal`; vocabulary enums are in schema (this slice).

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
   step outcomes (kettle + Domino’s microwave + wash-then-dry + oven bake +
   coffee brew + air fryer cook; `wait_dhw_reservoir` listed/bundled —
   `thermal_wait` run needs plant attach); covered by wasm `run_procedure`
   E2E tests).
   **Thermal-port UI slice is done** — `create_thermal_demo` /
   `thermal_state` / `thermal_negotiate_demo` / `thermal_tick` /
   `thermal_demo_transfer` expose the fridge→DHW plant; simulator-web has a
   Load demo / Negotiate / Tick / Transfer panel showing reservoirs, ports,
   and last transfer results. Device panel also surfaces catalog
   `thermal_port_*` chips + attach write for `water_heater` / `fridge` / `hvac` / `dishwasher` / `dryer` (auto-gated on `thermal_port_id`).
   **WASM module load:** simulator-web loads bindgen via **fetch + blob URL**
   (cache defeat after rebuilds) — **Done**.
   **Still open:** richer conformance-oriented screens.
2. Conformance suite: catalog id hygiene, capability advertisement rules,
   protocol major-version rejection, representative write denials.
   **Partial (smoke + denial matrix)** — `homecooked-conformance` runs named
   end-to-end scenarios (Tier-A/B catalog/sim/describe, `catalog_hygiene`,
   table-driven `write_denial_matrix` across Tier-A denial kinds, washer cotton
   controller, kettle procedure, oven bake, coffee brew, wash-then-dry, thermal fridge→DHW,
   thermal→dishwasher preheat dual-path, Modbus water_heater,
   Matter/Zigbee/BACnet kettle, TCP kettle, TCP PSK describe/ping, controller
   TCP washer + dryer interlock, hub lab-set discover/describe).
   **Protocol/transport robustness (table-driven):** `homecooked-transport`
   malformed length-prefixed frames + `homecooked-protocol` invalid Envelope
   JSON (oversize length, truncated body/header, invalid UTF-8, unknown kind,
   truncated JSON). **`cargo fuzz` deferred** — thorough unit tests keep CI
   free of nightly/libFuzzer deps; optional fuzz targets can land later if
   needed. Deeper write-denial matrix progressed (`write_denial_matrix` +
   `catalog_hygiene`); further matrices / richer UI remain follow-up.
3. CI runs the conformance suite (or a `cargo test` subset tagged as such).
   **Done (via workspace)** — `cargo test --workspace` includes
   `homecooked-conformance`; also `cargo test -p homecooked-conformance`.

**Definition of done**

- wasm-pack build remains in CI; UI documented in `apps/simulator-web`.
  *(Picker + procedure runner + thermal plant panel + catalog thermal-port
  device chips + list/spawn coverage is in; smoke suite + write-denial matrix
  are in; richer UI still open.)*
- Conformance failures are actionable (named assertions, not a single opaque
  binary).
  *(Smoke suite + per-case `write_denial_matrix` failures; further matrices
  still open.)*
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
| `oven` | Already tabled; bake procedure + stub heat tick |
| `steam_oven` | |
| `range` | |
| `cooktop` | |
| `induction_hob` | Already tabled |
| `air_fryer` | Already tabled |
| `kettle` | Already tabled |
| `coffee_machine` | Already tabled; brew procedure + stub boiler heat tick |
| `water_heater` | Thermal-port surface (catalog/sim) |
| `hvac` | Thermal-port surface (catalog/sim) |
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
| later | procedure + sim | 3 — **Done** (kettle + Domino's + wash-then-dry + `oven_bake_180` + `coffee_brew_espresso` + `air_fryer_cook_200` + thin `thermal_wait`) |
| later | HAL + controller-sim + TCP | 4 — TCP lab smoke + washer+dryer controller-sim-over-TCP interlock smoke **Done** |
| later | thermal ports | 5 — **Done (thin)** water_heater+fridge+hvac+dishwasher+dryer catalog/sim ports; schema vocabulary + `ClassTable.HeatPortSpec`; plant runtime still crate-local |
| later | `feat/bridges-modbus` | 6 — Modbus + stubs (first slice) |
| later | `feat/matter-mock-bridge` | 6 — Matter mock fabric + kettle map |
| later | `feat/simulator-tier-a-ui` | 7 — grouped Tier-A picker (first UI slice) |
| later | WASM UI + conformance suite | 7 — picker + procedure UI (kettle/Domino's/wash-then-dry/oven bake/coffee brew/air fryer cook) + thermal UI + device port chips + blob-load done; smoke suite + write-denial matrix + hub-in-suite done; richer UI remaining |
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
| 0.1.18 | Stream 4: controller-sim-over-TCP lab smoke (`ControllerEndpoint` + `RequestHandler` TCP; washer heater allow/deny; conformance `controller_tcp_washer_interlock`) |
| 0.1.19 | Stream 7: deeper write-denial / catalog hygiene conformance matrix (`write_denial_matrix` + `catalog_hygiene`) |
| 0.1.20 | Stream 4: dryer controller-sim-over-TCP (`DryerControllerEndpoint`; heater deny when unlocked; conformance `controller_tcp_dryer_interlock`) |
| 0.1.21 | Stream 5/7: simulator-web surfaces catalog `thermal_port_*` on `water_heater`/`fridge` (chips + attach write) |
| 0.1.22 | Stream 5: optional `thermal_port_*` on `hvac` (coil/sink/water/5000 W lab seeds); extend `water_heater_thermal_ports`; wire `hub_lab_set_discover_describe` into `all_scenarios` |
| 0.1.23 | Stream 3: `oven_bake_180` procedure fixture + minimal oven heat tick; wasm/`run_procedure` E2E + conformance |
| 0.1.24 | Current-state refresh: ~55% → **~65%** of 75% target; cite Stream 4/5/7 merges through oven bake, thermal ports, controller TCP, write-denial matrix, hub-in-suite, UI thermal panel; honest §2 gaps list |
| 0.1.25 | Stream 3: `coffee_brew_espresso` procedure fixture + minimal coffee boiler heat tick; wasm/`run_procedure` E2E + conformance |
| 0.1.26 | Stream 5: optional `thermal_port_*` on `dishwasher` (`inlet_preheat`/sink/water/1800 W); extend `water_heater_thermal_ports` |
| 0.1.27 | Stream 3/5: thin procedure⇄thermal bridge (`thermal_wait` / backend hooks / `wait_dhw_reservoir` + conformance `procedure_thermal_wait_dhw`); offer-as-steps + wasm UI deferred |
| 0.1.28 | Stream 3: `air_fryer_cook_200` procedure fixture + minimal air fryer heat tick; wasm/`run_procedure` E2E + conformance |
| 0.1.29 | Stream 5: optional `thermal_port_*` on `dryer` (`exhaust`/source/air/2000 W); extend `water_heater_thermal_ports` |
| 0.1.30 | Current-state refresh: ~65% → **~70%** (~68–72% band) of 75% target; cite PRs since v0.1.24 (coffee/air-fryer procedures, dishwasher+dryer thermal ports, `thermal_wait` / `wait_dhw_reservoir`, sim-web procedure copy); §2 gaps unchanged in kind |
| 0.1.31 | Stream 5: shared thermal vocabulary types (`Media` / `PortDirection` / `TempBandC` / `HeatPortSpec`) in `homecooked-schema`; `homecooked-thermal` re-exports; plant runtime still crate-local |
