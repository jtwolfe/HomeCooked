# Universal appliance control system

Version **0.1.0** — design extension (docs sketch).

HomeCooked already defines *what* an appliance is (class + traits + points)
and *how* peers talk (discover / describe / read / write / subscribe). This
document sketches a **universal control computer** that sits *inside* the
chassis and actually drives sensors, relays, and motors: take one appliance
out, put another in, hook the harness, load an I/O map and class profile,
program the controller, done.

The concrete worked example is laundry — a washer and a dryer sharing one
control module. Channel inventories, a sample I/O map, and a `cotton` cycle
outline live in [`examples/washer-dryer-io.md`](./examples/washer-dryer-io.md).

Related:

- [`overview.md`](./overview.md) — catalog → schema → wire; capability
  checks; local interlocks never bypassed
- [`procedures.md`](./procedures.md) — procedures and recipes the cycle
  runtime executes
- [`thermal-plant.md`](./thermal-plant.md) — heat ports as optional HAL
  channels later
- [`bridges.md`](./bridges.md) — adapters for *alien* stacks; this
  controller is a **native** HomeCooked device
- [`examples/washer-dryer-io.md`](./examples/washer-dryer-io.md) — washer /
  dryer I/O and sample map
- [`../catalog/appliances.md`](../catalog/appliances.md),
  [`../catalog/variables-and-settings.md`](../catalog/variables-and-settings.md)

This is a **design sketch**. It does not freeze wire messages, schema types,
I/O map YAML, HAL channel ids, or a hardware bill of materials. No certified
part numbers are implied. Implementations may experiment under `vendor.*`
until a later catalog revision promotes stable ids.

---

## 1. Goals

- One **compute module + backplane** that can run many catalog classes
  (washer, dryer, dishwasher, …) by swapping chassis, harness, config, and
  procedure library — not by designing a new controller each time.
- A layered stack from mains isolation up to HomeCooked advertisement, with
  a **single, auditable path** from a catalog write to a relay coil.
- **Declarative interlocks** that sit between protocol writes and high-
  voltage actuators. Software on the HomeCooked session is never the sole
  barrier for door-open + spin or door-open + heater.
- Config as data: an **I/O map** and a **device profile** are what change
  when the same controller moves from washer to dryer.
- Commissioning that is boring and checklist-driven: mount, plug, load map,
  prove sensors, enable remote, appear on the bus as `washer` or `dryer`.

## 2. Non-goals

- A certified functional-safety architecture (IEC 60335, IEC 61508, and
  similar). Local hardwired interlocks still exist; this sketch does not
  replace them or claim a SIL / PL rating.
- A frozen PCB, MCU, or relay BOM. Hardware below is **directional**.
- Bit-banging a universal motor, triac angle, or inverter PWM from the
  application CPU as the preferred first-rev motor path.
- Replacing [`bridges.md`](./bridges.md): this box *is* the appliance. A
  Matter washer still needs a bridge; a HomeCooked-native controller does
  not.
- Cloud identity, OAuth, or a global device CA ([`overview.md`](./overview.md)
  §1).
- Inventing core class / trait / point ids that are absent from
  `docs/catalog/`.

---

## 3. Universal controller thesis

Product differentiation for a family of heavy whitegoods is **not** a unique
control computer per SKU. It is:

| What changes | What stays |
|--------------|------------|
| Mechanical chassis and drum / tub / cavity | Compute module |
| Wiring harness and sensor/actuator set | Backplane and board *roles* (LV sensor, HV actuator, motor IF) |
| **`chassis.io_map.yaml`** | HAL channel *names* (the map binds them) |
| **`device.profile.json`** (class, traits, ranges) | Capability / validation engine |
| **`programs/`** + procedure library | Cycle runtime and interlock engine |
| Front-panel artwork / optional HMI | HomeCooked device role (hello / describe / …) |

Same compute, same backplane, many classes. The I/O map is the swap artifact
when a washer chassis comes out and a dryer chassis goes in.

