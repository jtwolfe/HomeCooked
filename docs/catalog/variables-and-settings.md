# Variables, Settings, and Commands

Source of truth for HomeCooked **traits**, **variables**, **settings**, and
**commands**. Schema types, capability checks, and the simulator must track this
file together with [`appliances.md`](./appliances.md).

Catalog version: **0.1.0** (docs-only; no schema crate in this revision).

---

## How to read this catalog

### Identity of a point

Every point has a stable `snake_case` **id**. On the wire, ids are qualified:

| Kind | Qualified form | Example |
|------|----------------|---------|
| Trait variable | `trait.<trait_id>.<id>` | `trait.temperature.current_c` |
| Trait command | `trait.<trait_id>.<id>` | `trait.cycle.start` |
| Class variable | `class.<class_id>.<id>` | `class.washer.spin_rpm` |
| Class command | `class.<class_id>.<id>` | `class.multi_cooker.vent` |
| Vendor | `vendor.<vendor_id>.<id>` | `vendor.acme.steam_pulse` |

Unqualified ids are unique **within** a trait or class. They are not globally
unique (`current_c` exists on several traits/zones). Devices and clients always
use the qualified form, plus an optional `zone` id when the point is zoned.

### Columns

Each point is specified as:

| Field | Meaning |
|-------|---------|
| `id` | Unqualified `snake_case` token |
| type | `bool`, `u8`, `u16`, `u32`, `i16`, `i32`, `f32`, `enum`, `string`, `timestamp_ms`, `duration_s`, `percent`, `list<T>` |
| unit | SI or catalog unit, or `—` |
| range / enum | Inclusive numeric range, enum tokens, or string constraints |
| access | `r` read, `w` write, `e` event. Combine as `r/e`, `r/w`, `r/w/e`, `w` |
| req | `req` = required if the trait/class is advertised; `opt` = optional |
| description | Short semantics |

**Settings** are points with `w` (usually also `r`). **Telemetry** is `r` and
usually `e`. **Commands** are `w` (sometimes `e` for completion). Commands use
type `command` with an argument type in the range column (`void`, `duration_s`,
a struct of fields, etc.). A `command` write is an action, not a stored
setpoint; success means the action was accepted, not that the cycle finished.

### Types

| type | Notes |
|------|-------|
| `bool` | JSON / schema boolean |
| `u8` `u16` `u32` `i16` `i32` | Integers; ranges are inclusive |
| `f32` | IEEE-754; devices advertise resolution (e.g. 0.1 °C) |
| `percent` | `f32` 0–100 unless a tighter range is given |
| `enum` | Closed token set in this catalog version. Unknown tokens must be preserved, not rejected, on read; writes of unknown tokens are `invalid_enum` |
| `string` | UTF-8; max length in range column |
| `timestamp_ms` | Unix time in milliseconds, UTC |
| `duration_s` | Unsigned seconds (`u32`) unless noted |
| `command` | Write-only action; argument listed under range |
| `list<T>` | Ordered list; max length in range |

### Units

Use these unit tokens (never invent aliases in core catalog):

| unit | Meaning |
|------|---------|
| `celsius` | Temperature |
| `percent` | 0–100 (or noted) |
| `second` | Time |
| `watt` | Instantaneous power |
| `watt_hour` | Energy |
| `volt` | Voltage |
| `ampere` | Current |
| `rpm` | Revolutions per minute |
| `liter` | Volume |
| `milliliter` | Volume |
| `liter_per_min` | Flow |
| `gram` | Mass |
| `kilogram` | Mass |
| `pascal` | Pressure (prefer kPa in descriptions, store Pa or kPa as noted) |
| `kilopascal` | Pressure |
| `bar` | Pressure (boilers) |
| `ppm` | Concentration (TDS, hardness as CaCO₃) |
| `gpg` | Grains per gallon (softener hardness US) |
| `rh_percent` | Relative humidity |
| `dBm` | RF signal |
| `dB` | Sound |
| `lux` | Illuminance |
| `degree` | Angle (louver) |
| `hertz` | Frequency |

Temperatures on the wire are **celsius**. UI conversion to Fahrenheit is a
client concern. Devices may advertise `display_unit` as a preference.

### Access and events

- `r` — included in `read` and in `describe`
- `w` — accepted by `write`; rejected with `not_writable` otherwise
- `e` — may be pushed as `event` after `subscribe`
- Commands (`w` only) may still emit related state events (`cycle_state`)

Reads of unknown ids → `unknown_variable`. Writes outside range →
`out_of_range`. Writes of unsupported optional points → `unsupported_capability`.

### Zones

Points marked **zoned** are repeated per zone id (`fridge`, `freezer`, `hob_1`,
`upper`, …). The zone id is a `snake_case` string advertised in
`trait.zone.zones`. Qualification: `trait.temperature.current_c#freezer`.

### Required vs optional

Advertising a trait **requires** every `req` point in that trait. Optional
points may be omitted from `describe`. Class-specific sections list extras that
are not in shared traits.

---

## Shared traits

Trait ids are `snake_case` and appear on the wire as `trait.<id>`.

Default trait versions: **1.0.0** unless noted.

---

### Trait `identity`

Who the device is. Advertised by every device.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `device_id` | string | — | 1–128 chars, `[a-zA-Z0-9._:-]+` | r | req | Stable device id on this fabric |
| `manufacturer` | string | — | 1–64 chars | r | req | Vendor display name |
| `model` | string | — | 1–64 chars | r | req | Model string |
| `serial` | string | — | 0–64 chars | r | opt | Serial number |
| `hw_version` | string | — | 0–32 chars | r | opt | Hardware revision |
| `fw_version` | string | — | 1–32 chars | r | req | Firmware version (free-form; semver recommended) |
| `class_id` | enum | — | catalog class ids | r | req | Primary class |
| `secondary_class_ids` | list<enum> | — | catalog class ids, max 8 | r | opt | Extra classes this endpoint implements |
| `catalog_version` | string | — | semver | r | req | Catalog version the firmware was built against |
| `protocol_version` | string | — | semver | r | req | Wire protocol version |
| `display_name` | string | — | 0–64 chars | r/w | opt | User label |
| `room` | string | — | 0–64 chars | r/w | opt | User room label |

No commands.

---

### Trait `power`

Mains / logical power. Not the same as a cycle start.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `power_state` | enum | — | `off` `standby` `on` `fault` | r/e | req | Logical power |
| `power_source` | enum | — | `mains` `battery` `mains_battery` `unknown` | r | opt | Supply |
| `battery_percent` | percent | percent | 0–100 | r/e | opt | If battery backed |
| `auto_off_s` | duration_s | second | 0–86400; 0 = never | r/w | opt | Inactivity auto-off |

**Commands**

| id | type | arg | access | req | description |
|----|------|-----|--------|-----|-------------|
| `power_on` | command | void | w | req | Enter `on` / ready (not start a cycle) |
| `power_off` | command | void | w | req | Enter `off` if the device allows remote off |
| `power_standby` | command | void | w | opt | Enter `standby` |

Writes that would cut a safety-critical actuator (boiler flame, pressurized
cooker) may return `safety_interlock`.

---

### Trait `connectivity`

Network and pairing. Optional on purely local-bus devices, required on IP/BLE
appliances in this standard.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `link_state` | enum | — | `offline` `connecting` `online` `degraded` | r/e | req | Fabric link |
| `transport` | enum | — | `ip` `ble` `thread` `zigbee` `matter` `uart` `unknown` | r | req | Primary transport |
| `rssi_dbm` | i16 | dBm | −120–0 | r/e | opt | RF RSSI |
| `ip_address` | string | — | dotted / IPv6 textual | r | opt | |
| `mac_address` | string | — | 17 chars | r | opt | |
| `pair_state` | enum | — | `unpaired` `pairing` `paired` | r/e | opt | |
| `cloud_state` | enum | — | `disabled` `disconnected` `connected` | r/e | opt | Vendor cloud, if any |

**Commands**

| id | type | arg | access | req | description |
|----|------|-----|--------|-----|-------------|
| `identify` | command | `duration_s` 1–60 | w | opt | Flash lights / beep to find the device |
| `reprovision` | command | void | w | opt | Enter pairing; usually local-only |

---

### Trait `time_schedule`

Clock and delayed / calendar starts.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `clock_ms` | timestamp_ms | — | unix ms | r/w | opt | Device clock; write sets time if no NTP |
| `timezone` | string | — | IANA tz, 0–64 | r/w | opt | |
| `delay_start_s` | duration_s | second | 0–86400 | r/w/e | opt | Seconds until a pending start; 0 = none |
| `delay_end_ms` | timestamp_ms | — | unix ms | r/e | opt | Absolute time of delayed start |
| `schedule_enabled` | bool | — | | r/w | opt | Honor stored schedules |

Schedule entries themselves are an optional list (vendor or later catalog
revision). v1 exposes delay start only as a first-class point.

---

### Trait `door_lid`

Doors, lids, drawers, carriages that gate safety.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `door_state` | enum | — | `open` `closed` `ajar` `unknown` | r/e | req | **zoned** if multiple doors |
| `door_lock_state` | enum | — | `unlocked` `locked` `locking` `unlocking` `fault` | r/e | opt | |
| `door_open_s` | duration_s | second | 0–86400 | r/e | opt | Time currently open; 0 if closed |
| `door_alarm` | bool | — | | r/e | opt | Open too long |
| `door_alarm_enable` | bool | — | | r/w | opt | |
| `door_alarm_s` | duration_s | second | 5–3600 | r/w | opt | Threshold |

**Commands**

| id | type | arg | access | req | description |
|----|------|-----|--------|-----|-------------|
| `lock_door` | command | void | w | opt | Lock if closed |
| `unlock_door` | command | void | w | opt | Unlock if safe (no pressure, no RF, no high water) |

---

### Trait `child_lock`

UI / local control lock, distinct from `door_lock_state`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `child_lock` | bool | — | | r/w/e | req | True = local buttons ignored (remote may still work) |

---

### Trait `lighting`

Interior or work lights.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `light_on` | bool | — | | r/w/e | req | **zoned** if multiple |
| `light_percent` | percent | percent | 0–100 | r/w | opt | Dimmer |
| `light_auto` | bool | — | | r/w | opt | Door-triggered |

---

### Trait `audio`

End-of-cycle and UI sounds.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `sound_enable` | bool | — | | r/w | req | |
| `volume_percent` | percent | percent | 0–100 | r/w | opt | |
| `end_signal` | enum | — | `off` `chime` `repeat` | r/w | opt | |

---

### Trait `temperature`

Thermal telemetry and setpoints. **Usually zoned.**

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `current_c` | f32 | celsius | −40–500 | r/e | req | Measured temperature |
| `setpoint_c` | f32 | celsius | device-advertised | r/w/e | opt | Target; required on closed-loop appliances |
| `setpoint_min_c` | f32 | celsius | | r | opt | Advertised min (also in capabilities) |
| `setpoint_max_c` | f32 | celsius | | r | opt | Advertised max |
| `resolution_c` | f32 | celsius | 0.01–1.0 | r | opt | Default 0.5 or 1 |
| `display_unit` | enum | — | `celsius` `fahrenheit` | r/w | opt | UI only; wire remains °C |
| `probe_c` | f32 | celsius | −40–300 | r/e | opt | Food / meat probe; extra probes as zones |
| `probe_target_c` | f32 | celsius | 0–100 | r/w | opt | |
| `probe_connected` | bool | — | | r/e | opt | |
| `preheat_complete` | bool | — | | r/e | opt | Cavity at setpoint |
| `super_mode` | bool | — | | r/w/e | opt | Super-cool / super-freeze / boost pull-down |
| `heater_active` | bool | — | | r/e | opt | Duplicate of heater trait if only a bit is needed |

