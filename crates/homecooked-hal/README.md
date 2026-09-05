# homecooked-hal

Firmware **HAL surface sketch** for HomeCooked — not production firmware.

Defines the logical channel API controller firmware will implement
(`din` / `dout` / `ain` / `aout` / `relay` / `motor`), plus a host-side
`MockHal` for unit tests. Aligns with:

- [`docs/standard/control-system.md`](../../docs/standard/control-system.md) §4.3
- [`docs/ROADMAP.md`](../../docs/ROADMAP.md) Stream 4 (HAL sketch milestone)
- Channel prefixes used by [`homecooked-io-map`](../homecooked-io-map)

## Public API

| Item | Role |
|------|------|
| `ChannelKind` | `DigitalIn`, `DigitalOut`, `AnalogIn`, `AnalogOut`, `Relay`, `Motor` |
| `ChannelId` | Validated `kind.suffix` string (`din.door_closed`, …) |
| `HalValue` / `MotorCommand` | Bool or number samples / commands |
| `Hal` trait | `read_di` / `read_ai` / `write_do` / `write_aout` / `write_relay` / `write_motor` / `tick_ms` |
| `MockHal` | In-memory backend: inject sensors, record actuator commands |
| `bridge` | Thin string helpers; documents that **io_map sits above HAL** |

No real GPIO or `embedded-hal` dependency. Omit the `std` feature on a
firmware target and implement `Hal` against your boards.

## Layering

```text
HomeCooked device role / cycle runtime / interlocks
                    │
              I/O map (config)     ← homecooked-io-map
                    │
                 HAL               ← this crate
                    │
            boards or MockHal
```

Higher layers resolve catalog points through an `IoMap` binding, then call
the HAL with the binding’s `channel` id. This crate does not load YAML maps.

## Features

| Feature | Default | Notes |
|---------|---------|--------|
| `std` | yes | `MockHal`, host clock |
| `interlock` | yes | Optional `MockHal` gate via `homecooked-interlock` |

## Tests

```bash
cargo test -p homecooked-hal
cargo test --workspace
```

Washer smoke test uses sample channels from
[`docs/standard/examples/washer-dryer-io.md`](../../docs/standard/examples/washer-dryer-io.md):
inject `din.door_lock_fb` + `water_present`, command `aout.heater_enable`,
assert the mock recorded it; with interlocks enabled, deny when water is
absent.
