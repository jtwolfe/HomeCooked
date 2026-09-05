# HomeCooked

Universal standard communication interface for whitegoods and kitchen appliances:
discover devices, read telemetry/state, and write settings/commands.

This repository is under active construction. The **docs catalog is the source
of truth**; code must track `docs/catalog/` and `docs/standard/`.

## Documentation

- [`docs/ROADMAP.md`](docs/ROADMAP.md) — ~75% of §2 in-scope bar **met in spirit**
  (lab/software depth; catalog optional depth improved via deepen series incl.
  `beverage_cooler` / `kegerator` / `warming_drawer` / `pizza_oven` / `electric_grill` / `electric_smoker` / `espresso_machine` / `drip_coffee_maker` / `coffee_grinder` / `water_dispenser` / `toaster` / `blender` / `food_processor` / `stand_mixer` / `juicer` / `rice_cooker` / `slow_cooker` / `bread_maker` / `dehydrator` / `vacuum_sealer` / `ice_cream_maker` / `yogurt_maker` / `waffle_maker` / `pasta_maker` / `steam_cooker` / `garbage_disposal` / `trash_compactor` / `boiler` / `water_softener` / `water_filter` / `washer` / `dryer` / `washer_dryer` / `fridge` / `dishwasher` / `microwave` / `oven` / `range` / `induction_hob` / `air_fryer`; all 31 Tier-B have optional-depth passes — undepened Tier-A deepen series: `washer` + `dryer` + `washer_dryer` + `fridge` + `dishwasher` + `microwave` + `oven` + `range` + `induction_hob` + `air_fryer`; remaining undepened Tier-A / real bridges / etc. still open); Tier-A/B classes, workstreams,
  `thermal_offer` + dryer cycle TCP called out; definitions of done
- [`docs/catalog/appliances.md`](docs/catalog/appliances.md) — appliance classes
  (ids, settings, state, composition, safety notes)
- [`docs/catalog/variables-and-settings.md`](docs/catalog/variables-and-settings.md) —
  shared traits and per-class variables, settings, and commands (types, units,
  ranges, access modes)
- [`docs/catalog/ADDING_A_CLASS.md`](docs/catalog/ADDING_A_CLASS.md) — step-by-step
  guide for adding a new appliance class (catalog → schema tables → sim → WASM)
- [`docs/standard/overview.md`](docs/standard/overview.md) — catalog → schema →
  wire protocol, discovery, versioning, errors, extensions
- [`docs/standard/thermal-plant.md`](docs/standard/thermal-plant.md) — house
  thermal / hydraulic coupling (design sketch)
- [`docs/standard/procedures.md`](docs/standard/procedures.md) — procedures and
  recipes as ordered HomeCooked steps (design sketch)
- [`docs/standard/bridges.md`](docs/standard/bridges.md) — Zigbee / Matter /
  Modbus / BACnet bridges; HomeCooked as whitegoods semantics layer
- [`docs/standard/control-system.md`](docs/standard/control-system.md) —
  universal control computer (HAL, I/O map, interlocks, cycle runtime);
  path from catalog semantics to real HV hardware (design sketch)
- [`docs/standard/examples/washer-dryer-io.md`](docs/standard/examples/washer-dryer-io.md)
  — washer and dryer I/O inventory, sample maps, washer/dryer `cotton` cycle outlines

## Workspace

Cargo workspace. Schema and catalog versions are **0.1.0**. Protocol version
is **0.1.0** (peers are rejected only on protocol **major** mismatch).

