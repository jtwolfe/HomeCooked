# House thermal / hydraulic coupling

Version **0.1.0** — design extension (docs sketch); device catalog points landed for water_heater/fridge/hvac (see history).

HomeCooked appliances do not live in thermal isolation. Washers, dryers,
fridges, heat-pump water heaters, dishwashers, and space-conditioning plant
exchange heat with shared **hot** and **cold** reservoirs (and sometimes
domestic hot water). This document sketches a **thermal plant** layer that
coordinates those exchanges without replacing appliance classes or local
safety.

Related:

- [`overview.md`](./overview.md) — catalog → schema → wire protocol
- [`procedures.md`](./procedures.md) — multi-step orchestration over points
- [`bridges.md`](./bridges.md) — mapping to Zigbee / Matter / Modbus / BACnet

This is a **design sketch**. It does not freeze wire messages or schema types.
Implementations may experiment under `vendor.*` or experimental core profiles
until a later catalog revision promotes stable ids.

---

## 1. Goals

- Make heat **producers** and **consumers** visible as first-class resources
  alongside appliance telemetry.
- Allow a hub or plant controller to **offer and accept** heat transfers
  between devices that share hydraulic or air loops.
- Keep **appliance classes** focused on device semantics; plant objects are
  separate resources that devices *attach to* via heat ports.
- Remain **best-effort**: coordination may fail open to local-only control.

## 2. Non-goals

- A full energy market, tariff optimizer, or carbon accounting system
- Replacing IEC 60335 / local interlocks, pressure relief, or refrigerant
  charge safety
- Mandating a particular pipe layout, glycol mix, or refrigerant circuit
- Requiring every SKU to participate (ports are optional capabilities)

---

## 3. Reservoirs

A **reservoir** is a shared thermal buffer with a usable temperature band and
finite capacity (energy or thermal mass proxy).

| Reservoir role | Typical media | Notes |
|----------------|---------------|-------|
| Hot | Hydronic loop, buffer tank, condenser water | Space heating, DHW preheat, dryer reject recovery |
| Cold | Chilled water, ground loop, outdoor sink | AC reject, fridge condenser assist, HPWH evaporator |
| DHW tank (optional) | Potable hot water | May act as both store and consumer; treat as a reservoir object when shared |

Reservoirs are **plant objects**, not appliance classes. A `water_heater` or
`boiler` class instance may *own* or *feed* a DHW / hot reservoir; the
reservoir id is still distinct from the appliance `device_id`.

Minimal conceptual fields (informative):

```
Reservoir {
  reservoir_id:     Id
  role:             hot | cold | dhw | other
  media:            water | air | glycol | refrigerant_proxy | unknown
  temp_c:           f32?              // measured or estimated
  usable_band_c:    { min, max }
  capacity_kwh:     f32?              // optional energy proxy
  headroom_kw:      f32?              // optional instantaneous accept/reject
}
```

---

## 4. Producers and consumers

Heat **producers** reject or export heat. Heat **consumers** absorb or import
heat. The same physical machine often exposes both (a heat-pump water heater
has an evaporator sink and a condenser source).

| Example | Port role | Typical coupling |
|---------|-----------|------------------|
| AC condenser rejection | source → cold or outdoor | Reject into shared cold loop or outdoor coil |
| Fridge / freezer condenser | source → ambient or cold loop | Optional recovery into DHW preheat |
| Heat-pump dryer | source (condenser) / sink (evaporator) | Condenser heat to DHW or space; evaporator from ambient/loop |
| HPWH | sink (evaporator) / source (condenser → tank) | Condenser tied to DHW reservoir |
| Dishwasher inlet preheat | sink from hot / DHW | Lower electrical boost when inlet is warm |
| Space heating | sink from hot reservoir | Radiators, underfloor, air handler |

Direction is from the appliance's point of view on each **heat port**.

---

## 5. Resource model: plant ≠ class

| Concept | What it is | Catalog home |
|---------|------------|--------------|
| Appliance class | Washer, fridge, HPWH, HVAC, … | `docs/catalog/appliances.md` |
| Plant object | Reservoir, loop segment, plant controller | Future plant catalog / this sketch |
| Heat port | Advertised attachment on an appliance | Optional class points (`thermal_port_*`) on `water_heater` / `fridge` / `hvac` / `dishwasher` / `dryer`; static `ClassTable.thermal_ports` (`HeatPortSpec`) in `homecooked-schema`; plant runtime still in `homecooked-thermal` |

