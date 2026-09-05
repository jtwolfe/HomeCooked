//! Integration demo: **fridge condenser (source) → DHW / water_heater preheat**.
//!
//! The fridge rejects compressor heat on a hydronic recovery port; the water
//! heater exposes a preheat sink attached to a shared DHW reservoir. A
//! best-effort offer is accepted at the source's 120 W cap and applied over
//! a one-hour tick. See `crates/homecooked-thermal/README.md`.

use homecooked_thermal::{
    energy_kwh, PortRef, PowerBandW, ThermalPlant, TransferOffer, TransferReply, TransferTarget,
};

#[test]
fn fridge_condenser_to_dhw_preheat_scenario() {
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo().expect("demo plant");

    assert!(plant.get_port("fridge-kitchen", "condenser").is_some());
    assert_eq!(
        plant
            .get_port("water-heater-plant", "preheat")
            .unwrap()
            .attached_reservoir_id
            .as_deref(),
        Some("dhw-tank")
    );
    let start = plant.get_reservoir("dhw-tank").unwrap().temp_c.unwrap();
    assert!((start - 35.0).abs() < 1e-4);

    let offer = TransferOffer::new(
        PortRef::new("fridge-kitchen", "condenser").unwrap(),
        TransferTarget::port("water-heater-plant", "preheat").unwrap(),
        PowerBandW::new(80, 120).unwrap(),
        None,
        1,
    );
    match plant.negotiate(offer) {
        TransferReply::Accept(a) => assert_eq!(a.accepted_power_w, 120),
        other => panic!("expected accept, got {other:?}"),
    }

    let results = plant.step(3_600.0).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].power_w, 120);
    assert!((results[0].energy_kwh - energy_kwh(120, 3_600.0)).abs() < 1e-6);
    // ΔT = (0.12 kWh / 4.0 kWh) × 40 °C = 1.2 °C
    assert!((results[0].delta_temp_c - 1.2).abs() < 1e-4);
    let end = plant.get_reservoir("dhw-tank").unwrap().temp_c.unwrap();
    assert!((end - 36.2).abs() < 1e-4, "dhw temp {end}");
}
