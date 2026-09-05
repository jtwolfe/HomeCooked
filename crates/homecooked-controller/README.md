# homecooked-controller

Host-side **controller simulator** — stand-in for the universal control
computer in [`docs/standard/control-system.md`](../../docs/standard/control-system.md).

Combines:

| Piece | Crate / role |
|-------|----------------|
| Chassis I/O map | [`homecooked-io-map`](../homecooked-io-map) (`WASHER_FRAGMENT_YAML`) |
| Logical HAL | [`homecooked-hal`](../homecooked-hal) `MockHal` (no GPIO) |
| Interlocks | [`homecooked-interlock`](../homecooked-interlock) `washer_rules` |
| Cycle runtime | Washer **cotton** state machine (this crate) |

Washer `cotton` outline:
[`docs/standard/examples/washer-dryer-io.md`](../../docs/standard/examples/washer-dryer-io.md) §6.

## API

```rust
use homecooked_controller::{Controller, CottonOptions};

let mut ctrl = Controller::washer_cotton_demo()?;
ctrl.start_cotton(CottonOptions::default())?;
ctrl.run_until_done(200)?;
assert_eq!(ctrl.cycle_state().as_str(), "complete");
```

States only command actuators through `MockHal` writes, so heater / spin
interlocks still apply. A small plant model raises water level when the
inlet is open, drains on pump, mirrors door lock feedback, and tracks drum
rpm — enough for sensor-driven transitions in tests.

**Not in this crate:** TCP transport, real GPIO, dryer cycle (follow-up),
full HomeCooked protocol device-role registration (drive `Controller`
directly from tests for now).

## Tests

```bash
cargo test -p homecooked-controller
cargo test -p homecooked-controller cotton_cycle_reaches_done -- --nocapture
```

## Roadmap

Stream 4 in [`docs/ROADMAP.md`](../../docs/ROADMAP.md): HAL sketch (done) →
**controller-sim (this crate)** → TCP transport (later).