Typical advertised setpoint ranges (override in capabilities):

| Context | Typical `setpoint_c` |
|---------|----------------------|
| Fridge zone | 1–7 |
| Freezer zone | −24–−12 |
| Wine | 5–20 |
| Oven bake | 50–250 |
| Broil | often enum `low`/`high` instead of °C |
| Kettle | 40–100 |
| Sous-vide | 20–95 |
| Water heater | 40–70 |
| HVAC heat | 10–30 |
| HVAC cool | 16–32 |
| Dehydrator | 30–75 |
| Warming drawer | 40–90 |
| Pizza stone | 200–450 |
| Air fryer | 80–200 |
| Slow cooker | named `low`/`high` *or* 70–95 |

---

### Trait `humidity`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `current_rh` | percent | rh_percent | 0–100 | r/e | req | |
| `setpoint_rh` | percent | rh_percent | 0–100 | r/w/e | opt | |
| `dew_point_c` | f32 | celsius | | r | opt | |

---

### Trait `cycle`

Run-state machine for anything that starts, pauses, and completes.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `cycle_state` | enum | — | `idle` `delayed` `running` `paused` `complete` `canceling` `error` | r/e | req | Coarse state |
| `cycle_phase` | string | — | 1–32 `snake_case` | r/e | opt | Fine phase (`rinse`, `spin`, `pressurizing`, …) |
| `progress_percent` | percent | percent | 0–100 | r/e | opt | |
| `remaining_s` | duration_s | second | 0–86400 | r/e | opt | 0 if unknown / idle |
| `elapsed_s` | duration_s | second | 0–86400 | r/e | opt | |
| `cycle_id` | u32 | — | | r/e | opt | Monotonic id of current/last cycle |
| `end_ms` | timestamp_ms | — | | r/e | opt | Estimated completion |

**Commands**

| id | type | arg | access | req | description |
|----|------|-----|--------|-----|-------------|
| `start` | command | void | w | req | Start selected program / setpoint |
| `pause` | command | void | w | opt | |
| `resume` | command | void | w | opt | |
| `cancel` | command | void | w | req | Abort; drain / cool as device policy |

`start` with door open, empty water, or remote-start disabled →
`safety_interlock` or `remote_disabled`. `pause` unsupported →
`unsupported_operation`.

---

### Trait `program`

Selectable named programs and options. Complements `cycle`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `program` | enum | — | class-specific; see per-class | r/w/e | req | Selected program |
| `available_programs` | list<enum> | — | max 64 | r | req | What this firmware offers |
| `option_flags` | list<enum> | — | max 32; class-specific tokens | r/w | opt | Extra rinse, steam, … |
| `custom_name` | string | — | 0–32 | r/w | opt | User-named custom program |

Writing `program` does not start the cycle. `start` does.

---

### Trait `fault`

Alerts and faults.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `fault_present` | bool | — | | r/e | req | Any active fault |
| `fault_code` | string | — | 0–32 | r/e | opt | Vendor or catalog code (`drain_fail`) |
| `fault_severity` | enum | — | `info` `warning` `error` `critical` | r/e | opt | |
| `fault_message` | string | — | 0–128 | r | opt | Human text; not a stable id |
| `alert_list` | list<string> | — | max 16 tokens | r/e | opt | Active alert ids |
| `last_fault_ms` | timestamp_ms | — | | r | opt | |

Catalog **alert tokens** (use in `alert_list` / `fault_code` when they fit):

`leak`, `overflow`, `dry`, `overtemp`, `undertemp`, `overpressure`,
`underpressure`, `unbalance`, `jam`, `stall`, `drain_fail`, `fill_fail`,
`door_open`, `door_lock_fail`, `filter_clogged`, `tank_full`, `tank_empty`,
`scale`, `salt_empty`, `flame_out`, `ignition_fail`, `sensor_fail`,
`comms_fail`, `power_fail`, `overcurrent`, `pan_missing`, `lid_open`,
`child_lock`, `service_required`, `ota_failed`.

**Commands**

| id | type | arg | access | req | description |
|----|------|-----|--------|-----|-------------|
| `ack_fault` | command | void | w | opt | Clear latching user-resettable faults |
| `mute_alert` | command | void | w | opt | Silence buzzer; does not clear the fault |

---

### Trait `energy`

Electrical metering.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `power_w` | f32 | watt | 0–50000 | r/e | opt | Instantaneous |
| `energy_wh` | f32 | watt_hour | 0+ | r/e | opt | Lifetime |
| `cycle_energy_wh` | f32 | watt_hour | 0+ | r/e | opt | Current/last cycle |
| `voltage_v` | f32 | volt | 0–500 | r | opt | |
| `current_a` | f32 | ampere | 0–200 | r | opt | |
| `energy_mode` | enum | — | `normal` `eco` `off_peak` | r/w | opt | Hint; device maps to internals |

---

### Trait `water`

Supply, drain, tanks, hardness.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `inlet_present` | bool | — | | r/e | opt | Supply pressure / valve ok |
| `inlet_valve` | enum | — | `closed` `open` `fault` | r/e | opt | |
| `drain_pump` | enum | — | `off` `on` `fault` | r/e | opt | |
| `level_percent` | percent | percent | 0–100 | r/e | opt | Tub / tank |
| `flow_l_min` | f32 | liter_per_min | 0–50 | r/e | opt | |
| `used_l` | f32 | liter | 0+ | r/e | opt | Lifetime |
| `cycle_used_l` | f32 | liter | 0+ | r/e | opt | |
| `hardness_ppm` | u16 | ppm | 0–1000 | r/w | opt | As CaCO₃; writable if no sensor |
| `hardness_gpg` | f32 | gpg | 0–50 | r/w | opt | US grains; prefer ppm on the wire |
| `leak` | bool | — | | r/e | opt | |
| `tank_state` | enum | — | `ok` `low` `empty` `full` `missing` | r/e | opt | Removable tanks |
| `tds_ppm` | u16 | ppm | 0–2000 | r/e | opt | RO / filter |

---

### Trait `filter`

Consumable filters.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `filter_state` | enum | — | `ok` `low` `replace` `missing` `clogged` | r/e | req | **zoned** per stage (`water`, `grease`, `hepa`, `lint`) |
| `life_percent` | percent | percent | 0–100 | r/e | opt | 0 = replace |
| `life_s` | duration_s | second | | r | opt | Estimated remaining |
| `stage_id` | string | — | 1–32 | r | opt | |

**Commands**

| id | type | arg | access | req | description |
|----|------|-----|--------|-----|-------------|
| `reset_filter` | command | optional `stage_id` | w | req | User replaced the filter |

---

### Trait `remote`

Remote start / control gating.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `remote_control_enabled` | bool | — | | r/w/e | req | Allow writes other than this flag |
| `remote_start_enabled` | bool | — | | r/w/e | opt | Allow `cycle.start` from remote |
| `local_only` | bool | — | | r/e | opt | Device forced local (service, demo) |

If `remote_control_enabled` is false, all writes except this trait return
`remote_disabled`. Many appliances require a physical “remote start” button
that sets `remote_start_enabled` until the next cycle.

---

### Trait `maintenance`

Cleaning and service counters.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `needs_clean` | bool | — | | r/e | opt | |
| `needs_descale` | bool | — | | r/e | opt | |
| `cycle_count` | u32 | — | | r | opt | Lifetime cycles |
| `last_clean_ms` | timestamp_ms | — | | r | opt | |
| `service_due` | bool | — | | r/e | opt | |

**Commands**

| id | type | arg | access | req | description |
|----|------|-----|--------|-----|-------------|
| `start_clean` | command | void | w | opt | Drum clean / descale / self-clean as class defines |
| `ack_clean` | command | void | w | opt | User ran a manual clean |

---

### Trait `safety`

Interlocks that reject actuator writes.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `interlock_ok` | bool | — | | r/e | req | False → actuator writes fail |
| `interlock_reason` | enum | — | `none` `door` `lid` `pan` `water` `pressure` `tilt` `leak` `overtemp` `child_lock` `remote` `other` | r/e | opt | |
| `hot_surface` | bool | — | | r/e | opt | Residual heat (hobs) |
| `tilt` | bool | — | | r/e | opt | Tip sensor |

Safety is device-enforced. The protocol never bypasses it.

---

### Trait `fan`

Air movers (hoods, dryers, HVAC, air fryers, circulators).

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `fan_state` | enum | — | `off` `on` `auto` `boost` | r/w/e | req | |
| `fan_speed` | u8 | — | 0–max (advertised, typically 0–5) | r/w/e | opt | 0 = off |
| `fan_percent` | percent | percent | 0–100 | r/w | opt | Continuous |
| `fan_remaining_s` | duration_s | second | | r/e | opt | Boost / delay-off |
| `swing_on` | bool | — | | r/w | opt | Louver swing (HVAC) |
| `louver_deg` | u16 | degree | 0–180 | r/w | opt | |

**Commands**

| id | type | arg | access | req | description |
|----|------|-----|--------|-----|-------------|
| `boost` | command | `duration_s` 30–900 | w | opt | Hood / HVAC boost |

---

### Trait `heater`

Heaters, elements, burners as actuators (not the temperature loop).

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `heater_state` | enum | — | `off` `on` `fault` | r/e | req | **zoned** |
| `heater_percent` | percent | percent | 0–100 | r/w | opt | Duty / modulation |
| `heat_source` | enum | — | `electric` `gas` `induction` `heat_pump` `steam` `mixed` `unknown` | r | opt | |
| `flame` | enum | — | `off` `on` `fault` | r/e | opt | Gas |

Raw `heater_percent` writes may be refused (`unsupported_operation`) on
closed-loop appliances that only accept `setpoint_c`.

---

### Trait `motor`

Drums, pumps (as motors), grinders, blades.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `motor_state` | enum | — | `off` `on` `stall` `fault` | r/e | req | |
| `rpm` | u16 | rpm | 0–20000 | r/e | opt | Measured |
| `rpm_setpoint` | u16 | rpm | device range | r/w | opt | |
| `speed_level` | u8 | — | 0–max | r/w/e | opt | Discrete speeds |
| `direction` | enum | — | `forward` `reverse` | r/w | opt | |

---

### Trait `zone`

Multi-compartment advertisement.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `zones` | list<string> | — | max 16 `snake_case` ids | r | req | e.g. `fridge`,`freezer`,`hob_1` |
| `zone_mode` | enum | — | class-specific (`fridge` `freezer` `off` `bar`) | r/w | opt | Convertible zones; **zoned** |
| `zone_enable` | bool | — | | r/w/e | opt | **zoned** HVAC heads / hob |

---

### Trait `dispense`

Water / ice / beverage dispensers.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `dispense_type` | enum | — | `water` `ice_cubed` `ice_crushed` `hot_water` `ambient` `cold` `beer` `other` | r/w | opt | Selected product |
| `portion_ml` | u16 | milliliter | 10–2000 | r/w | opt | |
| `hot_lock` | bool | — | | r/w/e | opt | Child lock for hot water |
| `dispensing` | bool | — | | r/e | req | |

**Commands**

