# Washer and dryer I/O (worked example)

Version **0.1.0** — design extension (docs sketch).

Concrete sensor and actuator inventory for running a **washer** or a
**dryer** on the universal control computer in
[`../control-system.md`](../control-system.md). Same compute module and
backplane; the **I/O map**, **device profile**, and **program library**
are what change.

Related:

- [`../control-system.md`](../control-system.md) — layers, HAL, interlocks,
  commissioning
- [`../overview.md`](../overview.md) — capability checks, writes, safety
- [`../procedures.md`](../procedures.md) — procedures the runtime executes
- [`../../catalog/appliances.md`](../../catalog/appliances.md) — `washer`,
  `dryer` classes
- [`../../catalog/variables-and-settings.md`](../../catalog/variables-and-settings.md)
  — traits and per-class points (`cotton`, `cycle_phase`, `spin_rpm`, …)

This is a **design sketch**. Channel names, YAML keys, and pin labels are
informative. They are not a frozen schema or a hardware BOM. No certified
part numbers are implied.

---

## 1. Scope

One laundry backplane, two class harnesses. Digital / analog / relay
*roles* stay; bindings in `chassis.io_map.yaml` re-label them.

HAL prefixes match the control-system sketch: `din.*`, `ain.*`, `aout.*`,
`dout.*`, `motor.*`. HomeCooked points use catalog qualified ids
(`trait.door_lid.door_state`, `class.washer.spin_rpm`, …).

Optional channels may be omitted from the map and from `describe`. Required
catalog points for the advertised class still need *some* source (sensor,
derived signal, or a honest “not implemented” by not advertising the
optional trait / point).

---

## 2. Washer I/O inventory

Class `washer`
([`appliances.md`](../../catalog/appliances.md)). Typical traits include
`door_lid`, `cycle`, `program`, `water`, `temperature`, `motor`, `heater`
(optional), `safety`, `remote`.

### 2.1 Sensors

| HAL channel (informative) | Kind | HomeCooked / internal | Notes |
|---------------------------|------|------------------------|-------|
| `din.door_closed` | DI | `trait.door_lid.door_state` | Map true → `closed`, false → `open` (ajar if a second contact exists) |
| `din.door_lock_fb` | DI | `trait.door_lid.door_lock_state` | Feedback; must match lock command before spin / heat |
| `ain.water_level_pa` | pressure | `trait.water.level_percent` | Tub level via pressure; derive `water_present` for interlocks |
| `ain.tub_temp_c` | NTC | `trait.temperature.current_c` | Wash water / tub |
| `ain.inlet_temp_c` | NTC | optional / `vendor.*` | Inlet if instrumented; not required |
| `ain.drum_rpm` | tach / Hall | `class.washer.drum_rpm`, `trait.motor.rpm` | Measured drum speed |
| `ain.motor_current_a` | current | `trait.energy.current_a` (opt.) | Stall / unbalance assist |
| `din.leak` | DI | `trait.water.leak` | Tray / overflow; catalog alert `leak` / `overflow` |
| `ain.unbalance` | accel (opt.) | `class.washer.unbalance` | May be derived from current / rpm wander instead |
| `ain.detergent_level` | analog (opt.) | `class.washer.detergent_level_percent` | Auto-dose reservoir |
| `ain.softener_level` | analog (opt.) | `class.washer.softener_level_percent` | |
| `din.user_*` / encoder | DI / count | UI only | Start, program knob; not catalog points unless advertised |

### 2.2 Actuators

| HAL channel (informative) | Kind | HomeCooked / internal | Interlock (sketch) |
|---------------------------|------|------------------------|--------------------|
| `aout.door_lock` | lock solenoid | `trait.door_lid.lock_door` / `unlock_door` | Door closed before lock; unlock only if spin-safe and water-safe |
| `aout.cold_inlet` | valve | `trait.water.inlet_valve` | Door closed; leak not active |
| `aout.hot_inlet` | valve (opt.) | same trait or mix | Same as cold; optional SKU |
| `aout.drain_pump` | pump | `trait.water.drain_pump` | Leak policy may *force* on |
| `aout.recirc` | pump (opt.) | internal / `vendor.*` | Door locked if spray would escape |
| `aout.detergent_dispense` | dispenser | dose settings / command | During `fill` / `wash` as the program defines |
| `aout.heater_enable` | heater | `trait.heater.heater_state` | water_present AND lock fb match |
| `motor.enable` + `motor.speed_rpm_cmd` | inverter IF | `trait.motor.rpm_setpoint`, `direction` | See spin / tumble notes below |
| `dout.buzzer` / `dout.led` | LV | `trait.audio`, UI | May be ungated |

Motor interlocks: tumble band requires door closed / lock as the SKU
defines. **Spin** requires lock feedback matching the lock command, rpm
command within advertised `class.washer.spin_rpm`, and water not above a
spin-safe level. `motor.direction` is an extra HAL channel on the same
inverter interface.

Catalog `class.washer.spin_rpm` is the **setpoint** the client writes;
`motor.speed_rpm_cmd` is the HAL command after the runtime and interlocks
have accepted it. `class.washer.drum_rpm` is measured.

