# homecooked-procedure

First executable slice of the HomeCooked **procedure / recipe** layer:
serde documents, structural (+ optional capability) validation, and a
sequential runner.

Aligns with [`docs/standard/procedures.md`](../../docs/standard/procedures.md)
(§2–5). Parallel steps, expression-language guards, dedicated thermal WASM/UI,
bridges, TCP transport, and HAL remain out of scope or deferred for this crate.

## Document

A procedure is an ordered list of steps. Each step is one of `read`,
`write`, `command`, `wait`, `assert` (the sketch’s `guard` op maps to
`assert`), or thin `thermal_wait` (alias `wait_reservoir`) on a plant
reservoir temperature. Simple comparison guards (`eq` / `ne` / `gt` /
`gte` / `lt` / `lte`) may be AND-combined. Multi-device roles may appear in
the document; the runner binds them through a role → device-id map.

Worked example (microwave-only Domino's reheat sketch):
[`examples/reheat_dominos_microwave.json`](examples/reheat_dominos_microwave.json).

Oven bake happy-path (`program` + setpoint + heat wait):
[`examples/oven_bake_180.json`](examples/oven_bake_180.json).

Coffee brew happy-path (power on + `espresso` + boiler heat wait):
[`examples/coffee_brew_espresso.json`](examples/coffee_brew_espresso.json).

Air fryer cook happy-path (`program` fries + setpoint 200 °C + heat wait):
[`examples/air_fryer_cook_200.json`](examples/air_fryer_cook_200.json).

Dishwasher companion to the fridge→DHW thermal demo (procedure leg only —
thermal transfer still out-of-band):
[`examples/dishwasher_dhw_preheat.json`](examples/dishwasher_dhw_preheat.json).

Thin procedure⇄thermal wait on DHW reservoir temp:
[`examples/wait_dhw_reservoir.json`](examples/wait_dhw_reservoir.json)
(`SimulatorBackend::with_plant`). Continuous re-queue while waiting:
[`examples/wait_dhw_with_requeue.json`](examples/wait_dhw_with_requeue.json)
(`requeue_offer`). Soft-decline offer:
[`examples/offer_fridge_dhw_soft.json`](examples/offer_fridge_dhw_soft.json).
See [`docs/standard/thermal-plant.md`](../../docs/standard/thermal-plant.md) §8.

## Runner

`DeviceBackend` is `read` / `write` / `tick` plus optional
`thermal_read_reservoir_temp` / `thermal_tick`. `Simulator` implements device
I/O so `Wait` advances **simulated** time (`tick`, default 1000 ms) instead of
sleeping on the wall clock. `SimulatorBackend` may hold a `ThermalPlant` for
`thermal_wait`. Kettle/oven heat and cycle progress therefore move under the
runner.

```bash
cargo test -p homecooked-procedure
cargo test -p homecooked-procedure -- kettle_happy
```

The kettle happy-path test writes `trait.temperature.setpoint_c` = 80,
commands `trait.cycle.start`, waits (ticking the sim) until
`trait.temperature.current_c` ≥ 75, then asserts.