| id | type | arg | access | req | description |
|----|------|-----|--------|-----|-------------|
| `dispense` | command | optional `portion_ml` | w | req | Start portion or continuous until `stop_dispense` |
| `stop_dispense` | command | void | w | req | |

Continuous dispense without a portion should time out device-side (seconds).

---

### Trait `ice`

Ice production (fridge or stand-alone).

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `ice_enabled` | bool | — | | r/w/e | req | |
| `ice_state` | enum | — | `off` `making` `harvest` `full` `fault` | r/e | req | |
| `ice_type` | enum | — | `cube` `crushed` `nugget` `clear` `crescent` | r/w | opt | |
| `bin_percent` | percent | percent | 0–100 | r/e | opt | |
| `bin_full` | bool | — | | r/e | opt | |

---

### Trait `ota`

Firmware update. Optional.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `update_available` | bool | — | | r/e | req | |
| `update_version` | string | — | 0–32 | r | opt | |
| `update_state` | enum | — | `idle` `downloading` `applying` `reboot` `failed` | r/e | opt | |
| `update_percent` | percent | percent | 0–100 | r/e | opt | |

**Commands**

| id | type | arg | access | req | description |
|----|------|-----|--------|-----|-------------|
| `start_update` | command | void | w | opt | Must refuse during unsafe cycles |

---

## Per-class variables

Points below are **in addition to** the traits listed in
[`appliances.md`](./appliances.md). Do not re-declare trait points. Enums here
refine `trait.program.program` and `trait.cycle.cycle_phase` for that class.

Unless noted, class points are `opt` so low-end models can omit them. Items
marked `req` are required **if** the class is the device’s primary class.

---

### Class `washer`

**Traits:** identity, power, connectivity, time_schedule, door_lid, child_lock,
cycle, program, water, temperature, motor, fault, energy, remote, maintenance,
audio, safety.

**`program` tokens:** `cotton` `eco` `wool` `delicates` `quick` `rinse` `spin`
`bedding` `allergy` `outdoor` `synthetic` `handwash` `drum_clean` `custom`.

**`cycle_phase` tokens:** `fill` `prewash` `wash` `rinse` `spin` `drain`
`soak` `steam` `complete`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `wash_temp_c` | f32 | celsius | 0–95 | r/w | req | Wash water target; 0 = cold |
| `wash_temp_band` | enum | — | `cold` `warm` `hot` `90` | r/w | opt | Alternate UI; maps to `wash_temp_c` |
| `spin_rpm` | u16 | rpm | 0–1600 | r/w | req | 0 = no spin / rinse hold |
| `spin_off` | bool | — | | r/w | opt | Alias for `spin_rpm=0` |
| `soil_level` | enum | — | `light` `normal` `heavy` | r/w | opt | |
| `load_size` | enum | — | `small` `medium` `large` `auto` | r/w | opt | |
| `extra_rinse` | bool | — | | r/w | opt | |
| `prewash` | bool | — | | r/w | opt | |
| `steam` | bool | — | | r/w | opt | |
| `rinse_hold` | bool | — | | r/w | opt | Pause before final drain |
| `auto_dose` | bool | — | | r/w | opt | |
| `detergent_ml` | u16 | milliliter | 0–200 | r/w | opt | Per cycle if auto-dose |
| `softener_ml` | u16 | milliliter | 0–100 | r/w | opt | |
| `detergent_level_percent` | percent | percent | 0–100 | r/e | opt | Reservoir |
| `softener_level_percent` | percent | percent | 0–100 | r/e | opt | |
| `bleach_level_percent` | percent | percent | 0–100 | r/e | opt | |
| `unbalance` | bool | — | | r/e | opt | |
| `drum_rpm` | u16 | rpm | 0–1600 | r/e | opt | Measured |
| `sabbath_mode` | bool | — | | r/w/e | opt | Suppress beeps / delay display |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving heater/motor duty (distinct from `eco` program) |
| `door_ajar` | bool | — | | r/e | opt | Door / lid ajar bit |
| `door_locked` | bool | — | | r/e | opt | Door lock engaged (cycle safety) |
| `water_temp_alarm` | bool | — | | r/e | opt | Wash water over/under temp |
| `overflow_alarm` | bool | — | | r/e | opt | Tub overflow / flood bit |
| `detergent_low` | bool | — | | r/e | opt | Auto-dose reservoir low (alongside `detergent_level_percent`) |
| `timer_s` | duration_s | second | 0–86400 | r/w/e | opt | Delay / countdown timer (also see `trait.time_schedule.delay_start_s`) |

---

### Class `dryer`

**Traits:** identity, power, connectivity, time_schedule, door_lid, child_lock,
cycle, program, temperature, humidity, heater, fan, filter, fault, energy,
remote, maintenance, audio, safety.

**`program` tokens:** `cotton` `synthetic` `delicates` `wool` `timed`
`air_fluff` `bedding` `hygiene` `rack` `eco` `custom`.

**`cycle_phase` tokens:** `heating` `drying` `cooling` `anti_crease` `complete`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `dryness` | enum | — | `iron` `cupboard` `extra` `damp` | r/w | opt | Sensor dry target |
| `timed_s` | duration_s | second | 0–18000 | r/w | opt | Timed dry; used when program is `timed` |
| `heat_level` | enum | — | `low` `medium` `high` `air` | r/w | opt | |
| `anti_crease` | bool | — | | r/w | opt | |
| `steam_refresh` | bool | — | | r/w | opt | |
| `lint_filter` | enum | — | `ok` `missing` `clogged` | r/e | req | Also reflected in `trait.filter` |
| `drain_tank` | enum | — | `ok` `full` `missing` `na` | r/e | opt | Condenser |
| `dryness_percent` | percent | percent | 0–100 | r/e | opt | Estimate |
| `vent_blocked` | bool | — | | r/e | opt | |
| `sabbath_mode` | bool | — | | r/w/e | opt | Suppress beeps / delay display |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving heater/fan duty (distinct from `eco` program) |
| `door_ajar` | bool | — | | r/e | opt | Door / lid ajar bit |
| `door_locked` | bool | — | | r/e | opt | Door lock engaged (cycle safety) |
| `high_temp_alarm` | bool | — | | r/e | opt | Drum / exhaust overtemp |
| `lint_full` | bool | — | | r/e | opt | Lint trap full bit (alongside `lint_filter`) |
| `timer_s` | duration_s | second | 0–86400 | r/w/e | opt | Delay / countdown timer (also see `trait.time_schedule.delay_start_s`) |
| `thermal_port_id` | string | — | local port id | r | opt | Device heat port (e.g. `exhaust`); see thermal-plant |
| `thermal_port_direction` | enum | — | `source` `sink` `bidirectional` | r | opt | Seed `source` (exhaust / heat reject) |
| `thermal_port_media` | enum | — | `water` `air` `glycol` `refrigerant_proxy` `unknown` | r | opt | Seed `air` |
| `thermal_port_max_power_w` | f32 | watt | | r | opt | Seed 2000 (demo; ~1.5–2.5 kW reject band) |
| `thermal_port_attached_reservoir_id` | string | — | reservoir id or empty | r/w | opt | Attach/detach plant reservoir |

---

### Class `washer_dryer`

Composition: all of `washer` class points + shared `DRYER_BASE` dryer points +
combo extras below. Washer deepen ids (sabbath/eco/door/alarms/detergent/timer)
are inherited from the washer slice; dryer-only `high_temp_alarm` / `lint_full`
are listed here on the combo EXTRA (not via `DRYER_DEPTH`, which would
duplicate laundry ids).

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `combo_mode` | enum | — | `wash_only` `dry_only` `wash_and_dry` | r/w | req | |
| `dry_after_wash` | bool | — | | r/w | opt | Implies `wash_and_dry` |
| `max_dry_s` | duration_s | second | 0–18000 | r/w | opt | Cap on the dry portion |
| `high_temp_alarm` | bool | — | | r/e | opt | Drum / exhaust overtemp (dryer-specific; washer uses `water_temp_alarm`) |
| `lint_full` | bool | — | | r/e | opt | Lint trap full bit (alongside `lint_filter`) |

---

### Class `fridge`

**Traits:** identity, power, connectivity, door_lid, temperature, zone,
lighting, fault, energy, remote, maintenance; optional ice, dispense, filter,
child_lock, audio.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `vacation_mode` | bool | — | | r/w/e | opt | |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | |
| `defrost_active` | bool | — | | r/e | opt | |
| `compressor_on` | bool | — | | r/e | opt | |
| `high_temp_alarm` | bool | — | | r/e | opt | |
| `power_fail_ms` | timestamp_ms | — | | r/e | opt | Last outage |
| `door_ajar` | bool | — | | r/e | opt | Door / lid ajar bit |
| `low_temp_alarm` | bool | — | | r/e | opt | Too-cold alarm |
| `thermal_port_id` | string | — | local port id | r | opt | Device heat port (e.g. `condenser`); see thermal-plant |
| `thermal_port_direction` | enum | — | `source` `sink` `bidirectional` | r | opt | Seed `source` (condenser reject) |
| `thermal_port_media` | enum | — | `water` `air` `glycol` `refrigerant_proxy` `unknown` | r | opt | Seed `water` (matches plant demo) |
| `thermal_port_max_power_w` | f32 | watt | | r | opt | Seed 120 (demo) |
| `thermal_port_attached_reservoir_id` | string | — | reservoir id or empty | r/w | opt | Attach/detach plant reservoir |

Fridge `setpoint_c` typical 1–7. `super_mode` is super-cool.
Shared cold-cabinet points plus fridge-only `door_ajar` / `low_temp_alarm`;
`thermal_port_*` condenser surface unchanged.

---

### Class `freezer`

Shared cold-cabinet extras plus freezer-only depth (merged like fridge thermal
ports — combo dual-zone depth lives on `fridge_freezer`):

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `vacation_mode` | bool | — | | r/w/e | opt | Shared cold-cabinet |
| `sabbath_mode` | bool | — | | r/w/e | opt | Shared cold-cabinet |
| `eco_mode` | bool | — | | r/w | opt | Shared cold-cabinet |
| `defrost_active` | bool | — | | r/e | opt | Shared cold-cabinet |
| `compressor_on` | bool | — | | r/e | opt | Shared cold-cabinet |
| `high_temp_alarm` | bool | — | | r/e | opt | Shared cold-cabinet |
| `power_fail_ms` | timestamp_ms | — | | r/e | opt | Shared cold-cabinet; last outage |
| `fast_freeze` | bool | — | | r/w/e | opt | Fast / boost freeze mode |
| `door_ajar` | bool | — | | r/e | opt | Door / lid ajar bit |
| `ice_buildup` | bool | — | | r/e | opt | Frost / ice buildup detected |
| `low_temp_alarm` | bool | — | | r/e | opt | Too-cold alarm |
| `anti_sweat` | bool | — | | r/w | opt | Anti-sweat heater (uprights) |
| `fast_freeze_remaining_s` | duration_s | second | 0–86400 | r/e | opt | Fast-freeze timer remaining |
| `frost_clean_needed` | bool | — | | r/e | opt | Manual frost clean suggested |

`setpoint_c` typical −24–−12. `trait.temperature.super_mode` is super-freeze.
Chest lid uses `door_lid`. No `thermal_port_*` on this class.

---

### Class `fridge_freezer`

