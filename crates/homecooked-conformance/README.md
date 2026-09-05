# homecooked-conformance

Light **conformance smoke suite** (Stream 7): exercises catalog ↔ schema ↔
sim ↔ protocol ↔ TCP ↔ optional hub (plus controller / procedure / thermal / Modbus + Matter + Zigbee + BACnet bridges)
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
4b. **thermal_then_dishwasher_preheat** — dual-path: thermal fridge→DHW
   (assert DHW rose), then `dishwasher_dhw_preheat` procedure writes eco +
   wash_temp reflecting preheat available
5. **modbus_water_heater_roundtrip** — Modbus map ↔ HomeCooked points via
   `homecooked-bridge`
6. **matter_kettle_roundtrip** — Matter mock fabric ↔ HomeCooked kettle points
7. **zigbee_kettle_roundtrip** — Zigbee mock network ↔ HomeCooked kettle points
   via `homecooked-bridge` (illustrative cluster IDs; no zigbee2mqtt)
8. **bacnet_kettle_roundtrip** — BACnet mock device ↔ HomeCooked kettle points
   via `homecooked-bridge` (illustrative object types; no BACnet stack)
9. **tcp_kettle_discover_describe_read_write** — TCP client against a sim
   kettle on an ephemeral port (`homecooked-transport`)
10. **tcp_psk_good_secret_describe_ping** — TCP PSK pairing with a good shared
   secret: describe + ping against a sim kettle (`homecooked-transport`)
11. **controller_tcp_washer_interlock** — TCP client against washer
   `ControllerEndpoint`: heater allow when water+lock, `safety_interlock` deny
   when dry (`homecooked-controller` + `homecooked-transport`)
12. **hub_lab_set_discover_describe** — optional `LabHub` lab set over TCP:
   Discover ≥3 devices, describe kettle (`homecooked-hub`)