**Rule:** do not invent parallel "thermal washer" classes. A `dryer` stays a
`dryer`. If it can export condenser heat, it advertises a heat port that
references a plant reservoir (or "unattached").

### 5.1 Heat port advertisement

Each port is a capability facet (informative shape):

```
HeatPort {
  port_id:          Id                 // local to the device
  direction:        source | sink | bidirectional
  max_power_w:      u32
  usable_temp_c:    { min, max }       // fluid/air band where the port is useful
  priority:         u8                 // 0 = best-effort scrap heat … higher = comfort-critical
  media:            water | air | …    // must be compatible with the reservoir
  attached_reservoir_id: Id | none
}
```

Priority is a **hint** for the negotiator (e.g. space-heating comfort above
opportunistic fridge-heat recovery). It is not a market price.

---

## 6. Negotiation sketch (offer / accept)

Avoid a full energy market. Use a small **offer → accept | decline** dialogue
among plant controller and participating devices.

Informative flow:

1. **Announce** — devices publish heat ports + instantaneous availability
   (e.g. "fridge condenser rejecting ~120 W, air, 35–45 °C").
2. **Offer** — plant controller (or a peer) proposes a transfer:
   `{ from_port, to_reservoir_or_port, power_w_band, duration_s?, priority }`.
3. **Accept / decline** — sink and source each confirm they can participate
   *without* violating local interlocks or cycle integrity.
4. **Active** — optional heartbeat / `transfer_state` while coupled.
5. **Release** — either side or the controller ends the transfer; devices
   revert to local thermal policy.

Properties:

- **No mandatory clearing price.** Optional future profiles may add cost
  signals; core sketch does not.
- **Partial fill OK.** Accepting 80 W of a 120 W offer is allowed if
  advertised.
- **Failure is normal.** Timeout or decline → devices continue standalone.
- **No silent override.** An offer never forces a magnetron, compressor, or
  valve past local limits; the device may accept and then deliver less (emit
  an event).

Wire encoding is deferred. Until then, hubs may coordinate out-of-band using
ordinary HomeCooked reads/writes on vendor or experimental points.

---

## 7. Safety

- **Local interlocks win.** Door, pressure, over-temp, leak, and refrigerant
  safeties remain device-local. The thermal bus does not bypass them (same
  rule as [`overview.md`](./overview.md) §11).
- **Best-effort coordination.** Loss of plant controller, reservoir sensor, or
  peer must leave each appliance in a safe autonomous mode.
- **Potable vs non-potable.** DHW reservoirs that touch drinking water need
  explicit media / isolation metadata; do not casually tee grey-water heat
  recovery into potable without a documented exchanger boundary.
- **Priority ≠ authority.** A high-priority space-heat sink cannot command a
  fridge to stop protecting food temperature.

---

## 8. Relation to procedures and bridges

- A **procedure** ([`procedures.md`](./procedures.md)) may include steps that
  read reservoir temperature or wait for an accepted transfer before starting
  a dishwasher preheat.
- A **bridge** ([`bridges.md`](./bridges.md)) may map BACnet/Modbus plant
  points into HomeCooked reservoir objects while whitegoods stay on richer
  appliance semantics.

### 8.1 Dual-path demo (v0.1 executable)

The original dual-path demo orchestrates **outside** the procedure JSON
(thermal transfer, then dishwasher settings). That path remains:

1. **Thermal path** — `ThermalPlant::fridge_condenser_dhw_demo()`, negotiate
   fridge condenser → water_heater preheat, `step(3600)` → DHW rises
   35.0 → 36.2 °C (`homecooked-thermal`, wasm `create_thermal_demo` /
   `thermal_demo_transfer`).
2. **Procedure path** — `dishwasher_dhw_preheat` writes `trait.program.program`
   = `eco` and `class.dishwasher.wash_temp_c` = 45 reflecting warm inlet /
   lower electrical boost (`homecooked-procedure`).

Orchestrators:

