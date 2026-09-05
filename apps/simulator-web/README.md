# HomeCooked web simulator

Minimal static UI that loads the `homecooked-wasm` package produced by
`wasm-pack`. No bundler or framework.

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

1. Pick a spawnable appliance class (static Tier-A catalog tables; see
   [`docs/ROADMAP.md`](../../docs/ROADMAP.md) §4).
2. Create a device.
3. Inspect identity, variables, and settings.
4. Write settings / fire commands (`start`, `power_on`, …).
5. Use **Tick** or **Auto tick** so simulated behavior (kettle heat, washer
   progress) advances.

Writes that fail capability checks (`out_of_range`, `not_writable`, …) show
an error banner and leave device state unchanged.
