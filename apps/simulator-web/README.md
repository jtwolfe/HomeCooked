# HomeCooked web simulator

Minimal static UI that loads the `homecooked-wasm` package produced by
`wasm-pack`. No bundler or framework.

See [`docs/ROADMAP.md`](../../docs/ROADMAP.md) Stream 7 (WASM UI +
conformance) and §4 (Tier-A ∪ Tier-B class sets).

## Build WASM

From the repository root (note `--out-dir` is relative to the wasm crate):

```bash
wasm-pack build crates/homecooked-wasm --target web --out-dir ../../apps/simulator-web/pkg
```

This writes `apps/simulator-web/pkg/` (`homecooked_wasm.js` +
`homecooked_wasm_bg.wasm`). That directory is gitignored; rebuild after
Rust changes.

Requires the `wasm32-unknown-unknown` target:

```bash
rustup target add wasm32-unknown-unknown
```

`wasm-pack` is already the documented tool; `cargo install wasm-pack` if
it is missing.

## Serve

ES modules and WASM will not load from `file://`. Serve this directory:

```bash
cd apps/simulator-web
python3 -m http.server 8080
```

Open <http://127.0.0.1:8080>.

## UI

1. Pick a spawnable appliance class from the grouped picker
   (`<optgroup>`s match the catalog Index in
   [`docs/catalog/appliances.md`](../../docs/catalog/appliances.md):
   Laundry / Cold / Wash / Cooking / Ventilation / Beverage / Countertop /
   Utility / Climate). All **56** statically tabled classes
   (`STATIC_CLASS_IDS` = Tier-A ∪ Tier-B) are listed; class id
   is shown next to the label.
2. Create a device.
3. Inspect identity (class id is highlighted in the device header), a few
   key telemetry chips (power / temperature / cycle when present),
   variables, and settings.
4. For classes that advertise `thermal_port_id` (`water_heater` / `fridge` /
   `hvac` / `dishwasher` / `dryer`), a compact **Thermal port** panel shows catalog `thermal_port_*`
   telemetry (id, direction, media, max power, attached reservoir) and a write
   field for `thermal_port_attached_reservoir_id` (e.g. `dhw-tank`). Auto-shown
   when the point is present; hidden otherwise (no class-id hardcoding).
5. Write settings / fire commands (`start`, `power_on`, …).
6. Use **Tick** or **Auto tick** so simulated behavior (kettle / oven / air fryer heat, washer / dryer /
   microwave progress) advances.

Writes that fail capability checks (`out_of_range`, `not_writable`, …) show
an error banner and leave device state unchanged.

## Procedure panel

The lower **Procedure** panel loads a bundled recipe or accepts pasted
procedure JSON, then runs it through `homecooked-procedure` against the
current simulator world.

1. Pick a sample (**Heat kettle to 80C**, Domino’s microwave, **Wash then dry**, **Oven bake at 180C**, **Brew espresso**, or **Air fryer cook at 200C**)
   or paste JSON.
2. **Load sample** fills the editor (WASM `get_example_procedure`, with a
   fetch fallback to `procedures/*.json`).
3. **Parse** validates the document (`parse_procedure`) and shows a short
   summary.
4. **Run** auto-binds each required role to an existing sim device of a
   matching class, or **spawns** that class if none is present. Optional
   roles bind only when a match already exists. After the run, the device
   list refreshes so spawned appliances appear.

Results show completed/failed status, role bindings, and per-step ok/fail
(with messages / fail reason). Static copies of the fixtures live in
[`procedures/`](procedures/) and stay in sync with
`crates/homecooked-procedure/examples/`.

The kettle sample is the happy-path demo (sim heats ~5 °C/s). The microwave
fixture writes cook settings and starts a cycle; sim ticks advance
`trait.cycle.elapsed_s` toward `class.microwave.cook_s` so the wait step can complete.
The oven bake sample sets `bake` + 180 °C; sim heats ~10 °C/s while the cycle runs.
The coffee brew sample powers on, selects `espresso`, and waits on boiler heat (~10 °C/s).
The air fryer cook sample sets `fries` + 200 °C; sim heats ~10 °C/s while the cycle runs.

## Thermal plant panel

The **Thermal plant** panel loads the fridge condenser → DHW demo from
`homecooked-thermal` (same scenario as the crate integration test).