Shared cold-cabinet extras plus dual-zone depth (merged like freezer extras —
no thermal-port surface; fridge owns condenser ports). Zones required: at least
`fridge` and `freezer`. Optional zones: `convertible`, `bar`, `pantry`,
`crisper`, `door`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `vacation_mode` | bool | — | | r/w/e | opt | Shared cold-cabinet |
| `sabbath_mode` | bool | — | | r/w/e | opt | Shared cold-cabinet |
| `eco_mode` | bool | — | | r/w | opt | Shared cold-cabinet |
| `defrost_active` | bool | — | | r/e | opt | Shared cold-cabinet |
| `compressor_on` | bool | — | | r/e | opt | Shared cold-cabinet |
| `high_temp_alarm` | bool | — | | r/e | opt | Shared chassis / any-zone high temp |
| `power_fail_ms` | timestamp_ms | — | | r/e | opt | Shared cold-cabinet; last outage |
| `door_ajar_fridge` | bool | — | | r/e | opt | Fresh-food door ajar |
| `door_ajar_freezer` | bool | — | | r/e | opt | Freezer door / drawer ajar |
| `fast_freeze` | bool | — | | r/w/e | opt | Fast / boost freeze (freezer side) |
| `ice_buildup` | bool | — | | r/e | opt | Freezer-side frost / ice buildup |
| `high_temp_alarm_fridge` | bool | — | | r/e | opt | Fresh-food zone high-temp alarm |
| `high_temp_alarm_freezer` | bool | — | | r/e | opt | Freezer zone high-temp alarm |
| `convertible_zone_mode` | enum | — | `fridge` `freezer` `off` `bar` | r/w | opt | Convertible compartment mode |

Convertible compartment also uses `trait.zone.zone_mode` when advertised as a
`zone`. Freezer-only points (`anti_sweat`, `frost_clean_needed`,
`fast_freeze_remaining_s`, single `door_ajar`, `low_temp_alarm`) stay on
`freezer`.

---

### Class `wine_cooler`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `vibration_reduce` | bool | — | | r/w | opt | Night / quiet compressor |
| `uv_protect` | bool | — | | r/w | opt | Disable interior light (UV protect) |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `compressor_on` | bool | — | | r/e | opt | |
| `high_temp_alarm` | bool | — | | r/e | opt | |
| `low_temp_alarm` | bool | — | | r/e | opt | Freeze / too-cold risk |
| `vibration_alert` | bool | — | | r/e | opt | Vibration / cork risk |
| `bottle_count` | u16 | — | 0–300 | r/e | opt | Estimated bottles loaded |

Setpoints typically 5–20 °C per zone (`upper` / `lower`). Humidity via
`trait.humidity` (`current_rh` / optional `setpoint_rh`). Dual-zone is typical.

---

### Class `beverage_cooler`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving compressor duty |
| `compressor_on` | bool | — | | r/e | opt | |
| `high_temp_alarm` | bool | — | | r/e | opt | |
| `low_temp_alarm` | bool | — | | r/e | opt | Freeze / too-cold risk |
| `door_ajar` | bool | — | | r/e | opt | Door ajar bit |
| `can_capacity` | u16 | — | 0–500 | r/e | opt | Rated / estimated can (or bottle) capacity |

Setpoint typically 1–10 °C. Humidity/vibration stay on `wine_cooler`, not here.

---

### Class `ice_maker`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `clean_cycle_needed` | bool | — | | r/e | opt | Descale / clean due |
| `water_temp_c` | f32 | celsius | 0–40 | r | opt | Inlet water temperature |
| `water_low` | bool | — | | r/e | opt | Low / no water supply |
| `scoop_light` | bool | — | | r/w | opt | Bin scoop light |
| `max_ice_mode` | bool | — | | r/w/e | opt | Boost ice production |
| `harvest_fail` | bool | — | | r/e | opt | Harvest / eject failure |
| `scale_alert` | bool | — | | r/e | opt | Mineral scale alert |
| `delayed_start_s` | duration_s | second | 0–86400 | r/w | opt | Delayed production start |

Bin level / full via `trait.ice` (`bin_percent` / `bin_full`). Filter life via
`trait.filter.life_percent`. Commands: `trait.maintenance.start_clean` starts a
clean cycle.

---

### Class `kegerator`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `co2_kpa` | f32 | kilopascal | 0–400 | r/e | opt | If sensed |
| `keg_percent` | percent | percent | 0–100 | r/e | opt | Estimated remaining keg |
| `keg_empty` | bool | — | | r/e | opt | Empty / near-empty bit |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving compressor duty |
| `compressor_on` | bool | — | | r/e | opt | |
| `high_temp_alarm` | bool | — | | r/e | opt | |
| `low_temp_alarm` | bool | — | | r/e | opt | Freeze / too-cold risk |
| `door_ajar` | bool | — | | r/e | opt | Door ajar bit |

Setpoint typically 1–10 °C. CO₂ setpoint writes are vendor unless advertised.

---

### Class `dishwasher`

**`program` tokens:** `auto` `eco` `intensive` `quick` `glass` `rinse`
`hygiene` `night` `custom`.

**`cycle_phase` tokens:** `prewash` `wash` `rinse` `dry` `complete`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `extra_dry` | bool | — | | r/w | opt | |
| `half_load` | bool | — | | r/w | opt | |
| `sanitize` | bool | — | | r/w | opt | |
| `zone_wash` | enum | — | `all` `upper` `lower` | r/w | opt | |
| `tab_mode` | bool | — | | r/w | opt | Combined detergent tab |
| `rinse_aid_level` | enum | — | `empty` `low` `ok` | r/e | opt | Thin-table level (typical) |
| `salt_level` | enum | — | `empty` `low` `ok` `na` | r/e | opt | Thin-table level (typical) |
| `rinse_aid_dose` | u8 | — | 0–6 | r/w | opt | |
| `turbidity` | u16 | — | 0–1000 device units | r | opt | Auto programs |
| `wash_temp_c` | f32 | celsius | 30–75 | r/w | opt | Typical |
| `sabbath_mode` | bool | — | | r/w/e | opt | Suppress beeps / delay display |
| `eco_mode` | bool | — | | r/w | opt | Prefer lower energy / water |
| `door_ajar` | bool | — | | r/e | opt | Door / lid ajar bit |
| `door_locked` | bool | — | | r/e | opt | Door lock engaged |
| `rinse_aid_low` | bool | — | | r/e | opt | Rinse-aid low alarm (alongside `rinse_aid_level`) |
| `salt_low` | bool | — | | r/e | opt | Salt / regenerant low alarm (alongside `salt_level`) |
| `overflow_alarm` | bool | — | | r/e | opt | Tub overflow / flood bit |
| `timer_s` | duration_s | second | 0–86400 | r/w/e | opt | End / delay timer |
| `thermal_port_id` | string | — | local port id | r | opt | Device heat port (e.g. `inlet_preheat`); see thermal-plant |
| `thermal_port_direction` | enum | — | `source` `sink` `bidirectional` | r | opt | Seed `sink` (DHW inlet preheat) |
| `thermal_port_media` | enum | — | `water` `air` `glycol` `refrigerant_proxy` `unknown` | r | opt | Seed `water` |
| `thermal_port_max_power_w` | f32 | watt | | r | opt | Seed 1800 (demo) |
| `thermal_port_attached_reservoir_id` | string | — | reservoir id or empty | r/w | opt | Attach/detach plant reservoir |

Optional depth (fifth undepened Tier-A deepen): sabbath/eco/door/alarms/timer;
typical also advertises thin-table `rinse_aid_level` / `salt_level` /
`wash_temp_c` and trait `delay_start_s`. `thermal_port_*` inlet-preheat surface
unchanged.

---

### Class `microwave`

**`program` tokens:** `manual` `sensor_reheat` `sensor_cook` `defrost`
`popcorn` `beverage` `potato` `grill` `convection` `combo` `custom`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `cook_s` | duration_s | second | 1–3600 | r/w | req | Cook duration (required surface) |
| `power_level_percent` | percent | percent | 0–100 | r/w | req | 10% steps typical |
| `power_w` | u16 | watt | 0–2000 | r/w | opt | Alternate to percent (typical) |
| `defrost_g` | u16 | gram | 50–4000 | r/w | opt | Thin-table defrost mass (typical) |
| `turntable` | bool | — | | r/w | opt | Turntable on/off (typical; not `turntable_on`) |
| `inverter` | bool | — | | r | opt | Capability flag (typical) |
| `add_30s` | command | void | — | w | opt | Shortcut |
| `sabbath_mode` | bool | — | | r/w/e | opt | Suppress beeps / delay display |
| `eco_mode` | bool | — | | r/w | opt | Prefer lower average power |
| `door_ajar` | bool | — | | r/e | opt | Door ajar bit |
| `magnetron_on` | bool | — | | r/e | opt | Magnetron energizing telemetry (not raw RF control) |
| `high_temp_alarm` | bool | — | | r/e | opt | Cavity / grill overtemp alarm |
| `timer_s` | duration_s | second | 0–86400 | r/w/e | opt | Kitchen timer (distinct from `cook_s`) |

Door open + start → `safety_interlock`. Remote start often `remote_disabled`
by default.

Optional depth (sixth undepened Tier-A deepen): sabbath/eco/door_ajar/
magnetron_on/high_temp_alarm/timer_s; typical also advertises thin-table
`power_w` / `defrost_g` / `turntable` / `inverter`. Required `cook_s` /
`power_level_percent` unchanged. `trait.child_lock.child_lock` already typical.

---

### Class `oven`

**`program` / mode tokens:** `bake` `convection_bake` `roast`
`convection_roast` `broil` `convection_broil` `proof` `keep_warm` `self_clean`
`pyrolytic` `air_fry` `steam_assist` `sabbath` `off`.

On ovens, `trait.program.program` **is** the cooking mode.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `broil_level` | enum | — | `low` `high` | r/w | opt | Broil intensity (typical) |
| `convection_fan` | bool | — | | r/w | opt | Convection fan enable (typical) |
| `steam_percent` | percent | percent | 0–100 | r/w | opt | Hybrid steam assist (typical) |
| `cook_s` | duration_s | second | 0–43200 | r/w | opt | Cook duration; 0 = no timer (typical) |
| `door_locked_clean` | bool | — | | r/e | opt | Pyro / self-clean lock (typical) |
| `element_bake` | bool | — | | r/e | opt | Bake element energizing (typical) |
| `element_broil` | bool | — | | r/e | opt | Broil element energizing (typical) |
| `sabbath_mode` | bool | — | | r/w/e | opt | Suppress beeps / delay display |
| `eco_mode` | bool | — | | r/w | opt | Prefer lower average power |
| `heater_on` | bool | — | | r/e | opt | Aggregate heater telemetry (not element control) |
| `high_temp_alarm` | bool | — | | r/e | opt | Cavity overtemp alarm |
| `door_ajar` | bool | — | | r/e | opt | Door ajar bit (distinct from pyro `door_locked_clean`) |
| `timer_s` | duration_s | second | 0–86400 | r/w/e | opt | Kitchen timer (distinct from `cook_s`) |

Self-clean: `program=self_clean` or `pyrolytic` then `cycle.start`. Mid-clean
setpoint writes → `busy`. Meat probe via `trait.temperature.probe_c` /
`probe_target_c` / `probe_connected`; preheat via `preheat_complete`.

