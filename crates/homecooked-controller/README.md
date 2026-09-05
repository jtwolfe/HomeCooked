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

**Lab TCP (thin):** [`ControllerEndpoint`](src/endpoint.rs) /
[`DryerControllerEndpoint`](src/dryer_endpoint.rs) map small washer/dryer
capabilities onto MockHal so clients over `homecooked-transport` get
`safety_interlock` on denied actuator writes (washer: water+lock; dryer:
lock+blower). Washer and dryer TCP also start cotton/dry via
`trait.cycle.start` and expose readable `trait.cycle.cycle_state` /
`cycle_phase`, plus lab-only `class.washer.sim_tick` /
`class.dryer.sim_tick`. CottonOptions / DryOptions / cancel / pause /
typical_capability remain follow-up. No GPIO, TLS, or OAuth.

## Tests

```bash
cargo test -p homecooked-controller
cargo test -p homecooked-controller cotton_cycle_reaches_done -- --nocapture
cargo test -p homecooked-controller dryer_cycle_reaches_done -- --nocapture
cargo test -p homecooked-controller dryer_heat_blocked_if_door_unlocked -- --nocapture
cargo test -p homecooked-controller --test tcp_interlock
```

## Roadmap

Stream 4 in [`docs/ROADMAP.md`](../../docs/ROADMAP.md): HAL sketch (done) →
**controller-sim (washer + dryer)** → TCP lab smoke (`homecooked-transport`) →
**controller-sim-over-TCP** lab smoke (washer + dryer interlock deny; washer/dryer cycle start/phase).