1. **Load demo** creates/resets the plant (`create_thermal_demo`).
2. **Negotiate** queues a best-effort fridge→water_heater offer
   (`thermal_negotiate_demo`).
3. **Tick** applies queued accepts over `dt` seconds (`thermal_tick`).
4. **Transfer + tick** negotiates and steps in one shot
   (`thermal_demo_transfer`). With the default `dt = 3600`, DHW rises from
   35.0 °C to 36.2 °C at 120 W.

The panel lists reservoirs (temps), heat ports, and the last transfer
results / reply.

### Dual-path: thermal then dishwasher preheat

Procedures cannot call thermal APIs yet. The **Orchestrations** block on the
Thermal plant panel exposes a one-click button that calls wasm
`run_thermal_then_dishwasher_preheat(dt)` (default **dt = 3600**):

1. Click **Thermal → dishwasher preheat**.
2. The panel shows DHW start → end (°C rise), dishwasher role bindings, and
   per-step write outcomes (`eco`, `wash_temp_c` = 45).
3. Spawned dishwasher appears in the device list and is selected.

You can still run the legs separately: **Transfer + tick** on the thermal
toolbar, then load **Dishwasher with DHW preheat available** in the procedure
panel and **Run**.

Automated coverage: `cargo test -p homecooked-wasm run_thermal_then_dishwasher`
and conformance scenario `thermal_then_dishwasher_preheat`.

## Rebuild note

After any Rust change under `crates/homecooked-wasm` (or crates it depends on),
rebuild the gitignored `pkg/` and hard-refresh the browser so blob-load boot
picks up new exports:

```bash
wasm-pack build crates/homecooked-wasm --target web --out-dir ../../apps/simulator-web/pkg
cd apps/simulator-web && python3 -m http.server 8080
```

`app.js` still uses the blob-load / `cache: "no-store"` boot path — do not
switch back to a plain static `import` of `./pkg/homecooked_wasm.js`.

### Optional manual test (orchestrator)

1. Rebuild + serve as above; open <http://127.0.0.1:8080>.
2. Scroll to **Thermal plant → Orchestrations**.
3. Leave **dt (s)** at `3600`; click **Thermal → dishwasher preheat**.
4. Expect: status shows DHW ~35.00 → ~36.20 °C; bindings include dishwasher;
   steps ok; device list shows a dishwasher.

## Manual smoke (after wasm-pack)

With the page served as above, spawn at least:

| Class id | Group | Expect |
|----------|-------|--------|
| `wine_cooler` | Cold | Device appears; header shows `wine_cooler`; no JS console errors |
| `hvac` | Climate | **Thermal port** panel: id `coil`, direction `sink`, media `water`, max 5000 W; Set attach works |
| `steam_oven` | Cooking | Same; steam / program points render |
| `water_heater` | Utility | **Thermal port** panel: id `preheat`, direction `sink`, media `water`, max 2000 W; Set attach to `dhw-tank` |
| `fridge` | Cold | **Thermal port** panel: id `condenser`, direction `source`, media `water`, max 120 W; Set attach works |
| `kettle` | Beverage | **Thermal port** panel stays hidden |

### Optional manual test (device thermal ports)

1. Ensure `pkg/` includes Stream 5 catalog thermal ports (rebuild if your
   package predates that slice):
   `wasm-pack build crates/homecooked-wasm --target web --out-dir ../../apps/simulator-web/pkg`
2. Serve `apps/simulator-web` and hard-refresh (blob-load boot).
3. Create a **water_heater**; confirm the **Thermal port** chips (`preheat` /
   `sink` / `water` / 2000) and empty attached reservoir.
4. Enter `dhw-tank` → **Set**; chips / raw state show the attach string.
5. Create a **fridge**; confirm `condenser` / `source` / 120 W chips.
6. Create an **hvac**; confirm `coil` / `sink` / `water` / 5000 W chips; Set attach.
7. Create a **kettle**; confirm the thermal-port panel is absent.

Automated coverage lives in `crates/homecooked-wasm` native tests:
`list_appliance_classes` length is 56 and matches `STATIC_CLASS_IDS`;
`create_device` + `describe` + `get_state` succeed for every tabled id.

After `wasm-pack build`, hard-refresh the browser. `app.js` boots via blob-load
(`cache: "no-store"` fetch of the bindgen JS + rewritten absolute `.wasm` URL)
so new exports (procedures, thermal, orchestrations) load after rebuilds.