Optional depth (seventh undepened Tier-A deepen): sabbath/eco/heater_on/
high_temp_alarm/door_ajar/timer_s on oven-only `OVEN_DEPTH` (kept off
`OVEN_BASE` so `range` / `steam_oven` / `toaster_oven` composition stays
unpolluted); typical also advertises thin-table broil/convection/steam/cook/
door_locked_clean/elements + Temperature probe/preheat points. Child lock
already typical.

---

### Class `steam_oven`

Oven points plus steam-specific. Oven-shared optional points (`broil_level`,
`convection_fan`, `steam_percent`, `cook_s`, `door_locked_clean`,
`element_bake`, `element_broil`, …) plus:

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `steam_mode` | enum | — | `steam` `combi` `convection` `sous_vide` `reheat` `descale` | r/w | req | Primary steam / combi mode family |
| `humidity_set_percent` | percent | percent | 0–100 | r/w | opt | Combi humidity / steam intensity setpoint |
| `water_tank` | enum | — | `ok` `low` `empty` `missing` | r/e | req | Tank status |
| `water_tank_level` | percent | percent | 0–100 | r/e | opt | Fine tank fill level |
| `descaling_needed` | bool | — | | r/e | opt | Descale due indicator |
| `steam_generator_on` | bool | — | | r/e | opt | Steam generator / boiler active |
| `cavity_humidity` | percent | percent | 0–100 | r/e | opt | Sensed cavity humidity |
| `door_locked` | bool | — | | r/e | opt | Cooking door lock (distinct from pyro `door_locked_clean`) |
| `drain_full` | bool | — | | r/e | opt | Condensate drain container full |
| `generator_fault` | bool | — | | r/e | opt | Steam generator fault |
| `delayed_start_s` | duration_s | second | 0–86400 | r/w | opt | Delayed cook start |

Cycle remaining via `trait.cycle.remaining_s`; water hardness via
`trait.water.hardness_ppm`.

---

### Class `toaster_oven`

Subset of `oven` modes: `toast` `bake` `broil` `air_fry` `keep_warm`
`convection`. Oven-shared optional points (`broil_level`, `convection_fan`,
`cook_s`, `element_bake`, `element_broil`, …) plus toaster-specific:

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `toast_shade` | u8 | — | 1–7 | r/w | opt | Toast darkness |
| `crumb_tray` | enum | — | `ok` `missing` `unknown` | r/e | opt | Crumb tray present / status |
| `door_open` | bool | — | | r/e | opt | Simple cavity door ajar bit |
| `timer_remaining_s` | duration_s | second | 0–43200 | r/e | opt | Toast/bake countdown |
| `delayed_start_s` | duration_s | second | 0–86400 | r/w | opt | Delayed cook start |
| `rack_position` | enum | — | `lower` `middle` `upper` | r/w | opt | Rack / shelf position |
| `bagel` | bool | — | | r/w | opt | Bagel / one-side toast mode |
| `preheating` | bool | — | | r/e | opt | Preheat in progress |
| `slices` | u8 | — | 1–6 | r/w | opt | Toast slice / portion count |
| `toast_done` | bool | — | | r/e | opt | End-of-toast / cycle complete latch |

Cycle remaining via `trait.cycle.remaining_s`.

---

### Class `range`

Composition: hob zones reuse `cooktop` class points; cavity reuses `oven`
`OVEN_BASE` thin surface. Zone ids: `hob_1`…`hob_n`, `oven`, `oven_lower`,
`warming_drawer`. Range-only extras live on `RANGE_EXTRA` (not `OVEN_DEPTH`).

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `surface` | enum | — | `gas` `electric` `radiant` `induction` `mixed` | r | req | Hob fuel / heat source |
| `sabbath_mode` | bool | — | | r/w/e | opt | Suppress beeps / delay display |
| `eco_mode` | bool | — | | r/w | opt | Prefer lower average power |
| `heater_on` | bool | — | | r/e | opt | Aggregate cavity heater telemetry |
| `high_temp_alarm` | bool | — | | r/e | opt | Cavity overtemp alarm |
| `door_ajar` | bool | — | | r/e | opt | Oven door ajar (distinct from pyro `door_locked_clean`) |

Also advertises composed cooktop depth (`boost` / zoned `timer_s` / `bridge` /
`keep_warm` / `hotspot_alert` / `timer_active` / `paused` / `surface_c` /
`element_fault` / `pan_detect` / `flame_on` / `flame_out` / `ignition_fail` /
`power_limit_w`) and `OVEN_BASE` (`broil_level` / `convection_fan` /
`steam_percent` / `cook_s` / `door_locked_clean` / `element_bake` /
`element_broil`). Do **not** add a second class `timer_s` from `OVEN_DEPTH`.
Meat probe via `trait.temperature.probe_c` / `probe_target_c` /
`probe_connected`; preheat via `preheat_complete`.

Optional depth (eighth undepened Tier-A deepen): `RANGE_EXTRA` cavity
sabbath/eco/heater_on/high_temp_alarm/door_ajar; typical reuses cooktop +
`OVEN_BASE` composition + Temperature probe/preheat. Child lock already typical.

---

### Class `cooktop`

**Zoned** per burner / element.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `level` | u8 | — | 0–max (9 or 17 typical) | r/w/e | req | 0 = off |
| `boost` | bool | — | | r/w/e | opt | |
| `timer_s` | duration_s | second | 0–10800 | r/w/e | opt | Per zone |
| `bridge` | bool | — | | r/w | opt | Zone paired with neighbor |
| `residual_heat` | bool | — | | r/e | req | |
| `flame_out` | bool | — | | r/e | opt | Gas |
| `ignition_fail` | bool | — | | r/e | opt | Gas |
| `power_limit_w` | u32 | watt | | r/w | opt | Load shed cap |
| `keep_warm` | bool | — | | r/w/e | opt | Low hold heat per zone |
| `hotspot_alert` | bool | — | | r/e | opt | Active hotspot warning beyond residual heat |
| `timer_active` | bool | — | | r/e | opt | Zone timer running |
| `paused` | bool | — | | r/e | opt | `pause_all` latch active |
| `surface_c` | f32 | celsius | 0–400 | r/e | opt | Glass / coil surface temp |
| `element_fault` | bool | — | | r/e | opt | Electric element fault |
| `pan_detect` | bool | — | | r/e | opt | Electric glass pan present (not induction `pan_present`) |
| `flame_on` | bool | — | | r/e | opt | Gas flame lit |

**Commands:** `pause_all` (w, void) — all zones to 0 preserving last levels;
`resume_all` (w, void).

Remote ignition of gas is default-deny (`safety_interlock`).

---

### Class `induction_hob`

All `cooktop` points plus:

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `pan_present` | bool | — | | r/e | req | **zoned** |
| `pan_size` | enum | — | `none` `small` `medium` `large` `unknown` | r/e | opt | |
| `power_w` | u16 | watt | 0–4000 | r/e | opt | Per zone |
| `limiter_active` | bool | — | | r/e | opt | Power share |
| `cookware_ok` | bool | — | | r/e | opt | |
| `temp_mode` | bool | — | | r/w | opt | Simulated pan temp vs power level |
| `flex_group` | string | — | zone id or empty | r/w | opt | Partner zone |

Writing `level` with `pan_present=false` is accepted; device times out and
emits `pan_missing`.

---

### Class `warming_drawer`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `level` | enum | — | `low` `medium` `high` | r/w | opt | Alternate to `setpoint_c` |
| `moist` | bool | — | | r/w | opt | Vent / humidity |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving heater duty |
| `heater_on` | bool | — | | r/e | opt | Element / heater active |
| `high_temp_alarm` | bool | — | | r/e | opt | Overtemp / too-hot |
| `door_ajar` | bool | — | | r/e | opt | Drawer ajar bit |
| `timer_s` | duration_s | second | 0–10800 | r/w/e | opt | Hold / countdown timer |

Setpoint typically 40–90 °C when using °C instead of `level`.

---

### Class `pizza_oven`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `stone_c` | f32 | celsius | | r/e | opt | Stone / deck temperature |
| `dome_c` | f32 | celsius | | r/e | opt | Dome / ceiling temperature |
| `top_bottom_balance` | i16 | percent | −100–100 | r/w | opt | + = more top heat |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving heater duty |
| `heater_on` | bool | — | | r/e | opt | Element / heater active |
| `high_temp_alarm` | bool | — | | r/e | opt | Overtemp / too-hot |
| `door_ajar` | bool | — | | r/e | opt | Door ajar bit |
| `timer_s` | duration_s | second | 0–10800 | r/w/e | opt | Cook / countdown timer |
| `steam_inject` | bool | — | | r/w/e | opt | Steam burst / inject |

Setpoint typically 200–450 °C.

---

### Class `air_fryer`

**`program` tokens:** `manual` `fries` `wings` `reheat` `bake` `dehydrate`
`fish` `veg` `custom`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `cook_s` | duration_s | second | 1–36000 | r/w | req | |
| `shake_enable` | bool | — | | r/w | opt | |
| `shake_due` | bool | — | | r/e | opt | Event when user should shake |
| `preheat` | bool | — | | r/w | opt | |
| `basket_present` | bool | — | | r/e | opt | **zoned** dual basket |
| `sync_finish` | bool | — | | r/w | opt | Dual zone |

---

### Class `electric_grill`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `plate_top_c` | f32 | celsius | | r/e | opt | Top contact plate temperature |
| `plate_bottom_c` | f32 | celsius | | r/e | opt | Bottom contact plate temperature |
| `sear` | bool | — | | r/w | opt | Sear / boost heat |
| `grease_tray` | enum | — | `ok` `missing` `full` | r/e | opt | Grease tray status |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving heater duty |
| `heater_on` | bool | — | | r/e | opt | Element / heater active |
| `high_temp_alarm` | bool | — | | r/e | opt | Overtemp / too-hot |
| `lid_open` | bool | — | | r/e | opt | Lid open / ajar bit |
| `timer_s` | duration_s | second | 0–10800 | r/w/e | opt | Cook / countdown timer |

Setpoint typically 100–250 °C.

---

### Class `electric_smoker`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `chamber_c` | f32 | celsius | | r/e | opt | Cabinet / chamber temperature |
| `smoke_on` | bool | — | | r/w/e | opt | Smoke generator / wood-tray heater |
| `fuel_percent` | percent | percent | 0–100 | r/e | opt | Pellets / puck / wood-chip level |
| `water_pan` | enum | — | `ok` `empty` `missing` `na` | r/e | opt | Water pan status |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving heater duty |
| `heater_on` | bool | — | | r/e | opt | Element / heater active |
| `high_temp_alarm` | bool | — | | r/e | opt | Overtemp / too-hot |
| `door_ajar` | bool | — | | r/e | opt | Door open / ajar bit |
| `timer_s` | duration_s | second | 0–10800 | r/w/e | opt | Cook / countdown timer |

Setpoint typically 50–150 °C.

Probe targets use `trait.temperature.probe_*` with zones `probe_1`…

---

### Class `range_hood`

