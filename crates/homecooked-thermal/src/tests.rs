//! Unit tests for the thermal plant slice.

use super::*;

const DT_HOUR: f32 = 3_600.0;

fn approx(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "actual={actual} expected={expected}"
    );
}

fn offer_fridge_to_preheat(plant: &ThermalPlant, power: PowerBandW, priority: u8) -> TransferOffer {
    assert!(plant.offer(&demo_offer(power, priority)).is_ok());
    demo_offer(power, priority)
}

fn demo_offer(power: PowerBandW, priority: u8) -> TransferOffer {
    TransferOffer::new(
        PortRef::new("fridge-kitchen", "condenser").unwrap(),
        TransferTarget::port("water-heater-plant", "preheat").unwrap(),
        power,
        None,
        priority,
    )
}

#[test]
fn registry_add_lookup_list() {
    let plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    assert_eq!(plant.list_reservoirs().len(), 1);
    assert_eq!(plant.list_ports().len(), 2);
    assert_eq!(
        plant.get_reservoir("dhw-tank").unwrap().role,
        ReservoirRole::Dhw
    );
    assert_eq!(
        plant
            .get_port("fridge-kitchen", "condenser")
            .unwrap()
            .max_power_w,
        120
    );
    assert_eq!(plant.list_ports_for_device("water-heater-plant").len(), 1);
}

#[test]
fn registry_rejects_duplicates_and_unknown_attach() {
    let mut plant = ThermalPlant::new();
    plant
        .add_reservoir(
            Reservoir::new(
                "hot",
                ReservoirRole::Hot,
                Media::Water,
                None,
                TempBandC::new(20.0, 80.0).unwrap(),
                Some(1.0),
                None,
            )
            .unwrap(),
        )
        .unwrap();
    let err = plant
        .add_reservoir(
            Reservoir::new(
                "hot",
                ReservoirRole::Hot,
                Media::Water,
                None,
                TempBandC::new(20.0, 80.0).unwrap(),
                Some(1.0),
                None,
            )
            .unwrap(),
        )
        .unwrap_err();
    assert!(matches!(err, Error::DuplicateReservoir(_)));

    let err = plant
        .attach_port(
            HeatPort::new(
                "dev",
                "p",
                PortDirection::Source,
                10,
                TempBandC::new(0.0, 10.0).unwrap(),
                0,
                Media::Water,
                Some("missing".into()),
            )
            .unwrap(),
        )
        .unwrap_err();
    assert!(matches!(err, Error::UnknownReservoir(_)));
}

#[test]
fn happy_path_transfer_updates_reservoir_temp() {
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    let before = plant.get_reservoir("dhw-tank").unwrap().temp_c.unwrap();
    let offer = offer_fridge_to_preheat(&plant, PowerBandW::new(80, 120).unwrap(), 1);
    let result = plant.apply(offer, 120, DT_HOUR).unwrap();

    // 120 W × 3600 s = 0.12 kWh; ΔT = (0.12 / 4.0) × 40 °C = 1.2 °C
    approx(result.energy_kwh, 0.12);
    approx(result.delta_temp_c, 1.2);
    approx(
        plant.get_reservoir("dhw-tank").unwrap().temp_c.unwrap(),
        before + 1.2,
    );
}

#[test]
fn reject_temp_outside_usable_band() {
    // Tank already hotter than the fridge condenser's useful band.
    let mut plant = ThermalPlant::new();
    plant
        .add_reservoir(
            Reservoir::new(
                "dhw-tank",
                ReservoirRole::Dhw,
                Media::Water,
                Some(70.0),
                TempBandC::new(10.0, 80.0).unwrap(),
                Some(4.0),
                None,
            )
            .unwrap(),
        )
        .unwrap();
    plant
        .attach_port(
            HeatPort::new(
                "fridge-kitchen",
                "condenser",
                PortDirection::Source,
                120,
                TempBandC::new(35.0, 45.0).unwrap(),
                1,
                Media::Water,
                None,
            )
            .unwrap(),
        )
        .unwrap();
    plant
        .attach_port(
            HeatPort::new(
                "water-heater-plant",
                "preheat",
                PortDirection::Sink,
                2_000,
                TempBandC::new(20.0, 80.0).unwrap(),
                4,
                Media::Water,
                Some("dhw-tank".into()),
            )
            .unwrap(),
        )
        .unwrap();

    let offer = demo_offer(PowerBandW::new(80, 120).unwrap(), 1);
    let err = plant.offer(&offer).unwrap_err();
    assert!(
        matches!(err, Error::TempOutOfBand { temp_c, .. } if (temp_c - 70.0).abs() < 1e-4),
        "{err:?}"
    );
    let before = plant.get_reservoir("dhw-tank").unwrap().temp_c;
    assert!(plant.apply(offer, 120, DT_HOUR).is_err());
    assert_eq!(plant.get_reservoir("dhw-tank").unwrap().temp_c, before);
}

