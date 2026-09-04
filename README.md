# HomeCooked

Universal standard communication interface for whitegoods and kitchen appliances:
discover devices, read telemetry/state, and write settings/commands.

This repository is under active construction. The **docs catalog is the source
of truth**; code (when it exists) must track `docs/catalog/` and
`docs/standard/`.

## Documentation

- [`docs/catalog/appliances.md`](docs/catalog/appliances.md) — appliance classes
  (ids, settings, state, composition, safety notes)
- [`docs/catalog/variables-and-settings.md`](docs/catalog/variables-and-settings.md) —
  shared traits and per-class variables, settings, and commands (types, units,
  ranges, access modes)
- [`docs/standard/overview.md`](docs/standard/overview.md) — catalog → schema →
  wire protocol, discovery, versioning, errors, extensions

This revision is **docs-only**. Schema, protocol, core, simulator, and WASM
crates are not in tree yet.

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
