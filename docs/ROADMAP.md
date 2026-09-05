# HomeCooked roadmap — ~75% project completeness

Version **0.1.99**. Planning doc for a long flesh-out of the catalog, control
stack, and simulator. It does **not** freeze APIs; crate and YAML shapes may
evolve with the code that implements each stream.

Related: [`../README.md`](../README.md), [`catalog/`](./catalog/),
[`standard/`](./standard/), especially
[`standard/control-system.md`](./standard/control-system.md) and
[`standard/examples/washer-dryer-io.md`](./standard/examples/washer-dryer-io.md).

---

## 1. Current state (~75% of the §2 in-scope bar — met in spirit)

What exists on `main` today (Done highlights called out):

| Area | Status |
|------|--------|
| Catalog docs | Appliance class index (**56** ids), traits, variables/settings in `docs/catalog/` |
| Standard docs | Overview, thermal-plant, procedures, bridges, control-system sketches; washer/dryer I/O example |
| `homecooked-schema` | Serde types, capability model, write validation; **56** static class tables (**25 Tier-A + 31 Tier-B**); optional thermal-port class points on `water_heater` + `fridge` + `hvac` + `dishwasher` + `dryer`; `ClassTable.thermal_ports: &[HeatPortSpec]` (static advertisement matching sim seeds); shared thermal **vocabulary** (`Media` / `PortDirection` / `TempBandC` / `HeatPortSpec`) — **Done** |
| `homecooked-protocol` | Envelope, request/response kinds, discovery, JSON, errors (v0.1.0); **invalid Envelope JSON table tests** |
| `homecooked-core` | Device registry, capability-enforced read/write |
| `homecooked-sim` | In-memory devices for all 56 statically tabled classes; microwave cook ticks advance `elapsed_s`; water_heater/fridge/hvac/dishwasher/dryer thermal-port seeds + RW attach |
| `homecooked-wasm` + `apps/simulator-web` | wasm-bindgen JSON API; full-catalog picker (56) + procedure runner (kettle + Domino's + wash-then-dry + oven bake + coffee brew + air fryer cook `run_procedure` E2E) + thermal panel + device `thermal_port_*` UI (auto when `thermal_port_id` present; `water_heater`/`fridge`/`hvac`/`dishwasher`/`dryer`) + read-only **Catalog heat ports** from `list_heat_port_specs` / `ClassTable.thermal_ports`; **WASM fetch+blob load** (module cache defeat) — **Done** |
| `homecooked-io-map` | Chassis I/O map serde + validate (washer + dryer fragments) |
| `homecooked-interlock` | Declarative interlock rules (washer heater/spin; dryer heater/motor) |
| `homecooked-hal` | Firmware HAL sketch + host `MockHal` |
| `homecooked-procedure` | Procedure documents + sequential runner; Domino's microwave + wash-then-dry + oven bake + coffee brew + air fryer cook + thin `thermal_wait` / `wait_dhw_reservoir` + `thermal_offer` / `offer_fridge_dhw` fixtures |
| `homecooked-controller` | Host controller sim: IoMap + MockHal + interlocks + washer cotton / **dryer cycle**; **lab TCP endpoints** (`ControllerEndpoint` + `DryerControllerEndpoint` interlock deny; washer/dryer **cycle start** + **pause/resume/cancel** + readable phase/state + lab tick; washer **CottonOptions** via adjacent `wash_temp_c`/`spin_rpm` writes; dryer **DryOptions** via adjacent `dryness`/`heat_level` writes) — **Done** |
| `homecooked-thermal` | First executable thermal plant slice (types, registry, offer/accept, tick); re-exports schema thermal vocabulary; plant **runtime** still crate-local (not promoted with ClassTable `HeatPortSpec`) |
| `homecooked-bridge` | **Modbus + Matter + Zigbee + BACnet mocks** (no real serial/TCP/CHIP/z2m/BACnet stacks) — **Done** |
| `homecooked-transport` | Lab TCP JSON envelopes; **optional PSK pairing**; sim-backed server + **pluggable `RequestHandler`**; malformed frame table tests — **Done** |
| `homecooked-hub` | Optional multi-device lab TCP aggregator (**not required for devices**) — **Done** |
| `homecooked-conformance` | Stream 7 smoke: Tier-A / Tier-B / `catalog_hygiene` / `write_denial_matrix` / cotton / kettle + oven bake + coffee brew + air fryer cook + wash-then-dry procedures / thermal / `procedure_thermal_wait_dhw` / `procedure_thermal_offer_dhw` / `water_heater_thermal_ports` / Modbus / Matter / Zigbee / BACnet / TCP / TCP PSK / `controller_tcp_washer_interlock` / `controller_tcp_dryer_interlock` / `controller_tcp_washer_cotton` / `controller_tcp_washer_cotton_options` / `controller_tcp_dryer_cycle` / `controller_tcp_dryer_dry_options` / `controller_tcp_washer_cycle_pause_cancel` / `controller_tcp_dryer_cycle_pause_cancel` / hub lab set |
| CI | rustfmt, clippy (`-D warnings`), `cargo test --workspace`, wasm-pack |

**Done (thin / lab depth):** Tier-A+B **56** static tables + sim; dryer controller
cycle; bridge family mocks; lab TCP + PSK; optional hub (in conformance suite);
simulator-web blob-load; procedure library (kettle + Domino's + wash-then-dry +
`oven_bake_180` + `coffee_brew_espresso` + `air_fryer_cook_200`) + thin
`thermal_wait` / **`thermal_offer`**; controller-sim-over-TCP interlock smoke
(washer + dryer) + washer cotton + **dryer cycle** start/phase + washer **CottonOptions** + dryer **DryOptions** + **cancel/pause/resume** over lab TCP;
catalog `thermal_port_*` on `water_heater` / `fridge` / `hvac` / `dishwasher` /
`dryer` + sim UI chips; schema thermal vocabulary + `ClassTable.thermal_ports`
(`HeatPortSpec`) + wasm/UI heat-port specs; **optional-depth deepen series**
(#56–#73 + follow-on) on `wine_cooler` + `ice_maker` + `sous_vide` + `multi_cooker` +
`toaster_oven` + `dehumidifier` + `range_hood` + `steam_oven` + `cooktop` +
`humidifier` + `freezer` + `fridge_freezer` + `beverage_cooler` + `kegerator` + `warming_drawer` + `pizza_oven` + `electric_grill` + `electric_smoker` + `espresso_machine` + `drip_coffee_maker` + `coffee_grinder` + `water_dispenser` + `toaster` + `blender` + `food_processor` + `stand_mixer` + `juicer` + `rice_cooker` + `slow_cooker` + `bread_maker` + `dehydrator` + `vacuum_sealer` + `ice_cream_maker` + `yogurt_maker` + `waffle_maker` + `pasta_maker` + `steam_cooker` + `garbage_disposal` + `trash_compactor` + `boiler` + `water_softener` + `water_filter` + `washer` + `dryer` + `washer_dryer` + `fridge` + `dishwasher` + `microwave` + `oven` + `range` + `induction_hob` + `air_fryer` + `kettle` + `coffee_machine` + `water_heater` + `hvac` (56 classes; Tier-B optional-depth passes complete; undepened Tier-A deepen series: `washer` + `dryer` + `washer_dryer` + `fridge` + `dishwasher` + `microwave` + `oven` + `range` + `induction_hob` + `air_fryer` + `kettle` + `coffee_machine` + `water_heater` + `hvac`; all listed undepened Tier-A classes now have optional-depth passes; honest caveats still apply for real bridges, TLS, typical_capability, etc.);
`write_denial_matrix` + `catalog_hygiene` conformance.

**Still open (beyond / still thin vs a strict §2 reading):** promote full plant
**runtime** into schema (`Media` / `PortDirection` / `TempBandC` / `HeatPortSpec`
+ `ClassTable.thermal_ports` landed; `ThermalPlant` / transfer dialogue still
crate-local); **real bridge SDKs** (Modbus serial/TCP or Matter/CHIP — mocks only
today); TLS (still out of scope for lab transport); **catalog optional-depth passes complete** for all listed classes — deepen series (#56–#73 + follow-on) landed optional-point passes on **56** classes
(mostly Tier-A + Tier-B `humidifier` / `beverage_cooler` / `kegerator` / `warming_drawer` / `pizza_oven` / `electric_grill` / `electric_smoker` / `espresso_machine` / `drip_coffee_maker` / `coffee_grinder` / `water_dispenser` / `toaster` / `blender` / `food_processor` / `stand_mixer` / `juicer` / `rice_cooker` / `slow_cooker` / `bread_maker` / `dehydrator` / `vacuum_sealer` / `ice_cream_maker` / `yogurt_maker` / `waffle_maker` / `pasta_maker` / `steam_cooker` / `garbage_disposal` / `trash_compactor` / `boiler` / `water_softener` / `water_filter` + undepened Tier-A `washer` + `dryer` + `washer_dryer` + `fridge` + `dishwasher` + `microwave` + `oven` + `range` + `induction_hob` + `air_fryer` + `kettle` + `coffee_machine` + `water_heater` + `hvac`); **0 of 31 Tier-B** ids remain thin tables
(all 31 have optional-depth passes; all listed undepened Tier-A classes now have optional-depth passes (0 remaining); see §4); procedure⇄thermal **multi-round negotiate dialogue** / soft decline /
richer wasm UI (thin `thermal_wait` + `thermal_offer` immediate-accept are present);
**typical_capability** over the wire still open (washer/dryer **CottonOptions** /
**DryOptions** + **cancel / pause / resume** via catalog cycle commands landed).

Rough completeness: foundation + Tier-A/B tables + procedure library (kettle /
Domino's / wash-then-dry / oven / coffee / air fryer + thin `thermal_wait` +
`thermal_offer`) +
HAL / controller TCP (washer+dryer interlock + washer cotton + **dryer cycle**
start/phase + washer **CottonOptions** + dryer **DryOptions** over lab TCP) + hub-in-suite + thermal-port surface (5 classes + UI +
schema vocabulary / `ClassTable.HeatPortSpec` + wasm heat-port chips) + bridge
mocks + write-denial matrix + **optional-depth deepen series** on
`wine_cooler` / `ice_maker` / `sous_vide` / `multi_cooker` / `toaster_oven` /
`dehumidifier` / `range_hood` / `steam_oven` / `cooktop` / `humidifier` /
`freezer` / `fridge_freezer` / `beverage_cooler` / `kegerator` / `warming_drawer` / `pizza_oven` / `electric_grill` / `electric_smoker` / `espresso_machine` / `drip_coffee_maker` / `coffee_grinder` / `water_dispenser` / `toaster` / `blender` / `food_processor` / `stand_mixer` / `juicer` / `rice_cooker` / `slow_cooker` / `bread_maker` / `dehydrator` / `vacuum_sealer` / `ice_cream_maker` / `yogurt_maker` / `waffle_maker` / `pasta_maker` / `steam_cooker` / `garbage_disposal` / `trash_compactor` / `boiler` / `water_softener` / `water_filter` / `washer` / `dryer` / `washer_dryer` / `fridge` / `dishwasher` / `microwave` / `oven` / `range` / `induction_hob` / `air_fryer` / `kettle` / `coffee_machine` / `water_heater` / `hvac` ≈ **~75% of the §2 in-scope bar, met in spirit** for
lab/software depth (was ~30% at roadmap start; ~72% at the v0.1.35 refresh; still
~75% after the deepen wave — all 31 Tier-B have optional-depth passes; all listed undepened Tier-A classes now have optional-depth passes (`washer` + `dryer` + `washer_dryer` + `fridge` + `dishwasher` + `microwave` + `oven` + `range` + `induction_hob` + `air_fryer` + `kettle` + `coffee_machine` + `water_heater` + `hvac`); honest caveats still apply for real bridges, TLS, etc.).
Recent grind: schema thermal vocab + `ClassTable.thermal_ports` + heat-port UI
(#54–#55, #59); washer cotton-over-TCP (#60) + **dryer cycle TCP** (#62);
thin **`thermal_offer`** (#65); catalog optional-depth PRs **#56–#57, #63–#64,
#66–#73** + `beverage_cooler` / `kegerator` / `warming_drawer` / `pizza_oven` / `electric_grill` / `electric_smoker` / `espresso_machine` / `drip_coffee_maker` / `coffee_grinder` / `water_dispenser` / `toaster` / `blender` / `food_processor` / `stand_mixer` / `juicer` / `rice_cooker` / `slow_cooker` / `bread_maker` / `dehydrator` / `vacuum_sealer` / `ice_cream_maker` / `yogurt_maker` / `waffle_maker` / `pasta_maker` / `steam_cooker` / `garbage_disposal` / `trash_compactor` / `boiler` / `water_softener` / `water_filter` / `washer` / `dryer` / `washer_dryer` / `fridge` / `dishwasher` / `microwave` / `oven` / `range` / `induction_hob` / `air_fryer` / `kettle` / `coffee_machine` / `water_heater` / `hvac` (the fifty-six classes above; undepened Tier-A deepen series: `washer` + `dryer` + `washer_dryer` + `fridge` + `dishwasher` + `microwave` + `oven` + `range` + `induction_hob` + `air_fryer` + `kettle` + `coffee_machine` + `water_heater` + `hvac`). Calling the target **substantially
achieved** remains honest — not that every §2 bullet is production-complete or
that real bridge SDKs / TLS are done. This is **not** IEC certification,
production firmware, or a shipping commercial appliance. Remaining work is depth
beyond the lab bar (real bridge SDK, full plant runtime schema promotion, TLS,
richer procedure⇄thermal dialogue beyond offer+immediate-accept; washer/dryer
lab typical_capability-over-wire landed — full HAL binding for every typical
point still not required).

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

### Honest gaps vs this 75% definition (beyond / still thin)

§1 treats the §2 in-scope bar as **~75% met in spirit** for lab/software depth.
The items below remain **beyond** that bar or still thin — they do **not**
undo the “substantially achieved” framing, and they do **not** claim IEC /
production firmware:

- **Real bridge SDK** — in-scope asks for *one* real Matter **or** Modbus
  implementation; `homecooked-bridge` has Modbus + Matter + Zigbee + BACnet
  **mocks** only (no serial/TCP Modbus, CHIP, z2m, or BACnet stack).
- **Plant runtime** — device `thermal_port_*` points exist on
  `water_heater` / `fridge` / `hvac` / `dishwasher` / `dryer`; shared vocabulary
  (`Media` / `PortDirection` / `TempBandC` / `HeatPortSpec`) +
  `ClassTable.thermal_ports` advertisement live in `homecooked-schema`; plant
  object runtime (`ThermalPlant`, reservoirs, offer/accept, tick) remains
  crate-local in `homecooked-thermal` (not full schema promotion).
- **Richer UI** — picker + procedure runner + thermal panel + port chips are
  in; conformance-oriented / deeper screens remain.
- **Deeper catalog optional points** — thin tables cover all 31 Tier-B ids;
  optional-point depth landed (PRs **#56–#57, #63–#64, #66–#73** + follow-on) on **56** classes —
  Tier-A `wine_cooler` + `ice_maker` + `sous_vide` + `multi_cooker` + `toaster_oven`
  + `dehumidifier` + `range_hood` + `steam_oven` + `cooktop` + `freezer` +
  `fridge_freezer` + `washer` + `dryer` + `washer_dryer` + `fridge` + `dishwasher` + `microwave` + `oven` + `range` + `induction_hob` + `air_fryer` + `kettle` + `coffee_machine` + `water_heater` + `hvac` (undepened Tier-A deepen series), plus Tier-B `humidifier` + `beverage_cooler` + `kegerator` + `warming_drawer` + `pizza_oven` + `electric_grill` + `electric_smoker` + `espresso_machine` + `drip_coffee_maker` + `coffee_grinder` + `water_dispenser` + `toaster` + `blender` + `food_processor` + `stand_mixer` + `juicer` + `rice_cooker` + `slow_cooker` + `bread_maker` + `dehydrator` + `vacuum_sealer` + `ice_cream_maker` + `yogurt_maker` + `waffle_maker` + `pasta_maker` + `steam_cooker` + `garbage_disposal` + `trash_compactor` + `boiler` + `water_softener` + `water_filter` (alarms / sabbath / bottle_count /
  humidity; ice bin/filter life + harvest/scale alerts; sous-vide
  water/lid/timer/overtemp + cycle remaining; multi-cooker pot/pressure/saute/keep-warm
  + cycle remaining; toaster-oven door/timer/rack/bagel/slices + convection/broil/elements
  + cycle remaining; dehumidifier compressor/RH alarms/continuous/quiet/bucket/filter
  + humidity setpoint + fan speed; range-hood filter/boost/light/grease/hob-link/overtemp
  + fan/light/filter traits; steam-oven tank/descale/generator/humidity/door/drain +
  delayed start + cycle remaining/hardness; cooktop keep_warm/hotspot/timer_active/paused/
  surface_c/element_fault/pan_detect/flame_on + boost/timer/bridge/gas faults/power_limit;
  humidifier warm_mist/auto_humidity/mineral_filter/uv_clean/scale_alert/tank_removed/
  misting/night_mode + output/mist/wick + humidity setpoint; freezer
  fast_freeze/door_ajar/ice_buildup/low_temp_alarm/anti_sweat/fast_freeze_remaining/
  frost_clean + cold-cabinet vacation/sabbath/eco/defrost/compressor/high_temp/power_fail;
  fridge_freezer dual-zone door_ajar_fridge/freezer/fast_freeze/ice_buildup/
  high_temp_alarm_fridge/freezer/convertible_zone_mode + cold-cabinet
  vacation/sabbath/eco/defrost/compressor/high_temp/power_fail;
  beverage_cooler sabbath/eco/compressor/temp alarms/door_ajar/can_capacity;
  kegerator sabbath/eco/compressor/temp alarms/door_ajar/co2_kpa/keg_percent/keg_empty;
  warming_drawer level/moist/sabbath/eco/heater_on/high_temp_alarm/door_ajar/timer_s;
  pizza_oven stone_c/dome_c/top_bottom_balance/sabbath/eco/heater_on/high_temp_alarm/door_ajar/timer_s/steam_inject;
  electric_grill plate_top_c/plate_bottom_c/sear/grease_tray/sabbath/eco/heater_on/high_temp_alarm/lid_open/timer_s;
  electric_smoker chamber_c/smoke_on/fuel_percent/water_pan/sabbath/eco/heater_on/high_temp_alarm/door_ajar/timer_s;
  espresso_machine brew_pressure_bar/shot_ml/pump_on/steam_wand_on/sabbath/eco/boiler_ready/high_temp_alarm/water_tank_empty/descaling_needed/timer_s;
  drip_coffee_maker cups/strength/keep_warm_s/carafe_present/sabbath/eco/heater_on/high_temp_alarm/water_tank_empty/descaling_needed/timer_s;
  coffee_grinder grind_s/dose_g/hopper_present/sabbath/eco/motor_on/hopper_empty/bean_level_percent/timer_s/single_dose;
  water_dispenser hot/cold_setpoint_c/bottle_empty/sabbath/eco/heater_on/cooler_on/high_temp_alarm/low_temp_alarm/water_tank_empty + trait filter life / child_lock;
  toaster shade/bagel/frozen/single_side/carriage/sabbath/eco/heater_on/high_temp_alarm/timer_s/crumb_tray_full/slots;
  blender speed_level/form_factor/pulse/jar_present/lid_locked/heated/sabbath/eco/motor_on/overload_trip/timer_s;
  food_processor speed_level/pulse/bowl_present/lid_locked/attachment/sabbath/eco/motor_on/overload_trip/timer_s;
  stand_mixer speed_level/bowl_present/head_down/mass_g/attachment/sabbath/eco/motor_on/overload_trip/timer_s;
  juicer speed_level/reverse/pulp_full/jug_present/sabbath/eco/motor_on/overload_trip/timer_s;
  rice_cooker texture/bowl_present/keep_warm/sabbath/eco/heater_on/high_temp_alarm/lid_open/timer_s/water_ratio;
  slow_cooker heat_level/cook_s/pot_present/keep_warm/sabbath/eco/heater_on/high_temp_alarm/lid_open/timer_s;
  bread_maker crust/loaf_size/pan_present/keep_warm/sabbath/eco/heater_on/high_temp_alarm/lid_open/timer_s;
  dehydrator cook_s/sabbath/eco/heater_on/fan_on/high_temp_alarm/door_ajar/timer_s/tray_count;
  vacuum_sealer mode/moist/vacuum_kpa/bag_detect/form_factor/sabbath/eco/pump_on/seal_heater_on/lid_locked/seal_fail/timer_s;
  ice_cream_maker doneness/sabbath/eco/compressor_on/motor_on/bowl_present/lid_locked/low_temp_alarm/timer_s;
  yogurt_maker incubate_s/sabbath/eco/heater_on/high_temp_alarm/low_temp_alarm/lid_open/jar_present/timer_s;
  waffle_maker shade/ready/sabbath/eco/heater_on/high_temp_alarm/lid_open/batter_done/timer_s;
  pasta_maker die/jam/sabbath/eco/motor_on/dough_ready/hopper_empty/die_present/overload_trip/timer_s;
  steam_cooker cook_s/water_empty/sabbath/eco/heater_on/high_temp_alarm/lid_open/steam_ready/timer_s;
  garbage_disposal run_s/jam/reset_needed/reverse/sabbath/eco/motor_on/overload_trip/air_switch/timer_s;
  trash_compactor ram_state/bin_full/sabbath/eco/motor_on/drawer_open/overload_trip/key_lock/timer_s;
  boiler pressure_bar/burner_on/flame_out/low_pressure/sabbath/eco/high_temp_alarm/lockout/ignition_fail/timer_s;
  water_softener capacity_remaining/salt_level/bypass/treated_l/sabbath/eco/regenerating/salt_low/timer_s + trait hardness_ppm / filter life;
  water_filter tds_in_ppm/tds_out_ppm/tank_full/sabbath/eco/bypass/filter_clogged/replace_needed/timer_s + trait filter life / flow_l_min;
  washer sabbath/eco/door_ajar/door_locked/water_temp_alarm/overflow_alarm/detergent_low/timer_s + typical detergent_level_percent/unbalance + trait delay_start_s;
  dryer sabbath/eco/door_ajar/door_locked/high_temp_alarm/lint_full/timer_s + typical anti_crease/dryness_percent/vent_blocked/drain_tank + trait delay_start_s;
  washer_dryer washer depth + dryer thin anti_crease/dryness/vent/drain + high_temp_alarm/lint_full on EXTRA (not DRYER_DEPTH) + typical dry_after_wash/max_dry_s + trait delay_start_s;
  fridge door_ajar/low_temp_alarm + typical cold-cabinet vacation/sabbath/eco/defrost/compressor/high_temp/power_fail + thermal ports;
  dishwasher sabbath/eco/door_ajar/door_locked/rinse_aid_low/salt_low/overflow_alarm/timer_s + typical rinse_aid_level/salt_level/wash_temp_c + trait delay_start_s + thermal ports;
  microwave sabbath/eco/door_ajar/magnetron_on/high_temp_alarm/timer_s + typical power_w/defrost_g/turntable/inverter; child_lock trait already typical;
  oven sabbath/eco/heater_on/high_temp_alarm/door_ajar/timer_s on OVEN_DEPTH (not range/steam/toaster composition) + typical broil/convection/steam/cook/door_locked_clean/elements + Temperature probe/preheat; child_lock already typical;
  range RANGE_EXTRA sabbath/eco/heater_on/high_temp_alarm/door_ajar (not OVEN_DEPTH — cooktop zoned timer_s already composed) + typical cooktop depth + OVEN_BASE broil/convection/steam/cook/door_locked_clean/elements + Temperature probe/preheat; child_lock already typical;
  induction_hob sabbath/eco/power_share/auto_boost/overtemp_alarm on EXTRA + typical cooktop depth + thin pan_size/power_w/limiter/cookware/temp_mode/flex; child_lock already typical; cooktop timer_s/pan_detect/residual_heat not redeclared;
  air_fryer sabbath/eco/heater_on/fan_on/high_temp_alarm/door_ajar/timer_s on AIR_FRYER_DEPTH + typical shake_enable/shake_due/preheat/basket_present/sync_finish; cook_s unchanged; Heater/Fan/DoorLid already typical;
  kettle sabbath/eco/heater_on/high_temp_alarm/lid_open/timer_s on KETTLE_DEPTH + typical keep_warm/keep_warm_s/boil_dry; on_base unchanged; Heater already typical;
  coffee_machine sabbath/eco/boiler_ready/high_temp_alarm/water_tank_empty/descaling_needed/carafe_present/timer_s on COFFEE_MACHINE_DEPTH + typical strength/volume_ml/milk_ml/grind_level/cups/drip_tray/grounds_bin/milk_present/capsule_present/boiler_c/brew_pressure_bar; water_tank unchanged;
  water_heater sabbath/eco/heater_on/high_temp_alarm/low_temp_alarm/leak_alarm/timer_s on WATER_HEATER_DEPTH + typical mode/inlet_c/outlet_c/hot_remaining_percent/leak/dry_fire/recirc_on/form_factor; Temperature setpoint + thermal_port_* unchanged; mode enum vacation — no vacation_mode bool;
  hvac sabbath/fan_on/high_temp_alarm/low_temp_alarm/timer_s on HVAC_DEPTH + typical heat/cool setpoints/deadband/outdoor/hold/quiet/eco/compressor/aux/defrost/reversing_valve + trait humidity setpoint / fan speed / filter life; hvac_mode / space_c + thermal_port_* unchanged; reuse eco (not eco_mode) and defrost (not defrost_active)).
  **Remaining thin Tier-B (0):**
  none — all 31 Tier-B classes now have optional-depth passes. **Remaining
  undepened Tier-A (0):** none — all listed undepened Tier-A classes now have
  optional-depth passes. Honest caveats still apply for real bridges, TLS,
  cancel/pause, plant runtime, etc.
- **Procedure⇄thermal depth** — thin `thermal_wait` on reservoir `temp_c` and
  thin `thermal_offer` (offer + immediate accept / decline; `offer_fridge_dhw`
  + conformance) are present; multi-round negotiate dialogue, soft
  decline-without-fail, and richer wasm/UI wiring remain open. Dual-path
  dishwasher demo can still orchestrate transfer outside the procedure JSON.
- **TLS** — lab TCP stays cleartext (+ optional PSK); TLS/OAuth remain out of
  scope for the lab path.
- **Richer controller-over-TCP** — interlock smoke for washer+dryer is done;
  washer cotton + dryer cycle **start + readable phase/state** (+ lab tick) over
  TCP landed; washer **CottonOptions** over the wire (adjacent `wash_temp_c` /
  `spin_rpm` before void start) landed; dryer **DryOptions** (adjacent
  `dryness` / `heat_level` before void start) landed; **cancel / pause /
  resume** over TCP landed; **typical_capability over the wire** for washer+dryer
  lab endpoints landed (advertise catalog typical + lab HAL/`sim_tick`;
  store/default for unbound typical points — full HAL binding not required).

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
   `kettle_heat_80` + `reheat_dominos_microwave` + `wash_then_dry` + `oven_bake_180` + `coffee_brew_espresso` + `air_fryer_cook_200` + thin `wait_dhw_reservoir` (`thermal_wait`) + `offer_fridge_dhw` (`thermal_offer`); wasm `run_procedure` E2E
   auto-spawns and completes device fixtures (microwave wait uses sim `elapsed_s` ticks). Dedicated wasm UI for thermal steps still thin (list + `run_procedure`).
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
   typical_capability over lab TCP landed
   (washer CottonOptions + dryer DryOptions + cancel/pause/resume + catalog
   typical Describe/store-default).
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
   Washer cotton + dryer cycle start + `cycle_state`/`cycle_phase` (+ lab tick)
   over TCP landed; washer CottonOptions + dryer DryOptions (adjacent catalog
   setpoints) over TCP landed; cancel/pause/resume over TCP landed;
   catalog **typical_capability** over TCP landed (merged with lab HAL /
   sim_tick; store/default for unbound points).

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
  `controller_tcp_washer_interlock` / `controller_tcp_dryer_interlock` /
  `controller_tcp_washer_cotton` / `controller_tcp_washer_cotton_options` / `controller_tcp_dryer_cycle` / `controller_tcp_dryer_dry_options` / `controller_tcp_washer_cycle_pause_cancel` / `controller_tcp_dryer_cycle_pause_cancel` / `controller_tcp_washer_typical_capability` / `controller_tcp_dryer_typical_capability`.
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
   coffee brew + air fryer cook; `wait_dhw_reservoir` / `offer_fridge_dhw`
   listed/bundled — thermal steps need plant attach); covered by wasm
   `run_procedure` E2E tests).
   **Thermal-port UI slice is done** — `create_thermal_demo` /
   `thermal_state` / `thermal_negotiate_demo` / `thermal_tick` /
   `thermal_demo_transfer` expose the fridge→DHW plant; simulator-web has a
   Load demo / Negotiate / Tick / Transfer panel showing reservoirs, ports,
   and last transfer results. Device panel also surfaces catalog
   `thermal_port_*` chips + attach write for `water_heater` / `fridge` / `hvac` / `dishwasher` / `dryer` (auto-gated on `thermal_port_id`).
   Read-only **Catalog heat ports** chips via wasm `list_heat_port_specs(class_id)`
   (`ClassTable.thermal_ports` / `HeatPortSpec`) sit alongside the live attach panel.
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
| `washer` | Optional depth: sabbath/eco/door_ajar/door_locked/water_temp_alarm/overflow_alarm/detergent_low/timer_s + typical detergent_level_percent/unbalance + trait delay_start_s (first undepened Tier-A deepen; I/O / interlock / cotton TCP already present) |
| `dryer` | Optional depth: sabbath/eco/door_ajar/door_locked/high_temp_alarm/lint_full/timer_s + typical anti_crease/dryness_percent/vent_blocked/drain_tank + trait delay_start_s (second undepened Tier-A deepen; I/O / interlock / cycle TCP / thermal already present) |
| `washer_dryer` | Optional depth: washer sabbath/eco/door/alarms/detergent/timer + dryer thin anti_crease/dryness/vent/drain + EXTRA high_temp_alarm/lint_full (not DRYER_DEPTH) + typical dry_after_wash/max_dry_s + trait delay_start_s (third undepened Tier-A deepen) |
| `fridge` | Optional depth: door_ajar/low_temp_alarm + typical cold-cabinet vacation/sabbath/eco/defrost/compressor/high_temp/power_fail (thermal ports unchanged; fourth undepened Tier-A deepen) |
| `freezer` | Optional depth: fast_freeze/door_ajar/ice_buildup/low_temp_alarm/anti_sweat/fast_freeze_remaining_s/frost_clean_needed + typical cold-cabinet vacation/sabbath/eco/defrost/compressor/high_temp/power_fail |
| `fridge_freezer` | Optional dual-zone depth: door_ajar_fridge/freezer, fast_freeze, ice_buildup, high_temp_alarm_fridge/freezer, convertible_zone_mode + typical cold-cabinet vacation/sabbath/eco/defrost/compressor/high_temp/power_fail (no thermal ports; freezer-only anti_sweat/frost_clean not copied) |
| `dishwasher` | Optional depth: sabbath/eco/door_ajar/door_locked/rinse_aid_low/salt_low/overflow_alarm/timer_s + typical rinse_aid_level/salt_level/wash_temp_c + trait delay_start_s (thermal ports unchanged; fifth undepened Tier-A deepen) |
| `microwave` | Optional depth: sabbath/eco/door_ajar/magnetron_on/high_temp_alarm/timer_s + typical power_w/defrost_g/turntable/inverter (child_lock trait already typical; cook_s/power_level_percent unchanged; sixth undepened Tier-A deepen) |
| `oven` | Optional depth: sabbath/eco/heater_on/high_temp_alarm/door_ajar/timer_s on OVEN_DEPTH (not merged into range/steam_oven/toaster_oven) + typical broil_level/convection_fan/steam_percent/cook_s/door_locked_clean/element_bake/element_broil + Temperature probe/preheat (self_clean stays program + door_locked_clean; seventh undepened Tier-A deepen; bake procedure + stub heat tick unchanged) |
| `steam_oven` | Optional depth: water_tank_level/descaling_needed/steam_generator_on/cavity_humidity/door_locked/drain_full/generator_fault/delayed_start + typical humidity_set/steam/cook/elements + cycle remaining/hardness |
| `range` | Optional depth: sabbath/eco/heater_on/high_temp_alarm/door_ajar on RANGE_EXTRA (not OVEN_DEPTH — avoids cooktop `timer_s` dup) + typical cooktop keep_warm/hotspot/timer_active/paused/surface_c/element_fault/pan_detect/flame_on + boost/timer/bridge/flame_out/ignition_fail/power_limit_w + OVEN_BASE broil/convection/steam/cook/door_locked_clean/elements + Temperature probe/preheat (eighth undepened Tier-A deepen) |
| `cooktop` | Optional depth: keep_warm/hotspot_alert/timer_active/paused/surface_c/element_fault/pan_detect/flame_on + typical boost/timer/bridge/flame_out/ignition_fail/power_limit_w |
| `induction_hob` | Optional depth: sabbath/eco/power_share/auto_boost/overtemp_alarm on INDUCTION_HOB_EXTRA + typical cooktop boost/timer/bridge/keep_warm/hotspot/timer_active/paused/surface_c/element_fault/pan_detect/flame_on/flame_out/ignition_fail/power_limit_w + thin pan_size/power_w/limiter/cookware/temp_mode/flex (child_lock already typical; do not redeclare cooktop timer_s/pan_detect/residual_heat; ninth undepened Tier-A deepen) |
| `air_fryer` | Optional depth: sabbath/eco/heater_on/fan_on/high_temp_alarm/door_ajar/timer_s on AIR_FRYER_DEPTH + typical shake_enable/shake_due/preheat/basket_present/sync_finish (cook_s unchanged; Heater/Fan/DoorLid already typical; tenth undepened Tier-A deepen; cook procedure + stub heat tick unchanged) |
| `kettle` | Optional depth: sabbath/eco/heater_on/high_temp_alarm/lid_open/timer_s on KETTLE_DEPTH + typical keep_warm/keep_warm_s/boil_dry (on_base unchanged; Heater already typical; eleventh undepened Tier-A deepen; Matter mock + boil setpoint/cycle surfaces unchanged) |
| `coffee_machine` | Optional depth: sabbath/eco/boiler_ready/high_temp_alarm/water_tank_empty/descaling_needed/carafe_present/timer_s on COFFEE_MACHINE_DEPTH + typical strength/volume_ml/milk_ml/grind_level/cups/drip_tray/grounds_bin/milk_present/capsule_present/boiler_c/brew_pressure_bar (water_tank unchanged; twelfth undepened Tier-A deepen; brew procedure + stub boiler heat tick unchanged) |
| `water_heater` | Optional depth: sabbath/eco/heater_on/high_temp_alarm/low_temp_alarm/leak_alarm/timer_s on WATER_HEATER_DEPTH + typical mode/inlet_c/outlet_c/hot_remaining_percent/leak/dry_fire/recirc_on/form_factor (Temperature setpoint + thermal_port_* unchanged; mode enum vacation — no vacation_mode bool; thirteenth undepened Tier-A deepen) |
| `hvac` | Optional depth: sabbath/fan_on/high_temp_alarm/low_temp_alarm/timer_s on HVAC_DEPTH + typical heat/cool setpoints/deadband/outdoor/hold/quiet/eco/compressor/aux/defrost/reversing_valve + trait humidity setpoint / fan speed / filter life (hvac_mode / space_c + thermal_port_* unchanged; reuse eco not eco_mode, defrost not defrost_active; fourteenth / last undepened Tier-A deepen) |
| `dehumidifier` | Optional depth: compressor/RH alarms/continuous/quiet/bucket/filter_dirty/delayed_start + tank_full/pump_mode/defrost + humidity setpoint + fan speed |
| `range_hood` | Optional depth: filter_dirty/boost/boost_remaining/light_level/grease_sensor/hob_linked/overtemp/charcoal_filter_life + typical auto_mode/delay_off/voc/grease+charcoal filters + fan speed/light_percent/filter life |
| `toaster_oven` | Optional depth: door_open/timer_remaining/delayed_start/rack/bagel/preheating/slices/toast_done + toast_shade/crumb_tray + convection/broil/cook/elements + cycle remaining |
| `sous_vide` | Optional depth: water_level_ok/lid_closed/timer_remaining/target_done/overtemp/delayed_start/alarm_offset + cycle remaining |
| `multi_cooker` | Optional depth: pot_detect/cook_s/delayed_start/keep_warm/saute_level/overpressure/lid_mismatch + pressure/float/burn + cycle remaining |
| `ice_maker` | Optional depth: water/scale/harvest alerts, scoop light, max-ice, delayed start + ice bin/filter life |
| `wine_cooler` | Optional depth: sabbath/compressor/alarms/vibration/bottle_count + humidity setpoint |

Count: **25** Tier-A ids, all with static tables + sim.

### Tier-B (thin static tables + sim) — done; optional depth mostly open

All remaining ids in the appliances catalog index (**31** = 56 − 25 Tier-A).
Each has a thinner `ClassTable` (typical traits + catalog class points) and
sim spawn via `typical_capability`. `STATIC_CLASS_IDS` = Tier-A ∪ Tier-B =
`ApplianceClassId::ALL`. To extend the catalog, see
[`catalog/ADDING_A_CLASS.md`](catalog/ADDING_A_CLASS.md).

**Optional-depth note:** `humidifier` (#71), `beverage_cooler`, `kegerator`,
`warming_drawer`, `pizza_oven`, `electric_grill`, `electric_smoker`, `espresso_machine`, `drip_coffee_maker`, `coffee_grinder`, `water_dispenser`, `toaster`, `blender`, `food_processor`, `stand_mixer`, `juicer`, `rice_cooker`, `slow_cooker`, `bread_maker`, `dehydrator`, `vacuum_sealer`, `ice_cream_maker`, `yogurt_maker`, `waffle_maker`, `pasta_maker`, `steam_cooker`, `garbage_disposal`, `trash_compactor`, `boiler`, `water_softener`, and `water_filter` received deepen-series optional-point passes. **0** Tier-B ids remain thin
(all 31 have optional-depth extras beyond the initial thin table). Most deepen-series work
(#56–#73) hit Tier-A classes listed in §4 Tier-A.

| Id | Notes |
|----|--------|
| `beverage_cooler` | Optional depth: sabbath/eco/compressor_on/high_temp_alarm/low_temp_alarm/door_ajar/can_capacity; setpoint 1–10 °C |
| `kegerator` | Optional depth: sabbath/eco/compressor_on/high_temp_alarm/low_temp_alarm/door_ajar/co2_kpa/keg_percent/keg_empty; setpoint 1–10 °C |
| `warming_drawer` | Optional depth: level/moist/sabbath/eco/heater_on/high_temp_alarm/door_ajar/timer_s; setpoint 40–90 °C |
| `pizza_oven` | Optional depth: stone_c/dome_c/top_bottom_balance/sabbath/eco/heater_on/high_temp_alarm/door_ajar/timer_s/steam_inject; setpoint 200–450 °C |
| `electric_grill` | Optional depth: plate_top_c/plate_bottom_c/sear/grease_tray/sabbath/eco/heater_on/high_temp_alarm/lid_open/timer_s; setpoint 100–250 °C |
| `electric_smoker` | Optional depth: chamber_c/smoke_on/fuel_percent/water_pan/sabbath/eco/heater_on/high_temp_alarm/door_ajar/timer_s; setpoint 50–150 °C |
| `espresso_machine` | Optional depth: brew_pressure_bar/shot_ml/pump_on/steam_wand_on/sabbath/eco/boiler_ready/high_temp_alarm/water_tank_empty/descaling_needed/timer_s; brew setpoint 85–100 °C |
| `drip_coffee_maker` | Optional depth: cups/strength/keep_warm_s/carafe_present/sabbath/eco/heater_on/high_temp_alarm/water_tank_empty/descaling_needed/timer_s; batch brew + keep-warm |
| `coffee_grinder` | Optional depth: grind_s/dose_g/hopper_present/sabbath/eco/motor_on/hopper_empty/bean_level_percent/timer_s/single_dose; burr dose / grind level |
| `water_dispenser` | Optional depth: hot/cold_setpoint_c/bottle_empty/sabbath/eco/heater_on/cooler_on/high_temp_alarm/low_temp_alarm/water_tank_empty; filter life + child_lock via traits |
| `toaster` | Optional depth: shade/bagel/frozen/single_side/carriage/sabbath/eco/heater_on/high_temp_alarm/timer_s/crumb_tray_full/slots |
| `blender` | Optional depth: speed_level/form_factor/pulse/jar_present/lid_locked/heated/sabbath/eco/motor_on/overload_trip/timer_s |
| `food_processor` | Optional depth: speed_level/pulse/bowl_present/lid_locked/attachment/sabbath/eco/motor_on/overload_trip/timer_s |
| `stand_mixer` | Optional depth: speed_level/bowl_present/head_down/mass_g/attachment/sabbath/eco/motor_on/overload_trip/timer_s |
| `juicer` | Optional depth: speed_level/reverse/pulp_full/jug_present/sabbath/eco/motor_on/overload_trip/timer_s |
| `rice_cooker` | Optional depth: texture/bowl_present/keep_warm/sabbath/eco/heater_on/high_temp_alarm/lid_open/timer_s/water_ratio |
| `slow_cooker` | Optional depth: heat_level/cook_s/pot_present/keep_warm/sabbath/eco/heater_on/high_temp_alarm/lid_open/timer_s |
| `bread_maker` | Optional depth: crust/loaf_size/pan_present/keep_warm/sabbath/eco/heater_on/high_temp_alarm/lid_open/timer_s |
| `dehydrator` | Optional depth: cook_s/sabbath/eco/heater_on/fan_on/high_temp_alarm/door_ajar/timer_s/tray_count; setpoint 30–75 °C |
| `vacuum_sealer` | Optional depth: mode/moist/vacuum_kpa/bag_detect/form_factor/sabbath/eco/pump_on/seal_heater_on/lid_locked/seal_fail/timer_s |
| `ice_cream_maker` | Optional depth: doneness/sabbath/eco/compressor_on/motor_on/bowl_present/lid_locked/low_temp_alarm/timer_s |
| `yogurt_maker` | Optional depth: incubate_s/sabbath/eco/heater_on/high_temp_alarm/low_temp_alarm/lid_open/jar_present/timer_s |
| `waffle_maker` | Optional depth: shade/ready/sabbath/eco/heater_on/high_temp_alarm/lid_open/batter_done/timer_s |
| `pasta_maker` | Optional depth: die/jam/sabbath/eco/motor_on/dough_ready/hopper_empty/die_present/overload_trip/timer_s |
| `steam_cooker` | Optional depth: cook_s/water_empty/sabbath/eco/heater_on/high_temp_alarm/lid_open/steam_ready/timer_s |
| `garbage_disposal` | Optional depth: run_s/jam/reset_needed/reverse/sabbath/eco/motor_on/overload_trip/air_switch/timer_s |
| `trash_compactor` | Optional depth: ram_state/bin_full/sabbath/eco/motor_on/drawer_open/overload_trip/key_lock/timer_s |
| `boiler` | Optional depth: pressure_bar/burner_on/flame_out/low_pressure/sabbath/eco/high_temp_alarm/lockout/ignition_fail/timer_s |
| `water_softener` | Optional depth: capacity_remaining/salt_level/bypass/treated_l/sabbath/eco/regenerating/salt_low/timer_s + trait hardness_ppm / filter life |
| `water_filter` | Optional depth: tds_in_ppm/tds_out_ppm/tank_full/sabbath/eco/bypass/filter_clogged/replace_needed/timer_s + trait filter life / flow_l_min |
| `humidifier` | Optional depth: warm_mist/auto_humidity/mineral_filter/uv_clean/scale_alert/tank_removed/misting/night_mode + typical output_level/mist_type/wick_state + humidity setpoint |

Count: **31** Tier-B ids, all with thin static tables + sim; **31** optional-depth
passes (`humidifier` + `beverage_cooler` + `kegerator` + `warming_drawer` + `pizza_oven` + `electric_grill` + `electric_smoker` + `espresso_machine` + `drip_coffee_maker` + `coffee_grinder` + `water_dispenser` + `toaster` + `blender` + `food_processor` + `stand_mixer` + `juicer` + `rice_cooker` + `slow_cooker` + `bread_maker` + `dehydrator` + `vacuum_sealer` + `ice_cream_maker` + `yogurt_maker` + `waffle_maker` + `pasta_maker` + `steam_cooker` + `garbage_disposal` + `trash_compactor` + `boiler` + `water_softener` + `water_filter`); **0** still thin.

---

## 5. Suggested sequencing (PRs)

| Order | Branch theme | Stream |
|------:|--------------|--------|
| A | `docs/roadmap-75` | 1 — this document |
| B | `feat/io-map-interlocks` | 1 — io_map + interlock crates |
| later | Tier-A table batches | 2 |
| later | procedure + sim | 3 — **Done** (kettle + Domino's + wash-then-dry + `oven_bake_180` + `coffee_brew_espresso` + `air_fryer_cook_200` + thin `thermal_wait` + `thermal_offer`) |
| later | HAL + controller-sim + TCP | 4 — TCP lab smoke + washer+dryer controller-sim-over-TCP interlock smoke **Done** |
| later | thermal ports | 5 — **Done (thin)** water_heater+fridge+hvac+dishwasher+dryer catalog/sim ports; schema vocabulary + `ClassTable.HeatPortSpec`; plant runtime still crate-local |
| later | `feat/bridges-modbus` | 6 — Modbus + stubs (first slice) |
| later | `feat/matter-mock-bridge` | 6 — Matter mock fabric + kettle map |
| later | `feat/simulator-tier-a-ui` | 7 — grouped Tier-A picker (first UI slice) |
| later | WASM UI + conformance suite | 7 — picker + procedure UI (kettle/Domino's/wash-then-dry/oven bake/coffee brew/air fryer cook) + thermal UI + device port chips + blob-load done; smoke suite + write-denial matrix + hub-in-suite done; richer UI remaining |
| later | Tier-B thin tables | 2 — **Done** (31 Tier-B → 56 total static + sim) |
| later | catalog optional depth | 7 — **Series progress** (#56–#73 + follow-on): 56 classes deepened (`wine_cooler` + `ice_maker` + `sous_vide` + `multi_cooker` + `toaster_oven` + `dehumidifier` + `range_hood` + `steam_oven` + `cooktop` + `humidifier` + `freezer` + `fridge_freezer` + `beverage_cooler` + `kegerator` + `warming_drawer` + `pizza_oven` + `electric_grill` + `electric_smoker` + `espresso_machine` + `drip_coffee_maker` + `coffee_grinder` + `water_dispenser` + `toaster` + `blender` + `food_processor` + `stand_mixer` + `juicer` + `rice_cooker` + `slow_cooker` + `bread_maker` + `dehydrator` + `vacuum_sealer` + `ice_cream_maker` + `yogurt_maker` + `waffle_maker` + `pasta_maker` + `steam_cooker` + `garbage_disposal` + `trash_compactor` + `boiler` + `water_softener` + `water_filter` + `washer` + `dryer` + `washer_dryer` + `fridge` + `dishwasher` + `microwave` + `oven` + `range` + `induction_hob` + `air_fryer` + `kettle` + `coffee_machine` + `water_heater` + `hvac`); **0/31 Tier-B still thin**; all listed undepened Tier-A classes now have optional-depth passes (0 remaining); honest caveats still apply for real bridges, TLS, typical_capability, etc.
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
| 0.1.32 | Stream 5: `ClassTable.thermal_ports: &[HeatPortSpec]` advertisement matching sim seeds (water_heater/fridge/hvac/dishwasher/dryer) |
| 0.1.33 | Stream 7 catalog depth: deepen `wine_cooler` optional class points (sabbath/compressor/alarms/vibration_alert/bottle_count + typical humidity setpoint); Tier-B deepen started |
| 0.1.34 | Stream 7 catalog depth: deepen `ice_maker` optional class points (water_low/scoop_light/max_ice_mode/harvest_fail/scale_alert/delayed_start_s + typical ice bin/filter life) |
| 0.1.35 | Current-state refresh: ~70% → **~72%** (~71–73% band) of 75% target; cite PRs #54–#57 (schema thermal vocab, `ClassTable.HeatPortSpec`, `wine_cooler` + `ice_maker` optional depth); §2 gaps narrowed in wording, not cleared |
| 0.1.36 | Stream 7: wasm `list_heat_port_specs(class_id)` exposes `ClassTable.thermal_ports`; simulator-web read-only Catalog heat ports chips alongside live `thermal_port_*` panel |
| 0.1.37 | Stream 4: washer controller TCP cotton start (`trait.cycle.start` + readable `cycle_state`/`cycle_phase` + `class.washer.sim_tick`); conformance `controller_tcp_washer_cotton`; CottonOptions/cancel/dryer cycle-over-TCP deferred |
| 0.1.38 | Current-state refresh: **~75% of the §2 in-scope bar met in spirit** for lab/software depth; cite PRs #54–#60 (thermal vocab + HeatPortSpec + UI, Tier-B wine_cooler/ice_maker, cotton-over-TCP); Still open reframed as beyond/thin (real bridge SDKs, plant runtime schema, TLS, fuller Tier-B, procedure offer/negotiate, dryer cycle-over-TCP, CottonOptions over wire); no IEC / production-firmware claim |
| 0.1.39 | Stream 4: dryer controller TCP cycle start (`trait.cycle.start` + readable `cycle_state`/`cycle_phase` + `class.dryer.sim_tick`); conformance `controller_tcp_dryer_cycle`; CottonOptions/DryOptions/cancel/pause deferred |
| 0.1.40 | Stream 7 catalog depth: deepen `sous_vide` optional class points (water_level_ok/lid_closed/timer_remaining_s/target_done/overtemp_alarm/delayed_start_s/alarm_offset_c + typical cycle remaining) |
| 0.1.41 | Stream 7 catalog depth: deepen `multi_cooker` optional class points (pot_detect/cook_s/delayed_start_s/keep_warm/keep_warm_s/saute_level/overpressure_alarm/lid_mismatch + typical pressure/float/burn/remote_vent + cycle remaining) |
| 0.1.42 | Stream 3/5: thin procedure⇄thermal `thermal_offer` / `offer_transfer` (offer + immediate accept) + backend hooks + `offer_fridge_dhw` + conformance `procedure_thermal_offer_dhw`; multi-round dialogue / soft decline / richer wasm UI deferred |
| 0.1.43 | Stream 7 catalog depth: deepen `toaster_oven` optional class points (door_open/timer_remaining_s/delayed_start_s/rack_position/bagel/preheating/slices/toast_done + typical toast_shade/crumb_tray/convection/broil/cook/elements + cycle remaining); copy `offer_fridge_dhw` into simulator-web procedures |
| 0.1.44 | Stream 7 catalog depth: deepen `dehumidifier` optional class points (compressor_on/high_rh_alarm/low_rh_alarm/continuous_mode/quiet_mode/bucket_removed/filter_dirty/delayed_start_s + typical tank_full/pump_mode/defrost + humidity setpoint + fan speed) |
| 0.1.45 | Stream 7 catalog depth: deepen `range_hood` optional class points (filter_dirty/boost/boost_remaining_s/light_level/grease_sensor/hob_linked/overtemp/charcoal_filter_life_percent + typical auto_mode/delay_off_s/voc_index/grease_filter/charcoal_filter + fan speed/light_percent/filter life) |
| 0.1.46 | Stream 7 catalog depth: deepen `steam_oven` optional class points (water_tank_level/descaling_needed/steam_generator_on/cavity_humidity/door_locked/drain_full/generator_fault/delayed_start_s + typical humidity_set_percent/steam_percent/convection/cook/elements + cycle remaining/hardness) |
| 0.1.47 | Stream 7 catalog depth: deepen `cooktop` optional class points (keep_warm/hotspot_alert/timer_active/paused/surface_c/element_fault/pan_detect/flame_on + typical boost/timer_s/bridge/flame_out/ignition_fail/power_limit_w) |
| 0.1.48 | Stream 7 catalog depth: deepen `humidifier` optional class points (warm_mist/auto_humidity/mineral_filter/uv_clean/scale_alert/tank_removed/misting/night_mode + typical output_level/mist_type/wick_state + humidity setpoint) |
| 0.1.49 | Stream 7 catalog depth: deepen `freezer` optional class points (fast_freeze/door_ajar/ice_buildup/low_temp_alarm/anti_sweat/fast_freeze_remaining_s/frost_clean_needed merged onto shared cold-cabinet; typical also advertises vacation/sabbath/eco/defrost/compressor/high_temp/power_fail; fridge_freezer unchanged) |
| 0.1.50 | Stream 7 catalog depth: deepen `fridge_freezer` optional dual-zone class points (door_ajar_fridge/freezer, fast_freeze, ice_buildup, high_temp_alarm_fridge/freezer, convertible_zone_mode merged onto shared cold-cabinet; typical also advertises vacation/sabbath/eco/defrost/compressor/high_temp/power_fail; fridge thermal ports and freezer FREEZER_EXTRA unchanged) |
| 0.1.51 | Docs refresh after Tier-B / catalog optional-depth series (#56–#73) + related: list 12 deepened classes; call out remaining thin Tier-B (30/31) and undepened Tier-A; keep **~75% met in spirit** (depth improved, not catalog-complete); highlight `thermal_offer` (#65) + dryer cycle TCP (#62); no fabricated metrics |
| 0.1.52 | Stream 7 catalog depth: deepen `beverage_cooler` optional class points (sabbath_mode/eco_mode/compressor_on/high_temp_alarm/low_temp_alarm/door_ajar/can_capacity); remaining thin Tier-B 30→29 |
| 0.1.53 | Stream 7 catalog depth: deepen `kegerator` optional class points (sabbath_mode/eco_mode/compressor_on/high_temp_alarm/low_temp_alarm/door_ajar + typical co2_kpa/keg_percent/keg_empty); remaining thin Tier-B 29→28 |
| 0.1.54 | Stream 7 catalog depth: deepen `warming_drawer` optional class points (sabbath_mode/eco_mode/heater_on/high_temp_alarm/door_ajar/timer_s + typical level/moist); remaining thin Tier-B 28→27 |
| 0.1.55 | Stream 7 catalog depth: deepen `pizza_oven` optional class points (sabbath_mode/eco_mode/heater_on/high_temp_alarm/door_ajar/timer_s/steam_inject + typical stone_c/dome_c/top_bottom_balance); remaining thin Tier-B 27→26 |
| 0.1.56 | Stream 7 catalog depth: deepen `electric_grill` optional class points (sabbath_mode/eco_mode/heater_on/high_temp_alarm/lid_open/timer_s + typical plate_top_c/plate_bottom_c/sear/grease_tray); remaining thin Tier-B 26→25 |
| 0.1.57 | Stream 7 catalog depth: deepen `electric_smoker` optional class points (sabbath_mode/eco_mode/heater_on/high_temp_alarm/door_ajar/timer_s + typical chamber_c/smoke_on/fuel_percent/water_pan); remaining thin Tier-B 25→24 |
| 0.1.58 | Stream 7 catalog depth: deepen `espresso_machine` optional class points (sabbath_mode/eco_mode/boiler_ready/high_temp_alarm/water_tank_empty/descaling_needed/timer_s/steam_wand_on + typical brew_pressure_bar/shot_ml/pump_on); remaining thin Tier-B 24→23 |
| 0.1.59 | Stream 7 catalog depth: deepen `drip_coffee_maker` optional class points (sabbath_mode/eco_mode/heater_on/high_temp_alarm/water_tank_empty/descaling_needed/timer_s + typical cups/strength/keep_warm_s/carafe_present); remaining thin Tier-B 23→22 |
| 0.1.60 | Stream 7 catalog depth: deepen `coffee_grinder` optional class points (sabbath_mode/eco_mode/motor_on/hopper_empty/bean_level_percent/timer_s/single_dose + typical grind_s/dose_g/hopper_present); remaining thin Tier-B 22→21 |
| 0.1.61 | Stream 7 catalog depth: deepen `water_dispenser` optional class points (sabbath_mode/eco_mode/heater_on/cooler_on/high_temp_alarm/low_temp_alarm/water_tank_empty + typical hot/cold_setpoint_c/bottle_empty; advertise trait filter life + child_lock); remaining thin Tier-B 21→20 |
| 0.1.62 | Stream 7 catalog depth: deepen `toaster` optional class points (sabbath_mode/eco_mode/heater_on/high_temp_alarm/timer_s/crumb_tray_full/slots + typical shade/bagel/frozen/single_side/carriage); remaining thin Tier-B 20→19 |
| 0.1.63 | Stream 7 catalog depth: deepen `blender` optional class points (sabbath_mode/eco_mode/motor_on/overload_trip/timer_s + typical form_factor/pulse/jar_present/lid_locked/heated); remaining thin Tier-B 19→18 |
| 0.1.64 | Stream 7 catalog depth: deepen `food_processor` optional class points (sabbath_mode/eco_mode/motor_on/overload_trip/timer_s + typical pulse/bowl_present/lid_locked/attachment); remaining thin Tier-B 18→17 |
| 0.1.65 | Stream 7 catalog depth: deepen `stand_mixer` optional class points (sabbath_mode/eco_mode/motor_on/overload_trip/timer_s/attachment + typical bowl_present/head_down/mass_g); remaining thin Tier-B 17→16 |
| 0.1.66 | Stream 7 catalog depth: deepen `juicer` optional class points (sabbath_mode/eco_mode/motor_on/overload_trip/timer_s + typical reverse/pulp_full/jug_present); remaining thin Tier-B 16→15 |
| 0.1.67 | Stream 7 catalog depth: deepen `rice_cooker` optional class points (sabbath_mode/eco_mode/heater_on/high_temp_alarm/lid_open/timer_s/water_ratio + typical texture/bowl_present/keep_warm); remaining thin Tier-B 15→14 |
| 0.1.68 | Stream 7 catalog depth: deepen `slow_cooker` optional class points (sabbath_mode/eco_mode/heater_on/high_temp_alarm/lid_open/timer_s + typical pot_present/keep_warm; required heat_level/cook_s); remaining thin Tier-B 14→13 |
| 0.1.69 | Stream 7 catalog depth: deepen `bread_maker` optional class points (sabbath_mode/eco_mode/heater_on/high_temp_alarm/lid_open/timer_s/keep_warm + typical crust/loaf_size/pan_present); remaining thin Tier-B 13→12 |
| 0.1.70 | Stream 7 catalog depth: deepen `dehydrator` optional class points (sabbath_mode/eco_mode/heater_on/fan_on/high_temp_alarm/door_ajar/timer_s/tray_count + required cook_s); remaining thin Tier-B 12→11 |
| 0.1.71 | Stream 7 catalog depth: deepen `vacuum_sealer` optional class points (sabbath_mode/eco_mode/pump_on/seal_heater_on/lid_locked/seal_fail/timer_s + typical moist/vacuum_kpa/bag_detect/form_factor); remaining thin Tier-B 11→10 |
| 0.1.72 | Stream 7 catalog depth: deepen `ice_cream_maker` optional class points (sabbath_mode/eco_mode/compressor_on/motor_on/bowl_present/lid_locked/low_temp_alarm/timer_s + typical doneness); remaining thin Tier-B 10→9 |
| 0.1.73 | Stream 7 catalog depth: deepen `yogurt_maker` optional class points (sabbath_mode/eco_mode/heater_on/high_temp_alarm/low_temp_alarm/lid_open/jar_present/timer_s + required incubate_s); remaining thin Tier-B 9→8 |
| 0.1.74 | Stream 7 catalog depth: deepen `waffle_maker` optional class points (sabbath_mode/eco_mode/heater_on/high_temp_alarm/lid_open/batter_done/timer_s + typical shade/ready); remaining thin Tier-B 8→7 |
| 0.1.75 | Stream 7 catalog depth: deepen `pasta_maker` optional class points (sabbath_mode/eco_mode/motor_on/dough_ready/hopper_empty/die_present/overload_trip/timer_s + typical die/jam); remaining thin Tier-B 7→6 |
| 0.1.76 | Stream 7 catalog depth: deepen `steam_cooker` optional class points (sabbath_mode/eco_mode/heater_on/high_temp_alarm/lid_open/steam_ready/timer_s + required cook_s/water_empty); remaining thin Tier-B 6→5 |
| 0.1.77 | Stream 7 catalog depth: deepen `garbage_disposal` optional class points (sabbath_mode/eco_mode/motor_on/overload_trip/air_switch/timer_s + typical run_s/jam/reset_needed/reverse); remaining thin Tier-B 5→4 |
| 0.1.78 | Stream 7 catalog depth: deepen `trash_compactor` optional class points (sabbath_mode/eco_mode/motor_on/drawer_open/overload_trip/key_lock/timer_s + typical ram_state/bin_full); remaining thin Tier-B 4→3 |
| 0.1.79 | Stream 7 catalog depth: deepen `boiler` optional class points (sabbath_mode/eco_mode/high_temp_alarm/lockout/ignition_fail/timer_s + typical pressure_bar/burner_on/flame_out/low_pressure); remaining thin Tier-B 3→2 |
| 0.1.80 | Stream 7 catalog depth: deepen `water_softener` optional class points (sabbath_mode/eco_mode/regenerating/salt_low/timer_s + typical capacity_remaining/salt_level/bypass/treated_l + trait hardness_ppm / filter life); remaining thin Tier-B 2→1 |
| 0.1.81 | Stream 7 catalog depth: deepen `water_filter` optional class points (sabbath_mode/eco_mode/bypass/filter_clogged/replace_needed/timer_s + typical tds_in_ppm/tds_out_ppm/tank_full + trait filter life / flow_l_min); remaining thin Tier-B 1→0 (all 31 Tier-B optional-depth passes done) |
| 0.1.82 | Stream 7 catalog depth: deepen `washer` optional class points (sabbath_mode/eco_mode/door_ajar/door_locked/water_temp_alarm/overflow_alarm/detergent_low/timer_s + typical detergent_level_percent/unbalance + trait delay_start_s); first undepened Tier-A deepen in the series (0.1.82) |
| 0.1.83 | Stream 7 catalog depth: deepen `dryer` optional class points (sabbath_mode/eco_mode/door_ajar/door_locked/high_temp_alarm/lint_full/timer_s + typical anti_crease/dryness_percent/vent_blocked/drain_tank + trait delay_start_s); second undepened Tier-A deepen in the series (0.1.83) |
| 0.1.84 | Stream 7 catalog depth: deepen `washer_dryer` optional class points (advertise washer sabbath/eco/door/alarms/detergent/timer + dryer thin anti_crease/dryness/vent/drain; add high_temp_alarm/lint_full on EXTRA not DRYER_DEPTH; typical dry_after_wash/max_dry_s + trait delay_start_s); third undepened Tier-A deepen in the series (0.1.84) |
| 0.1.85 | Stream 7 catalog depth: deepen `fridge` optional class points (advertise cold-cabinet vacation/sabbath/eco/defrost/compressor/high_temp/power_fail; add door_ajar/low_temp_alarm; thermal ports unchanged); fourth undepened Tier-A deepen in the series (0.1.85) |
| 0.1.86 | Stream 7 catalog depth: deepen `dishwasher` optional class points (sabbath_mode/eco_mode/door_ajar/door_locked/rinse_aid_low/salt_low/overflow_alarm/timer_s + typical rinse_aid_level/salt_level/wash_temp_c + trait delay_start_s; thermal ports unchanged); fifth undepened Tier-A deepen in the series (0.1.86) |
| 0.1.87 | Stream 7 catalog depth: deepen `microwave` optional class points (sabbath_mode/eco_mode/door_ajar/magnetron_on/high_temp_alarm/timer_s + typical power_w/defrost_g/turntable/inverter; child_lock trait already typical; cook_s/power_level_percent unchanged); sixth undepened Tier-A deepen in the series (0.1.87) |
| 0.1.88 | Stream 7 catalog depth: deepen `oven` optional class points (sabbath_mode/eco_mode/heater_on/high_temp_alarm/door_ajar/timer_s on OVEN_DEPTH; typical broil_level/convection_fan/steam_percent/cook_s/door_locked_clean/element_bake/element_broil + Temperature probe/preheat; OVEN_BASE kept for range/steam_oven/toaster_oven composition; self_clean stays program + door_locked_clean); seventh undepened Tier-A deepen in the series (0.1.88) |
| 0.1.89 | Stream 7 catalog depth: deepen `range` optional class points (advertise cooktop depth + OVEN_BASE cavity thin surface; add RANGE_EXTRA sabbath_mode/eco_mode/heater_on/high_temp_alarm/door_ajar — not OVEN_DEPTH to avoid cooktop timer_s duplicate; typical Temperature probe/preheat); eighth undepened Tier-A deepen in the series (0.1.89) |
| 0.1.90 | Stream 7 catalog depth: deepen `induction_hob` optional class points (advertise cooktop depth + thin pan_size/power_w/limiter/cookware/temp_mode/flex; add EXTRA sabbath_mode/eco_mode/power_share/auto_boost/overtemp_alarm — do not redeclare cooktop timer_s/pan_detect/residual_heat; child_lock already typical); ninth undepened Tier-A deepen in the series (0.1.90) |
| 0.1.91 | Stream 7 catalog depth: deepen `air_fryer` optional class points (advertise thin shake_enable/shake_due/preheat/basket_present/sync_finish; add AIR_FRYER_DEPTH sabbath_mode/eco_mode/heater_on/fan_on/high_temp_alarm/door_ajar/timer_s — cook_s unchanged; Heater/Fan/DoorLid already typical); tenth undepened Tier-A deepen in the series (0.1.91) |
| 0.1.92 | Stream 7 catalog depth: deepen `kettle` optional class points (advertise thin keep_warm/keep_warm_s/boil_dry; add KETTLE_DEPTH sabbath_mode/eco_mode/heater_on/high_temp_alarm/lid_open/timer_s — on_base unchanged; Heater already typical; Matter mock / boil surfaces unchanged); eleventh undepened Tier-A deepen in the series (0.1.92) |
| 0.1.93 | Stream 7 catalog depth: deepen `coffee_machine` optional class points (advertise thin strength/volume_ml/milk_ml/grind_level/cups/drip_tray/grounds_bin/milk_present/capsule_present/boiler_c/brew_pressure_bar; add COFFEE_MACHINE_DEPTH sabbath_mode/eco_mode/boiler_ready/high_temp_alarm/water_tank_empty/descaling_needed/carafe_present/timer_s — water_tank unchanged; brew procedure + stub boiler heat tick unchanged); twelfth undepened Tier-A deepen in the series (0.1.93) |
| 0.1.94 | Stream 7 catalog depth: deepen `water_heater` optional class points (advertise thin mode/inlet_c/outlet_c/hot_remaining_percent/leak/dry_fire/recirc_on/form_factor; add WATER_HEATER_DEPTH sabbath_mode/eco_mode/heater_on/high_temp_alarm/low_temp_alarm/leak_alarm/timer_s — Temperature setpoint + thermal_port_* unchanged; mode enum vacation — no vacation_mode bool); thirteenth undepened Tier-A deepen in the series (0.1.94) |
| 0.1.95 | Stream 7 catalog depth: deepen `hvac` optional class points (advertise thin heat/cool setpoints/deadband/outdoor/hold/quiet/eco/compressor_on/aux_heat/defrost/reversing_valve; add HVAC_DEPTH sabbath_mode/fan_on/high_temp_alarm/low_temp_alarm/timer_s — hvac_mode / space_c + thermal_port_* unchanged; reuse eco not eco_mode, defrost not defrost_active; advertise trait humidity setpoint / fan speed / filter life); fourteenth / last undepened Tier-A deepen in the series (0.1.95); remaining undepened Tier-A now 0 |
| 0.1.96 | Stream 4: washer **CottonOptions** over lab TCP (adjacent `class.washer.wash_temp_c` / `spin_rpm` writes before void `trait.cycle.start`); conformance `controller_tcp_washer_cotton_options`; DryOptions / cancel / pause / typical_capability remain follow-up |
| 0.1.97 | Stream 4: dryer **DryOptions** over lab TCP (adjacent `class.dryer.dryness` / `heat_level` writes before void `trait.cycle.start`; map onto host humidity / temp targets); conformance `controller_tcp_dryer_dry_options`; cancel / pause / typical_capability remain follow-up |
| 0.1.98 | Stream 4: washer + dryer **cycle cancel / pause / resume** over lab TCP (`trait.cycle.cancel` / `pause` / `resume`; host `Paused`/`Canceling` → drain/cool → `idle`); conformance `controller_tcp_washer_cycle_pause_cancel` / `controller_tcp_dryer_cycle_pause_cancel`; typical_capability remains follow-up |
| 0.1.99 | Stream 4: washer + dryer **typical_capability over lab TCP** (Describe = catalog typical ∪ lab HAL/`sim_tick`/cycle pause-phase/DryOptions; unbound typical points store/default); conformance `controller_tcp_washer_typical_capability` / `controller_tcp_dryer_typical_capability` |