| Surface | Entry |
|---------|--------|
| Conformance | `thermal_then_dishwasher_preheat` in `homecooked-conformance` |
| WASM | `run_thermal_then_dishwasher_preheat(dt_s)` |
| Web sim | **Thermal → dishwasher preheat** (Orchestrations; one-click) or Thermal panel + procedure picker |

```bash
cargo test -p homecooked-conformance thermal_then_dishwasher
cargo test -p homecooked-wasm run_thermal_then_dishwasher
```

### 8.2 Thin procedure⇄thermal bridge (`thermal_wait`)

Procedures can now **wait/assert on a plant reservoir temperature** without
inventing parallel appliance classes:

- Step action `thermal_wait` (alias `wait_reservoir`) with
  `{ reservoir_id, cmp, temp_c, timeout_s }`.
- `DeviceBackend::thermal_read_reservoir_temp` / `thermal_tick` (default:
  unsupported / no-op). `SimulatorBackend` holds an optional `ThermalPlant`
  and implements both.
- Bundled fixture `wait_dhw_reservoir` waits until `dhw-tank` ≥ 36 °C.
  Plant accepts are one-shot per `step`, so demos typically negotiate+step
  (or otherwise seed the reservoir) **before** the wait; the wait loop still
  polls + `thermal_tick` until the comparison holds or `timeout_s` elapses.
  For continuous heat while waiting, see §8.4 (`requeue_offer`).

| Surface | Entry |
|---------|--------|
| Procedure crate | `WAIT_DHW_RESERVOIR_JSON` + `SimulatorBackend::with_plant` |
| Conformance | `procedure_thermal_wait_dhw` |

```bash
cargo test -p homecooked-procedure thermal_wait
cargo test -p homecooked-conformance procedure_thermal_wait_dhw
```

**Continuous re-queue** across wait polls lives under §8.4. **Still deferred:**
promoting full plant **runtime** into schema (vocabulary types +
`ClassTable.HeatPortSpec` live in `homecooked-schema`; `ThermalPlant` / transfer
dialogue remain crate-local); wasm/UI wiring beyond bundled list/`run_procedure`
for thermal steps (dual-path orchestrator UI remains). Soft decline + thin
fallback retry live under §8.3.

### 8.3 Thin procedure⇄thermal bridge (`thermal_offer`)

Procedures can **submit a heat-transfer offer** aligned with `TransferOffer` and
immediately negotiate (accept at max allowable power, or decline):

- Step action `thermal_offer` (alias `offer_transfer`) with
  `{ from_port, to_port | to_reservoir_id, power_w, duration_s?, priority?,
  fallback_power_w?, on_decline? }`.
- `DeviceBackend::thermal_offer` / `thermal_accept` / `thermal_negotiate`
  (default: unsupported). `SimulatorBackend` with an attached plant implements
  them via `ThermalPlant::{offer,accept,negotiate}`.
- On **Accept**, `read_value` is the accepted power (u32). When `duration_s` is
  set, the runner applies **one** `thermal_tick` of that length so the queued
  accept moves energy (fridge→DHW demo).
- On **Decline**, default `on_decline: fail` fails the step (`InvalidRequest`)
  with the decline reason. `on_decline: continue` soft-continues (`ok: true`,
  `read_value` null, message notes the decline) so later steps can run.
- Thin multi-round: optional `fallback_power_w` retries **once** with that band
  after a first decline; the final decline still respects `on_decline`. Plant
  `negotiate` declines when available max is below the offered min (no silent
  partial below min). No new `TransferReply` variant.
- Bundled fixture `offer_fridge_dhw` offers fridge condenser → water-heater
  preheat at 80–120 W for 3600 s. `offer_fridge_dhw_soft` demos a first band
  that declines (min above condenser max) then fallback accept.

| Surface | Entry |
|---------|--------|
| Procedure crate | `OFFER_FRIDGE_DHW_JSON` / `OFFER_FRIDGE_DHW_SOFT_JSON` + `SimulatorBackend::with_plant` |
| Conformance | `procedure_thermal_offer_dhw` / `procedure_thermal_offer_soft_decline` |

```bash
cargo test -p homecooked-procedure thermal_offer
cargo test -p homecooked-conformance procedure_thermal_offer_dhw
cargo test -p homecooked-conformance procedure_thermal_offer_soft_decline
```