### 3.1 “Boom done” workflow

1. **Mount** the control computer on the chassis backplane.
2. **Plug** the class harness (sensors, valves, lock, motor inverter).
3. **Load** `chassis.io_map.yaml` + `device.profile.json` for the class
   (`washer` or `dryer`).
4. **Commission** sensors (door, lock feedback, NTC, level / humidity,
   tach) against the checklist in §6.4.
5. **Flash / enable** remote only after interlocks pass dry-run.
6. Device **appears on the bus** as that class: hello / describe advertise
   `class_id`, traits, and point ranges from the profile.

A client that already speaks HomeCooked does not care that the compute
module is shared. It sees a `washer` or a `dryer`.

---

## 4. Layers (bottom → top)

```
 HomeCooked clients / hub
        │  discover / describe / read / write / subscribe
        ▼
 ┌──────────────────────────────────────────┐
 │ 7. HomeCooked device role                │
 ├──────────────────────────────────────────┤
 │ 6. Cycle / procedure runtime             │
 ├──────────────────────────────────────────┤
 │ 5. Interlock engine                      │
 ├──────────────────────────────────────────┤
 │ 4. I/O map (per-chassis config)          │
 ├──────────────────────────────────────────┤
 │ 3. HAL (logical channels)                │
 ├──────────────────────────────────────────┤
 │ 2. I/O hardware (modular boards)         │
 ├──────────────────────────────────────────┤
 │ 1. Power & safety (isolation, e-stop)    │
 └──────────────────────────────────────────┘
        │
        ▼
  chassis harness → sensors / relays / motor
```

Writes flow **down**. Telemetry and faults flow **up**. A layer MUST NOT
reach around the one below it: the protocol stack cannot toggle a GPIO, and
the cycle runtime cannot energize a heater except through a HAL channel that
the interlock engine has allowed.

### 4.1 Power & safety

Mains enters through isolation (transformer / SMPS as the product requires)
with a **protective earth** and clear creepage on the HV board. An
**e-stop** or equivalent cut and the **door / lid interlocks** that prevent
spin or heat with the door open are **hardwired or on a separate safety
path** (safety relay, lock solenoid holding current, door-switch in series
with the heater and motor-enable). Application firmware on the control CPU
is **never the sole barrier** for door-open + spin or door-open + heater.

This layer also owns:

- Per-circuit fusing on HV actuator outputs
- Watchdog-driven **fail-safe de-energize** of actuators on firmware hang
  (see §7)
- Separation of SELV sensor rails from mains-referenced loads

HomeCooked `trait.safety.interlock_ok` *reports* this layer; it does not
implement it.

### 4.2 I/O hardware

Modular boards on a shared backplane. First rev is **local GPIO / analog /
UART**, not a plant fieldbus.

| Board role | Typical I/O |
|------------|-------------|
| Digital in | Door closed, lock feedback, lint present, leak, user buttons, e-stop sense |
| Analog in | NTC (tub, drum, exhaust), pressure (water level), 4–20 mA, humidity, optional thermocouple |
| HV out | Relay or SSR for valves, pump, heater, lock solenoid, gas valve (if present) |
| LV out | LED, buzzer, drum light, 24 V / 5 V solenoids that are not mains |
| Motor-drive IF | Enable + speed command to an **external VFD / inverter** (UART or discrete); tach / Hall in |
| Current sense (opt.) | Motor or heater current for stall / dry-fire detection |

Board *roles* are stable across classes. Which circuits are populated, and
what they mean, is the I/O map.

### 4.3 HAL

The hardware abstraction layer exposes **logical channels**, not GPIO
numbers. Firmware above this line never names a pin.

Channel ids are `snake_case` with a kind prefix (informative, not frozen):

