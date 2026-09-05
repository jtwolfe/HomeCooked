# homecooked-thermal

First executable **thermal plant** slice: shared reservoirs, device heat
ports, a best-effort offer / accept / decline dialogue, and a coarse
simulator tick. Aligns with
[`docs/standard/thermal-plant.md`](../../docs/standard/thermal-plant.md).

This is **not** a catalog promotion, a sim class-table change, a bridge, or
CFD / plumbing physics. Types stay crate-local and experimental until a
later revision promotes stable ids. Coordination may fail open (decline /
timeout → appliances keep local thermal policy).

## Types

| Type | Role |
|------|------|
| `Reservoir` | Plant buffer: `id`, `role` (`hot` / `cold` / `dhw` / `other`), `media`, optional `temp_c`, `usable_band_c`, optional `capacity_kwh` / `headroom_kw` |
| `HeatPort` | Device attachment: `port_id` + owning `device_id`, `direction`, `max_power_w`, `usable_temp_c`, `priority`, `media`, optional `attached_reservoir_id` |
| `TransferOffer` | `{ from_port, to: port \| reservoir, power_w band, duration_s?, priority }` |
| `TransferAccept` / `TransferDecline` | Accept may **partial-fill**; decline leaves plant state unchanged |

`ThermalPlant` registers reservoirs, attaches ports, looks up / lists them,
validates offers, and applies accepted transfers on `step(dt_s)`.

## Tick energy and ΔT

When a source offer is accepted by a compatible sink, the tick transfers
`min(available, requested)` watts (also capped by remaining reservoir
`headroom_kw` when several accepts compete). Energy and temperature:

```text
E_kWh  = (P_W × dt_s) / 3_600_000
ΔT_C   = (E_kWh / capacity_kWh) × (T_max − T_min)
```

`capacity_kwh` is a coarse thermal-mass proxy: the energy that would
traverse the reservoir's `usable_band_c`. Missing capacity → energy is
accounted but temperature is left unchanged. Source ports attached to a
reservoir are cooled by `−ΔT`; sink-attached / target reservoirs are heated
by `+ΔT`. Temps are clamped to the usable band.

Transfers are **rejected** on media mismatch, non-overlapping usable
temperature bands, current reservoir temp outside a participant's usable
band, wrong port direction, or power above `max_power_w` / the offer max.
`Media::Unknown` is treated as compatible with any media.

Priority is a hint (`0` = scrap heat … higher = comfort-critical). On a
tick, queued accepts are sorted by priority descending and share limited
headroom greedily (later offers may partial-fill).

## Demo scenario

**Fridge condenser (source) → DHW / water_heater preheat sink.**

`ThermalPlant::fridge_condenser_dhw_demo()` builds:

- reservoir `dhw-tank` — role `dhw`, water, 35 °C, band 20–60 °C, 4 kWh, 2 kW headroom
- device `fridge-kitchen` port `condenser` — source, water recovery loop, 35–55 °C, 120 W, priority 1
- device `water-heater-plant` port `preheat` — sink attached to `dhw-tank`, 20–60 °C, 2 kW, priority 4

Device ids are instances. Catalog class ids `fridge` and `water_heater`
already exist; this crate does not add class or point ids.

A 120 W accept over a 3600 s tick delivers 0.12 kWh and raises the tank
**1.2 °C** (35 → 36.2).

```bash
cargo test -p homecooked-thermal fridge_condenser
cargo test -p homecooked-thermal
```

The filter matches the unit demo `demo_fridge_condenser_to_dhw_preheat` and
the integration test in `tests/fridge_condenser_dhw.rs`.

## Still sketch / vendor / experimental

- No wire encoding; no `homecooked-schema` catalog types or sim port
  read/write on Tier-A classes (`water_heater`, `hvac`, `fridge`).
- No isolation / potable-boundary enforcement beyond a media tag.
- No CFD, pipe network, glycol mix, or refrigerant charge model.
- Local interlocks still win; this layer never commands a compressor past
  device limits.
