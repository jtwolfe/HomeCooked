# Contributing to HomeCooked

Thanks for helping. The **docs catalog is the source of truth**; code under
`crates/` must track `docs/catalog/` and `docs/standard/`.

## Workflow

1. Branch from **`main`** with a short prefix (`feat/…`, `fix/…`, `docs/…`).
   Do not commit directly to `main`.
2. Open a **small, focused PR** against `main`. One concern per PR when you can.
3. Wait for CI (rustfmt, clippy `-D warnings`, `cargo test --workspace`,
   wasm-pack). Docs-only PRs still run the full job; they should stay green.
4. **Never force-push `main`.** Do not rewrite published history on the default
   branch.
5. Do not paste secrets into the repo or PR descriptions.
6. Catalog / standard docs land **before or with** the code that implements
   them. Do not invent core class / trait / point ids in code that are missing
   from `docs/catalog/`.

## Local checks

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
# When touching wasm / simulator-web:
wasm-pack build crates/homecooked-wasm --target web --out-dir ../../apps/simulator-web/pkg
```

See the root [`README.md`](README.md) for crate map, catalog JSON export, TCP /
hub demos, and the web simulator.

## Adding a new appliance class

Full step-by-step (catalog → `ids.rs` → `ClassTable` → sim → tests → WASM →
optional procedure):

**[`docs/catalog/ADDING_A_CLASS.md`](docs/catalog/ADDING_A_CLASS.md)**

Current layout: **25** Tier-A + **31** Tier-B thin tables;
`STATIC_CLASS_IDS` = `ApplianceClassId::ALL` (**56** classes). See
[`docs/ROADMAP.md`](docs/ROADMAP.md) §4.

## License

Contributions are under the repository [LICENSE](LICENSE) (Apache-2.0).