| Prefix | Meaning | Examples |
|--------|---------|----------|
| `din.*` | Digital input | `din.door_closed`, `din.door_lock_fb`, `din.leak` |
| `ain.*` | Analog input | `ain.tub_temp_c`, `ain.water_level_pa`, `ain.drum_rpm` |
| `aout.*` | Actuator / HV or gated out | `aout.heater_enable`, `aout.cold_inlet`, `aout.door_lock` |
| `dout.*` | Ungated LV out | `dout.buzzer`, `dout.drum_light` |
| `motor.*` | Motor-drive interface | `motor.speed_rpm_cmd`, `motor.enable`, `motor.direction` |

A later thermal-plant attachment MAY appear as optional channels
(`thermal.port_source_enable`, …) without changing this prefix set; see
[`thermal-plant.md`](./thermal-plant.md).

The HAL is responsible for unit conversion at the edge (NTC resistance →
°C, pressure → Pa, tach period → rpm) so upper layers see catalog units
([`variables-and-settings.md`](../catalog/variables-and-settings.md)).

### 4.4 I/O map (config)

Per-chassis YAML or JSON that binds **HomeCooked points and internal
signals ↔ HAL channels**. This is the artifact you swap when the same
controller moves washer → dryer.

Informative shape:

```
IoMap {
  version:          SemVer
  class_id:         ClassId          // washer | dryer | …
  bindings:         [Binding]
}

Binding {
  channel:          HalChannel       // din.door_closed
  source_or_sink:   BoardPin         // board role + local pin / circuit
  point:            QualifiedId?     // trait.door_lid.door_state, or none if internal
  encode:           mapping?         // bool → enum tokens, scale, invert
  gated_by:         [InterlockId]?   // actuators only
}
```

A sample washer fragment is in
[`examples/washer-dryer-io.md`](./examples/washer-dryer-io.md) §5.

Rules:

- Every `aout.*` / `motor.*` that can energize a hazardous load MUST name
  an interlock id (or an explicit `ungated: true` with a documented reason,
  e.g. buzzer).
- Points in the map MUST exist in the loaded `device.profile.json` ∩
  catalog. The map MUST NOT mint core ids.
- Unused HAL channels stay in the firmware image; the map simply omits
  them. Dryer maps do not bind `aout.cold_inlet`.

### 4.5 Interlock engine

Declarative rules evaluated **continuously**, not only at `write` time.
The cycle runtime and the HomeCooked device role may *request* a channel;
the engine is the last software gate before the HAL.

Informative examples (laundry):

- `aout.heater_enable` requires `water_present` (from `ain.water_level_pa`
  or a derived `din.water_present`) **AND** `din.door_lock_fb` consistent
  with lock command (see §7 dual-path).
- `motor.speed_rpm_cmd` above a tumble band (spin) requires
  `din.door_lock_fb` **AND** the commanded rpm within the advertised
  `class.washer.spin_rpm` range **AND** water not above a spin-safe level.
- Leak / overflow (`din.leak`) forces inlet valves closed and drain pump
  policy as the class defines; heater and spin stay off.

Rejected requests surface as HomeCooked `safety_interlock` (and
`trait.safety.interlock_reason`) — same codes as
[`overview.md`](./overview.md) §9. There is no protocol “force” flag.

The engine is **not** the hardwired door/heater series path in §4.1. It is
the software twin that keeps firmware honest and gives clients an
auditable reason. Both must agree to energize.

### 4.6 Cycle / procedure runtime

A state machine that executes:

1. **Catalog named programs** (`trait.program.program` = `cotton`, …) as
   class-local cycles. Prefer these when they encode vendor-validated
   chemistry (wash temperatures, spin profiles) —
   [`procedures.md`](./procedures.md) §6.
2. **HomeCooked procedures** (ordered reads / writes / commands + guards +
   timeouts) when a hub or tool downloads them. The runtime is a procedure
   runner that happens to live *on the device* for the chassis it owns.

Every actuator command the runtime issues is a HAL request that still
passes the interlock engine. The runtime updates `trait.cycle.cycle_state`
and `trait.cycle.cycle_phase` using **catalog tokens** (`fill`, `wash`,
`rinse`, `spin`, `drain`, … for `washer`; `heating`, `drying`, `cooling`,
… for `dryer`). Finer internal sub-states (heat vs tumble inside `wash`)
are not required on the wire.