#[test]
fn reject_media_mismatch_and_wrong_direction() {
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    plant
        .attach_port(
            HeatPort::new(
                "hvac-1",
                "condenser",
                PortDirection::Source,
                500,
                TempBandC::new(30.0, 50.0).unwrap(),
                1,
                Media::Air,
                None,
            )
            .unwrap(),
        )
        .unwrap();
    let air_offer = TransferOffer::new(
        PortRef::new("hvac-1", "condenser").unwrap(),
        TransferTarget::port("water-heater-plant", "preheat").unwrap(),
        PowerBandW::new(100, 500).unwrap(),
        None,
        1,
    );
    assert!(matches!(
        plant.offer(&air_offer).unwrap_err(),
        Error::MediaMismatch { .. }
    ));

    let sink_as_source = TransferOffer::new(
        PortRef::new("water-heater-plant", "preheat").unwrap(),
        TransferTarget::reservoir("dhw-tank").unwrap(),
        PowerBandW::new(10, 20).unwrap(),
        None,
        1,
    );
    assert!(matches!(
        plant.offer(&sink_as_source).unwrap_err(),
        Error::WrongDirection { .. }
    ));
}

#[test]
fn reject_over_max_power() {
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    let offer = demo_offer(PowerBandW::new(80, 120).unwrap(), 1);
    let err = plant.accept(offer, 500).unwrap_err();
    assert!(matches!(
        err,
        Error::PowerExceedsMax {
            requested: 500,
            max: 120
        }
    ));
}

#[test]
fn partial_fill_ok() {
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    let offer = offer_fridge_to_preheat(&plant, PowerBandW::new(50, 120).unwrap(), 1);
    let result = plant.apply(offer, 80, DT_HOUR).unwrap();
    // 80 W × 3600 s = 0.08 kWh; ΔT = (0.08 / 4.0) × 40 = 0.8 °C
    approx(result.energy_kwh, 0.08);
    approx(result.power_w as f32, 80.0);
    approx(result.delta_temp_c, 0.8);
    approx(
        plant.get_reservoir("dhw-tank").unwrap().temp_c.unwrap(),
        35.8,
    );
}

#[test]
fn decline_leaves_state_unchanged() {
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    let before = plant.get_reservoir("dhw-tank").cloned().unwrap();
    let offer = demo_offer(PowerBandW::new(80, 120).unwrap(), 1);
    plant.offer(&offer).unwrap();
    let decline = plant.decline("local interlock: compressor already in defrost");
    assert!(decline.reason.contains("defrost"));
    let results = plant.step(DT_HOUR).unwrap();
    assert!(results.is_empty());
    assert_eq!(plant.get_reservoir("dhw-tank").unwrap(), &before);
}

#[test]
fn negotiate_decline_on_band_mismatch() {
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    let offer = TransferOffer::new(
        PortRef::new("fridge-kitchen", "condenser").unwrap(),
        TransferTarget::port("water-heater-plant", "preheat").unwrap(),
        PowerBandW::new(80, 120).unwrap(),
        None,
        1,
    );
    // Compatible — should accept.
    assert!(plant.negotiate(offer).is_accept());
    // Pending accept does not change temp until step; decline path:
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    let before = plant.get_reservoir("dhw-tank").unwrap().temp_c;
    let bad = TransferOffer::new(
        PortRef::new("missing", "x").unwrap(),
        TransferTarget::reservoir("dhw-tank").unwrap(),
        PowerBandW::new(10, 20).unwrap(),
        None,
        0,
    );
    assert!(plant.negotiate(bad).is_decline());
    assert_eq!(plant.get_reservoir("dhw-tank").unwrap().temp_c, before);
}

#[test]
fn negotiate_counters_when_min_above_available_max() {
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    // Condenser max is 120 W; offer min 150 → Counter (no silent partial below min).
    let high = TransferOffer::new(
        PortRef::new("fridge-kitchen", "condenser").unwrap(),
        TransferTarget::port("water-heater-plant", "preheat").unwrap(),
        PowerBandW::new(150, 200).unwrap(),
        None,
        1,
    );
    let reply = plant.negotiate(high);
    assert!(reply.is_counter(), "{reply:?}");
    match reply {
        TransferReply::Counter(c) => {
            assert_eq!(c.suggested_power_w.min, 120);
            assert_eq!(c.suggested_power_w.max, 120);
            assert!(c.reason.contains("below offer min"), "{}", c.reason);
        }
        other => panic!("expected counter, got {other:?}"),
    }
    // Plant unchanged until Accept.
    assert!(plant.step(DT_HOUR).unwrap().is_empty());
}