Fan + lighting traits. Extra:

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `auto_mode` | bool | — | | r/w | opt | Follow hob or air quality |
| `delay_off_s` | duration_s | second | 0–1800 | r/w | opt | Delayed fan-off after cooking |
| `voc_index` | u16 | — | 0–500 | r/e | opt | Air-quality / VOC index if sensed |
| `grease_filter` | enum | — | `ok` `clogged` `missing` | r/e | opt | Grease filter status |
| `charcoal_filter` | enum | — | `ok` `replace` `na` | r/e | opt | Recirculating charcoal filter |
| `filter_dirty` | bool | — | | r/e | opt | Filter dirty indicator light |
| `boost` | bool | — | | r/w | opt | High-speed boost mode engaged |
| `boost_remaining_s` | duration_s | second | 0–900 | r/e | opt | Boost auto-expire countdown |
| `light_level` | u8 | — | 0–5 | r/w | opt | Discrete hood light steps |
| `grease_sensor` | bool | — | | r/e | opt | Cooking / grease plume detected |
| `hob_linked` | bool | — | | r/w | opt | Auto-follow linked hob activity |
| `overtemp` | bool | — | | r/e | opt | Motor / hood over-temperature |
| `charcoal_filter_life_percent` | percent | percent | 0–100 | r/e | opt | Recirculating charcoal life |

Fan speed via `trait.fan.fan_speed`; light dimming via
`trait.lighting.light_percent`; grease-filter life via
`trait.filter.life_percent`.

---

### Class `coffee_machine`

**`program` tokens:** `espresso` `double_espresso` `americano` `lungo`
`cappuccino` `latte` `macchiato` `hot_water` `steam` `rinse` `descale`
`custom`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `strength` | enum | — | `mild` `normal` `strong` `extra` | r/w | opt | |
| `volume_ml` | u16 | milliliter | 15–400 | r/w | opt | |
| `milk_ml` | u16 | milliliter | 0–400 | r/w | opt | |
| `grind_level` | u8 | — | 1–16 | r/w | opt | |
| `cups` | u8 | — | 1–2 | r/w | opt | |
| `water_tank` | enum | — | `ok` `low` `empty` `missing` | r/e | req | |
| `drip_tray` | enum | — | `ok` `full` `missing` | r/e | opt | |
| `grounds_bin` | enum | — | `ok` `full` `missing` | r/e | opt | |
| `milk_present` | bool | — | | r/e | opt | |
| `capsule_present` | bool | — | | r/e | opt | Capsule variant |
| `boiler_c` | f32 | celsius | | r/e | opt | |
| `brew_pressure_bar` | f32 | bar | 0–20 | r/e | opt | |

---

### Class `espresso_machine`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `brew_setpoint_c` | f32 | celsius | 85–100 | r/w | req | |
| `steam_setpoint_c` | f32 | celsius | 110–135 | r/w | opt | Dual boiler |
| `preinfusion_s` | duration_s | second | 0–15 | r/w | opt | |
| `shot_ml` | u16 | milliliter | 10–100 | r/w | opt | Volumetric |
| `shot_s` | duration_s | second | 0–90 | r/e | opt | Elapsed |
| `brew_pressure_bar` | f32 | bar | 0–16 | r/e | opt | Group / brew pressure |
| `pump_on` | bool | — | | r/e | opt | Pump active |
| `water_source` | enum | — | `tank` `plumbed` | r/w | opt | |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving boiler duty |
| `boiler_ready` | bool | — | | r/e | opt | Brew boiler at setpoint |
| `high_temp_alarm` | bool | — | | r/e | opt | Overtemp / too-hot |
| `water_tank_empty` | bool | — | | r/e | opt | Tank empty / inhibit brew |
| `descaling_needed` | bool | — | | r/e | opt | Descale due indicator |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Delayed brew / countdown |
| `steam_wand_on` | bool | — | | r/e | opt | Steam wand active |

**Commands:** `start_shot`, `stop_shot` (w, void). Raw `pump_on` write is
optional and safety-gated (`remote_brew_enabled` bool, r/w, opt).

---

### Class `drip_coffee_maker`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `cups` | u8 | — | 1–12 | r/w | opt | Batch size |
| `strength` | enum | — | `mild` `normal` `strong` | r/w | opt | Brew strength |
| `keep_warm_s` | duration_s | second | 0–7200 | r/w | opt | 0 = off |
| `carafe_present` | bool | — | | r/e | opt | Carafe / pot detect |
| `bloom` | bool | — | | r/w | opt | Pre-infusion bloom |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving keep-warm duty |
| `heater_on` | bool | — | | r/e | opt | Keep-warm plate active |
| `high_temp_alarm` | bool | — | | r/e | opt | Overtemp / too-hot plate |
| `water_tank_empty` | bool | — | | r/e | opt | Tank empty / inhibit brew |
| `descaling_needed` | bool | — | | r/e | opt | Descale due indicator |
| `timer_s` | duration_s | second | 0–7200 | r/w/e | opt | Delayed brew / countdown |

---

### Class `coffee_grinder`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `grind_s` | duration_s | second | 1–60 | r/w | opt | Time dose |
| `dose_g` | f32 | gram | 5–30 | r/w | opt | Mass dose |
| `grind_level` | u8 | — | 1–40 | r/w | req | Grind size ticks |
| `hopper_present` | bool | — | | r/e | opt | Hopper / lid detect |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving idle |
| `motor_on` | bool | — | | r/e | opt | Burr motor active |
| `hopper_empty` | bool | — | | r/e | opt | Hopper empty / inhibit grind |
| `bean_level_percent` | percent | percent | 0–100 | r/e | opt | Bean hopper fill level |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Delayed grind / countdown |
| `single_dose` | bool | — | | r/w | opt | Single-dose / bypass hopper feed |

---

### Class `kettle`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `keep_warm` | bool | — | | r/w/e | opt | |
| `keep_warm_s` | duration_s | second | 0–3600 | r/w | opt | |
| `on_base` | bool | — | | r/e | req | |
| `boil_dry` | bool | — | | r/e | opt | Latched trip |

Heat while `on_base=false` → `safety_interlock`.

---

### Class `water_dispenser`

Dispense + temperature traits. Extra:

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `hot_setpoint_c` | f32 | celsius | 60–100 | r/w | opt | Hot tank setpoint |
| `cold_setpoint_c` | f32 | celsius | 4–15 | r/w | opt | Cold tank setpoint |
| `bottle_empty` | bool | — | | r/e | opt | Bottle / reservoir empty (bottle units) |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving heater/cooler duty |
| `heater_on` | bool | — | | r/e | opt | Hot tank heater active |
| `cooler_on` | bool | — | | r/e | opt | Cold tank cooler / compressor active |
| `high_temp_alarm` | bool | — | | r/e | opt | Hot tank overtemp |
| `low_temp_alarm` | bool | — | | r/e | opt | Cold tank freeze / too-cold risk |
| `water_tank_empty` | bool | — | | r/e | opt | Plumbed / internal tank empty |

Filter life via `trait.filter.life_percent`. Hot-water lock via
`trait.child_lock.child_lock` (required on this class).

---

### Class `toaster`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `shade` | u8 | — | 1–7 | r/w | req | Toast darkness |
| `bagel` | bool | — | | r/w | opt | Bagel / one-side preference |
| `frozen` | bool | — | | r/w | opt | Frozen / defrost assist |
| `single_side` | bool | — | | r/w | opt | Heat one side only |
| `carriage` | enum | — | `up` `down` | r/e | opt | Carriage position |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving heater duty |
| `heater_on` | bool | — | | r/e | opt | Elements energized |
| `high_temp_alarm` | bool | — | | r/e | opt | Overtemp / jam heat alarm |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Optional countdown / remaining |
| `crumb_tray_full` | bool | — | | r/e | opt | Crumb tray needs emptying |
| `slots` | u8 | — | 1–4 | r | opt | Hardware slot count |

---

### Class `blender`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `form_factor` | enum | — | `jar` `immersion` | r | opt | Jar vs immersion form |
| `speed_level` | u8 | — | 0–10 | r/w | req | Blade speed |
| `pulse` | command | void | — | w | opt | Momentary high-speed pulse |
| `jar_present` | bool | — | | r/e | opt | Jar seated on base |
| `lid_locked` | bool | — | | r/e | opt | Lid interlock closed |
| `heated` | bool | — | | r | opt | Capability: soup blender |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving motor duty |
| `motor_on` | bool | — | | r/e | opt | Blade motor active |
| `overload_trip` | bool | — | | r/e | opt | Stall / thermal overload trip |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Timed blend / remaining |

---

### Class `food_processor`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `speed_level` | u8 | — | 0–10 | r/w | req | Blade / disc speed |
| `pulse` | command | void | — | w | opt | Momentary high-speed pulse |
| `bowl_present` | bool | — | | r/e | opt | Work bowl seated on base |
| `lid_locked` | bool | — | | r/e | opt | Lid / pusher interlock closed |
| `attachment` | enum | — | `unknown` `blade` `dough` `disc` | r | opt | Sensed tool attachment |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving motor duty |
| `motor_on` | bool | — | | r/e | opt | Drive motor active |
| `overload_trip` | bool | — | | r/e | opt | Stall / thermal overload trip |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Timed run / remaining |

---

### Class `stand_mixer`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `speed_level` | u8 | — | 0–10 | r/w | req | Planetary drive speed |
| `bowl_present` | bool | — | | r/e | opt | Mixing bowl seated / locked |
| `head_down` | bool | — | | r/e | opt | Mixer head lowered interlock |
| `mass_g` | f32 | gram | | r/e | opt | Bowl scale reading |
| `attachment` | enum | — | `unknown` `beater` `dough_hook` `whisk` | r | opt | Sensed tool attachment |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving motor duty |
| `motor_on` | bool | — | | r/e | opt | Drive motor active |
| `overload_trip` | bool | — | | r/e | opt | Stall / thermal overload trip |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Timed mix / remaining |

Head up → `safety_interlock` on start.

---

### Class `juicer`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `speed_level` | u8 | — | 0–10 | r/w | req | Auger / centrifuge speed |
| `reverse` | command | void | — | w | opt | Momentary reverse to clear jam |
| `pulp_full` | bool | — | | r/e | opt | Pulp bin full |
| `jug_present` | bool | — | | r/e | opt | Juice jug / pitcher seated |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving motor duty |
| `motor_on` | bool | — | | r/e | opt | Drive motor active |
| `overload_trip` | bool | — | | r/e | opt | Stall / thermal overload trip |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Timed juice / remaining |

---

### Class `rice_cooker`

**`program` tokens:** `white` `brown` `sushi` `porridge` `steam` `keep_warm`
`quick` `custom`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `texture` | enum | — | `soft` `normal` `firm` | r/w | opt | Cooked grain texture |
| `bowl_present` | bool | — | | r/e | opt | Inner bowl / pot seated |
| `keep_warm` | bool | — | | r/w/e | opt | Keep-warm enable |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving cook / hold |
| `heater_on` | bool | — | | r/e | opt | Heater element active |
| `high_temp_alarm` | bool | — | | r/e | opt | Bowl / boil-dry over-temp |
| `lid_open` | bool | — | | r/e | opt | Lid open |
| `timer_s` | duration_s | second | 0–86400 | r/w/e | opt | Delay start / cook remaining |
| `water_ratio` | f32 | — | 0.5–3.0 | r/w | opt | Water:rice volume ratio |

---

### Class `slow_cooker`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `heat_level` | enum | — | `low` `high` `warm` | r/w | req | If no numeric setpoint |
| `cook_s` | duration_s | second | 0–43200 | r/w | req | Cook duration |
| `pot_present` | bool | — | | r/e | opt | Ceramic / insert pot seated |
| `keep_warm` | bool | — | | r/w/e | opt | Keep-warm enable |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving cook / hold |
| `heater_on` | bool | — | | r/e | opt | Heater element active |
| `high_temp_alarm` | bool | — | | r/e | opt | Pot / dry-run over-temp |
| `lid_open` | bool | — | | r/e | opt | Lid open |
| `timer_s` | duration_s | second | 0–86400 | r/w/e | opt | Delay start / cook remaining |

