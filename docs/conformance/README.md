# Conformance scenario catalog

Checked-in browse catalog for Stream 7’s `homecooked-conformance` suite and the
thin simulator-web **Conformance** panel.

| File | Role |
|------|------|
| [`scenarios.json`](./scenarios.json) | Names + tags + `native_only` for every `all_scenarios()` entry |

- **Runnable in wasm** (`native_only: false`): schema/sim/procedure/thermal lab
  checks via `list_conformance_scenarios` / `run_conformance_lab_check` in
  `homecooked-wasm` (isolated in-process; not a full browser CI runner).
- **Native only** (`native_only: true`): TCP / Modbus TCP / hub / controller /
  bridge — run with `cargo test -p homecooked-conformance`.

Keep names in sync with `homecooked_conformance::all_scenarios()` (enforced by
`crates/homecooked-conformance/tests/catalog_sync.rs`).
