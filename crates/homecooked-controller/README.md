# homecooked-controller

Host-side **controller simulator** — stand-in for the universal control
computer in [`docs/standard/control-system.md`](../../docs/standard/control-system.md).

Combines:

| Piece | Crate / role |
|-------|----------------|
| Chassis I/O map | [`homecooked-io-map`](../homecooked-io-map) (`WASHER_FRAGMENT_YAML` / `DRYER_FRAGMENT_YAML`) |
| Logical HAL | [`homecooked-hal`](../homecooked-hal) `MockHal` (no GPIO) |
| Interlocks | [`homecooked-interlock`](../homecooked-interlock) `washer_rules` / `dryer_rules` |
| Cycle runtime | Washer **cotton** (`Controller`) and dryer Idle→Dry/Heat→Cool→Done (`DryerController`) |

Washer `cotton` outline:
[`docs/standard/examples/washer-dryer-io.md`](../../docs/standard/examples/washer-dryer-io.md) §6.
Dryer I/O: same doc §3 (sensors/actuators) + §5 dryer map notes.

## API

```rust
use homecooked_controller::{Controller, CottonOptions, DryOptions, DryerController};

let mut washer = Controller::washer_cotton_demo()?;
washer.start_cotton(CottonOptions::default())?;
washer.run_until_done(200)?;
assert_eq!(washer.cycle_state().as_str(), "complete");

let mut dryer = DryerController::dryer_cotton_demo()?;
dryer.start_dry(DryOptions::default())?;
dryer.run_until_done(200)?;
assert_eq!(dryer.cycle_state().as_str(), "complete");
```

States only command actuators through `MockHal` writes, so heater / spin /
dryer-heat interlocks still apply. A small plant model raises water level when
the inlet is open (washer), drains on pump, drops humidity while drying, mirrors
door lock feedback, and tracks drum rpm — enough for sensor-driven transitions
in tests.

**Not in this crate:** real GPIO, full HomeCooked protocol device-role
registration (drive controllers directly from tests). Lab TCP for protocol
envelopes is in `homecooked-transport` (auth/TLS out of scope); wiring these
controllers onto that transport is optional follow-up.

## Tests

```bash
cargo test -p homecooked-controller
cargo test -p homecooked-controller cotton_cycle_reaches_done -- --nocapture
cargo test -p homecooked-controller dryer_cycle_reaches_done -- --nocapture
cargo test -p homecooked-controller dryer_heat_blocked_if_door_unlocked -- --nocapture
```

## Roadmap

Stream 4 in [`docs/ROADMAP.md`](../../docs/ROADMAP.md): HAL sketch (done) →
**controller-sim (washer + dryer)** → TCP lab smoke (`homecooked-transport`).