---

### Class `multi_cooker`

**`program` tokens:** `pressure` `saute` `slow` `steam` `rice` `yogurt`
`sous_vide` `keep_warm` `sterilize` `custom`.

**`cycle_phase` tokens:** `preheat` `pressurizing` `at_pressure` `cooking`
`venting` `keep_warm` `safe_to_open`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `pressure_band` | enum | — | `low` `high` | r/w | opt | Pressure cook band |
| `pressure_kpa` | f32 | kilopascal | 0–150 | r/e | opt | Gauge pressure |
| `lid_locked` | bool | — | | r/e | req | Lid locked while pressurized |
| `float_valve` | enum | — | `down` `up` | r/e | opt | Float / pin valve position |
| `safe_to_open` | bool | — | | r/e | req | Safe to unlock / open lid |
| `remote_vent_enabled` | bool | — | | r/w | opt | Default false; unlocks remote `vent` |
| `vent` | command | void | — | w | opt | Quick release; default `safety_interlock` |
| `burn_detected` | bool | — | | r/e | opt | Burn / high-temp pot sensor |
| `pot_detect` | bool | — | | r/e | opt | Inner pot detected / seated |
| `cook_s` | duration_s | second | 0–172800 | r/w | opt | Cook duration setpoint |
| `delayed_start_s` | duration_s | second | 0–86400 | r/w | opt | Delayed cook start |
| `keep_warm` | bool | — | | r/w/e | opt | Keep-warm enable |
| `keep_warm_s` | duration_s | second | 0–14400 | r/w | opt | Keep-warm duration (0 = off / until cancel) |
| `saute_level` | enum | — | `low` `normal` `high` | r/w | opt | Sauté heat level |
| `overpressure_alarm` | bool | — | | r/e | opt | Over-pressure fault / alarm |
| `lid_mismatch` | bool | — | | r/e | opt | Lid / program mismatch fault |

Cycle remaining via `trait.cycle.remaining_s`.

---

### Class `sous_vide`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `low_water` | bool | — | | r/e | req | Cuts heat |
| `circulating` | bool | — | | r/e | opt | Pump / circulator running |
| `cook_s` | duration_s | second | 0–259200 | r/w | opt | Cook duration setpoint |
| `water_level_ok` | bool | — | | r/e | opt | Bath level healthy (inverse of low_water when both present) |
| `lid_closed` | bool | — | | r/e | opt | Cover / lid closed |
| `timer_remaining_s` | duration_s | second | 0–259200 | r/e | opt | Cook timer remaining |
| `target_done` | bool | — | | r/e | opt | Cook duration / target reached |
| `overtemp_alarm` | bool | — | | r/e | opt | Bath over-temperature alarm |
| `delayed_start_s` | duration_s | second | 0–86400 | r/w | opt | Delayed cook start |
| `alarm_offset_c` | f32 | celsius | −5–5 | r/w | opt | Done / alert offset from setpoint |

Cycle remaining via `trait.cycle.remaining_s`. Setpoint resolution typically 0.1 °C.

---

### Class `bread_maker`

**`program` tokens:** `basic` `whole_wheat` `french` `quick` `dough` `jam`
`bake_only` `custom`.

**`cycle_phase` tokens:** `knead` `rise` `punch_down` `bake` `keep_warm`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `crust` | enum | — | `light` `medium` `dark` | r/w | opt | Crust color |
| `loaf_size` | enum | — | `small` `medium` `large` | r/w | opt | |
| `pan_present` | bool | — | | r/e | opt | Baking pan seated |
| `keep_warm` | bool | — | | r/w/e | opt | Keep-warm after bake |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving bake / hold |
| `heater_on` | bool | — | | r/e | opt | Heater element active |
| `high_temp_alarm` | bool | — | | r/e | opt | Chamber / dry-run over-temp |
| `lid_open` | bool | — | | r/e | opt | Lid open |
| `timer_s` | duration_s | second | 0–86400 | r/w/e | opt | Delay start / bake remaining |

---

### Class `dehydrator`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `cook_s` | duration_s | second | 0–172800 | r/w | req | Dry / dehydrate duration |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving dry / hold |
| `heater_on` | bool | — | | r/e | opt | Heater element active |
| `fan_on` | bool | — | | r/e | opt | Circulating fan active |
| `high_temp_alarm` | bool | — | | r/e | opt | Chamber over-temp |
| `door_ajar` | bool | — | | r/e | opt | Door / lid ajar bit |
| `timer_s` | duration_s | second | 0–172800 | r/w/e | opt | Delay start / remaining |
| `tray_count` | u8 | — | 1–15 | r | opt | Hardware tray count |

---

### Class `vacuum_sealer`

**`cycle_phase` tokens:** `pump` `seal` `vent` `complete`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `mode` | enum | — | `vacuum_seal` `seal_only` `pulse` `chamber` | r/w | req | |
| `moist` | bool | — | | r/w | opt | Moist-food vacuum profile |
| `vacuum_kpa` | f32 | kilopascal | 0–101 | r/e | opt | Remaining absolute or gauge as advertised |
| `bag_detect` | bool | — | | r/e | opt | Bag present in seal channel |
| `form_factor` | enum | — | `bar` `chamber` | r | opt | Hardware form |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving vacuum / seal profile |
| `pump_on` | bool | — | | r/e | opt | Vacuum pump running |
| `seal_heater_on` | bool | — | | r/e | opt | Seal-bar heater active |
| `lid_locked` | bool | — | | r/e | opt | Lid / clamp interlock engaged |
| `seal_fail` | bool | — | | r/e | opt | Seal incomplete / fault bit |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Delay start / cycle timer |

---

### Class `ice_cream_maker`

**`program` tokens:** `ice_cream` `gelato` `sorbet` `keep_cool`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `doneness` | percent | percent | 0–100 | r/e | opt | Motor-load proxy |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving churn / cool profile |
| `compressor_on` | bool | — | | r/e | opt | Compressor running (compressor models) |
| `motor_on` | bool | — | | r/e | opt | Churn / dasher motor running |
| `bowl_present` | bool | — | | r/e | opt | Freezer bowl seated |
| `lid_locked` | bool | — | | r/e | opt | Lid / paddle interlock engaged |
| `low_temp_alarm` | bool | — | | r/e | opt | Bowl / mix below safe low |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Delay start / churn timer |

---

### Class `yogurt_maker`

**`program` tokens:** `yogurt` `greek` `proof`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `incubate_s` | duration_s | second | 3600–86400 | r/w | req | Incubation duration |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving incubation profile |
| `heater_on` | bool | — | | r/e | opt | Heater element active |
| `high_temp_alarm` | bool | — | | r/e | opt | Chamber over-temp |
| `low_temp_alarm` | bool | — | | r/e | opt | Chamber under-temp (culture risk) |
| `lid_open` | bool | — | | r/e | opt | Lid ajar bit |
| `jar_present` | bool | — | | r/e | opt | Jar / vessel seated |
| `timer_s` | duration_s | second | 0–86400 | r/w/e | opt | Delay start / remaining timer |

---

### Class `waffle_maker`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `shade` | u8 | — | 1–7 | r/w | opt | Doneness / browning level |
| `ready` | bool | — | | r/e | opt | Preheat complete |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving preheat / bake profile |
| `heater_on` | bool | — | | r/e | opt | Plate heater element active |
| `high_temp_alarm` | bool | — | | r/e | opt | Plate over-temp |
| `lid_open` | bool | — | | r/e | opt | Lid ajar bit |
| `batter_done` | bool | — | | r/e | opt | Bake cycle complete / waffle ready |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Delay start / remaining bake timer |

---

### Class `pasta_maker`

**`program` tokens:** `mix` `extrude` `mix_extrude`.

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `die` | enum | — | `spaghetti` `fettuccine` `penne` `other` | r/w | opt | Extrusion die shape |
| `jam` | bool | — | | r/e | opt | Extruder / mixer jam |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving mix / extrude profile |
| `motor_on` | bool | — | | r/e | opt | Drive motor active |
| `dough_ready` | bool | — | | r/e | opt | Mix complete / dough ready to extrude |
| `hopper_empty` | bool | — | | r/e | opt | Flour / dough hopper empty |
| `die_present` | bool | — | | r/e | opt | Extrusion die seated |
| `overload_trip` | bool | — | | r/e | opt | Motor overload / thermal trip |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Delay start / remaining cycle timer |

---

### Class `steam_cooker`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `cook_s` | duration_s | second | 1–7200 | r/w | req | Steam cook duration |
| `water_empty` | bool | — | | r/e | req | Reservoir / tank empty |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving steam profile |
| `heater_on` | bool | — | | r/e | opt | Steam heater / boiler active |
| `high_temp_alarm` | bool | — | | r/e | opt | Over-temperature alarm |
| `lid_open` | bool | — | | r/e | opt | Lid / door ajar |
| `steam_ready` | bool | — | | r/e | opt | Steam generator at cook temperature |
| `timer_s` | duration_s | second | 0–7200 | r/w/e | opt | Delay start / remaining timer |

---

### Class `garbage_disposal`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `run_s` | duration_s | second | 1–60 | r/w | opt | Max pulse length |
| `jam` | bool | — | | r/e | opt | Impeller / grind chamber jammed |
| `reset_needed` | bool | — | | r/e | opt | Manual reset after overload / jam |
| `reverse` | command | void | — | w | opt | Reverse grind direction |
| `run` | command | optional `run_s` | w | req | Timed pulse; default deny if `remote_control_enabled` is false |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving grind profile |
| `motor_on` | bool | — | | r/e | opt | Disposer motor running |
| `overload_trip` | bool | — | | r/e | opt | Overcurrent / thermal overload trip |
| `air_switch` | bool | — | | r/w | opt | Air-switch control enabled |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Delay start / remaining timer |

---

### Class `trash_compactor`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `ram_state` | enum | — | `up` `down` `moving` `jam` | r/e | opt | Compactor ram position / motion |
| `bin_full` | bool | — | | r/e | opt | Compaction bin / bag full |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving compact profile |
| `motor_on` | bool | — | | r/e | opt | Compactor drive motor running |
| `drawer_open` | bool | — | | r/e | opt | Drawer / door open (interlocks compact) |
| `overload_trip` | bool | — | | r/e | opt | Overcurrent / thermal overload trip |
| `key_lock` | bool | — | | r/w/e | opt | Physical key lock engaged |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Delay start / remaining timer |

---

### Class `water_heater`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `mode` | enum | — | `heat_pump` `hybrid` `electric` `vacation` `high_demand` `off` | r/w | opt | |
| `inlet_c` | f32 | celsius | | r | opt | |
| `outlet_c` | f32 | celsius | | r/e | opt | Tankless |
| `hot_remaining_percent` | percent | percent | 0–100 | r/e | opt | Tank / HPWH |
| `leak` | bool | — | | r/e | opt | |
| `dry_fire` | bool | — | | r/e | opt | |
| `recirc_on` | bool | — | | r/w/e | opt | |
| `form_factor` | enum | — | `tank` `tankless` `heat_pump` | r | opt | |
| `thermal_port_id` | string | — | local port id | r | opt | Device heat port (e.g. `preheat`); see thermal-plant |
| `thermal_port_direction` | enum | — | `source` `sink` `bidirectional` | r | opt | Seed `sink` (DHW preheat) |
| `thermal_port_media` | enum | — | `water` `air` `glycol` `refrigerant_proxy` `unknown` | r | opt | Seed `water` |
| `thermal_port_max_power_w` | f32 | watt | | r | opt | Seed 2000 (demo) |
| `thermal_port_attached_reservoir_id` | string | — | reservoir id or empty | r/w | opt | Attach/detach plant reservoir |