---

## 3. Dryer I/O inventory

Class `dryer`. Typical traits include `door_lid`, `cycle`, `program`,
`temperature`, `humidity`, `heater`, `fan`, `filter`, `safety`, `remote`.

### 3.1 Sensors

| HAL channel (informative) | Kind | HomeCooked / internal | Notes |
|---------------------------|------|------------------------|-------|
| `din.door_closed` | DI | `trait.door_lid.door_state` | Same HAL name as washer |
| `din.door_lock_fb` | DI | `trait.door_lid.door_lock_state` | Same dual-path rule before heat / tumble |
| `ain.drum_temp_c` | NTC | `trait.temperature.current_c` | Drum / inlet-to-load |
| `ain.exhaust_temp_c` | NTC | zone or opt. point | Overtemp cutout still device-local |
| `ain.moisture` / `ain.humidity_rh` | humidity | `trait.humidity.current_rh`, `class.dryer.dryness_percent` | Sensor-dry programs |
| `din.lint_present` | DI | `class.dryer.lint_filter` | `ok` / `missing` / `clogged` (clog may be ΔP or a second DI) |
| `ain.drum_rpm` | tach / Hall | `trait.motor.rpm` | Tumble proof; no-tumble → fault |
| `ain.flame_c` | thermocouple (gas, opt.) | `trait.heater.flame` | Gas SKU only; ignition path default-deny remote |
| `din.user_*` / encoder | DI / count | UI only | |

Condenser drain-tank full (`class.dryer.drain_tank`) is an extra DI or
level input on condenser SKUs.

### 3.2 Actuators

| HAL channel (informative) | Kind | HomeCooked / internal | Interlock (sketch) |
|---------------------------|------|------------------------|--------------------|
| `aout.door_lock` | lock solenoid | `trait.door_lid.lock_door` | Door closed |
| `motor.enable` + `motor.speed_rpm_cmd` | drum motor / inverter | `trait.motor.*` | Door lock fb match for run |
| `aout.heater_enable` | electric heater | `trait.heater.heater_state` | Lock fb match; overtemp clear; blower as required |
| `aout.gas_valve` + igniter path | gas (opt.) | `trait.heater.flame` | Same as heater plus flame-sense window; remote ignite default deny |
| `aout.blower` / `motor.fan_*` | fan | `trait.fan.fan_state` | Often required before heater |
| `dout.drum_light` | LV | `trait.lighting.light_on` | Door or UI; ungated OK |
| `dout.buzzer` | LV | `trait.audio` | Ungated OK |

`class.dryer.lint_filter` is required on the class when `dryer` is primary.
Do not start a heat phase with `missing` / `clogged` if the profile says so
— `safety_interlock` or `busy` as the device policy documents.

---

## 4. Shared vs class-specific channels

When the same controller is moved washer → dryer (or the reverse), keep
the compute module and board *roles*. Rewire the harness and **load a
different I/O map + profile + programs**.

| Channel | Shared? | Washer | Dryer |
|---------|---------|--------|-------|
| `din.door_closed` | yes | door | door |
| `din.door_lock_fb` | yes | lock fb | lock fb |
| `aout.door_lock` | yes | lock solenoid | lock solenoid |
| `ain.drum_rpm` | yes | drum tach | drum tach |
| `motor.*` | yes (same IF) | wash / spin inverter | tumble inverter |
| `dout.buzzer`, UI DIs | yes | panel | panel |
| `ain.tub_temp_c` | washer | tub NTC | unused (map omits) |
| `ain.water_level_pa` | washer | level | unused |
| `din.leak` | washer (typical) | leak tray | unused or condenser leak |
| `aout.cold_inlet`, `aout.hot_inlet` | washer | valves | unused |
| `aout.drain_pump` | washer | drain | condenser pump on some SKUs — **rebind**, do not assume |
| `aout.detergent_dispense` | washer | dispenser | unused |
| `aout.heater_enable` | **same HAL name, different load** | tub heater | air heater or gas path |
| `ain.drum_temp_c`, `ain.exhaust_temp_c` | dryer | unused | NTCs |
| `ain.humidity_rh` | dryer | unused | moisture |
| `din.lint_present` | dryer | unused | lint |
| `aout.blower` | dryer | unused | fan |
| `dout.drum_light` | dryer (typical) | rare | light |

`aout.heater_enable` is the important collision: the **channel id stays**,
the **physical circuit and interlock rule change** with the map (washer:
water_present; dryer: airflow / overtemp). Do not load a washer map on a
dryer chassis.

---

## 5. Sample I/O map (washer)

Informative fragment — **not** a frozen schema. Five to ten bindings to
show the shape; a real chassis map lists every connected channel.