A `cotton` washer outline is in
[`examples/washer-dryer-io.md`](./examples/washer-dryer-io.md) §6.

### 4.7 HomeCooked device role

The controller **is** a HomeCooked device, not a bridge:

- Advertises `class_id` + traits + capabilities from `device.profile.json`
- Speaks discover / describe / read / write / subscribe over **IP** in
  the first rev (Ethernet or Wi-Fi). Other transports later via
  `trait.connectivity.transport`
- Validates writes with the same pipeline as
  [`overview.md`](./overview.md) §7.4 and §9.2
- Maps accepted writes onto internal signals / HAL requests
- Emits events for cycle, fault, and subscribed points

**Rule:** the HomeCooked protocol **MUST NOT** bypass the interlock
engine. A well-formed `write` of `trait.heater.heater_percent` with the
door open fails `safety_interlock` the same way a cycle step would. Bridges
([`bridges.md`](./bridges.md)) exist for appliances that already have a
foreign stack; do not wrap this controller in a bridge to talk to itself.

---

## 5. Hardware sketch (directional, not a BOM)

Nothing here is a certified part number or a purchase list. Cycles need
deterministic I/O; the protocol stack does not.

- **Control computer** — MCU-class for hard realtime (laundry fill / spin
  timing) with an **optional Linux companion** for rich UI, logging, and
  IP; **or** a single CM4-class module **if** I/O is offloaded to a
  co-MCU that owns the watchdog and actuator enable.
- **HV actuator board** — relay or SSR per load (valves, pump, heater,
  lock solenoid); **clear creepage**; **fuse per circuit**; lock and
  heater enables also gated by the safety path in §4.1 so a single failed
  load does not take the backplane with it.
- **LV sensor board** — 24 V or 5 V digital in; NTC inputs; 4–20 mA or
  pressure for water level; tach / Hall for drum rpm; leak sense. Same
  board *role* on washer and dryer.
- **Motor** — prefer an **external inverter** with speed feedback (UART
  or discrete enable + analog/digital speed). Do not bit-bang a universal
  motor from the application CPU in v1; isolation, stall, and rpm loops
  belong next to the motor.
- **Fieldbus** — not in first rev. Local GPIO + HomeCooked over Ethernet
  / Wi-Fi. Plant buses (Modbus / BACnet) stay on
  [`bridges.md`](./bridges.md) / [`thermal-plant.md`](./thermal-plant.md)
  until a later pass.

Gas dryer paths (valve + igniter + flame sense) are class-specific HV
circuits with default-deny remote ignition, matching catalog safety notes
on `dryer` — still this controller, still the same interlock engine.

---

## 6. Software config artifacts

Loaded at commission time (local file, signed bundle, or equivalent). Not
wire protocol.

### 6.1 `chassis.io_map.yaml`

Pin / channel bindings (§4.4). Swapped with the chassis. Informative
keys: `version`, `class_id`, `bindings[]` (`channel`, board pin, optional
`point`, `encode`, `gated_by`).

### 6.2 `device.profile.json`

Feeds the capability object in [`overview.md`](./overview.md) §4.3:

- `class_id` (`washer`, `dryer`, …) and optional `secondary_class_ids`
- Trait list and per-point ranges (e.g. `class.washer.spin_rpm` 0–1400)
- `SafetyFlags` (`remote_start_supported` default **false**, and the other
  catalog flags)
- `catalog_version` / `protocol_version` the firmware tracks

The profile MUST be a subset of the catalog. Omitting an optional point is
fine; advertising a point the firmware cannot drive is not.

### 6.3 `programs/`

Cycle definitions that reference **catalog program tokens**
(`cotton`, `eco`, `wool`, … on washer;
`cotton`, `synthetic`, `timed`, … on dryer) and catalog
`cycle_phase` tokens. A program file is data for the runtime in §4.6, not
a new wire type. Fine-grained recipes still use
[`procedures.md`](./procedures.md).