---

### Class `boiler`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `ch_enable` | bool | — | | r/w/e | req | Central heating |
| `dhw_enable` | bool | — | | r/w/e | opt | Domestic hot water |
| `ch_setpoint_c` | f32 | celsius | 30–90 | r/w | req | |
| `dhw_setpoint_c` | f32 | celsius | 35–65 | r/w | opt | |
| `flow_c` | f32 | celsius | | r/e | req | |
| `return_c` | f32 | celsius | | r | opt | |
| `pressure_bar` | f32 | bar | 0–4 | r/e | req | |
| `modulation_percent` | percent | percent | 0–100 | r/e | opt | |
| `burner_on` | bool | — | | r/e | opt | |
| `pump_on` | bool | — | | r/e | opt | |
| `outdoor_c` | f32 | celsius | | r | opt | Weather compensation |
| `summer_mode` | bool | — | | r/w | opt | CH off, DHW on |
| `flame_out` | bool | — | | r/e | opt | Flame failure / flame-out fault |
| `low_pressure` | bool | — | | r/e | opt | System water pressure low |
| `sabbath_mode` | bool | — | | r/w/e | opt | |
| `eco_mode` | bool | — | | r/w | opt | Energy-saving CH/DHW profile |
| `high_temp_alarm` | bool | — | | r/e | opt | Overheat / flow overtemp alarm |
| `lockout` | bool | — | | r/e | opt | Safety lockout after flame/ignition faults |
| `ignition_fail` | bool | — | | r/e | opt | Ignition failure |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Delay start / remaining timer |

No raw gas-valve command.

---

### Class `water_softener`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `capacity_remaining` | f32 | — | grains or m³ as advertised | r/e | opt | Unit in capabilities |
| `capacity_unit` | enum | — | `grain` `m3` | r | opt | |
| `salt_level` | enum | — | `empty` `low` `ok` `unknown` | r/e | opt | |
| `bypass` | bool | — | | r/w/e | opt | Softener valve bypass |
| `regen_now` | command | void | — | w | opt | |
| `treated_l` | f32 | liter | 0+ | r | opt | |
| `sabbath_mode` | bool | — | | r/w/e | opt | Suppress beeps / delay regen display |
| `eco_mode` | bool | — | | r/w | opt | Salt / water-saving regen profile |
| `regenerating` | bool | — | | r/e | opt | Regeneration cycle active |
| `salt_low` | bool | — | | r/e | opt | Salt low alarm (alongside `salt_level`) |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Delay start / remaining timer |

Hardness input uses `trait.water.hardness_ppm` / `hardness_gpg`. Resin bed life
uses `trait.filter.life_percent` (not a class `resin_life_percent`).

---

### Class `water_filter`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `tds_in_ppm` | u16 | ppm | 0–2000 | r/e | opt | Inlet TDS |
| `tds_out_ppm` | u16 | ppm | 0–1000 | r/e | opt | Outlet TDS |
| `tank_full` | bool | — | | r/e | opt | RO storage tank full |
| `flush` | command | void | — | w | opt | |
| `sabbath_mode` | bool | — | | r/w/e | opt | Suppress beeps / delay flush display |
| `eco_mode` | bool | — | | r/w | opt | Lower flush / pump duty |
| `bypass` | bool | — | | r/w/e | opt | Filter / RO bypass valve |
| `filter_clogged` | bool | — | | r/e | opt | Clog alarm (alongside filter_state) |
| `replace_needed` | bool | — | | r/e | opt | Replace reminder (alongside filter_state) |
| `timer_s` | duration_s | second | 0–3600 | r/w/e | opt | Delay flush / remaining timer |

Filter stages use `trait.filter` zones (`pre`, `ro`, `post`, `remin`). Filter
life uses `trait.filter.life_percent`. Measured flow uses
`trait.water.flow_l_min` (not a class `flow_lpm`).

---

### Class `hvac`

**Mode** is the primary setting (not `trait.program`).

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `hvac_mode` | enum | — | `off` `heat` `cool` `auto` `fan_only` `dry` `emergency_heat` | r/w/e | req | |
| `heat_setpoint_c` | f32 | celsius | 10–32 | r/w/e | opt | Required if heat capable |
| `cool_setpoint_c` | f32 | celsius | 10–32 | r/w/e | opt | Required if cool capable |
| `deadband_c` | f32 | celsius | 0.5–5 | r/w | opt | Auto mode |
| `space_c` | f32 | celsius | | r/e | req | Indoor |
| `outdoor_c` | f32 | celsius | | r | opt | |
| `hold` | bool | — | | r/w/e | opt | Ignore schedule |
| `quiet` | bool | — | | r/w | opt | |
| `eco` | bool | — | | r/w | opt | |
| `compressor_on` | bool | — | | r/e | opt | |
| `aux_heat` | bool | — | | r/e | opt | |
| `defrost` | bool | — | | r/e | opt | Heat pump |
| `reversing_valve` | enum | — | `heat` `cool` `unknown` | r | opt | |
| `thermal_port_id` | string | — | local port id | r | opt | Device heat port (e.g. `coil`); see thermal-plant |
| `thermal_port_direction` | enum | — | `source` `sink` `bidirectional` | r | opt | Seed `sink` (space heat from hot reservoir) |
| `thermal_port_media` | enum | — | `water` `air` `glycol` `refrigerant_proxy` `unknown` | r | opt | Seed `water` (hydronic coil) |
| `thermal_port_max_power_w` | f32 | watt | | r | opt | Seed 5000 (lab) |
| `thermal_port_attached_reservoir_id` | string | — | reservoir id or empty | r/w | opt | Attach/detach plant reservoir |

Fan uses `trait.fan`. Filter uses `trait.filter`. Multi-head: `trait.zone`.

---

### Class `dehumidifier`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `tank_full` | bool | — | | r/e | opt | Condensate tank full / stop compressor |
| `pump_mode` | bool | — | | r/w | opt | Continuous drain pump enabled |
| `defrost` | bool | — | | r/e | opt | Coil defrost active |
| `compressor_on` | bool | — | | r/e | opt | Compressor running |
| `high_rh_alarm` | bool | — | | r/e | opt | Room RH above high alarm |
| `low_rh_alarm` | bool | — | | r/e | opt | Room RH below low alarm |
| `continuous_mode` | bool | — | | r/w | opt | Ignore RH setpoint; run continuous |
| `quiet_mode` | bool | — | | r/w | opt | Reduced fan / sleep noise mode |
| `bucket_removed` | bool | — | | r/e | opt | Condensate bucket not seated |
| `filter_dirty` | bool | — | | r/e | opt | Filter dirty indicator light |
| `delayed_start_s` | duration_s | second | 0–86400 | r/w | opt | Delayed start countdown |

Humidity setpoint via `trait.humidity.setpoint_rh`; fan speed via
`trait.fan.fan_speed`.

---

### Class `humidifier`

| id | type | unit | range / enum | access | req | description |
|----|------|------|--------------|--------|-----|-------------|
| `output_level` | u8 | — | 1–10 | r/w | opt | Mist / output intensity |
| `mist_type` | enum | — | `cool` `warm` `steam` `evaporative` | r | opt | Hardware mist capability |
| `water_empty` | bool | — | | r/e | req | Dry tank / stop misting |
| `wick_state` | enum | — | `ok` `replace` `na` | r/e | opt | Evaporative wick condition |
| `warm_mist` | bool | — | | r/w | opt | Warm mist enabled (when dual-mode) |
| `auto_humidity` | bool | — | | r/w | opt | Follow `trait.humidity.setpoint_rh` |
| `mineral_filter` | bool | — | | r/e | opt | Demineralization cartridge needs replace |
| `uv_clean` | bool | — | | r/w | opt | UV sterilization lamp enabled |
| `scale_alert` | bool | — | | r/e | opt | Mineral scale buildup detected |
| `tank_removed` | bool | — | | r/e | opt | Water tank not seated |
| `misting` | bool | — | | r/e | opt | Currently producing mist |
| `night_mode` | bool | — | | r/w | opt | Quiet / dim night operation |

Humidity setpoint via `trait.humidity.setpoint_rh`.

---

## Capability advertisement (normative for later schema)

A device `describe` payload (see the standard overview) must be able to list:

1. `class_id` + `class_version` (semver of the class slice of this catalog)
2. `secondary_class_ids` if any
3. Each advertised `trait_id` + `trait_version`
4. For every advertised point: qualified id, type, unit, access, current
   range (may be tighter than this catalog), enum subset, zone list,
   resolution, and `required`/`optional`
5. Safety flags: `remote_start_supported`, `gas_remote_ignite` (default
   false), `rf_remote_start` (microwave, default false), `remote_vent`
   (multi-cooker, default false)

Validation rules the future core crate must implement:

| Write condition | Error |
|-----------------|-------|
| Unknown device id | `unknown_device` |
| Unknown trait or class point | `unknown_variable` |
| Trait not advertised | `unsupported_capability` |
| Type mismatch | `invalid_type` |
| Enum token not in advertised subset | `invalid_enum` |
| Numeric outside advertised range | `out_of_range` |
| Access lacks `w` | `not_writable` |
| Access lacks `r` | `not_readable` |
| `interlock_ok=false` on actuator command | `safety_interlock` |
| `remote_control_enabled=false` | `remote_disabled` |
| Cycle running and point is program-select | `busy` |
| Optional command not implemented | `unsupported_operation` |

---

## Naming rules (normative)

1. Ids are `snake_case`, ASCII `[a-z0-9_]+`, start with a letter, length ≤ 64.
2. Do not encode units in the id except when disambiguating (`current_c`,
   `spin_rpm`, `volume_ml`). Prefer a `unit` field plus a short id.
3. Booleans are positive (`ice_enabled`, not `ice_disabled`).
4. Commands are verbs: `start`, `pause`, `cancel`, `dispense`, `vent`.
5. State enums are nouns/adjectives: `running`, `full`, `clogged`.
6. Never reuse an id with a new type in a minor version.
7. Vendor ids must start with `vendor.` and a registered vendor slug.

---

## Typical phase tokens (non-exhaustive)

Devices may emit other `cycle_phase` strings; clients must tolerate unknowns.

| Domain | Tokens |
|--------|--------|
| Laundry wash | `fill` `prewash` `wash` `rinse` `spin` `drain` `soak` `steam` |
| Laundry dry | `heating` `drying` `cooling` `anti_crease` |
| Dish | `prewash` `wash` `rinse` `dry` |
| Pressure cook | `preheat` `pressurizing` `at_pressure` `cooking` `venting` `safe_to_open` |
| Coffee | `heating` `grinding` `brewing` `steaming` `rinsing` |
| Bread | `knead` `rise` `punch_down` `bake` |
| Vacuum | `pump` `seal` `vent` |

---

## Out of this catalog revision

- Exact JSON Schema / protobuf artifacts (later `homecooked-schema`)
- Transport bindings (later `homecooked-protocol`)
- User accounts, scenes, whole-home automations
- Raw service-technician registers
- Currency, recipes as documents, camera images
