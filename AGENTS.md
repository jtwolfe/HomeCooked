# HomeCooked — Agent Instructions

## Project

Universal standard communication interface for whitegoods / kitchen appliances:
discover devices, read telemetry/state, write settings/commands. Capability-based,
versioned, extensible.

## Stack

- Rust workspace (Cargo), segmented crates under `crates/` and `apps/`
- Schema → protocol → core → sim → wasm → web simulator
- Docs catalog is the source of truth; code must track `docs/catalog/` and `docs/standard/`

## Crates (do not collapse into a god-crate)

- `crates/homecooked-schema` — types from catalog (serde), capability model, versioning
- `crates/homecooked-protocol` — framing, request/response, discovery, errors
- `crates/homecooked-core` — device registry, validation against capabilities
- `crates/homecooked-sim` — in-memory simulated devices driven by catalog
- `crates/homecooked-wasm` — wasm-bindgen API for the web simulator
- `apps/simulator-web` — minimal web UI loading WASM

## Workflow

- Feature branches, small focused PRs to `main`
- Never force-push `main`; never `rm -rf`; never paste secrets
- Run `cargo test` and relevant wasm build before claiming done
- Prefer MIT or Apache-2.0 LICENSE already in repo

## Conventions

- Idiomatic Rust 2021+, `clippy` clean, `rustfmt`
- Serde for wire/schema types; explicit units in docs and type names where useful
- Capability checks reject out-of-range / unsupported writes
- Tests: unit + integration for schema validation, protocol roundtrips, capability enforcement, sim behavior

## Grok Build

Use `/design` then `/implement` for non-trivial work. Keep sessions continuous with `--resume`.
