# Procedures — fine-grained dynamic control

Version **0.1.0** — design extension (docs sketch).

HomeCooked already exposes **named programs** on many appliances
(`trait.program`). Homes also need **timed and conditional sequences** that
span one or many devices, and eventually **AI-generated protocols** that
compose catalog points safely. This document defines **procedures** as that
layer.

Related:

- [`overview.md`](./overview.md) — capability checks, writes, safety
- [`thermal-plant.md`](./thermal-plant.md) — optional heat-transfer steps
- [`bridges.md`](./bridges.md) — devices reached via automation bridges

**Recipes are first-class as procedures.** Earlier overview text treated
"recipe documents" as a non-goal; that blanket ban is revised. A recipe is a
procedure (or a package of procedures) whose steps are ordinary HomeCooked
reads, writes, and commands plus guards and timeouts. Pixel cooking UIs and
camera streams remain out of scope for the core standard.

---

## 1. Three layers of control

| Layer | What it is | Who authors it | Bound to |
|-------|------------|----------------|----------|
| Named program | Vendor/SKU cycle token (`eco`, `fan_bake`, …) | Firmware / catalog enum | Single device |
| Procedure | Ordered steps of HomeCooked ops + guards + timeouts | User, integrator, or tool | One or many devices |
| AI-generated protocol | Procedure (or procedure graph) proposed by a model | Model + validator | Same as procedure; **must** validate before run |

Named programs stay on the device. Procedures live in a client, hub, or
procedure runner that speaks HomeCooked. AI protocols are not a separate wire
type: they compile to procedures and face the same capability gates.

---

## 2. Procedure model

A **procedure** is an ordered list of **steps**. Each step is one of:

- **read** — sample points (for guards or logging)
- **write** — settings
- **command** — actions (`trait.cycle.start`, …)
- **wait** — duration and/or condition
- **guard** — assert a condition; fail or branch if false
- **parallel** (optional) — run a set of steps concurrently across devices

Informative shape:

```
Procedure {
  id:            Id
  title:         string
  version:       SemVer
  devices:       [DeviceRef]          // class_id hints + optional device_id
  params:        [Param]              // e.g. slice_count, cheese_on_top
  steps:         [Step]
  on_fail:       abort | continue | goto
}

Step {
  id:            Id
  op:            read | write | command | wait | guard | parallel
  target:        DeviceRef | none
  point:         QualifiedId?         // for read/write/command
  value:         Value?
  timeout_s:     u32?
  guard:         Expr?                // e.g. probe_c >= 70 || elapsed_s >= 90
}
```

Expression language for guards is intentionally small in this sketch
(comparisons on recently read points, elapsed time, door state). A later
revision may freeze a grammar; until then runners SHOULD stick to obvious
boolean combinations of HomeCooked values.

---

## 3. Multi-device orchestration

Procedures may name several devices, for example:

- Pre-heat oven while microwave thaws
- Open/monitor `fridge` door state only long enough to fetch cheese
- Stagger hob zones with range-hood boost

The runner is responsible for **session management** (discover / describe /
subscribe) per device. A procedure that cannot bind a required device at
start fails closed unless marked optional.

Example device roles (informative): `heating` → `oven` or `air_fryer`;
`reheat_fast` → `microwave`; `cold_store` → `fridge`.

---

## 4. Example — Domino's supreme slices + cold grated cheese

**Scenario (informative):** Reheat two Domino's supreme pizza slices and finish
with cold grated cheese on top. Prefer microwave then oven/air-fryer crisp;
use probe or time guards; pull cheese from the fridge only at the end so it
stays cold.