#[test]
fn negotiate_accepts_after_counter_suggested_band() {
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    let high = TransferOffer::new(
        PortRef::new("fridge-kitchen", "condenser").unwrap(),
        TransferTarget::port("water-heater-plant", "preheat").unwrap(),
        PowerBandW::new(150, 200).unwrap(),
        None,
        1,
    );
    let suggested = match plant.negotiate(high) {
        TransferReply::Counter(c) => c.suggested_power_w,
        other => panic!("expected counter, got {other:?}"),
    };
    let retry = TransferOffer::new(
        PortRef::new("fridge-kitchen", "condenser").unwrap(),
        TransferTarget::port("water-heater-plant", "preheat").unwrap(),
        suggested,
        None,
        1,
    );
    match plant.negotiate(retry) {
        TransferReply::Accept(a) => assert_eq!(a.accepted_power_w, 120),
        other => panic!("expected accept, got {other:?}"),
    }
}

#[test]
fn priority_prefers_higher_sink_when_headroom_limited() {
    let mut plant = ThermalPlant::new();
    plant
        .add_reservoir(
            Reservoir::new(
                "dhw-tank",
                ReservoirRole::Dhw,
                Media::Water,
                Some(35.0),
                TempBandC::new(20.0, 60.0).unwrap(),
                Some(4.0),
                Some(0.1), // 100 W instantaneous headroom
            )
            .unwrap(),
        )
        .unwrap();
    plant
        .attach_port(
            HeatPort::new(
                "fridge-kitchen",
                "condenser",
                PortDirection::Source,
                200,
                TempBandC::new(30.0, 55.0).unwrap(),
                1,
                Media::Water,
                None,
            )
            .unwrap(),
        )
        .unwrap();
    plant
        .attach_port(
            HeatPort::new(
                "hvac-1",
                "reject",
                PortDirection::Source,
                200,
                TempBandC::new(30.0, 55.0).unwrap(),
                1,
                Media::Water,
                None,
            )
            .unwrap(),
        )
        .unwrap();
    plant
        .attach_port(
            HeatPort::new(
                "water-heater-plant",
                "preheat",
                PortDirection::Sink,
                2_000,
                TempBandC::new(20.0, 60.0).unwrap(),
                8,
                Media::Water,
                Some("dhw-tank".into()),
            )
            .unwrap(),
        )
        .unwrap();

    let high = TransferOffer::new(
        PortRef::new("fridge-kitchen", "condenser").unwrap(),
        TransferTarget::port("water-heater-plant", "preheat").unwrap(),
        PowerBandW::new(50, 80).unwrap(),
        None,
        8,
    );
    let low = TransferOffer::new(
        PortRef::new("hvac-1", "reject").unwrap(),
        TransferTarget::port("water-heater-plant", "preheat").unwrap(),
        PowerBandW::new(50, 80).unwrap(),
        None,
        1,
    );
    plant.accept(high, 80).unwrap();
    plant.accept(low, 80).unwrap();
    let results = plant.step(DT_HOUR).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].from_port.device_id, "fridge-kitchen");
    assert_eq!(results[0].power_w, 80);
    assert_eq!(results[1].from_port.device_id, "hvac-1");
    assert_eq!(results[1].power_w, 20); // leftover headroom
                                        // Combined 100 W × 1 h = 0.10 kWh; ΔT = (0.10 / 4.0) × 40 = 1.0 °C
    approx(
        plant.get_reservoir("dhw-tank").unwrap().temp_c.unwrap(),
        36.0,
    );
}

/// Demo scenario: fridge condenser (source) → DHW / water_heater preheat sink.
#[test]
fn demo_fridge_condenser_to_dhw_preheat() {
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    let offer = demo_offer(PowerBandW::new(80, 120).unwrap(), 1);
    let reply = plant.negotiate(offer);
    match reply {
        TransferReply::Accept(a) => assert_eq!(a.accepted_power_w, 120),
        other => panic!("expected accept, got {other:?}"),
    }
    let results = plant.step(DT_HOUR).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].from_port.device_id, "fridge-kitchen");
    assert_eq!(results[0].heated_reservoir_id.as_deref(), Some("dhw-tank"));
    approx(results[0].delta_temp_c, 1.2);
    approx(
        plant.get_reservoir("dhw-tank").unwrap().temp_c.unwrap(),
        36.2,
    );
}

#[test]
fn serde_roundtrip_reservoir_and_offer() {
    let plant = ThermalPlant::fridge_condenser_dhw_demo().unwrap();
    let r = plant.get_reservoir("dhw-tank").unwrap();
    let json = serde_json::to_string(r).unwrap();
    let back: Reservoir = serde_json::from_str(&json).unwrap();
    assert_eq!(&back, r);

    let offer = demo_offer(PowerBandW::new(80, 120).unwrap(), 3);
    let json = serde_json::to_string(&offer).unwrap();
    let back: TransferOffer = serde_json::from_str(&json).unwrap();
    assert_eq!(back, offer);
    assert!(json.contains("\"kind\":\"port\""));
}
