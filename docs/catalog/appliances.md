# Appliance Class Catalog

Source of truth for HomeCooked appliance **classes**. Every class has a stable
`snake_case` id. Code that later generates schema types must track this file
together with [`variables-and-settings.md`](./variables-and-settings.md).

This catalog lists kitchen and whitegoods appliances that expose a
**controllable-settings / readable-state** interface: discover the device, read
telemetry, write settings, and issue commands. Classes that are only passive
(no useful state or settings) are omitted.

Related documents:

- [`variables-and-settings.md`](./variables-and-settings.md) — traits, variables,
  settings, commands, types, units, ranges, access modes.
- [`ADDING_A_CLASS.md`](./ADDING_A_CLASS.md) — step-by-step for adding a new
  class (docs → `ids.rs` → `ClassTable` → sim → WASM).
- [`../standard/overview.md`](../standard/overview.md) — catalog → schema → wire
  protocol, versioning, and extensions.

Catalog version: **0.1.0** (docs-only; no schema crate in this revision).

---

## Conventions

| Rule | Detail |
|------|--------|
| Class id | Stable `snake_case` token, unique in this catalog. Never rename in a minor version. |
| Human name | Informal English label; not used on the wire. |
| Traits | Reusable capability bundles a class typically advertises. A device may omit a trait it does not implement. |
| Composition | Combo products advertise a primary class plus extra traits (and optionally extra class ids). See [Combo devices](#combo-devices). |
| Settings vs state | Settings are writable (sometimes also readable). State is readable and usually evented. Commands are write-only actions. |
| Ranges | Typical consumer ranges, not certified limits. Devices advertise actual ranges in capabilities. |
| Safety | Heating, water, motion, gas, and high-power actuators require interlocks. Remote start is opt-in. |
| Variants | Documented in Notes, not as extra class ids, unless controls genuinely diverge. |
| Out of catalog | Vendor-only gadgets use `vendor.<vendor_id>.*` class ids. See the standard overview. |

Access to variables, units, and enums lives in the variables catalog. This file
answers: *what is this class, what do you typically set, what do you typically
read, and what can go wrong.*

---

## Index

| id | Name | Group |
|----|------|-------|
| `washer` | Washer | Laundry |
| `dryer` | Dryer | Laundry |
| `washer_dryer` | Washer-dryer combo | Laundry |
| `fridge` | Refrigerator | Cold |
| `freezer` | Freezer | Cold |
| `fridge_freezer` | Fridge-freezer | Cold |
| `wine_cooler` | Wine cooler | Cold |
| `beverage_cooler` | Beverage cooler | Cold |
| `ice_maker` | Ice maker | Cold |
| `kegerator` | Kegerator | Cold |
| `dishwasher` | Dishwasher | Wash |
| `microwave` | Microwave | Cooking |
| `oven` | Oven | Cooking |
| `steam_oven` | Steam oven | Cooking |
| `toaster_oven` | Toaster oven | Cooking |
| `range` | Range | Cooking |
| `cooktop` | Cooktop | Cooking |
| `induction_hob` | Induction hob | Cooking |
| `warming_drawer` | Warming drawer | Cooking |
| `pizza_oven` | Pizza oven | Cooking |
| `air_fryer` | Air fryer | Cooking |
| `electric_grill` | Electric grill | Cooking |
| `electric_smoker` | Electric smoker | Cooking |
| `range_hood` | Range hood | Ventilation |
| `coffee_machine` | Automatic coffee machine | Beverage |
| `espresso_machine` | Espresso machine | Beverage |
| `drip_coffee_maker` | Drip coffee maker | Beverage |
| `coffee_grinder` | Coffee grinder | Beverage |
| `kettle` | Kettle | Beverage |
| `water_dispenser` | Water dispenser | Beverage |
| `toaster` | Toaster | Countertop |
| `blender` | Blender | Countertop |
| `food_processor` | Food processor | Countertop |
| `stand_mixer` | Stand mixer | Countertop |
| `juicer` | Juicer | Countertop |
| `rice_cooker` | Rice cooker | Countertop |
| `slow_cooker` | Slow cooker | Countertop |
| `multi_cooker` | Multi-cooker / Instant Pot | Countertop |
| `sous_vide` | Sous-vide / immersion circulator | Countertop |
| `bread_maker` | Bread maker | Countertop |
| `dehydrator` | Dehydrator | Countertop |
| `vacuum_sealer` | Vacuum sealer | Countertop |
| `ice_cream_maker` | Ice cream maker | Countertop |
| `yogurt_maker` | Yogurt maker | Countertop |
| `waffle_maker` | Waffle maker | Countertop |
| `pasta_maker` | Pasta maker | Countertop |
| `steam_cooker` | Steam cooker | Countertop |
| `garbage_disposal` | Garbage disposal | Utility |
| `trash_compactor` | Trash compactor | Utility |
| `water_heater` | Water heater | Utility |
| `boiler` | Boiler | Utility |
| `water_softener` | Water softener | Utility |
| `water_filter` | Water filter | Utility |
| `hvac` | HVAC | Climate |
| `dehumidifier` | Dehumidifier | Climate |
| `humidifier` | Humidifier | Climate |

---

## Combo devices

A physical product may implement more than one class. HomeCooked models that as
**composition**, not a new ad-hoc class, except where the combination is a
widely shipped product with fused controls.

| Pattern | How to advertise |
|---------|------------------|
| Washer-dryer combo | Class `washer_dryer`, which includes washer + dryer traits. May also advertise `washer` and `dryer` as secondary classes if the two drums are independently controllable. |
| Fridge-freezer | Class `fridge_freezer` with two `zone`s (`fridge`, `freezer`). Independent cabinets that share a chassis still use this class. |
| Range (cooktop + oven) | Class `range`. Exposes hob zones via `cooktop` / `induction_hob` traits plus `oven` traits. A dual-oven range uses two oven zones. |
| Microwave-oven combo | Primary `oven` or `microwave` plus the other class as secondary, or a vendor class if the cavity is truly dual-mode. Prefer two classes on one device over a new core id. |
| Coffee machine with grinder | `coffee_machine` or `espresso_machine` plus trait `grind` (see coffee classes). Do not require a separate `coffee_grinder` device unless it is a standalone unit. |
| Fridge with ice / water | `fridge` or `fridge_freezer` plus traits `ice` and `dispense`. |
| HVAC with dehumidifier | `hvac` plus trait `humidity` and optional secondary class `dehumidifier` if the dehumidifier is independently scheduled. |

Rules:

1. One device id, one network endpoint, one or more class ids.
2. Variables are namespaced by trait or class so fridge and freezer setpoints do not collide (`zone` + variable id).
3. Safety interlocks are per actuator (door, heater, motor, gas valve), not per chassis.
4. Do not mint a new core class because two products share a badge or an app.

---

## Laundry

### `washer`

Front- or top-loading clothes washer. Washes, rinses, and spins a textile load
using water, detergent, and a rotating drum.

**Typical traits:** `identity`, `power`, `connectivity`, `time_schedule`,
`door_lid`, `child_lock`, `cycle`, `program`, `water`, `temperature`, `motor`,
`fault`, `energy`, `remote`, `maintenance`, `audio`, `safety`.

**Typical controllable settings:**

- Program (`cotton`, `eco`, `wool`, `delicates`, `quick`, `rinse`, `spin`,
  `bedding`, `allergy`, `outdoor`, `custom`, …)
- Wash temperature setpoint (°C) or named band (`cold`, `warm`, `hot`)
- Spin speed (rpm) or `spin_off` / `rinse_hold`
- Soil / stain level, load size, extra rinse, prewash, steam, soak
- Detergent and softener dose (if auto-dosing)
- Delay start, remote start enable, child lock, end-of-cycle signal
- Water hardness override (if not using `water_softener` telemetry)

**Typical readable state:**

- Cycle state (`idle`, `delayed`, `running`, `paused`, `rinsing`, `spinning`,
  `draining`, `complete`, `error`)
- Phase, progress percent, remaining / elapsed time
- Drum / door lock, door open, water level, inlet valve, drain pump
- Drum rpm, motor current, tub temperature
- Detergent / softener / bleach reservoir levels
- Unbalance, leak, overflow, drain-fail, door-unlocked-while-running
- Energy and water consumed this cycle / lifetime
- Fault codes and maintenance (clean drum, clean filter)

**Notes:**

- Safety: door must stay locked above a water-level and rpm threshold. Remote
  start is refused unless `remote_start_enabled` is true and the door is closed.
- Do not command spin rpm above the advertised max for the selected program.
- Unbalance may automatically limit spin; that is device policy, not a protocol
  error.
- Heat-pump or steam washers still use this class; extra traits `heater` /
  `humidity` are optional.
- Compact / portable washers omit auto-dose and sometimes temperature control.

### `dryer`

Tumble dryer (vented, condenser, or heat-pump) that removes moisture from a
textile load.

**Typical traits:** `identity`, `power`, `connectivity`, `time_schedule`,
`door_lid`, `child_lock`, `cycle`, `program`, `temperature`, `humidity`,
`heater`, `fan`, `filter`, `fault`, `energy`, `remote`, `maintenance`, `audio`,
`safety`.

**Typical controllable settings:**

- Program (`cotton`, `synthetic`, `delicates`, `wool`, `timed`, `air_fluff`,
  `bedding`, `hygiene`, `rack`, …)
- Dryness target (`iron`, `cupboard`, `extra`) or timed duration
- Temperature / heat level, anti-crease / wrinkle-prevent
- Steam refresh, delay start, child lock, remote start
- Eco / energy mode, drum light (if present)

**Typical readable state:**

- Cycle state and phase (`heating`, `drying`, `cooling`, `anti_crease`,
  `complete`)
- Remaining time, dryness estimate, exhaust / drum temperature
- Lint filter present / clogged, condenser / drain tank full
- Door, child lock, heater on, fan rpm
- Energy this cycle, fault codes (overtemp, no-tumble, blocked airflow)

**Notes:**

- Safety: over-temperature cutout is device-local and must be readable as a
  fault. Remote start requires closed door and empty drain tank (condenser).
- Heat-pump dryers advertise lower heater power and a `heat_pump` option flag;
  they are not a separate class.
- Gas dryers may expose an extra `gas_valve` safety bit; still class `dryer`.
- Never model “smart rack dryers” without a drum as `dryer` if they have no
  tumbling motor — use a vendor class or `dehydrator` if they are cabinets.
- Optional `thermal_port_*` class points advertise an exhaust / heat-reject
  source into the plant (not a parallel thermal class; see thermal-plant).

### `washer_dryer`

Single-cabinet washer with a drying function on the same drum (true combo), or
a stacked pair that is sold and addressed as one endpoint.

**Typical traits:** All of `washer` plus `dryer` drying traits (`humidity`,
`heater`, `filter`) and a `wash_then_dry` program option.

**Typical controllable settings:**

- Everything on `washer`
- Dry after wash (bool), dryness target, max dry time
- Combined programs (`wash_and_dry`, `wash_only`, `dry_only`)

**Typical readable state:**

- Washer state plus dryer phase when drying
- Combined remaining time for wash+dry
- Lint filter and drain-tank state (often more constrained than a standalone dryer)

**Notes:**

- Prefer this class when there is **one drum** and one cycle that can wash then
  dry. Independent stacked machines behind one Wi-Fi module should advertise
  two device ids (`washer` and `dryer`) if they can run independently.
- Drying capacity is usually smaller than wash capacity; devices should reject
  `dry_after_wash` when load exceeds advertised dry mass.
- Water used for condenser drying is still `water` trait telemetry.

---

## Cold

### `fridge`

Refrigerated fresh-food cabinet without a frozen compartment (or with only an
ice box that is not a controllable freezer zone).

**Typical traits:** `identity`, `power`, `connectivity`, `door_lid`,
`temperature`, `zone`, `lighting`, `fault`, `energy`, `remote`, `maintenance`,
`filter`, `ice` (optional), `dispense` (optional), `child_lock`, `audio`.

**Typical controllable settings:**

- Compartment setpoint (°C), vacation / sabbath / eco mode
- Super-cool (temporary pull-down)
- Door alarm enable, interior light, ice maker on/off (if present)
- Water filter reset after replacement

**Typical readable state:**

- Current air / evap temperature, compressor running, defrost cycle
- Door open per door, open duration, door alarm
- Ice bin level, water filter life
- Energy, fault (sensor fail, high temp, leak)

**Notes:**

- Setpoint range is typically about 1–7 °C. Writes outside advertised range are
  `out_of_range`.
- Optional `thermal_port_*` class points advertise a condenser heat port for
  plant coupling (not a parallel thermal class; see thermal-plant).
- A fridge that also has a freezer compartment **must** use `fridge_freezer`
  (or expose a `freezer` zone). Do not overload `fridge` with a freezer
  setpoint.
- Convertible “fridge or freezer” cabinets are `fridge_freezer` with a zone
  mode setting.

### `freezer`

Dedicated frozen-food cabinet (upright or chest).

**Typical traits:** `identity`, `power`, `connectivity`, `door_lid`,
`temperature`, `zone`, `lighting`, `fault`, `energy`, `remote`, `maintenance`,
`child_lock`, `audio`.

**Typical controllable settings:**

- Setpoint (°C), super-freeze / fast-freeze
- Door alarm enable, eco / vacation, sabbath
- Chest freezers: little else; uprights may have lights and anti-sweat heaters

**Typical readable state:**

- Current temperature, compressor, defrost
- Door / lid state, high-temp alarm, power-fail timestamp
- Energy, frost-build suggestion (maintenance)

**Notes:**

- Typical setpoint −24 to −16 °C. Chest lids map to `door_lid` (open = lid up).
- Medical / laboratory freezers are out of core catalog (vendor class) because
  alarm and validation requirements differ.
- Drawer freezers still use this class; multiple drawers are `zone`s.

### `fridge_freezer`

Combined fresh and frozen storage, including side-by-side, bottom-freezer,
French-door, and multi-door cabinets.

**Typical traits:** `fridge` + `freezer` via two or more `zone`s, plus optional
`ice`, `dispense`, `filter`, `humidity` (crisper).

**Typical controllable settings:**

- Per-zone setpoints and super-cool / super-freeze
- Convertible-zone mode (`fridge` / `freezer` / `off`)
- Ice maker, crushed/cubed, water dispense temperature (if chilled)
- Sabbath, vacation, door alarm, lights, eco

**Typical readable state:**

- Per-zone temperatures and door states
- Ice production state, dispenser tray, drip tray
- Filter life, energy, faults per loop if dual-evaporator

**Notes:**

- Always model compartments as `zone`s (`fridge`, `freezer`, `convertible`,
  `bar`, `pantry`, `crisper`). Do not flatten two setpoints into one variable.
- Dual independent appliances in one kitchen are two devices, not this class.

### `wine_cooler`

Temperature-controlled cabinet for bottled wine, often dual-zone.

**Typical traits:** `identity`, `power`, `connectivity`, `door_lid`,
`temperature`, `zone`, `humidity`, `lighting`, `child_lock`, `fault`, `energy`,
`audio`.

**Typical controllable settings:**

- Per-zone setpoint (typically 5–20 °C), lighting, sabbath, humidity target
  (if actively humidified), vibration-reduction / compressor night mode,
  UV protect

**Typical readable state:**

- Per-zone temperature and humidity, door, compressor, high/low temp alarm,
  vibration alert, bottle-count estimate
- UV protect / interior light (optional), filter if present

**Notes:**

- Not a `fridge`: ranges, humidity, and vibration matter; freezing is a fault.
- Dual-zone is expected. Single-zone devices advertise one zone.
- Catalog depth: optional class points include sabbath, compressor, temp alarms,
  vibration alert, and `bottle_count` (see variables-and-settings).

### `beverage_cooler`

Undercounter or mini cooler for cans and bottles at serving temperature. No
freezer. Distinct from `wine_cooler` because humidity/vibration are not primary
and setpoints run colder (often 1–10 °C).

**Typical traits:** `identity`, `power`, `connectivity`, `door_lid`,
`temperature`, `lighting`, `fault`, `energy`.

**Typical controllable settings:** setpoint, light, eco, door alarm.

**Typical readable state:** temperature, door, compressor, energy.

**Notes:** Mini-fridges with a tiny freezer box stay `fridge` if that box is
uncontrolled, or `fridge_freezer` if it has a setpoint.

### `ice_maker`

Stand-alone automatic ice machine (undercounter, countertop, or modular), not
the ice maker inside a fridge.

**Typical traits:** `identity`, `power`, `connectivity`, `door_lid`, `ice`,
`water`, `filter`, `fault`, `energy`, `maintenance`, `child_lock`.

**Typical controllable settings:**

- Ice production on/off, ice size / type (`cube`, `nugget`, `clear`), max bin
  level, clean cycle, water filter reset, delayed start

**Typical readable state:**

- Making / harvest / full / off, bin level, water supply, inlet temperature
- Clean needed, scale, low water, harvest fail, filter life

**Notes:**

- Fridge-integrated ice uses trait `ice` on `fridge` / `fridge_freezer`, not
  this class.
- Requires potable water. Leak and no-water are safety-relevant faults.
- Never command a clean cycle with ice in the bin if the device forbids it;
  device returns `busy` or `safety_interlock`.

### `kegerator`

Refrigerated keg cabinet with optional CO₂ / dispense controls.

**Typical traits:** `identity`, `power`, `connectivity`, `door_lid`,
`temperature`, `dispense`, `lighting`, `fault`, `energy`.

**Typical controllable settings:** beer temperature setpoint, light, maybe
CO₂ solenoid enable (if instrumented).

**Typical readable state:** temperature, door, compressor, keg empty (if
load-cell or flow), CO₂ pressure (optional).

**Notes:** Gas pressure writes are vendor extensions unless a device advertises
a calibrated `co2_pressure_kpa` setting. Uncontrolled party fridges are
`beverage_cooler`.

---

## Wash

### `dishwasher`

Automatic dishwasher (built-in, drawer, or countertop).

**Typical traits:** `identity`, `power`, `connectivity`, `time_schedule`,
`door_lid`, `child_lock`, `cycle`, `program`, `water`, `temperature`, `heater`,
`fault`, `energy`, `remote`, `maintenance`, `audio`, `safety`, `filter`.

**Typical controllable settings:**

- Program (`auto`, `eco`, `intensive`, `quick`, `glass`, `rinse`, `hygiene`,
  `night`, `custom`)
- Options: extra dry, extra rinse, half load, zone wash, steam, sanitize,
  delay start, tab vs powder/rinse-aid dosing
- Water hardness, rinse-aid dose, child lock, remote start

**Typical readable state:**

- Cycle state / phase (`prewash`, `wash`, `rinse`, `dry`, `complete`)
- Remaining time, water temperature, turbidity (auto programs)
- Door / lock, leak tray, drain, inlet
- Rinse-aid and salt / regenerant levels, filter clogged
- Energy and water this cycle, faults (drain, heat, leak, spray-arm)

**Notes:**

- Safety: door lock during wash; leak sensor trips `safety_interlock` and
  typically forces drain. Remote start needs closed door.
- Optional `thermal_port_*` class points advertise a DHW inlet-preheat sink
  (not a parallel thermal class; see thermal-plant).
- Drawer dishwashers: one device with two `zone`s if independently runnable,
  else one cycle.
- Do not confuse with `washer` (laundry). Different programs and no spin rpm.

---

## Cooking

### `microwave`

Microwave oven (solo, grill, or inverter). Combination microwave-convection
cavities that are primarily microwaves stay this class and add convection
options; a true dual oven should also advertise `oven`.

**Typical traits:** `identity`, `power`, `connectivity`, `door_lid`, `cycle`,
`program`, `heater` (grill / convection), `lighting`, `audio`, `child_lock`,
`fault`, `energy`, `remote`, `safety`.

**Typical controllable settings:**

- Time, power level (percent or watts), program / sensor cook, defrost mass
- Grill / convection / combination mode if present
- Inverter vs duty-cycle is device-internal; expose `power_level_percent`
- Kitchen timer, child lock, clock, light, turntable on/off

**Typical readable state:**

- Running / paused / idle / door-open, remaining time, power level
- Cavity temperature if convection, magnetron / inverter fault
- Door interlock, child lock

**Notes:**

- Safety: **door interlock is mandatory**. Writes that start RF with door open
  are `safety_interlock`. Remote start is commonly forbidden or requires a
  local confirmation flag.
- Never expose raw magnetron control. Power level and time only.
- Over-the-range microwaves may also advertise `range_hood` as a secondary
  class for the vent fan / light.

### `oven`

Thermal oven cavity (conventional, convection, European range oven, wall oven).
Does not include microwave-only cavities (`microwave`) or dedicated steam-only
cavities (`steam_oven`).

**Typical traits:** `identity`, `power`, `connectivity`, `door_lid`,
`temperature`, `cycle`, `program`, `heater`, `fan`, `lighting`, `child_lock`,
`fault`, `energy`, `remote`, `time_schedule`, `safety`, `audio`.

**Typical controllable settings:**

- Mode (`bake`, `convection_bake`, `broil`, `convection_roast`, `proof`,
  `keep_warm`, `self_clean`, `pyrolytic`, `air_fry` if offered)
- Setpoint °C, timer / duration, delay start, probe target
- Steam assist percent (if hybrid), Sabbath, child lock, light
- Self-clean start (local confirmation strongly recommended)

**Typical readable state:**

- Cavity temperature, preheat complete, mode, remaining time
- Door, lock (especially during pyrolytic), broiler / bake element on
- Meat-probe temperature, steam generator water level (hybrid)
- Faults: sensor, overtemp, door-lock fail

**Notes:**

- Typical setpoint 50–250 °C (broil may be an element percent, not °C).
- Pyrolytic clean locks the door and ignores setpoint writes until complete.
- Double wall ovens: one device, two `zone`s (`upper`, `lower`), each with oven
  variables.
- Gas ovens add ignition / flame-out faults; still this class.

### `steam_oven`

Cavity whose primary heat/moisture source is steam (combi-steam included).

**Typical traits:** `oven` traits plus `water`, `humidity`, `filter` (descale).

**Typical controllable settings:**

- Steam / combi / convection / sous-vide modes
- Temperature, humidity or steam intensity, duration
- Descale / rinse, water hardness

**Typical readable state:**

- Cavity temp and humidity, water tank level, drain, descale due
- Door, cycle phase, generator faults, overheat

**Notes:**

- Hybrid ovens that steam-assist a conventional bake still use `oven` with a
  steam option. Use `steam_oven` when steam is a first-class mode family.
- Empty tank is `busy` / fault, not a successful start.

### `toaster_oven`

Countertop oven / toaster oven / mini convection oven, including many
“air-fryer toaster ovens”.

**Typical traits:** subset of `oven` plus toaster programs. Usually no pyrolytic
clean, no meat probe.

**Typical controllable settings:** toast shade, bake/broil/air-fry temp and
time, convection fan, light.

**Typical readable state:** running, remaining, cavity temp (if sensed), crumb
tray missing (optional), door.

**Notes:**

- Distinct from `toaster` (slots, no cavity setpoint) and `air_fryer` (basket).
- If the SKU is a basket air fryer with no bake cavity, use `air_fryer`.

### `range`

Freestanding or slide-in range: cooktop + oven in one appliance.

**Typical traits:** union of `cooktop` or `induction_hob` and `oven`. Optional
second oven zone, warming drawer as extra class or zone, `range_hood` only if
the hood is integrated and addressable.

**Typical controllable settings / state:** see `cooktop` / `induction_hob` and
`oven`, namespaced by zone (`hob_1` … `hob_n`, `oven`, `oven_lower`).

**Notes:**

- Gas, electric coil, radiant glass, and mixed-fuel ranges all use `range`.
  Surface technology is a capability flag (`heat_source`: `gas`, `electric`,
  `radiant`, `induction`, `mixed`).
- If the cooktop is induction, advertise trait set of `induction_hob` on the
  hob zones rather than inventing `induction_range`.
- Oven door lock and hob residual-heat are independent safety bits.

### `cooktop`

Cooktop / hob that is not specifically induction (gas, electric coil, radiant
ceramic). Built-in independent of an oven.

**Typical traits:** `identity`, `power`, `connectivity`, `child_lock`, `heater`,
`lighting` (optional), `fault`, `energy`, `safety`, `zone`.

**Typical controllable settings:**

- Per-zone power level (0–9, 0–17, or percent — device advertises the enum or
  range), boost, timer per zone, bridge / combo zones
- Pause all, child lock, keep-warm

**Typical readable state:**

- Per-zone power, residual heat (`hot_surface`), pan detect (electric)
- Gas: flame on, ignition fail, flame-out
- Total power limit / load shedding active

**Notes:**

- Use `induction_hob` when the surface is induction. Mixed surfaces on one
  glass may use `cooktop` with per-zone `heat_source`.
- Safety: residual-heat must remain readable with main power “off” if the UI
  still warns. Child lock blocks level writes.
- Remote start of a gas burner is **out of default policy**; devices may refuse
  with `safety_interlock` unless a local “remote cook enabled” setting exists.

### `induction_hob`

Induction cooktop. Separate class because pan-detect, power-sharing, and
boost/no-pan behavior differ from gas/radiant.

**Typical traits:** `cooktop` traits plus induction-specific pan and limiter
variables.

**Typical controllable settings:**

- Per-zone power or simulated temperature, boost, flex-zone grouping
- Total power cap, pan-size assist, timer, pause, child lock
- Some units: pan-temperature probe / sous-vide mode

**Typical readable state:**

- Per-zone: power W, requested level, pan present, pan size, coil temp
- Residual heat, limiter active, cookware incompatible
- Energy

**Notes:**

- Writing a level with no pan typically results in a timeout to off and an
  event `pan_missing` — not necessarily an error on write.
- Power-share: a write that exceeds the cabinet cap is accepted and clamped, or
  rejected `out_of_range` if the device advertises strict mode. Devices must
  document which.

### `warming_drawer`

Heated drawer for plates or food holding.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`,
`door_lid`, `fault`, `energy`, `safety`.

**Typical controllable settings:** mode (`low`, `medium`, `high` or °C), timer,
moist/crisp vent if present.

**Typical readable state:** on, setpoint, drawer open, overtemp.

**Notes:** Often a zone of `range` rather than a standalone device. Either is
valid; standalone built-ins use this class.

### `pizza_oven`

Dedicated high-temperature pizza oven (countertop electric or instrumented
gas/wood with electronic control).

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`heater`, `door_lid`, `fault`, `energy`, `safety`.

**Typical controllable settings:** stone / deck setpoint (often 200–450 °C),
timer, top/bottom balance, steam burst (optional).

**Typical readable state:** stone temp, dome temp, preheat ready, door, heater.

**Notes:** Consumer outdoor Ooni-style units with Bluetooth fit here if they
expose a setpoint. Uncontrolled wood-fired ovens without electronics are out of
catalog.

### `air_fryer`

Basket or drawer hot-air fryer.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`program`, `heater`, `fan`, `door_lid`, `fault`, `energy`, `audio`, `safety`.

**Typical controllable settings:** temperature, time, preset (`fries`, `wings`,
`reheat`, `bake`, `dehydrate` if offered), shake reminder, preheat.

**Typical readable state:** running, remaining, cavity temp, basket present,
shake event, done.

**Notes:**

- Dual-basket units use two `zone`s with optional sync-finish.
- A toaster-oven that air-fries is `toaster_oven` with an `air_fry` program,
  not this class.

### `electric_grill`

Indoor contact grill, George-Foreman-style, or countertop electric grill with
plates.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`heater`, `fault`, `energy`, `safety`.

**Typical controllable settings:** plate setpoint or doneness program, timer,
sear boost.

**Typical readable state:** plate temps (top/bottom), lid, grease tray missing,
done.

**Notes:** Outdoor gas grills with Wi-Fi probes may use this class plus vendor
gas variables, or a vendor class if burners are the primary interface.

### `electric_smoker`

Electric smoker cabinet with temperature and optional smoke generator.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`heater`, `humidity` (optional), `fault`, `energy`, `time_schedule`, `safety`.

**Typical controllable settings:** cabinet setpoint, duration, smoke on/off or
wood-tray heater, probe targets, delay start.

**Typical readable state:** cabinet temp, probe temps, smoke generating, door,
element, water pan (optional).

**Notes:** Pellet grills are vendor-adjacent; if they expose PID temp + pellet
auger they may still use this class with a `fuel_level` vendor variable.

---

## Ventilation

### `range_hood`

Extractor hood / cooker hood over a hob: fan, lights, sometimes auto-boost
linked to hob.

**Typical traits:** `identity`, `power`, `connectivity`, `fan`, `lighting`,
`filter`, `fault`, `energy`, `remote`, `audio`.

**Typical controllable settings:**

- Fan speed (off / 1–n / boost / auto), light on/dim, delay-off
- Auto mode (follow hob or VOC/temp sensor), grease-filter reset

**Typical readable state:**

- Fan speed, boost remaining, air quality / VOC if sensed
- Grease filter life / present, charcoal filter life (recirculating)
- Light, motor fault, overtemp

**Notes:**

- Over-the-range microwave ventilation is this class as **secondary** on a
  `microwave` device, or a separate endpoint if the hood is independent.
- Make-up air dampers are vendor extensions.
- Boost usually auto-expires; that is expected, not a write failure.

---

## Beverage

### `coffee_machine`

Automatic bean-to-cup / super-automatic machine (grind, tamp, brew, milk in one
box). Distinct from `espresso_machine` (semi-auto group head) and
`drip_coffee_maker` (batch filter).

**Typical traits:** `identity`, `power`, `connectivity`, `water`, `temperature`,
`cycle`, `program`, `child_lock`, `fault`, `energy`, `maintenance`, `filter`,
`audio`, `lighting` (optional).

**Typical controllable settings:**

- Drink (`espresso`, `double`, `americano`, `lungo`, `cappuccino`, `latte`,
  `hot_water`, `steam`, custom)
- Strength, grind level (if exposed), volume ml, temperature, milk volume
- Cup preheat, 1 vs 2 cups, user profiles
- Rinse, descale start, water hardness, auto-off timeout

**Typical readable state:**

- Ready / heating / brewing / steaming / rinsing / off / water-empty /
  grounds-full / descale-needed
- Water tank, drip tray, grounds bin, milk present / empty
- Boiler temp, brew pressure (optional), shot in progress
- Maintenance counters

**Notes:**

- Safety: hot water / steam writes require ready state; drip-tray-full may
  block brew (`busy`).
- Capsule machines (Nespresso-style) use this class with `grind` omitted and a
  `capsule` present bit if instrumented.
- Do not merge with `espresso_machine`: workflows and settings differ.

### `espresso_machine`

Semi-automatic or prosumer espresso machine (portafilter, independent steam
wand). Includes many dual-boiler connected machines.

**Typical traits:** `identity`, `power`, `connectivity`, `water`, `temperature`,
`cycle`, `fault`, `energy`, `maintenance`, `safety`.

**Typical controllable settings:**

- Boiler / brew temp, steam temp, pre-infusion, shot timer / volumetric stop
  (ml or pulses), pump on (if the API allows group control)
- Flush, descale, water source (tank vs plumbed)

**Typical readable state:**

- Brew and steam boiler temps, pressure, shot elapsed, heating
- Water level, scale, overtemp, group idle/brewing

**Notes:**

- Direct pump-on without a volumetric goal is a **safety-sensitive** command
  (flood). Prefer timed / volumetric shots. Devices may require
  `remote_brew_enabled`.
- Standalone grinders are `coffee_grinder`.

### `drip_coffee_maker`

Batch drip / filter coffee machine, including heat-plate and thermal-carafe
connected models.

**Typical traits:** `identity`, `power`, `connectivity`, `water`, `temperature`,
`cycle`, `time_schedule`, `fault`, `energy`, `maintenance`, `audio`.

**Typical controllable settings:** brew now, cups / volume, strength, keep-warm
time, bloom, schedule, carafe preheat.

**Typical readable state:** brewing, keep-warm, water empty, carafe present,
keep-warm remaining, descale.

**Notes:** Carafe-missing should inhibit brew (`safety_interlock` or `busy`).
Cold-brew pitchers without heaters are out of core catalog.

### `coffee_grinder`

Standalone burr grinder.

**Typical traits:** `identity`, `power`, `connectivity`, `motor`, `cycle`,
`fault`, `energy`, `child_lock` (optional).

**Typical controllable settings:** grind time or dose grams, grind size ticks,
single/double, RPM if variable.

**Typical readable state:** grinding, remaining, hopper present, clog, motor
stall.

**Notes:** Built-in grinders on `coffee_machine` use trait-level grind
settings, not this class.

### `kettle`

Electric kettle, including variable-temperature and gooseneck models.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`,
`heater`, `cycle`, `fault`, `energy`, `safety`, `audio`, `child_lock`
(optional), `keep_warm` via temperature trait.

**Typical controllable settings:** setpoint °C (typically 40–100), start/cancel
boil, keep-warm, hold time.

**Typical readable state:** current water temp, boiling, keep-warm, lifted off
base, empty, boil-dry trip.

**Notes:**

- Safety: boil-dry and off-base are interlocks. A write to heat while off-base
  is `safety_interlock`.
- No water-level sensor is common; `water_level` is optional.

### `water_dispenser`

Point-of-use water dispenser (hot / cold / ambient), bottle or plumbed.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`,
`dispense`, `water`, `child_lock`, `filter`, `fault`, `energy`, `safety`.

**Typical controllable settings:** hot setpoint, cold setpoint, dispense
volume, hot-lock, filter reset, energy saving.

**Typical readable state:** tank temps, dispensing, bottle empty, drip tray,
filter life, overtemp.

**Notes:** Fridge door dispensers use trait `dispense` on the fridge, not this
class. Child lock on hot water is expected.

---

## Countertop

### `toaster`

Slot toaster.

**Typical traits:** `identity`, `power`, `connectivity`, `cycle`, `heater`,
`fault`, `energy`, `audio`, `safety`.

**Typical controllable settings:** shade 1–n, bagel, frozen, defrost, cancel,
single-side.

**Typical readable state:** toasting, remaining (if timed), carriage down,
done, jam / overtemp.

**Notes:** No cavity temperature. Toaster ovens are `toaster_oven`. Lift-to-
cancel is a command (`cancel`) plus state `carriage`.

### `blender`

Jar blender (countertop). Immersion / stick blenders with no jar are a variant
flag (`form_factor: immersion`) if they are connected at all.

**Typical traits:** `identity`, `power`, `connectivity`, `motor`, `cycle`,
`program`, `door_lid` (lid), `fault`, `energy`, `safety`, `audio`.

**Typical controllable settings:** speed 1–n or rpm, timed run, pulse, presets
(`smoothie`, `ice_crush`, `soup` if heated), heat target for heated blenders.

**Typical readable state:** running, speed, jar present, lid locked, motor
temp / stall, remaining.

**Notes:** Safety: lid and jar interlocks. Heated blenders add `temperature`
and `heater`. Do not start with lid open.

### `food_processor`

Bowl-and-blade food processor, optional dicing / slicing attachments.

**Typical traits:** `identity`, `power`, `connectivity`, `motor`, `cycle`,
`door_lid`, `fault`, `energy`, `safety`.

**Typical controllable settings:** speed, pulse, timed run, program.

**Typical readable state:** running, bowl / lid / pusher interlock, stall.

**Notes:** Attachment type may be an enum if sensed; otherwise omit.

### `stand_mixer`

Planetary stand mixer.

**Typical traits:** `identity`, `power`, `connectivity`, `motor`, `cycle`,
`fault`, `energy`, `safety`, `lighting` (bowl light, optional).

**Typical controllable settings:** speed 1–n, timed mix, direction (if allowed),
pause.

**Typical readable state:** running, speed, bowl present, head-up interlock,
stall, remaining.

**Notes:** Head-up or missing bowl is `safety_interlock`. Smart scales on the
bowl are a vendor extension or optional `mass_g` read.

### `juicer`

Centrifugal or masticating juicer.

**Typical traits:** `identity`, `power`, `connectivity`, `motor`, `cycle`,
`fault`, `energy`, `safety`.

**Typical controllable settings:** on/speed, pulse, reverse (masticating).

**Typical readable state:** running, pulp bin full, juice jug present, jam /
reverse needed.

### `rice_cooker`

Rice cooker / fuzzy-logic cooker, including many multi-grain cookers that are
not full pressure multi-cookers.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`program`, `heater`, `fault`, `energy`, `time_schedule`, `audio`.

**Typical controllable settings:** program (`white`, `brown`, `sushi`, `porridge`,
`steam`, `keep_warm`), delay start, texture, keep-warm.

**Typical readable state:** cooking / steaming / keep-warm / idle, remaining
(estimate), lid, bowl present, boil-dry.

**Notes:** Pressure rice cookers that expose pressure cook programs should use
`multi_cooker` instead.

### `slow_cooker`

Low-and-slow crock / slow cooker.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`heater`, `time_schedule`, `fault`, `energy`, `audio`.

**Typical controllable settings:** heat (`low`, `high`, `warm`) or °C, duration,
delay, keep-warm after.

**Typical readable state:** running, remaining, probe temp (optional), lid.

**Notes:** If the product is a pressure multi-cooker, use `multi_cooker`.

### `multi_cooker`

Electric pressure multi-cooker (Instant Pot and similar): pressure, sauté,
slow, rice, yogurt, steam in one pot.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`program`, `heater`, `door_lid`, `child_lock`, `fault`, `energy`, `time_schedule`,
`safety`, `audio`.

**Typical controllable settings:**

- Program (`pressure`, `saute`, `slow`, `steam`, `rice`, `yogurt`, `sous_vide`,
  `keep_warm`, `sterilize`)
- Pressure band (`low`, `high`) or kPa, setpoint °C, duration, delay
- Keep-warm, vent / quick-release command if the device allows **and**
  advertises it (many will refuse remote vent)

**Typical readable state:**

- Phase (`preheat`, `pressurizing`, `at_pressure`, `cooking`, `keep_warm`,
  `safe_to_open`)
- Pot temp, pressure, remaining, lid locked, float valve
- Faults: overpressure, burn / high-temp, lid mismatch

**Notes:**

- Safety: lid lock while pressurized. Remote quick-release is default-deny
  (`safety_interlock`) unless `remote_vent_enabled`.
- Distinct from `slow_cooker` and `rice_cooker`.

### `sous_vide`

Immersion circulator or dedicated sous-vide bath.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`heater`, `fan` (pump), `fault`, `energy`, `time_schedule`, `safety`, `audio`.

**Typical controllable settings:** water setpoint (typically 20–95 °C, 0.1 °C
resolution), duration, start/stop, delay, alarm offset.

**Typical readable state:** water temp, heating, circulating, remaining, low
water, cover (optional).

**Notes:** Low-water must cut heat. A sous-vide *program* on an oven or
multi-cooker is not this class.

### `bread_maker`

Automatic bread machine (mix, knead, rise, bake in a pan).

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`program`, `heater`, `motor`, `time_schedule`, `fault`, `energy`, `audio`.

**Typical controllable settings:** program (`basic`, `whole_wheat`, `french`,
`quick`, `dough`, `jam`, `bake_only`), crust, loaf size, delay.

**Typical readable state:** phase (`knead`, `rise`, `bake`, `keep_warm`),
remaining, pan present, lid, overtemp.

### `dehydrator`

Food dehydrator cabinet or stacked trays.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`heater`, `fan`, `humidity` (optional), `time_schedule`, `fault`, `energy`.

**Typical controllable settings:** temp (typically 30–75 °C), time, fan, tray
programs.

**Typical readable state:** temp, remaining, fan, heater, done.

### `vacuum_sealer`

Vacuum sealer (bar or chamber).

**Typical traits:** `identity`, `power`, `connectivity`, `cycle`, `motor`
(pump), `heater` (seal bar), `fault`, `energy`, `safety`.

**Typical controllable settings:** vacuum then seal, seal-only, pulse vacuum,
moist/dry, chamber vacuum target (kPa) if chamber.

**Typical readable state:** pumping, sealing, complete, bag detect, overheat
seal bar, vacuum pressure.

**Notes:** Chamber vs bar is a variant flag. Lid interlock on chamber units.

### `ice_cream_maker`

Compressor or pre-freeze-bowl ice cream maker.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`motor`, `fault`, `energy`.

**Typical controllable settings:** time or doneness, program (`ice_cream`,
`gelato`, `sorbet`), keep-cool.

**Typical readable state:** churning, bowl temp, motor load (doneness proxy),
done.

**Notes:** Pre-freeze-bowl units may have no temperature sensor.

### `yogurt_maker`

Yogurt (and sometimes cheese) fermenter.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`heater`, `time_schedule`, `fault`, `energy`.

**Typical controllable settings:** incubation temp and time, program (`yogurt`,
`greek`, `proof`).

**Typical readable state:** temp, remaining, done.

**Notes:** Yogurt mode on `multi_cooker` stays on that class.

### `waffle_maker`

Connected waffle iron.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`heater`, `fault`, `energy`, `audio`.

**Typical controllable settings:** shade / temp, count, start.

**Typical readable state:** preheat ready, baking, lid, done.

### `pasta_maker`

Electric pasta extruder / mixer.

**Typical traits:** `identity`, `power`, `connectivity`, `motor`, `cycle`,
`program`, `fault`, `energy`, `safety`.

**Typical controllable settings:** program (`mix`, `extrude`), die type
(if not sensed), portion.

**Typical readable state:** mixing / extruding, lid, hopper, jam.

### `steam_cooker`

Dedicated countertop steam cooker / food steamer (not a steam oven cavity).

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`, `cycle`,
`heater`, `water`, `time_schedule`, `fault`, `energy`, `safety`.

**Typical controllable settings:** duration, programs per tray, keep-warm.

**Typical readable state:** steaming, water empty, remaining, done.

**Notes:** Steam ovens are `steam_oven`. Rice-cooker steam trays stay
`rice_cooker`.

---

## Utility

### `garbage_disposal`

Food waste disposer under a sink.

**Typical traits:** `identity`, `power`, `connectivity`, `motor`, `cycle`,
`fault`, `energy`, `safety`, `audio` (optional).

**Typical controllable settings:** run (momentary or timed), reverse (if
supported), air-switch enable, batch-feed interlock ignore is **not** allowed.

**Typical readable state:** running, jam, overcurrent, reset needed, lid /
batch-feed stopper (if sensed).

**Notes:**

- Safety: continuous remote run is dangerous. Prefer timed pulses (seconds)
  and require `remote_enable`. Default deny unattended start.
- Not a dishwasher. Dishwasher drain through a disposal is plumbing, not a
  HomeCooked relationship.

### `trash_compactor`

Kitchen trash compactor.

**Typical traits:** `identity`, `power`, `connectivity`, `motor`, `cycle`,
`door_lid`, `child_lock`, `fault`, `energy`, `safety`.

**Typical controllable settings:** compact cycle, lock.

**Typical readable state:** running, drawer closed, ram position, jam, full.

**Notes:** Child lock is expected. Remote compact only with drawer closed.

### `water_heater`

Domestic hot water heater (tank electric, heat-pump water heater, or
instrumented tankless) for potable water. Not a space-heating boiler.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`,
`heater`, `water`, `fault`, `energy`, `time_schedule`, `safety`, `maintenance`.

**Typical controllable settings:**

- Setpoint °C (typically 40–70), mode (`heat_pump`, `hybrid`, `electric`,
  `vacation`, `high_demand`)
- Schedule, vacation, recirculation pump on/schedule (if present)

**Typical readable state:**

- Tank / outlet temperature, element or compressor on, inlet temp
- Leak, dry-fire, overtemp / T&P, anode / maintenance
- Energy, estimated hot water remaining (heat-pump WH)

**Notes:**

- Legionella: some regions require periodic high-temp cycles; that is device
  policy. The catalog does not mandate it.
- Tankless units use the same class with flow-based state (`flow_l_min`,
  `outlet_temp_c`) and no tank remaining estimate.
- Distinct from `boiler` (hydronic / space heat) and `kettle` (countertop).
- Optional `thermal_port_*` class points advertise a DHW-preheat heat port
  (device telemetry surface; plant objects stay in `homecooked-thermal`).

### `boiler`

Hydronic boiler for space heating and optionally domestic hot water (combi).
Whitegoods-adjacent plant in the home.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`,
`heater`, `water`, `fault`, `energy`, `time_schedule`, `safety`, `maintenance`.

**Typical controllable settings:**

- CH (central heat) enable, DHW enable, CH setpoint, DHW setpoint
- Weather-compensation curve (optional), summer mode, pump overrun
- Flame / burner is not directly writable

**Typical readable state:**

- Flow / return temp, DHW temp, burner on, modulation percent, pressure bar
- Pump, faults (ignition, flame-out, low pressure, overheat, flue)
- Outdoor temp if used for compensation

**Notes:**

- Gas valve is never a raw write. Setpoints and enable bits only.
- A combi boiler still uses this class (CH + DHW zones), not `water_heater`,
  when CH is present.
- Pairing with `hvac` thermostats is a system concern; this class is the plant.

### `water_softener`

Ion-exchange or similar water softener.

**Typical traits:** `identity`, `power`, `connectivity`, `water`, `cycle`,
`fault`, `energy`, `maintenance`, `time_schedule`.

**Typical controllable settings:** hardness input (if not sensed), regeneration
now, regen schedule / threshold, salt reminder reset.

**Typical readable state:**

- Softening / regenerating / bypass, remaining capacity (grains or m³)
- Salt level, water treated volume, valve position, leak

**Notes:** Bypass may be writable. Do not expose raw brine-valve timing unless
as a vendor service mode.

### `water_filter`

Point-of-use or whole-home filter / RO system with electronics.

**Typical traits:** `identity`, `power`, `connectivity`, `water`, `filter`,
`fault`, `maintenance`, `dispense` (if a faucet is instrumented).

**Typical controllable settings:** filter-reset per stage, flush, RO tank empty
(service).

**Typical readable state:** filter life per stage, TDS in/out (RO), leak, flow,
tank full.

**Notes:** Fridge filters stay on the fridge `filter` trait. This class is a
standalone appliance.

---

## Climate

### `hvac`

Residential HVAC plant as a whitegoods-adjacent endpoint: thermostat-facing
heat/cool/fan, including heat pumps and mini-splits that are controlled like an
appliance.

**Typical traits:** `identity`, `power`, `connectivity`, `temperature`,
`humidity`, `fan`, `filter`, `fault`, `energy`, `time_schedule`, `zone`,
`safety`, `maintenance`.

**Typical controllable settings:**

- Mode (`off`, `heat`, `cool`, `auto`, `fan_only`, `dry`, `emergency_heat`)
- Heat and cool setpoints, deadband, fan (`auto`, `on`, `circulate`)
- Swing / louver, quiet, eco, schedule, hold vs schedule
- Zone enable on multi-head mini-splits

**Typical readable state:**

- Space temp, humidity, outdoor temp, coil temp
- Compressor, reversing valve, defrost, aux heat
- Filter life, fault codes (from the indoor or outdoor unit)
- Energy if metered

**Notes:**

- This is **not** a full BMS. No BACnet object model here. The class exists so
  kitchen/home appliance fabrics can talk to the same climate box.
- Multi-split: one outdoor + N indoor endpoints, each an `hvac` device or one
  device with `zone`s. Prefer one device id per indoor head.
- Heat / cool setpoint writes in the wrong mode may be stored and applied later
  or rejected `unsupported_operation`; devices advertise which.
- Gas furnaces and boilers used only as HVAC plants may be `hvac` (thermostat
  face) with `boiler` as a separate plant device if both are connected.
- Optional `thermal_port_*` class points advertise a hydronic coil heat port
  for plant coupling (device telemetry; plant objects stay in `homecooked-thermal`).

### `dehumidifier`

Dedicated dehumidifier (portable or whole-home).

**Typical traits:** `identity`, `power`, `connectivity`, `humidity`, `fan`,
`water`, `filter`, `fault`, `energy`, `time_schedule`.

**Typical controllable settings:** humidity setpoint (% RH), fan, pump / drain
mode, timer.

**Typical readable state:** RH, tank full, defrost, compressor, filter.

**Notes:** HVAC dry mode stays on `hvac`. Tank-full is `busy` for compressor
start.

### `humidifier`

Dedicated humidifier (evaporative, steam, or ultrasonic) as HVAC-adjacent
whitegoods.

**Typical traits:** `identity`, `power`, `connectivity`, `humidity`, `water`,
`fault`, `energy`, `maintenance`, `safety`.

**Typical controllable settings:** humidity setpoint, output level, warm vs
cool mist.

**Typical readable state:** RH, water empty, scale, filter / wick life.

**Notes:** Safety: ultrasonic units may expose a dry-tank interlock. Whole-home
bypass humidifiers attached to HVAC ducts can be this class or a trait on
`hvac` if they are not independently addressed.

---

## Class ids reserved / not used

Do not mint core classes for:

| Idea | Use instead |
|------|-------------|
| “Smart plug on a dumb oven” | Not an appliance class. Out of scope. |
| Generic `appliance` | Too vague; pick a class or a vendor id. |
| `cooker` | Ambiguous (hob vs oven vs rice). Use the specific class. |
| `hob` | Use `cooktop` or `induction_hob`. |
| `fridge_freezer_ice` | `fridge_freezer` + trait `ice`. |
| `washer_dryer_vented` | Variant on `washer_dryer` / `dryer`. |
| Medical / lab cold chain | Vendor class. |
| Robots, vacuums, mops | Out of this catalog (not whitegoods/kitchen). |
| Lighting, speakers | Out of this catalog. |
| Utility meters | Out of this catalog (energy on the appliance is enough). |

Vendor-specific classes use `vendor.<vendor_id>.<class_id>` and must not collide
with ids in this table.

---

## Typical trait advertisement (summary)

Full variable lists are in [`variables-and-settings.md`](./variables-and-settings.md).
This table is the default **trait set** a well-implemented device of that class
should consider advertising. Optional traits are in parentheses.

| Class | Typical traits |
|-------|----------------|
| `washer` | identity, power, connectivity, time_schedule, door_lid, child_lock, cycle, program, water, temperature, motor, fault, energy, remote, maintenance, audio, safety |
| `dryer` | identity, power, connectivity, time_schedule, door_lid, child_lock, cycle, program, temperature, humidity, heater, fan, filter, fault, energy, remote, maintenance, audio, safety |
| `washer_dryer` | washer ∪ dryer |
| `fridge` | identity, power, connectivity, door_lid, temperature, zone, lighting, fault, energy, remote, maintenance, (ice, dispense, filter, child_lock, audio) |
| `freezer` | identity, power, connectivity, door_lid, temperature, zone, (lighting), fault, energy, remote, maintenance, (child_lock, audio) |
| `fridge_freezer` | fridge ∪ freezer ∪ (ice, dispense, filter, humidity) |
| `wine_cooler` | identity, power, connectivity, door_lid, temperature, zone, humidity, lighting, (child_lock), fault, energy, audio |
| `beverage_cooler` | identity, power, connectivity, door_lid, temperature, (lighting), fault, energy |
| `ice_maker` | identity, power, connectivity, door_lid, ice, water, filter, fault, energy, maintenance, (child_lock) |
| `kegerator` | identity, power, connectivity, door_lid, temperature, dispense, (lighting), fault, energy |
| `dishwasher` | identity, power, connectivity, time_schedule, door_lid, child_lock, cycle, program, water, temperature, heater, filter, fault, energy, remote, maintenance, audio, safety |
| `microwave` | identity, power, connectivity, door_lid, cycle, program, (heater), lighting, audio, child_lock, fault, energy, remote, safety |
| `oven` | identity, power, connectivity, door_lid, temperature, cycle, program, heater, fan, lighting, child_lock, fault, energy, remote, time_schedule, safety, audio |
| `steam_oven` | oven ∪ water, humidity, filter |
| `toaster_oven` | subset of oven |
| `range` | cooktop or induction_hob ∪ oven |
| `cooktop` | identity, power, connectivity, child_lock, heater, zone, fault, energy, safety, (lighting) |
| `induction_hob` | cooktop + pan-detect variables |
| `warming_drawer` | identity, power, connectivity, temperature, door_lid, fault, energy, safety |
| `pizza_oven` | identity, power, connectivity, temperature, cycle, heater, door_lid, fault, energy, safety |
| `air_fryer` | identity, power, connectivity, temperature, cycle, program, heater, fan, door_lid, fault, energy, audio, safety |
| `electric_grill` | identity, power, connectivity, temperature, cycle, heater, fault, energy, safety |
| `electric_smoker` | identity, power, connectivity, temperature, cycle, heater, (humidity), fault, energy, time_schedule, safety |
| `range_hood` | identity, power, connectivity, fan, lighting, filter, fault, energy, remote, (audio) |
| `coffee_machine` | identity, power, connectivity, water, temperature, cycle, program, (child_lock), fault, energy, maintenance, filter, audio |
| `espresso_machine` | identity, power, connectivity, water, temperature, cycle, fault, energy, maintenance, safety |
| `drip_coffee_maker` | identity, power, connectivity, water, temperature, cycle, time_schedule, fault, energy, maintenance, audio |
| `coffee_grinder` | identity, power, connectivity, motor, cycle, fault, energy |
| `kettle` | identity, power, connectivity, temperature, heater, cycle, fault, energy, safety, audio |
| `water_dispenser` | identity, power, connectivity, temperature, dispense, water, child_lock, filter, fault, energy, safety |
| `toaster` | identity, power, connectivity, cycle, heater, fault, energy, audio, safety |
| `blender` | identity, power, connectivity, motor, cycle, program, door_lid, fault, energy, safety, audio, (temperature, heater) |
| `food_processor` | identity, power, connectivity, motor, cycle, door_lid, fault, energy, safety |
| `stand_mixer` | identity, power, connectivity, motor, cycle, fault, energy, safety, (lighting) |
| `juicer` | identity, power, connectivity, motor, cycle, fault, energy, safety |
| `rice_cooker` | identity, power, connectivity, temperature, cycle, program, heater, fault, energy, time_schedule, audio |
| `slow_cooker` | identity, power, connectivity, temperature, cycle, heater, time_schedule, fault, energy, audio |
| `multi_cooker` | identity, power, connectivity, temperature, cycle, program, heater, door_lid, child_lock, fault, energy, time_schedule, safety, audio |
| `sous_vide` | identity, power, connectivity, temperature, cycle, heater, fan, fault, energy, time_schedule, safety, audio |
| `bread_maker` | identity, power, connectivity, temperature, cycle, program, heater, motor, time_schedule, fault, energy, audio |
| `dehydrator` | identity, power, connectivity, temperature, cycle, heater, fan, (humidity), time_schedule, fault, energy |
| `vacuum_sealer` | identity, power, connectivity, cycle, motor, heater, fault, energy, safety |
| `ice_cream_maker` | identity, power, connectivity, temperature, cycle, motor, fault, energy |
| `yogurt_maker` | identity, power, connectivity, temperature, cycle, heater, time_schedule, fault, energy |
| `waffle_maker` | identity, power, connectivity, temperature, cycle, heater, fault, energy, audio |
| `pasta_maker` | identity, power, connectivity, motor, cycle, program, fault, energy, safety |
| `steam_cooker` | identity, power, connectivity, temperature, cycle, heater, water, time_schedule, fault, energy, safety |
| `garbage_disposal` | identity, power, connectivity, motor, cycle, fault, energy, safety |
| `trash_compactor` | identity, power, connectivity, motor, cycle, door_lid, child_lock, fault, energy, safety |
| `water_heater` | identity, power, connectivity, temperature, heater, water, fault, energy, time_schedule, safety, maintenance |
| `boiler` | identity, power, connectivity, temperature, heater, water, fault, energy, time_schedule, safety, maintenance |
| `water_softener` | identity, power, connectivity, water, cycle, fault, energy, maintenance, time_schedule |
| `water_filter` | identity, power, connectivity, water, filter, fault, maintenance, (dispense) |
| `hvac` | identity, power, connectivity, temperature, humidity, fan, filter, fault, energy, time_schedule, zone, safety, maintenance |
| `dehumidifier` | identity, power, connectivity, humidity, fan, water, filter, fault, energy, time_schedule |
| `humidifier` | identity, power, connectivity, humidity, water, fault, energy, maintenance, safety |

Devices **must** advertise the subset they actually implement. Advertising a
trait commits the device to the trait’s required variables (see the variables
catalog). Optional variables inside a trait may still be omitted.