```yaml
# INFORMATIVE EXAMPLE — not a frozen schema
# chassis.io_map.yaml  — washer on the laundry backplane
version: "0.1.0"
class_id: washer
bindings:
  - channel: din.door_closed
    source: { board: lv_sensor, pin: di_0, active: high }
    point: trait.door_lid.door_state
    encode: { true: closed, false: open }

  - channel: din.door_lock_fb
    source: { board: lv_sensor, pin: di_1, active: high }
    point: trait.door_lid.door_lock_state
    encode: { true: locked, false: unlocked }

  - channel: ain.water_level_pa
    source: { board: lv_sensor, pin: pressure_0, unit: pascal }
    point: trait.water.level_percent
    # runtime derives water_present for interlocks from this span
    scale: { zero: 0, full: 4000, to: percent }

  - channel: ain.tub_temp_c
    source: { board: lv_sensor, pin: ntc_0 }
    point: trait.temperature.current_c

  - channel: ain.drum_rpm
    source: { board: lv_sensor, pin: tach_0 }
    point: class.washer.drum_rpm

  - channel: din.leak
    source: { board: lv_sensor, pin: di_2, active: high }
    point: trait.water.leak

  - channel: aout.door_lock
    sink: { board: hv_actuator, circuit: lock_solenoid }
    point: trait.door_lid.door_lock_state

  - channel: aout.cold_inlet
    sink: { board: hv_actuator, circuit: cold_inlet }
    gated_by: [il.fill]
    point: trait.water.inlet_valve

  - channel: aout.heater_enable
    sink: { board: hv_actuator, circuit: heater }
    gated_by: [il.heater]
    # il.heater: water_present AND din.door_lock_fb matches aout.door_lock

  - channel: motor.speed_rpm_cmd
    sink: { board: motor_if, proto: inverter_uart }
    gated_by: [il.motor]
    point: trait.motor.rpm_setpoint
    # il.motor spin band: lock fb match AND rpm within advertised
    # class.washer.spin_rpm AND water at or below spin-safe level
```

`board` / `pin` / `circuit` values are **logical names** on the backplane,
not vendor SKUs.

A dryer map would keep `din.door_closed`, `din.door_lock_fb`,
`aout.door_lock`, `ain.drum_rpm`, and `motor.speed_rpm_cmd`; drop water
and inlet bindings; add `ain.drum_temp_c`, `ain.humidity_rh`,
`din.lint_present`, `aout.blower`; retarget `il.heater` to dryer rules.

---

## 6. Minimal cycle outline — washer `cotton`

Catalog program token: `trait.program.program` = `cotton`
([`variables-and-settings.md`](../../catalog/variables-and-settings.md)
class `washer`). Client also writes `class.washer.wash_temp_c` and
`class.washer.spin_rpm` (and optional soil / extra rinse) **before**
`trait.cycle.start` — same order as [`overview.md`](../overview.md) §12.

Advertised `trait.cycle.cycle_phase` tokens stay in the catalog set:
`fill` `wash` `drain` `rinse` `spin` `complete` (plus `prewash` / `soak`
if the program uses them). **Heat** and **tumble** are internal sub-states
of `wash`; they are not extra wire tokens.

Every line that names an `aout.*` or `motor.*` is a HAL request. The
interlock engine may refuse it; the runtime then faults or waits, it does
not poke GPIO.

| Internal state | `cycle_phase` | HAL (sketch) | Guards |
|----------------|---------------|--------------|--------|
| idle | — (`cycle_state=idle`) | none | Door closed to accept `start` if required; remote flag for remote start |
| lock | `fill` (or idle until locked) | `aout.door_lock` | `din.door_closed`; wait for `din.door_lock_fb` |
| fill | `fill` | `aout.cold_inlet` until level ≥ target | Leak not active; lock fb matched |
| heat | `wash` | `aout.heater_enable` until tub ≥ `wash_temp_c` | **il.heater**: water_present AND lock fb |
| tumble | `wash` | `motor.speed_rpm_cmd` tumble band; reverse on a timer | Door locked; rpm_cmd in tumble envelope |
| drain | `drain` | heater off; `aout.drain_pump` until empty | — |
| rinse | `rinse` | fill cold → short tumble → drain | Same fill / motor interlocks; extra rinse if set |
| spin | `spin` | `motor.speed_rpm_cmd` → `class.washer.spin_rpm` | **il.motor spin**: lock fb, rpm in range, water spin-safe |
| complete | `complete` | motor off; unlock when spin-safe; buzzer | — |

Unbalance may limit spin rpm (device policy, not a protocol error —
[`appliances.md`](../../catalog/appliances.md) washer notes). Skip heat
when `wash_temp_c` is cold / 0. Repeat rinse if `class.washer.extra_rinse`.

`trait.cycle.cycle_state` moves `idle` → `running` → `complete` (or
`error`). `cancel` drains / unlocks as device policy and still cannot skip
interlocks.

This outline is a **named program** on the device. A hub procedure
([`procedures.md`](../procedures.md)) may instead write `program=cotton`,
setpoints, and `start`, then wait on `cycle_state` — it should not
micro-step valves over IP unless the device advertised those points as
writable, which v1 washers typically do not.

---

## 7. Document history

| Version | Notes |
|---------|--------|
| 0.1.0 | Initial washer / dryer I/O inventory, sample map, `cotton` outline |