| Crate / app | Path | Role |
|-------------|------|------|
| `homecooked-schema` | [`crates/homecooked-schema`](crates/homecooked-schema) | Catalog-backed serde types, capability model, static tables, write validation; **catalog JSON export** for tooling |
| `homecooked-protocol` | [`crates/homecooked-protocol`](crates/homecooked-protocol) | Envelope framing, request/response kinds, discovery, JSON, errors |
| `homecooked-core` | [`crates/homecooked-core`](crates/homecooked-core) | Device registry, capability-enforced read/write, request handling |
| `homecooked-sim` | [`crates/homecooked-sim`](crates/homecooked-sim) | In-memory devices for static Tier-A ∪ Tier-B class tables |
| `homecooked-wasm` | [`crates/homecooked-wasm`](crates/homecooked-wasm) | wasm-bindgen JSON API over the simulator |
| `homecooked-io-map` | [`crates/homecooked-io-map`](crates/homecooked-io-map) | Chassis I/O map serde+validate |
| `homecooked-interlock` | [`crates/homecooked-interlock`](crates/homecooked-interlock) | Declarative interlock rules |
| `homecooked-hal` | [`crates/homecooked-hal`](crates/homecooked-hal) | Firmware HAL surface sketch + host `MockHal` ([control-system.md](docs/standard/control-system.md) §4.3) |
| `homecooked-procedure` | [`crates/homecooked-procedure`](crates/homecooked-procedure) | Procedure / recipe documents, validation, sequential runner; thin `thermal_wait` / `thermal_offer` ([procedures.md](docs/standard/procedures.md)) |
| `homecooked-controller` | [`crates/homecooked-controller`](crates/homecooked-controller) | Host controller sim: IoMap + MockHal + interlocks + washer cotton / dryer cycle + lab TCP (interlock + cycle start/phase) ([control-system.md](docs/standard/control-system.md)) |
| `homecooked-thermal` | [`crates/homecooked-thermal`](crates/homecooked-thermal) | Thermal plant slice: reservoirs, heat ports, offer/accept, tick transfer ([thermal-plant.md](docs/standard/thermal-plant.md)) |
| `homecooked-bridge` | [`crates/homecooked-bridge`](crates/homecooked-bridge) | Fabric bridges: Modbus + Matter + Zigbee + BACnet mock adapters ([bridges.md](docs/standard/bridges.md)) |
| `homecooked-transport` | [`crates/homecooked-transport`](crates/homecooked-transport) | Lab TCP transport for protocol envelopes (length-prefixed JSON + optional PSK); sim or pluggable `RequestHandler` server + client ([overview.md](docs/standard/overview.md) §6) |
| `homecooked-hub` | [`crates/homecooked-hub`](crates/homecooked-hub) | Optional lab hub: multi-device sim registry behind one TCP listener (not required for devices) |
| `homecooked-conformance` | [`crates/homecooked-conformance`](crates/homecooked-conformance) | Light Stream 7 conformance smoke (catalog↔schema↔sim↔protocol↔TCP) |
| simulator-web | [`apps/simulator-web`](apps/simulator-web) | Static HTML/JS UI: full catalog picker (Tier-A ∪ Tier-B), procedure runner, thermal plant panel, device **thermal port** chips + read-only **Catalog heat ports** (`list_heat_port_specs` / `ClassTable.thermal_ports`) |

`list_all_class_ids` covers the full class index in
`docs/catalog/appliances.md`. Static capability tables (and therefore
simulated devices) cover all **56** catalog class ids: **25 Tier-A** plus
**31 Tier-B** (`TIER_A_CLASS_IDS` ∪ `TIER_B_CLASS_IDS` =
`STATIC_CLASS_IDS` = `ApplianceClassId::ALL` in
`crates/homecooked-schema/src/catalog/classes.rs`). See
[`docs/ROADMAP.md`](docs/ROADMAP.md) §4.



## Catalog JSON export (tooling)

Machine-readable dump of all **56** class ids and their typical capability
points (traits + required class points). This is a small auditable JSON
document for generators / validators — **not** a full OpenAPI server.

```bash
cargo run -p homecooked-schema --example export_catalog
cargo run -p homecooked-schema --example export_catalog -- /tmp/homecooked-catalog.json
cargo test -p homecooked-schema export_is_valid_json
```

