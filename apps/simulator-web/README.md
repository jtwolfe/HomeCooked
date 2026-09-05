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
4. Write settings / fire commands (`start`, `power_on`, …).
5. Use **Tick** or **Auto tick** so simulated behavior (kettle heat, washer / dryer /
   microwave progress) advances.

Writes that fail capability checks (`out_of_range`, `not_writable`, …) show
an error banner and leave device state unchanged.

## Procedure panel

The lower **Procedure** panel loads a bundled recipe or accepts pasted
procedure JSON, then runs it through `homecooked-procedure` against the
current simulator world.

1. Pick a sample (**Heat kettle to 80C**, Domino’s microwave, or **Wash then dry**)
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

Procedures cannot call thermal APIs yet. Demo both legs:

1. Use this thermal panel (or wasm `run_thermal_then_dishwasher_preheat(3600)`).
2. In the procedure panel, load **Dishwasher with DHW preheat available**
   (`dishwasher_dhw_preheat`) and Run — writes `eco` + `wash_temp_c` = 45.

Automated coverage: `cargo test -p homecooked-wasm run_thermal_then_dishwasher`
and conformance scenario `thermal_then_dishwasher_preheat`.

## Manual smoke (after wasm-pack)

With the page served as above, spawn at least:

| Class id | Group | Expect |
|----------|-------|--------|
| `wine_cooler` | Cold | Device appears; header shows `wine_cooler`; no JS console errors |
| `hvac` | Climate | Same; `class.hvac.*` points render |
| `steam_oven` | Cooking | Same; steam / program points render |

Automated coverage lives in `crates/homecooked-wasm` native tests:
`list_appliance_classes` length is 56 and matches `STATIC_CLASS_IDS`;
`create_device` + `describe` + `get_state` succeed for every tabled id.

After `wasm-pack build`, hard-refresh the browser. `app.js` cache-busts the WASM module URL via `pkg/package.json` so new exports (procedures, thermal) load after rebuilds.
