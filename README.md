# HomeCooked

Universal standard communication interface for whitegoods and kitchen appliances:
discover devices, read telemetry/state, and write settings/commands.

This repository is under active construction. The **docs catalog is the source
of truth**; code must track `docs/catalog/` and `docs/standard/`.

## Documentation

- [`docs/ROADMAP.md`](docs/ROADMAP.md) — ~30% → ~75% completeness plan, Tier-A
  classes, workstreams and definitions of done
- [`docs/catalog/appliances.md`](docs/catalog/appliances.md) — appliance classes
  (ids, settings, state, composition, safety notes)
- [`docs/catalog/variables-and-settings.md`](docs/catalog/variables-and-settings.md) —
  shared traits and per-class variables, settings, and commands (types, units,
  ranges, access modes)
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
  — washer and dryer I/O inventory, sample I/O map, `cotton` cycle outline

## Workspace

Cargo workspace. Schema and catalog versions are **0.1.0**. Protocol version
is **0.1.0** (peers are rejected only on protocol **major** mismatch).

| Crate / app | Path | Role |
|-------------|------|------|
| `homecooked-schema` | [`crates/homecooked-schema`](crates/homecooked-schema) | Catalog-backed serde types, capability model, static tables, write validation |
| `homecooked-protocol` | [`crates/homecooked-protocol`](crates/homecooked-protocol) | Envelope framing, request/response kinds, discovery, JSON, errors |
| `homecooked-core` | [`crates/homecooked-core`](crates/homecooked-core) | Device registry, capability-enforced read/write, request handling |
| `homecooked-sim` | [`crates/homecooked-sim`](crates/homecooked-sim) | In-memory devices for static Tier-A class tables |
| `homecooked-wasm` | [`crates/homecooked-wasm`](crates/homecooked-wasm) | wasm-bindgen JSON API over the simulator |
| `homecooked-io-map` | [`crates/homecooked-io-map`](crates/homecooked-io-map) | Chassis I/O map serde+validate |
| `homecooked-interlock` | [`crates/homecooked-interlock`](crates/homecooked-interlock) | Declarative interlock rules |
| `homecooked-hal` | [`crates/homecooked-hal`](crates/homecooked-hal) | Firmware HAL surface sketch + host `MockHal` ([control-system.md](docs/standard/control-system.md) §4.3) |
| simulator-web | [`apps/simulator-web`](apps/simulator-web) | Static HTML/JS UI that loads the wasm-pack output |

`list_all_class_ids` covers the full class index in
`docs/catalog/appliances.md`. Static capability tables (and therefore
simulated devices) cover all **25 Tier-A** class ids listed in
[`docs/ROADMAP.md`](docs/ROADMAP.md) §4 and `TIER_A_CLASS_IDS`
(`crates/homecooked-schema/src/catalog/classes.rs`).

## Tests

```bash
cargo test
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

CI on `main` (and PRs targeting `main`) runs rustfmt, clippy (`-D warnings`),
`cargo test`, and a wasm-pack build.

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

See [`apps/simulator-web/README.md`](apps/simulator-web/README.md).

## Contributing

1. Branch from **`main`** using a short prefix (`feat/…`, `fix/…`, `docs/…`).
   Do not commit directly to `main`.
2. Open a **small, focused PR** against `main`. One concern per PR when you can.
3. CI must pass. For Rust changes run `cargo test` (and clippy/fmt). For wasm
   or simulator-web changes also run the `wasm-pack build` command above.
4. **Never force-push `main`.** Do not rewrite published history on default
   branches.
5. Do not paste secrets into the repo or PR descriptions.
6. Catalog and standard docs land before or with the code that implements them.
   Do not invent core class / trait / point ids in code that are missing from
   `docs/catalog/`.

## License

Apache-2.0