**Still deferred:** dedicated wasm UI controls beyond listing/`run_procedure`;
fuller multi-round dialogue as separate typed steps / plant Counter replies.
Continuous re-queue across wait polls lives under §8.4.

### 8.4 Continuous re-queue across wait polls (`requeue_offer`)

Plant accepts remain **one-shot per `step`**: after a `thermal_tick` consumes
pending accepts, a bare `thermal_wait` only polls + ticks and the plant sits
idle. To keep energy flowing while waiting on a reservoir threshold, set
`requeue_offer: true` on a `thermal_wait` and inline the same transfer fields
used by `thermal_offer` (`from_port`, `to_port` | `to_reservoir_id`, `power_w`,
`priority?`):

- Before each wait-poll `thermal_tick`, the runner builds a `TransferOffer`
  (forcing `duration_s = None` so the poll interval drives applied energy),
  calls `thermal_offer` + `thermal_negotiate`, then ticks.
- A mid-wait decline fails the step (`InvalidRequest`) with the decline reason.
- Prefer this explicit step field over silent plant-level “keep last accept”
  magic.

Bundled fixture `wait_dhw_with_requeue` waits until `dhw-tank` ≥ 36 °C while
re-queuing fridge condenser → water-heater preheat each poll (no prior
`duration_s` apply required).

| Surface | Entry |
|---------|--------|
| Procedure crate | `WAIT_DHW_WITH_REQUEUE_JSON` + `SimulatorBackend::with_plant` |
| Conformance | `procedure_thermal_wait_requeue` |

```bash
cargo test -p homecooked-procedure thermal_wait_with_requeue
cargo test -p homecooked-conformance procedure_thermal_wait_requeue
```

**Still deferred:** dedicated wasm UI controls; fuller typed multi-round /
plant Counter replies; promoting full plant runtime into schema.

---

## 9. Document history

| Version | Notes |
|---------|--------|
| 0.1.0 | Initial thermal / hydraulic coupling sketch |
| 0.1.0+ | First executable plant slice in `homecooked-thermal` (types, registry, offer/accept, tick). Sketch text unchanged; types remain experimental / not catalog ids. |
| 0.1.0+ | Dual-path demo: thermal fridge→DHW then `dishwasher_dhw_preheat` procedure (conformance + wasm). |
| 0.1.0+ | Catalog/sim device telemetry surface: optional `thermal_port_id` / `direction` / `media` / `max_power_w` / `attached_reservoir_id` (RW) on `water_heater`, `fridge`, `hvac`, `dishwasher` (`inlet_preheat` sink), and `dryer` (`exhaust` source / air / 2000 W). Plant types remain crate-local in `homecooked-thermal`. |
| 0.1.0+ | Thin procedure⇄thermal: `thermal_wait` step + backend hooks + `wait_dhw_reservoir` fixture + conformance `procedure_thermal_wait_dhw`. Offer/negotiate-as-steps and wasm UI still deferred. |
| 0.1.0+ | Thin procedure⇄thermal: `thermal_offer` / `offer_transfer` + backend `thermal_offer`/`thermal_accept`/`thermal_negotiate` + `offer_fridge_dhw` fixture + conformance `procedure_thermal_offer_dhw`. Multi-round dialogue / soft decline / richer wasm UI still deferred. |
| 0.1.0+ | Schema thermal vocabulary (`Media` / `PortDirection` / `TempBandC` / `HeatPortSpec`) in `homecooked-schema`; plant runtime remains crate-local in `homecooked-thermal`. |
| 0.1.0+ | `ClassTable.thermal_ports` advertises static `HeatPortSpec` for the five thermal-port classes (match sim seeds); catalog `thermal_port_*` points remain the device RW surface; plant runtime still crate-local. |
| 0.1.0+ | Soft decline (`on_decline: fail|continue`) + thin `fallback_power_w` retry; plant negotiate declines when max < offer min; fixture `offer_fridge_dhw_soft` + conformance `procedure_thermal_offer_soft_decline`. Continuous re-queue / dedicated wasm UI still deferred. |
| 0.1.0+ | Continuous re-queue: `thermal_wait` + `requeue_offer` + inline transfer fields re-negotiate each poll; fixture `wait_dhw_with_requeue` + conformance `procedure_thermal_wait_requeue`. Dedicated wasm UI / plant Counter / schema plant runtime still deferred. |