```yaml
# INFORMATIVE EXAMPLE — not a frozen schema
id: reheat_dominos_supreme_cheese_top
title: "Reheat 2 Domino's supreme slices, cold grated cheese on top"
version: "0.1.0"
params:
  - { name: slice_count, type: u8, default: 2 }
  - { name: cheese_ready, type: bool, default: true }
devices:
  - { role: microwave, class_id: microwave }
  - { role: crisp, class_id: [oven, air_fryer] }   # runner picks one capable device
  - { role: fridge, class_id: fridge, optional: true }
steps:
  - id: check_caps
    op: guard
    guard: "microwave supports trait.cycle and trait.temperature"

  - id: mw_power
    op: write
    target: microwave
    point: class.microwave.power_percent
    value: 70
    timeout_s: 5

  - id: mw_start
    op: command
    target: microwave
    point: trait.cycle.start
    timeout_s: 5

  - id: mw_wait
    op: wait
    target: microwave
    timeout_s: 90
    guard: "elapsed_s >= 45 || trait.temperature.probe_c >= 55"

  - id: mw_stop
    op: command
    target: microwave
    point: trait.cycle.stop
    timeout_s: 5

  - id: crisp_set
    op: write
    target: crisp
    point: trait.temperature.setpoint_c
    value: 200
    timeout_s: 5

  - id: crisp_start
    op: command
    target: crisp
    point: trait.cycle.start
    timeout_s: 5

  - id: crisp_wait
    op: wait
    target: crisp
    timeout_s: 360
    guard: "elapsed_s >= 180 || trait.temperature.probe_c >= 70"

  - id: crisp_stop
    op: command
    target: crisp
    point: trait.cycle.stop
    timeout_s: 5

  - id: fetch_cheese
    op: guard
    target: fridge
    guard: "params.cheese_ready == true"
    # Human or robot fetches grated cheese; procedure only checks door briefly
  - id: fridge_door_watch
    op: wait
    target: fridge
    timeout_s: 30
    guard: "trait.door_lid.door_state == closed || elapsed_s >= 20"

  - id: done
    op: guard
    guard: "true"   # plating: cheese on hot slices — outside appliance scope
```

Notes:

- Point ids above are illustrative; runners must `describe` and map to the
  actual advertised set (power may be watts or percent; probe may be absent).
- If `air_fryer` is chosen for `crisp`, setpoint and time guards still apply
  via the same trait ids when advertised.
- Cheese handling is mostly human; the fridge step only avoids leaving the
  door open.

---

## 5. Validation rules

Every write and command in a procedure **MUST** pass the same capability
pipeline as an interactive client ([`overview.md`](./overview.md)):

1. Point is advertised on that device.
2. Type / enum / range checks succeed.
3. Safety flags respected (`remote_start`, RF, gas ignite, pressurized vent).
4. Device-local interlocks still apply at execution time (`safety_interlock`,
   `remote_disabled`, …).

Reject **before run** when static checks fail (unknown point, hard
out-of-range constant, missing required device). Reject **at step** when
dynamic state makes the op unsafe; do not clamp unless the point advertised
`clamp: true`.

AI-generated protocols:

- MUST be validated as procedures before enqueue
- MUST NOT introduce vendor points the runner is not allowed to use
- SHOULD surface a human-readable diff of points touched

---

## 6. Relation to named programs

Bundled oven example `oven_bake_180` writes `trait.program.program = bake` then a cavity setpoint before `start`. Bundled coffee example `coffee_brew_espresso` powers on, selects `espresso`, then waits on `class.coffee_machine.boiler_c`.

A procedure step may **select** a named program (`trait.program.program = eco`)
and then `start`, or it may drive fine-grained setpoints when the device
allows. Prefer named programs when they encode vendor-validated chemistry
(wash temperatures, spin profiles). Use fine-grained writes when the catalog
exposes them and the recipe needs them (pizza crisp finish).

---

## 7. Out of scope (this sketch)

- Pixel-perfect cooking UI, step photos, or camera doneness models
- Guaranteed food-safety certification of community recipes
- Cloud recipe marketplace identity (may appear later as a profile)

---

## 8. Document history

| Version | Notes |
|---------|--------|
| 0.1.0 | Initial procedures sketch; recipes as procedures |
| 0.1.1 | Bundled `oven_bake_180` example (program + setpoint + sim heat wait) |
| 0.1.2 | Bundled `coffee_brew_espresso` example (program espresso + sim boiler heat wait) |