Document shape: `format` = `homecooked.catalog_export`, `classes[].class_id`,
`classes[].group`, `classes[].typical` (same serde shape as the capability
model). See `homecooked_schema::catalog_export` /
`export_catalog_json`.

## Tests

```bash
cargo test
cargo test -p homecooked-conformance   # Stream 7 end-to-end smoke
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

CI on `main` (and PRs targeting `main`) runs rustfmt, clippy (`-D warnings`),
`cargo test`, and a wasm-pack build. Conformance smoke lives in
[`crates/homecooked-conformance`](crates/homecooked-conformance).

## Web simulator

Requires `wasm-pack` and the `wasm32-unknown-unknown` target
(`rustup target add wasm32-unknown-unknown`). `--out-dir` is relative to the
wasm crate, not the repo root:

```bash
wasm-pack build crates/homecooked-wasm --target web --out-dir ../../apps/simulator-web/pkg
cd apps/simulator-web
python3 -m http.server 8080
```

Open <http://127.0.0.1:8080>. Do not open `index.html` as a `file://` URL;
the browser will not load the ES module / WASM.

See [`apps/simulator-web/README.md`](apps/simulator-web/README.md) (grouped
Tier-A picker, rebuild/serve commands, manual smoke) and
[`docs/ROADMAP.md`](docs/ROADMAP.md) Stream 7.


## TCP lab transport

Cleartext TCP binding for protocol envelopes (no TLS / OAuth). Framing is
**length-prefixed JSON** (`u32` BE length + compact envelope). Optional lab
**PSK pairing** refuses anonymous clients when configured (dedicated auth
preamble; not a TLS substitute). See
[`crates/homecooked-transport`](crates/homecooked-transport).

```bash
cargo test -p homecooked-transport
cargo run -p homecooked-transport --example homecooked-tcp-demo
```


## Lab hub (optional)

Aggregate multiple sim devices on one TCP port for multi-appliance labs.
**Devices do not need a hub** — single-device transport is enough. See
[`crates/homecooked-hub`](crates/homecooked-hub).

```bash
cargo test -p homecooked-hub
cargo run -p homecooked-hub --example hub_demo
```

## Dual-path demo: thermal preheat → dishwasher

Procedures cannot call thermal APIs yet. The runnable bridge is:

1. Fridge condenser → DHW plant tick (`homecooked-thermal` /
   `create_thermal_demo` + `thermal_demo_transfer`) — DHW rises 35 → 36.2 °C.
2. Procedure `dishwasher_dhw_preheat` writes eco + `wash_temp_c` = 45 reflecting
   warm inlet / lower electrical boost.

In the web simulator (**Thermal plant → Orchestrations**), use
**Thermal → dishwasher preheat** (`run_thermal_then_dishwasher_preheat`, default
dt=3600) for both legs in one click. Rebuild WASM after Rust changes:

```bash
wasm-pack build crates/homecooked-wasm --target web --out-dir ../../apps/simulator-web/pkg
cd apps/simulator-web && python3 -m http.server 8080
```

```bash
cargo test -p homecooked-conformance   # includes thermal_then_dishwasher_preheat
cargo test -p homecooked-wasm run_thermal_then_dishwasher
```

See [`docs/standard/thermal-plant.md`](docs/standard/thermal-plant.md) §8.1 and
[`apps/simulator-web/README.md`](apps/simulator-web/README.md).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for branch / PR / CI norms.

To add a **new appliance class** (catalog docs → `ids.rs` → `ClassTable` →
sim defaults → tests → WASM list → optional procedure), follow
[`docs/catalog/ADDING_A_CLASS.md`](docs/catalog/ADDING_A_CLASS.md). Today
`STATIC_CLASS_IDS` = `ApplianceClassId::ALL` (Tier-A ∪ Tier-B, **56** ids).

## License

Apache-2.0
