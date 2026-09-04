# HomeCooked

Universal standard communication interface for whitegoods and kitchen appliances:
discover devices, read telemetry/state, and write settings/commands.

This repository is under active construction. The **docs catalog is the source
of truth**; code must track `docs/catalog/` and `docs/standard/`.

## Documentation

- [`docs/catalog/appliances.md`](docs/catalog/appliances.md) — appliance classes
  (ids, settings, state, composition, safety notes)
- [`docs/catalog/variables-and-settings.md`](docs/catalog/variables-and-settings.md) —
  shared traits and per-class variables, settings, and commands (types, units,
  ranges, access modes)
- [`docs/standard/overview.md`](docs/standard/overview.md) — catalog → schema →
  wire protocol, discovery, versioning, errors, extensions

## Workspace

Cargo workspace. The first code crate is
[`crates/homecooked-schema`](crates/homecooked-schema): catalog-backed serde
types, capability model, static tables for nine appliance classes, and write
validation. Schema and catalog versions are **0.1.0**.

Protocol, core, simulator, and WASM crates are **not** in this revision.

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

`list_all_class_ids` covers the full class index in
`docs/catalog/appliances.md`. Static capability tables are provided for
`washer`, `dryer`, `fridge`, `dishwasher`, `microwave`, `oven`,
`induction_hob`, `kettle`, and `air_fryer`.

## Contributing

- Work on **feature branches**; open **small, focused PRs** against `main`.
- CI must pass on the PR (when workflows exist). Do not claim done without the
  relevant tests / wasm build for code changes.
- **Never force-push `main`.** Do not rewrite published history on default
  branches.
- Do not paste secrets into the repo or PR descriptions.
- Catalog and standard docs land before or with the code that implements them.
  Do not invent core class / trait / point ids in code that are missing from
  `docs/catalog/`.

## License

Apache-2.0