### 6.4 Commissioning checklist

Minimum, auditable, class-aware. Expand per chassis; do not skip.

1. Isolation and e-stop / door series path verified **with actuators
   de-energized** (hardware, not a software self-test alone).
2. I/O map and profile **class_id match** the chassis in front of you.
3. Each bound `din.*` / `ain.*`: exercise the real sensor (open/close
   door, lock/unlock, known NTC temperature, level / humidity span, tach
   while turning the drum by hand or jog).
4. Dual-path lock: command `aout.door_lock`, confirm `din.door_lock_fb`
   matches **before** any heater or spin enable is armed.
5. Interlock dry-run: request heater and spin with door open / unlocked
   and confirm both software reject (`safety_interlock`) **and** hardware
   path stays de-energized.
6. Watchdog: force a firmware stall in a service mode and confirm
   actuators drop.
7. Named program smoke: one short `cotton` (washer) or `timed` (dryer)
   with a technician present.
8. Only then set `trait.remote.remote_control_enabled` /
   `remote_start_enabled` as the product allows, and publish hello on IP.

---

## 7. Safety and high voltage

- **Dual-path for critical actuators.** Door lock **command**
  (`aout.door_lock`) and door lock **feedback** (`din.door_lock_fb`) must
  agree before spin or heat. Mismatch → `trait.safety.interlock_ok = false`,
  `interlock_reason = door` (or `other` if the catalog token is a poor
  fit), heater and motor-enable off. The hardwired series path in §4.1 is
  the second path; do not omit it because feedback exists.
- **Watchdog.** Firmware hang, HAL timeout, or co-MCU loss de-energizes
  every gated actuator. Recovery is a defined restart, not a silent
  resume of a half-run spin.
- **Protocol cannot bypass interlocks.** HomeCooked writes, procedure
  steps, and front-panel requests all enter the same engine. No `force`
  flag ([`overview.md`](./overview.md) §9.3).
- **Leak / overtemp / flame-out** are fail-safe: inlets closed, heat off,
  motor policy as the class defines, `trait.fault` + `alert_list` tokens
  from the catalog (`leak`, `overtemp`, `flame_out`, `door_lock_fail`, …).
- **Remote start** remains opt-in (`remote_start_supported` default false;
  physical enable commonly required). Commissioning §6.4 step 8 is last
  for a reason.

This document does **not** certify a product. It states how a HomeCooked-
native controller is *supposed* to be structured so audits have a place to
look.

---

## 8. Relation to existing documents

| Document | Relation |
|----------|----------|
| [`overview.md`](./overview.md) | Wire, capabilities, errors. This controller implements the **device** role. Local enforcement in overview §11 is the interlock engine + hardwired path here. |
| [`procedures.md`](./procedures.md) | The cycle / procedure runtime **executes** procedures and named programs. Validation rules in procedures §5 apply on-device, not only on a hub. |
| [`thermal-plant.md`](./thermal-plant.md) | Heat **ports** are optional later HAL channels / capability facets. A washer or heat-pump dryer stays `washer` / `dryer`; it does not become a parallel thermal class. |
| [`bridges.md`](./bridges.md) | Bridges map **alien** fabrics (Zigbee, Matter, Modbus, vendor Wi-Fi) into HomeCooked. This control system *is* HomeCooked. Do not require a bridge in front of it. |
| Catalog | Class ids, program tokens, `cycle_phase` tokens, point ranges. The I/O map and profile MUST track the catalog; they MUST NOT invent core ids. |
| [`examples/washer-dryer-io.md`](./examples/washer-dryer-io.md) | Concrete washer / dryer channels, shared vs class-specific bindings, sample map, `cotton` cycle. |

---

## 9. Document history

| Version | Notes |
|---------|--------|
| 0.1.0 | Initial universal control-system sketch; laundry as worked example |
