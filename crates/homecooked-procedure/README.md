# homecooked-procedure

First executable slice of the HomeCooked **procedure / recipe** layer:
serde documents, structural (+ optional capability) validation, and a
sequential runner.

Aligns with [`docs/standard/procedures.md`](../../docs/standard/procedures.md)
(§2–5). Parallel steps, expression-language guards, WASM/UI, thermal-plant,
bridges, TCP transport, and HAL are out of scope for this crate.

## Document

A procedure is an ordered list of steps. Each step is one of `read`,
`write`, `command`, `wait`, or `assert` (the sketch’s `guard` op maps to
`assert`). Simple comparison guards (`eq` / `ne` / `gt` / `gte` / `lt` /
`lte`) may be AND-combined. Multi-device roles may appear in the document;
the runner binds them through a role → device-id map.

Worked example (microwave-only Domino's reheat sketch):
[`examples/reheat_dominos_microwave.json`](examples/reheat_dominos_microwave.json).

Oven bake happy-path (`program` + setpoint + heat wait):
[`examples/oven_bake_180.json`](examples/oven_bake_180.json).

Dishwasher companion to the fridge→DHW thermal demo (procedure leg only —
thermal must run out-of-band first):
[`examples/dishwasher_dhw_preheat.json`](examples/dishwasher_dhw_preheat.json).
See [`docs/standard/thermal-plant.md`](../../docs/standard/thermal-plant.md) §8.1.

## Runner

`DeviceBackend` is `read` / `write` / `tick`. `Simulator` implements it so
`Wait` advances **simulated** time (`tick`, default 1000 ms) instead of
sleeping on the wall clock. Kettle/oven heat and cycle progress therefore move
under the runner.

```bash
cargo test -p homecooked-procedure
cargo test -p homecooked-procedure -- kettle_happy
```

The kettle happy-path test writes `trait.temperature.setpoint_c` = 80,
commands `trait.cycle.start`, waits (ticking the sim) until
`trait.temperature.current_c` ≥ 75, then asserts.
