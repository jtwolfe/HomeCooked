# homecooked-conformance

Light **conformance smoke suite** (Stream 7): exercises catalog ↔ schema ↔
sim ↔ protocol ↔ TCP (plus controller / procedure / thermal / Modbus + Matter + Zigbee bridges)
end-to-end without a heavy test framework.

Failures are named by scenario so CI output stays actionable.

## Run

```bash
cargo test -p homecooked-conformance
```

Also covered by `cargo test --workspace` (CI).

## Scenarios

1. **tier_a_catalog_sim_describe** — every Tier-A class has
   `typical_capability`; sim can spawn; protocol describe returns the class id
2. **washer_cotton_controller** — `homecooked-controller` cotton path reaches
   `WasherState::Done`
3. **procedure_kettle_happy_path** — kettle heat procedure via
   `homecooked-procedure` against the sim
4. **thermal_fridge_dhw_demo** — fridge condenser → DHW preheat via
   `homecooked-thermal`
5. **modbus_water_heater_roundtrip** — Modbus map ↔ HomeCooked points via
   `homecooked-bridge`
6. **matter_kettle_roundtrip** — Matter mock fabric ↔ HomeCooked kettle points
7. **zigbee_kettle_roundtrip** — Zigbee mock network ↔ HomeCooked kettle points
   via `homecooked-bridge` (illustrative cluster IDs; no zigbee2mqtt)
8. **tcp_kettle_discover_describe_read_write** — TCP client against a sim
   kettle on an ephemeral port (`homecooked-transport`)
