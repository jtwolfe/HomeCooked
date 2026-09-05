//! In-memory simulated HomeCooked devices driven by the catalog.

mod behavior;
mod defaults;
mod simulator;

pub use behavior::{
    DEFAULT_MICROWAVE_COOK_S, DRYER_CYCLE_S, KETTLE_HEAT_RATE_C_PER_S, WASHER_CYCLE_S,
};
pub use defaults::{seed_identity, seed_state, sim_capability};
pub use simulator::Simulator;

#[cfg(test)]
mod tests {
    use homecooked_schema::{
        ApplianceClassId, ErrorCode, Value, STATIC_CLASS_IDS, TIER_A_CLASS_IDS, TIER_B_CLASS_IDS,
    };

    use super::*;

    fn f32_val(v: &Value) -> f32 {
        match v {
            Value::F32(x) => *x,
            Value::Percent(x) => *x,
            other => panic!("expected f32, got {other:?}"),
        }
    }

    #[test]
    fn spawn_all_static_classes() {
        let mut sim = Simulator::new();
        let ids = sim.spawn_static_kitchen().unwrap();
        assert_eq!(ids.len(), STATIC_CLASS_IDS.len());
        assert_eq!(sim.list().len(), STATIC_CLASS_IDS.len());
        assert_eq!(STATIC_CLASS_IDS.len(), 56);
        assert_eq!(
            STATIC_CLASS_IDS.len(),
            TIER_A_CLASS_IDS.len() + TIER_B_CLASS_IDS.len()
        );
        for class in TIER_B_CLASS_IDS {
            assert!(STATIC_CLASS_IDS.contains(class));
            let mut one = Simulator::new();
            one.spawn(*class)
                .unwrap_or_else(|e| panic!("spawn {class}: {e}"));
        }
    }

    #[test]
    fn kettle_heats_toward_setpoint_when_on() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::Kettle).unwrap();
        let start = f32_val(&sim.read_value(&id, "trait.temperature.current_c").unwrap());
        assert!((start - 20.0).abs() < f32::EPSILON);
        let setpoint = f32_val(&sim.read_value(&id, "trait.temperature.setpoint_c").unwrap());
        assert!((setpoint - 100.0).abs() < f32::EPSILON);

        sim.write(&id, "trait.power.power_on", Value::Void).unwrap();
        assert_eq!(
            sim.read_value(&id, "trait.power.power_state").unwrap(),
            Value::Enum("on".into())
        );

        sim.tick(&id, 5_000).unwrap();
        let mid = f32_val(&sim.read_value(&id, "trait.temperature.current_c").unwrap());
        let expected = 20.0 + KETTLE_HEAT_RATE_C_PER_S * 5.0;
        assert!(
            (mid - expected).abs() < 0.01,
            "mid={mid} expected={expected}"
        );

        sim.tick(&id, 20_000).unwrap();
        let end = f32_val(&sim.read_value(&id, "trait.temperature.current_c").unwrap());
        assert!((end - 100.0).abs() < 0.01, "end={end}");
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("complete".into())
        );
        assert_eq!(
            sim.read_value(&id, "trait.power.power_state").unwrap(),
            Value::Enum("standby".into())
        );
    }

    #[test]
    fn washer_cycle_progress_stub() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::Washer).unwrap();
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("idle".into())
        );

        sim.write(&id, "trait.cycle.start", Value::Void).unwrap();
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("running".into())
        );

        sim.tick(&id, 30_000).unwrap();
        let progress = f32_val(&sim.read_value(&id, "trait.cycle.progress_percent").unwrap());
        assert!((progress - 50.0).abs() < 1.0, "progress={progress}");
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("running".into())
        );

        sim.tick(&id, 30_000).unwrap();
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("complete".into())
        );
        let done = f32_val(&sim.read_value(&id, "trait.cycle.progress_percent").unwrap());
        assert!((done - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn dryer_cycle_progress_stub() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::Dryer).unwrap();
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("idle".into())
        );

        sim.write(&id, "trait.cycle.start", Value::Void).unwrap();
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("running".into())
        );

        sim.tick(&id, 15_000).unwrap();
        let progress = f32_val(&sim.read_value(&id, "trait.cycle.progress_percent").unwrap());
        assert!((progress - 50.0).abs() < 1.0, "progress={progress}");
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("running".into())
        );

        sim.tick(&id, 15_000).unwrap();
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("complete".into())
        );
        let done = f32_val(&sim.read_value(&id, "trait.cycle.progress_percent").unwrap());
        assert!((done - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn capability_rejection_through_sim() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::Kettle).unwrap();
        let err = sim
            .write(&id, "trait.temperature.setpoint_c", Value::F32(20.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfRange);

        let washer = sim.spawn(ApplianceClassId::Washer).unwrap();
        let err = sim
            .write(&washer, "class.washer.spin_rpm", Value::U16(2000))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfRange);
        assert_eq!(
            sim.read_value(&washer, "class.washer.spin_rpm").unwrap(),
            Value::U16(800)
        );

        let err = sim
            .write(
                &washer,
                "trait.cycle.cycle_state",
                Value::Enum("running".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn spawn_pr1_tier_a_classes_identity_power_and_writes() {
        let mut sim = Simulator::new();
        let batch = [
            ApplianceClassId::WasherDryer,
            ApplianceClassId::Freezer,
            ApplianceClassId::FridgeFreezer,
            ApplianceClassId::WineCooler,
            ApplianceClassId::IceMaker,
            ApplianceClassId::WaterHeater,
            ApplianceClassId::Hvac,
            ApplianceClassId::Dehumidifier,
            ApplianceClassId::RangeHood,
        ];
        for class in batch {
            let id = sim.spawn(class).unwrap();
            assert_eq!(
                sim.read_value(&id, "trait.identity.class_id").unwrap(),
                Value::Enum(class.as_str().into())
            );
            let power = sim.read_value(&id, "trait.power.power_state").unwrap();
            assert!(matches!(power, Value::Enum(_)));
        }

        let wd = sim.spawn(ApplianceClassId::WasherDryer).unwrap();
        sim.write(
            &wd,
            "class.washer_dryer.combo_mode",
            Value::Enum("wash_only".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&wd, "class.washer_dryer.combo_mode")
                .unwrap(),
            Value::Enum("wash_only".into())
        );

        let freezer = sim.spawn(ApplianceClassId::Freezer).unwrap();
        sim.write(
            &freezer,
            "trait.temperature.setpoint_c#freezer",
            Value::F32(-18.0),
        )
        .unwrap();
        let err = sim
            .write(
                &freezer,
                "trait.temperature.setpoint_c#freezer",
                Value::F32(4.0),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::OutOfRange);

        let ff = sim.spawn(ApplianceClassId::FridgeFreezer).unwrap();
        assert!(
            (f32_val(
                &sim.read_value(&ff, "trait.temperature.current_c#fridge")
                    .unwrap()
            ) - 4.0)
                .abs()
                < f32::EPSILON
        );
        assert!(
            (f32_val(
                &sim.read_value(&ff, "trait.temperature.current_c#freezer")
                    .unwrap()
            ) + 18.0)
                .abs()
                < f32::EPSILON
        );

        let wine = sim.spawn(ApplianceClassId::WineCooler).unwrap();
        sim.write(
            &wine,
            "trait.temperature.setpoint_c#upper",
            Value::F32(12.0),
        )
        .unwrap();
        assert!(
            (f32_val(
                &sim.read_value(&wine, "trait.temperature.current_c#upper")
                    .unwrap()
            ) - 16.0)
                .abs()
                < f32::EPSILON
        );
        assert!(
            (f32_val(
                &sim.read_value(&wine, "trait.temperature.current_c#lower")
                    .unwrap()
            ) - 10.0)
                .abs()
                < f32::EPSILON
        );

        let heater = sim.spawn(ApplianceClassId::WaterHeater).unwrap();
        sim.write(&heater, "trait.temperature.setpoint_c", Value::F32(60.0))
            .unwrap();

        let hvac = sim.spawn(ApplianceClassId::Hvac).unwrap();
        sim.write(&hvac, "class.hvac.hvac_mode", Value::Enum("heat".into()))
            .unwrap();
        assert!(
            (f32_val(&sim.read_value(&hvac, "class.hvac.space_c").unwrap()) - 21.0).abs()
                < f32::EPSILON
        );

        let hood = sim.spawn(ApplianceClassId::RangeHood).unwrap();
        sim.write(&hood, "trait.fan.fan_state", Value::Enum("on".into()))
            .unwrap();
    }

    #[test]
    fn wine_cooler_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let wine = sim.spawn(ApplianceClassId::WineCooler).unwrap();

        assert_eq!(
            sim.read_value(&wine, "class.wine_cooler.bottle_count")
                .unwrap(),
            Value::U16(24)
        );
        assert_eq!(
            sim.read_value(&wine, "class.wine_cooler.vibration_alert")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&wine, "class.wine_cooler.compressor_on")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&wine, "class.wine_cooler.high_temp_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&wine, "class.wine_cooler.low_temp_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&wine, "trait.humidity.current_rh").unwrap(),
            Value::Percent(60.0)
        );
        assert_eq!(
            sim.read_value(&wine, "trait.humidity.setpoint_rh").unwrap(),
            Value::Percent(60.0)
        );

        sim.write(
            &wine,
            "class.wine_cooler.vibration_reduce",
            Value::Bool(true),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&wine, "class.wine_cooler.vibration_reduce")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&wine, "class.wine_cooler.uv_protect", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&wine, "class.wine_cooler.uv_protect")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&wine, "class.wine_cooler.sabbath_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&wine, "class.wine_cooler.sabbath_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&wine, "trait.humidity.setpoint_rh", Value::Percent(70.0))
            .unwrap();
        assert_eq!(
            sim.read_value(&wine, "trait.humidity.setpoint_rh").unwrap(),
            Value::Percent(70.0)
        );
        sim.write(&wine, "trait.temperature.setpoint_c#lower", Value::F32(8.0))
            .unwrap();
        assert!(
            (f32_val(
                &sim.read_value(&wine, "trait.temperature.setpoint_c#lower")
                    .unwrap()
            ) - 8.0)
                .abs()
                < f32::EPSILON
        );
        let err = sim
            .write(&wine, "class.wine_cooler.bottle_count", Value::U16(40))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn ice_maker_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let ice = sim.spawn(ApplianceClassId::IceMaker).unwrap();

        assert_eq!(
            sim.read_value(&ice, "class.ice_maker.water_temp_c")
                .unwrap(),
            Value::F32(12.0)
        );
        assert_eq!(
            sim.read_value(&ice, "class.ice_maker.water_low").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ice, "class.ice_maker.clean_cycle_needed")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ice, "class.ice_maker.harvest_fail")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ice, "class.ice_maker.scale_alert").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ice, "trait.ice.bin_percent").unwrap(),
            Value::Percent(45.0)
        );
        assert_eq!(
            sim.read_value(&ice, "trait.ice.bin_full").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ice, "trait.filter.life_percent").unwrap(),
            Value::Percent(80.0)
        );

        sim.write(&ice, "class.ice_maker.scoop_light", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&ice, "class.ice_maker.scoop_light").unwrap(),
            Value::Bool(true)
        );
        sim.write(&ice, "class.ice_maker.max_ice_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&ice, "class.ice_maker.max_ice_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(
            &ice,
            "class.ice_maker.delayed_start_s",
            Value::DurationS(1800),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&ice, "class.ice_maker.delayed_start_s")
                .unwrap(),
            Value::DurationS(1800)
        );
        let err = sim
            .write(&ice, "class.ice_maker.water_low", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&ice, "trait.ice.bin_percent", Value::Percent(90.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn sous_vide_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let sv = sim.spawn(ApplianceClassId::SousVide).unwrap();

        assert_eq!(
            sim.read_value(&sv, "class.sous_vide.low_water").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&sv, "class.sous_vide.water_level_ok")
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            sim.read_value(&sv, "class.sous_vide.lid_closed").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            sim.read_value(&sv, "class.sous_vide.circulating").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&sv, "class.sous_vide.target_done").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&sv, "class.sous_vide.overtemp_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&sv, "class.sous_vide.timer_remaining_s")
                .unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&sv, "class.sous_vide.cook_s").unwrap(),
            Value::DurationS(600)
        );
        assert_eq!(
            sim.read_value(&sv, "class.sous_vide.delayed_start_s")
                .unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&sv, "class.sous_vide.alarm_offset_c")
                .unwrap(),
            Value::F32(0.0)
        );
        assert_eq!(
            sim.read_value(&sv, "trait.cycle.remaining_s").unwrap(),
            Value::DurationS(0)
        );

        sim.write(&sv, "class.sous_vide.cook_s", Value::DurationS(3600))
            .unwrap();
        assert_eq!(
            sim.read_value(&sv, "class.sous_vide.cook_s").unwrap(),
            Value::DurationS(3600)
        );
        sim.write(
            &sv,
            "class.sous_vide.delayed_start_s",
            Value::DurationS(900),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&sv, "class.sous_vide.delayed_start_s")
                .unwrap(),
            Value::DurationS(900)
        );
        sim.write(&sv, "class.sous_vide.alarm_offset_c", Value::F32(0.5))
            .unwrap();
        assert!(
            (f32_val(
                &sim.read_value(&sv, "class.sous_vide.alarm_offset_c")
                    .unwrap()
            ) - 0.5)
                .abs()
                < f32::EPSILON
        );
        let err = sim
            .write(&sv, "class.sous_vide.water_level_ok", Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&sv, "class.sous_vide.overtemp_alarm", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&sv, "class.sous_vide.target_done", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&sv, "trait.cycle.remaining_s", Value::DurationS(120))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn multi_cooker_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let mc = sim.spawn(ApplianceClassId::MultiCooker).unwrap();

        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.lid_locked")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.safe_to_open")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.pot_detect")
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.burn_detected")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.overpressure_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.lid_mismatch")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.keep_warm").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.cook_s").unwrap(),
            Value::DurationS(600)
        );
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.delayed_start_s")
                .unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.keep_warm_s")
                .unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.pressure_band")
                .unwrap(),
            Value::Enum("low".into())
        );
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.saute_level")
                .unwrap(),
            Value::Enum("low".into())
        );
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.float_valve")
                .unwrap(),
            Value::Enum("down".into())
        );
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.remote_vent_enabled")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&mc, "trait.cycle.remaining_s").unwrap(),
            Value::DurationS(0)
        );

        sim.write(&mc, "class.multi_cooker.cook_s", Value::DurationS(2400))
            .unwrap();
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.cook_s").unwrap(),
            Value::DurationS(2400)
        );
        sim.write(
            &mc,
            "class.multi_cooker.delayed_start_s",
            Value::DurationS(900),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.delayed_start_s")
                .unwrap(),
            Value::DurationS(900)
        );
        sim.write(&mc, "class.multi_cooker.keep_warm", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.keep_warm").unwrap(),
            Value::Bool(true)
        );
        sim.write(
            &mc,
            "class.multi_cooker.keep_warm_s",
            Value::DurationS(1800),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.keep_warm_s")
                .unwrap(),
            Value::DurationS(1800)
        );
        sim.write(
            &mc,
            "class.multi_cooker.saute_level",
            Value::Enum("high".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.saute_level")
                .unwrap(),
            Value::Enum("high".into())
        );
        sim.write(
            &mc,
            "class.multi_cooker.pressure_band",
            Value::Enum("high".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.pressure_band")
                .unwrap(),
            Value::Enum("high".into())
        );
        sim.write(
            &mc,
            "class.multi_cooker.remote_vent_enabled",
            Value::Bool(true),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&mc, "class.multi_cooker.remote_vent_enabled")
                .unwrap(),
            Value::Bool(true)
        );
        let err = sim
            .write(&mc, "class.multi_cooker.pot_detect", Value::Bool(false))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &mc,
                "class.multi_cooker.overpressure_alarm",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&mc, "class.multi_cooker.lid_mismatch", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&mc, "class.multi_cooker.burn_detected", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&mc, "trait.cycle.remaining_s", Value::DurationS(120))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn toaster_oven_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let to = sim.spawn(ApplianceClassId::ToasterOven).unwrap();

        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.toast_shade")
                .unwrap(),
            Value::U8(4)
        );
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.crumb_tray")
                .unwrap(),
            Value::Enum("ok".into())
        );
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.door_open").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.timer_remaining_s")
                .unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.delayed_start_s")
                .unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.rack_position")
                .unwrap(),
            Value::Enum("middle".into())
        );
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.bagel").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.preheating")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.slices").unwrap(),
            Value::U8(2)
        );
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.toast_done")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.convection_fan")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.cook_s").unwrap(),
            Value::DurationS(600)
        );
        assert_eq!(
            sim.read_value(&to, "trait.cycle.remaining_s").unwrap(),
            Value::DurationS(0)
        );

        sim.write(&to, "class.toaster_oven.toast_shade", Value::U8(6))
            .unwrap();
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.toast_shade")
                .unwrap(),
            Value::U8(6)
        );
        sim.write(
            &to,
            "class.toaster_oven.delayed_start_s",
            Value::DurationS(120),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.delayed_start_s")
                .unwrap(),
            Value::DurationS(120)
        );
        sim.write(
            &to,
            "class.toaster_oven.rack_position",
            Value::Enum("upper".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.rack_position")
                .unwrap(),
            Value::Enum("upper".into())
        );
        sim.write(&to, "class.toaster_oven.bagel", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.bagel").unwrap(),
            Value::Bool(true)
        );
        sim.write(&to, "class.toaster_oven.slices", Value::U8(4))
            .unwrap();
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.slices").unwrap(),
            Value::U8(4)
        );
        sim.write(&to, "class.toaster_oven.convection_fan", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.convection_fan")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&to, "class.toaster_oven.cook_s", Value::DurationS(1200))
            .unwrap();
        assert_eq!(
            sim.read_value(&to, "class.toaster_oven.cook_s").unwrap(),
            Value::DurationS(1200)
        );
        let err = sim
            .write(&to, "class.toaster_oven.door_open", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &to,
                "class.toaster_oven.timer_remaining_s",
                Value::DurationS(30),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&to, "class.toaster_oven.preheating", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&to, "class.toaster_oven.toast_done", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &to,
                "class.toaster_oven.crumb_tray",
                Value::Enum("ok".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&to, "trait.cycle.remaining_s", Value::DurationS(120))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn dehumidifier_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let dh = sim.spawn(ApplianceClassId::Dehumidifier).unwrap();

        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.tank_full").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.pump_mode").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.defrost").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.compressor_on")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.high_rh_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.low_rh_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.continuous_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.quiet_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.bucket_removed")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.filter_dirty")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.delayed_start_s")
                .unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&dh, "trait.humidity.current_rh").unwrap(),
            Value::Percent(55.0)
        );
        assert_eq!(
            sim.read_value(&dh, "trait.humidity.setpoint_rh").unwrap(),
            Value::Percent(45.0)
        );
        assert_eq!(
            sim.read_value(&dh, "trait.fan.fan_speed").unwrap(),
            Value::U8(2)
        );

        sim.write(&dh, "class.dehumidifier.pump_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.pump_mode").unwrap(),
            Value::Bool(true)
        );
        sim.write(&dh, "class.dehumidifier.continuous_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.continuous_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&dh, "class.dehumidifier.quiet_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.quiet_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(
            &dh,
            "class.dehumidifier.delayed_start_s",
            Value::DurationS(1800),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&dh, "class.dehumidifier.delayed_start_s")
                .unwrap(),
            Value::DurationS(1800)
        );
        sim.write(&dh, "trait.humidity.setpoint_rh", Value::Percent(40.0))
            .unwrap();
        assert_eq!(
            sim.read_value(&dh, "trait.humidity.setpoint_rh").unwrap(),
            Value::Percent(40.0)
        );
        sim.write(&dh, "trait.fan.fan_speed", Value::U8(4)).unwrap();
        assert_eq!(
            sim.read_value(&dh, "trait.fan.fan_speed").unwrap(),
            Value::U8(4)
        );

        let err = sim
            .write(&dh, "class.dehumidifier.tank_full", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&dh, "class.dehumidifier.defrost", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&dh, "class.dehumidifier.compressor_on", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&dh, "class.dehumidifier.high_rh_alarm", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&dh, "class.dehumidifier.low_rh_alarm", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&dh, "class.dehumidifier.bucket_removed", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&dh, "class.dehumidifier.filter_dirty", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn range_hood_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let hood = sim.spawn(ApplianceClassId::RangeHood).unwrap();

        assert_eq!(
            sim.read_value(&hood, "class.range_hood.auto_mode").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.delay_off_s")
                .unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.voc_index").unwrap(),
            Value::U16(40)
        );
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.grease_filter")
                .unwrap(),
            Value::Enum("ok".into())
        );
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.charcoal_filter")
                .unwrap(),
            Value::Enum("ok".into())
        );
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.filter_dirty")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.boost").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.boost_remaining_s")
                .unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.light_level")
                .unwrap(),
            Value::U8(2)
        );
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.grease_sensor")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.hob_linked")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.overtemp").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.charcoal_filter_life_percent")
                .unwrap(),
            Value::Percent(70.0)
        );
        assert_eq!(
            sim.read_value(&hood, "trait.fan.fan_speed").unwrap(),
            Value::U8(2)
        );
        assert_eq!(
            sim.read_value(&hood, "trait.lighting.light_percent")
                .unwrap(),
            Value::Percent(80.0)
        );
        assert_eq!(
            sim.read_value(&hood, "trait.filter.life_percent").unwrap(),
            Value::Percent(75.0)
        );

        sim.write(&hood, "class.range_hood.auto_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.auto_mode").unwrap(),
            Value::Bool(true)
        );
        sim.write(&hood, "class.range_hood.boost", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.boost").unwrap(),
            Value::Bool(true)
        );
        sim.write(&hood, "class.range_hood.hob_linked", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.hob_linked")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&hood, "class.range_hood.delay_off_s", Value::DurationS(300))
            .unwrap();
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.delay_off_s")
                .unwrap(),
            Value::DurationS(300)
        );
        sim.write(&hood, "class.range_hood.light_level", Value::U8(4))
            .unwrap();
        assert_eq!(
            sim.read_value(&hood, "class.range_hood.light_level")
                .unwrap(),
            Value::U8(4)
        );
        sim.write(&hood, "trait.fan.fan_speed", Value::U8(3))
            .unwrap();
        assert_eq!(
            sim.read_value(&hood, "trait.fan.fan_speed").unwrap(),
            Value::U8(3)
        );
        sim.write(&hood, "trait.lighting.light_percent", Value::Percent(50.0))
            .unwrap();
        assert_eq!(
            sim.read_value(&hood, "trait.lighting.light_percent")
                .unwrap(),
            Value::Percent(50.0)
        );

        let err = sim
            .write(&hood, "class.range_hood.voc_index", Value::U16(100))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &hood,
                "class.range_hood.grease_filter",
                Value::Enum("clogged".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &hood,
                "class.range_hood.charcoal_filter",
                Value::Enum("replace".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&hood, "class.range_hood.filter_dirty", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &hood,
                "class.range_hood.boost_remaining_s",
                Value::DurationS(60),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&hood, "class.range_hood.grease_sensor", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&hood, "class.range_hood.overtemp", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &hood,
                "class.range_hood.charcoal_filter_life_percent",
                Value::Percent(50.0),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn steam_oven_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let steam = sim.spawn(ApplianceClassId::SteamOven).unwrap();

        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.steam_mode")
                .unwrap(),
            Value::Enum("steam".into())
        );
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.water_tank")
                .unwrap(),
            Value::Enum("ok".into())
        );
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.humidity_set_percent")
                .unwrap(),
            Value::Percent(60.0)
        );
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.water_tank_level")
                .unwrap(),
            Value::Percent(85.0)
        );
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.descaling_needed")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.steam_generator_on")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.cavity_humidity")
                .unwrap(),
            Value::Percent(45.0)
        );
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.door_locked")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.drain_full")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.generator_fault")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.delayed_start_s")
                .unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.steam_percent")
                .unwrap(),
            Value::Percent(40.0)
        );
        assert_eq!(
            sim.read_value(&steam, "trait.cycle.remaining_s").unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&steam, "trait.water.hardness_ppm").unwrap(),
            Value::U16(120)
        );

        sim.write(
            &steam,
            "class.steam_oven.steam_mode",
            Value::Enum("combi".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.steam_mode")
                .unwrap(),
            Value::Enum("combi".into())
        );
        sim.write(
            &steam,
            "class.steam_oven.humidity_set_percent",
            Value::Percent(80.0),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.humidity_set_percent")
                .unwrap(),
            Value::Percent(80.0)
        );
        sim.write(
            &steam,
            "class.steam_oven.delayed_start_s",
            Value::DurationS(900),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.delayed_start_s")
                .unwrap(),
            Value::DurationS(900)
        );
        sim.write(&steam, "class.steam_oven.convection_fan", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.convection_fan")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(
            &steam,
            "class.steam_oven.steam_percent",
            Value::Percent(55.0),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.steam_percent")
                .unwrap(),
            Value::Percent(55.0)
        );
        sim.write(&steam, "class.steam_oven.cook_s", Value::DurationS(2400))
            .unwrap();
        assert_eq!(
            sim.read_value(&steam, "class.steam_oven.cook_s").unwrap(),
            Value::DurationS(2400)
        );
        sim.write(&steam, "trait.water.hardness_ppm", Value::U16(180))
            .unwrap();
        assert_eq!(
            sim.read_value(&steam, "trait.water.hardness_ppm").unwrap(),
            Value::U16(180)
        );

        let err = sim
            .write(
                &steam,
                "class.steam_oven.water_tank_level",
                Value::Percent(50.0),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &steam,
                "class.steam_oven.descaling_needed",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &steam,
                "class.steam_oven.steam_generator_on",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &steam,
                "class.steam_oven.cavity_humidity",
                Value::Percent(30.0),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&steam, "class.steam_oven.door_locked", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&steam, "class.steam_oven.drain_full", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &steam,
                "class.steam_oven.generator_fault",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &steam,
                "class.steam_oven.water_tank",
                Value::Enum("low".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn cooktop_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let cooktop = sim.spawn(ApplianceClassId::Cooktop).unwrap();

        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.level#hob_1")
                .unwrap(),
            Value::U8(0)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.residual_heat#hob_1")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.boost#hob_1")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.timer_s#hob_1")
                .unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.bridge#hob_1")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.flame_out").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.ignition_fail")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.power_limit_w")
                .unwrap(),
            Value::U32(7200)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.keep_warm#hob_1")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.hotspot_alert#hob_1")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.timer_active#hob_1")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.paused").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.surface_c#hob_1")
                .unwrap(),
            Value::F32(20.0)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.element_fault#hob_1")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.pan_detect#hob_1")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.flame_on#hob_1")
                .unwrap(),
            Value::Bool(false)
        );

        sim.write(&cooktop, "class.cooktop.boost#hob_1", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.boost#hob_1")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(
            &cooktop,
            "class.cooktop.timer_s#hob_2",
            Value::DurationS(900),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.timer_s#hob_2")
                .unwrap(),
            Value::DurationS(900)
        );
        sim.write(&cooktop, "class.cooktop.bridge#hob_1", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.bridge#hob_1")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&cooktop, "class.cooktop.power_limit_w", Value::U32(4800))
            .unwrap();
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.power_limit_w")
                .unwrap(),
            Value::U32(4800)
        );
        sim.write(&cooktop, "class.cooktop.keep_warm#hob_3", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.keep_warm#hob_3")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&cooktop, "class.cooktop.level#hob_1", Value::U8(5))
            .unwrap();
        assert_eq!(
            sim.read_value(&cooktop, "class.cooktop.level#hob_1")
                .unwrap(),
            Value::U8(5)
        );

        let err = sim
            .write(
                &cooktop,
                "class.cooktop.hotspot_alert#hob_1",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &cooktop,
                "class.cooktop.timer_active#hob_1",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&cooktop, "class.cooktop.paused", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&cooktop, "class.cooktop.surface_c#hob_1", Value::F32(180.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &cooktop,
                "class.cooktop.element_fault#hob_1",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &cooktop,
                "class.cooktop.pan_detect#hob_1",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&cooktop, "class.cooktop.flame_on#hob_1", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&cooktop, "class.cooktop.flame_out", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&cooktop, "class.cooktop.ignition_fail", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &cooktop,
                "class.cooktop.residual_heat#hob_1",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn humidifier_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let hum = sim.spawn(ApplianceClassId::Humidifier).unwrap();

        assert_eq!(
            sim.read_value(&hum, "class.humidifier.output_level")
                .unwrap(),
            Value::U8(3)
        );
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.mist_type").unwrap(),
            Value::Enum("cool".into())
        );
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.water_empty")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.wick_state").unwrap(),
            Value::Enum("ok".into())
        );
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.warm_mist").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.auto_humidity")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.mineral_filter")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.uv_clean").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.scale_alert")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.tank_removed")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.misting").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.night_mode").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&hum, "trait.humidity.setpoint_rh").unwrap(),
            Value::Percent(45.0)
        );
        assert_eq!(
            sim.read_value(&hum, "trait.humidity.current_rh").unwrap(),
            Value::Percent(40.0)
        );

        sim.write(&hum, "class.humidifier.output_level", Value::U8(7))
            .unwrap();
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.output_level")
                .unwrap(),
            Value::U8(7)
        );
        sim.write(&hum, "class.humidifier.warm_mist", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.warm_mist").unwrap(),
            Value::Bool(true)
        );
        sim.write(&hum, "class.humidifier.auto_humidity", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.auto_humidity")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&hum, "class.humidifier.uv_clean", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.uv_clean").unwrap(),
            Value::Bool(true)
        );
        sim.write(&hum, "class.humidifier.night_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&hum, "class.humidifier.night_mode").unwrap(),
            Value::Bool(true)
        );
        sim.write(&hum, "trait.humidity.setpoint_rh", Value::Percent(55.0))
            .unwrap();
        assert_eq!(
            sim.read_value(&hum, "trait.humidity.setpoint_rh").unwrap(),
            Value::Percent(55.0)
        );

        let err = sim
            .write(
                &hum,
                "class.humidifier.mist_type",
                Value::Enum("warm".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&hum, "class.humidifier.water_empty", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &hum,
                "class.humidifier.wick_state",
                Value::Enum("replace".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&hum, "class.humidifier.mineral_filter", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&hum, "class.humidifier.scale_alert", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&hum, "class.humidifier.tank_removed", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&hum, "class.humidifier.misting", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn freezer_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let fz = sim.spawn(ApplianceClassId::Freezer).unwrap();

        assert_eq!(
            sim.read_value(&fz, "class.freezer.vacation_mode").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&fz, "class.freezer.sabbath_mode").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&fz, "class.freezer.eco_mode").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&fz, "class.freezer.defrost_active").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&fz, "class.freezer.compressor_on").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&fz, "class.freezer.high_temp_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&fz, "class.freezer.fast_freeze").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&fz, "class.freezer.door_ajar").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&fz, "class.freezer.ice_buildup").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&fz, "class.freezer.low_temp_alarm").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&fz, "class.freezer.anti_sweat").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&fz, "class.freezer.fast_freeze_remaining_s")
                .unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&fz, "class.freezer.frost_clean_needed")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&fz, "trait.temperature.setpoint_c#freezer")
                .unwrap(),
            Value::F32(-18.0)
        );

        sim.write(&fz, "class.freezer.fast_freeze", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&fz, "class.freezer.fast_freeze").unwrap(),
            Value::Bool(true)
        );
        sim.write(&fz, "class.freezer.anti_sweat", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&fz, "class.freezer.anti_sweat").unwrap(),
            Value::Bool(true)
        );
        sim.write(&fz, "class.freezer.vacation_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&fz, "class.freezer.vacation_mode").unwrap(),
            Value::Bool(true)
        );
        sim.write(&fz, "class.freezer.sabbath_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&fz, "class.freezer.sabbath_mode").unwrap(),
            Value::Bool(true)
        );
        sim.write(&fz, "class.freezer.eco_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&fz, "class.freezer.eco_mode").unwrap(),
            Value::Bool(true)
        );

        let err = sim
            .write(&fz, "class.freezer.door_ajar", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&fz, "class.freezer.ice_buildup", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&fz, "class.freezer.low_temp_alarm", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&fz, "class.freezer.frost_clean_needed", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &fz,
                "class.freezer.fast_freeze_remaining_s",
                Value::DurationS(120),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&fz, "class.freezer.defrost_active", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&fz, "class.freezer.compressor_on", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&fz, "class.freezer.high_temp_alarm", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn fridge_freezer_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let ff = sim.spawn(ApplianceClassId::FridgeFreezer).unwrap();

        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.vacation_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.sabbath_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.eco_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.defrost_active")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.compressor_on")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.high_temp_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.door_ajar_fridge")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.door_ajar_freezer")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.fast_freeze")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.ice_buildup")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.high_temp_alarm_fridge")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.high_temp_alarm_freezer")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.convertible_zone_mode")
                .unwrap(),
            Value::Enum("fridge".into())
        );
        assert_eq!(
            sim.read_value(&ff, "trait.temperature.setpoint_c#fridge")
                .unwrap(),
            Value::F32(4.0)
        );
        assert_eq!(
            sim.read_value(&ff, "trait.temperature.setpoint_c#freezer")
                .unwrap(),
            Value::F32(-18.0)
        );

        sim.write(&ff, "class.fridge_freezer.fast_freeze", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.fast_freeze")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&ff, "class.fridge_freezer.vacation_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.vacation_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&ff, "class.fridge_freezer.sabbath_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.sabbath_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&ff, "class.fridge_freezer.eco_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.eco_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(
            &ff,
            "class.fridge_freezer.convertible_zone_mode",
            Value::Enum("freezer".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&ff, "class.fridge_freezer.convertible_zone_mode")
                .unwrap(),
            Value::Enum("freezer".into())
        );

        let err = sim
            .write(
                &ff,
                "class.fridge_freezer.door_ajar_fridge",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &ff,
                "class.fridge_freezer.door_ajar_freezer",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&ff, "class.fridge_freezer.ice_buildup", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &ff,
                "class.fridge_freezer.high_temp_alarm_fridge",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &ff,
                "class.fridge_freezer.high_temp_alarm_freezer",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &ff,
                "class.fridge_freezer.defrost_active",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&ff, "class.fridge_freezer.compressor_on", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &ff,
                "class.fridge_freezer.high_temp_alarm",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn beverage_cooler_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let bev = sim.spawn(ApplianceClassId::BeverageCooler).unwrap();

        assert_eq!(
            sim.read_value(&bev, "class.beverage_cooler.can_capacity")
                .unwrap(),
            Value::U16(120)
        );
        assert_eq!(
            sim.read_value(&bev, "class.beverage_cooler.sabbath_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&bev, "class.beverage_cooler.eco_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&bev, "class.beverage_cooler.compressor_on")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&bev, "class.beverage_cooler.high_temp_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&bev, "class.beverage_cooler.low_temp_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&bev, "class.beverage_cooler.door_ajar")
                .unwrap(),
            Value::Bool(false)
        );

        sim.write(
            &bev,
            "class.beverage_cooler.sabbath_mode",
            Value::Bool(true),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&bev, "class.beverage_cooler.sabbath_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&bev, "class.beverage_cooler.eco_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&bev, "class.beverage_cooler.eco_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&bev, "trait.temperature.setpoint_c", Value::F32(4.0))
            .unwrap();
        assert!(
            (f32_val(
                &sim.read_value(&bev, "trait.temperature.setpoint_c")
                    .unwrap()
            ) - 4.0)
                .abs()
                < f32::EPSILON
        );

        let err = sim
            .write(&bev, "class.beverage_cooler.can_capacity", Value::U16(200))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&bev, "class.beverage_cooler.door_ajar", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &bev,
                "class.beverage_cooler.compressor_on",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &bev,
                "class.beverage_cooler.high_temp_alarm",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &bev,
                "class.beverage_cooler.low_temp_alarm",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn kegerator_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let keg = sim.spawn(ApplianceClassId::Kegerator).unwrap();

        assert_eq!(
            sim.read_value(&keg, "class.kegerator.co2_kpa").unwrap(),
            Value::F32(110.0)
        );
        assert_eq!(
            sim.read_value(&keg, "class.kegerator.keg_percent").unwrap(),
            Value::Percent(75.0)
        );
        assert_eq!(
            sim.read_value(&keg, "class.kegerator.keg_empty").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&keg, "class.kegerator.sabbath_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&keg, "class.kegerator.eco_mode").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&keg, "class.kegerator.compressor_on")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&keg, "class.kegerator.high_temp_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&keg, "class.kegerator.low_temp_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&keg, "class.kegerator.door_ajar").unwrap(),
            Value::Bool(false)
        );

        sim.write(&keg, "class.kegerator.sabbath_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&keg, "class.kegerator.sabbath_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&keg, "class.kegerator.eco_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&keg, "class.kegerator.eco_mode").unwrap(),
            Value::Bool(true)
        );
        sim.write(&keg, "trait.temperature.setpoint_c", Value::F32(4.0))
            .unwrap();
        assert!(
            (f32_val(
                &sim.read_value(&keg, "trait.temperature.setpoint_c")
                    .unwrap()
            ) - 4.0)
                .abs()
                < f32::EPSILON
        );

        let err = sim
            .write(&keg, "class.kegerator.co2_kpa", Value::F32(200.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&keg, "class.kegerator.keg_percent", Value::Percent(10.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&keg, "class.kegerator.keg_empty", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&keg, "class.kegerator.door_ajar", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&keg, "class.kegerator.compressor_on", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&keg, "class.kegerator.high_temp_alarm", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&keg, "class.kegerator.low_temp_alarm", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn warming_drawer_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let wd = sim.spawn(ApplianceClassId::WarmingDrawer).unwrap();

        assert_eq!(
            sim.read_value(&wd, "class.warming_drawer.level").unwrap(),
            Value::Enum("medium".into())
        );
        assert_eq!(
            sim.read_value(&wd, "class.warming_drawer.moist").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&wd, "class.warming_drawer.sabbath_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&wd, "class.warming_drawer.eco_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&wd, "class.warming_drawer.heater_on")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&wd, "class.warming_drawer.high_temp_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&wd, "class.warming_drawer.door_ajar")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&wd, "class.warming_drawer.timer_s").unwrap(),
            Value::DurationS(0)
        );

        sim.write(
            &wd,
            "class.warming_drawer.level",
            Value::Enum("high".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&wd, "class.warming_drawer.level").unwrap(),
            Value::Enum("high".into())
        );
        sim.write(&wd, "class.warming_drawer.moist", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&wd, "class.warming_drawer.moist").unwrap(),
            Value::Bool(true)
        );
        sim.write(&wd, "class.warming_drawer.sabbath_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&wd, "class.warming_drawer.sabbath_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&wd, "class.warming_drawer.eco_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&wd, "class.warming_drawer.eco_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&wd, "class.warming_drawer.timer_s", Value::DurationS(3600))
            .unwrap();
        assert_eq!(
            sim.read_value(&wd, "class.warming_drawer.timer_s").unwrap(),
            Value::DurationS(3600)
        );
        sim.write(&wd, "trait.temperature.setpoint_c", Value::F32(70.0))
            .unwrap();
        assert!(
            (f32_val(&sim.read_value(&wd, "trait.temperature.setpoint_c").unwrap()) - 70.0).abs()
                < f32::EPSILON
        );

        let err = sim
            .write(&wd, "class.warming_drawer.heater_on", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &wd,
                "class.warming_drawer.high_temp_alarm",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&wd, "class.warming_drawer.door_ajar", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn pizza_oven_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let oven = sim.spawn(ApplianceClassId::PizzaOven).unwrap();

        assert!(
            (f32_val(&sim.read_value(&oven, "class.pizza_oven.stone_c").unwrap()) - 20.0).abs()
                < f32::EPSILON
        );
        assert!(
            (f32_val(&sim.read_value(&oven, "class.pizza_oven.dome_c").unwrap()) - 20.0).abs()
                < f32::EPSILON
        );
        assert_eq!(
            sim.read_value(&oven, "class.pizza_oven.top_bottom_balance")
                .unwrap(),
            Value::I16(0)
        );
        assert_eq!(
            sim.read_value(&oven, "class.pizza_oven.sabbath_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&oven, "class.pizza_oven.eco_mode").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&oven, "class.pizza_oven.heater_on").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&oven, "class.pizza_oven.high_temp_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&oven, "class.pizza_oven.door_ajar").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&oven, "class.pizza_oven.timer_s").unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&oven, "class.pizza_oven.steam_inject")
                .unwrap(),
            Value::Bool(false)
        );

        sim.write(&oven, "class.pizza_oven.top_bottom_balance", Value::I16(40))
            .unwrap();
        assert_eq!(
            sim.read_value(&oven, "class.pizza_oven.top_bottom_balance")
                .unwrap(),
            Value::I16(40)
        );
        sim.write(&oven, "class.pizza_oven.sabbath_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&oven, "class.pizza_oven.sabbath_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&oven, "class.pizza_oven.eco_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&oven, "class.pizza_oven.eco_mode").unwrap(),
            Value::Bool(true)
        );
        sim.write(&oven, "class.pizza_oven.timer_s", Value::DurationS(600))
            .unwrap();
        assert_eq!(
            sim.read_value(&oven, "class.pizza_oven.timer_s").unwrap(),
            Value::DurationS(600)
        );
        sim.write(&oven, "class.pizza_oven.steam_inject", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&oven, "class.pizza_oven.steam_inject")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&oven, "trait.temperature.setpoint_c", Value::F32(400.0))
            .unwrap();
        assert!(
            (f32_val(
                &sim.read_value(&oven, "trait.temperature.setpoint_c")
                    .unwrap()
            ) - 400.0)
                .abs()
                < f32::EPSILON
        );

        let err = sim
            .write(&oven, "class.pizza_oven.stone_c", Value::F32(300.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&oven, "class.pizza_oven.dome_c", Value::F32(350.0))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&oven, "class.pizza_oven.heater_on", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&oven, "class.pizza_oven.high_temp_alarm", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&oven, "class.pizza_oven.door_ajar", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn electric_grill_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let grill = sim.spawn(ApplianceClassId::ElectricGrill).unwrap();

        assert!(
            (f32_val(
                &sim.read_value(&grill, "class.electric_grill.plate_top_c")
                    .unwrap()
            ) - 20.0)
                .abs()
                < f32::EPSILON
        );
        assert!(
            (f32_val(
                &sim.read_value(&grill, "class.electric_grill.plate_bottom_c")
                    .unwrap()
            ) - 20.0)
                .abs()
                < f32::EPSILON
        );
        assert_eq!(
            sim.read_value(&grill, "class.electric_grill.sear").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&grill, "class.electric_grill.grease_tray")
                .unwrap(),
            Value::Enum("ok".into())
        );
        assert_eq!(
            sim.read_value(&grill, "class.electric_grill.sabbath_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&grill, "class.electric_grill.eco_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&grill, "class.electric_grill.heater_on")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&grill, "class.electric_grill.high_temp_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&grill, "class.electric_grill.lid_open")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&grill, "class.electric_grill.timer_s")
                .unwrap(),
            Value::DurationS(0)
        );

        sim.write(&grill, "class.electric_grill.sear", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&grill, "class.electric_grill.sear").unwrap(),
            Value::Bool(true)
        );
        sim.write(
            &grill,
            "class.electric_grill.sabbath_mode",
            Value::Bool(true),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&grill, "class.electric_grill.sabbath_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&grill, "class.electric_grill.eco_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&grill, "class.electric_grill.eco_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(
            &grill,
            "class.electric_grill.timer_s",
            Value::DurationS(600),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&grill, "class.electric_grill.timer_s")
                .unwrap(),
            Value::DurationS(600)
        );
        sim.write(&grill, "trait.temperature.setpoint_c", Value::F32(220.0))
            .unwrap();
        assert!(
            (f32_val(
                &sim.read_value(&grill, "trait.temperature.setpoint_c")
                    .unwrap()
            ) - 220.0)
                .abs()
                < f32::EPSILON
        );

        let err = sim
            .write(
                &grill,
                "class.electric_grill.plate_top_c",
                Value::F32(200.0),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &grill,
                "class.electric_grill.plate_bottom_c",
                Value::F32(200.0),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &grill,
                "class.electric_grill.grease_tray",
                Value::Enum("full".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&grill, "class.electric_grill.heater_on", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &grill,
                "class.electric_grill.high_temp_alarm",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(&grill, "class.electric_grill.lid_open", Value::Bool(true))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn electric_smoker_optional_depth_points_read_and_write() {
        let mut sim = Simulator::new();
        let smoker = sim.spawn(ApplianceClassId::ElectricSmoker).unwrap();

        assert!(
            (f32_val(
                &sim.read_value(&smoker, "class.electric_smoker.chamber_c")
                    .unwrap()
            ) - 20.0)
                .abs()
                < f32::EPSILON
        );
        assert_eq!(
            sim.read_value(&smoker, "class.electric_smoker.smoke_on")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&smoker, "class.electric_smoker.fuel_percent")
                .unwrap(),
            Value::Percent(80.0)
        );
        assert_eq!(
            sim.read_value(&smoker, "class.electric_smoker.water_pan")
                .unwrap(),
            Value::Enum("ok".into())
        );
        assert_eq!(
            sim.read_value(&smoker, "class.electric_smoker.sabbath_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&smoker, "class.electric_smoker.eco_mode")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&smoker, "class.electric_smoker.heater_on")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&smoker, "class.electric_smoker.high_temp_alarm")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&smoker, "class.electric_smoker.door_ajar")
                .unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            sim.read_value(&smoker, "class.electric_smoker.timer_s")
                .unwrap(),
            Value::DurationS(0)
        );

        sim.write(&smoker, "class.electric_smoker.smoke_on", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&smoker, "class.electric_smoker.smoke_on")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(
            &smoker,
            "class.electric_smoker.sabbath_mode",
            Value::Bool(true),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&smoker, "class.electric_smoker.sabbath_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(&smoker, "class.electric_smoker.eco_mode", Value::Bool(true))
            .unwrap();
        assert_eq!(
            sim.read_value(&smoker, "class.electric_smoker.eco_mode")
                .unwrap(),
            Value::Bool(true)
        );
        sim.write(
            &smoker,
            "class.electric_smoker.timer_s",
            Value::DurationS(600),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&smoker, "class.electric_smoker.timer_s")
                .unwrap(),
            Value::DurationS(600)
        );
        sim.write(&smoker, "trait.temperature.setpoint_c", Value::F32(110.0))
            .unwrap();
        assert!(
            (f32_val(
                &sim.read_value(&smoker, "trait.temperature.setpoint_c")
                    .unwrap()
            ) - 110.0)
                .abs()
                < f32::EPSILON
        );

        let err = sim
            .write(
                &smoker,
                "class.electric_smoker.chamber_c",
                Value::F32(100.0),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &smoker,
                "class.electric_smoker.fuel_percent",
                Value::Percent(50.0),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &smoker,
                "class.electric_smoker.water_pan",
                Value::Enum("empty".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &smoker,
                "class.electric_smoker.heater_on",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &smoker,
                "class.electric_smoker.high_temp_alarm",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
        let err = sim
            .write(
                &smoker,
                "class.electric_smoker.door_ajar",
                Value::Bool(true),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn spawn_cooking_tier_a_classes_identity_power_and_writes() {
        let mut sim = Simulator::new();
        for class in TIER_A_CLASS_IDS {
            let id = sim.spawn(*class).unwrap();
            assert_eq!(
                sim.read_value(&id, "trait.identity.class_id").unwrap(),
                Value::Enum(class.as_str().into())
            );
            let power = sim.read_value(&id, "trait.power.power_state").unwrap();
            assert!(matches!(power, Value::Enum(_)));
        }

        let steam = sim.spawn(ApplianceClassId::SteamOven).unwrap();
        sim.write(
            &steam,
            "class.steam_oven.steam_mode",
            Value::Enum("steam".into()),
        )
        .unwrap();

        let cooktop = sim.spawn(ApplianceClassId::Cooktop).unwrap();
        sim.write(&cooktop, "class.cooktop.level#hob_1", Value::U8(4))
            .unwrap();

        let range = sim.spawn(ApplianceClassId::Range).unwrap();
        sim.write(&range, "class.range.level#hob_1", Value::U8(2))
            .unwrap();
        sim.write(
            &range,
            "trait.temperature.setpoint_c#oven",
            Value::F32(180.0),
        )
        .unwrap();

        let coffee = sim.spawn(ApplianceClassId::CoffeeMachine).unwrap();
        assert_eq!(
            sim.read_value(&coffee, "trait.power.power_state").unwrap(),
            Value::Enum("standby".into())
        );
        sim.write(
            &coffee,
            "trait.program.program",
            Value::Enum("latte".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&coffee, "class.coffee_machine.water_tank")
                .unwrap(),
            Value::Enum("ok".into())
        );

        let sv = sim.spawn(ApplianceClassId::SousVide).unwrap();
        sim.write(&sv, "trait.temperature.setpoint_c", Value::F32(55.0))
            .unwrap();
        assert_eq!(
            sim.read_value(&sv, "class.sous_vide.low_water").unwrap(),
            Value::Bool(false)
        );

        let multi = sim.spawn(ApplianceClassId::MultiCooker).unwrap();
        sim.write(
            &multi,
            "trait.program.program",
            Value::Enum("pressure".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&multi, "class.multi_cooker.safe_to_open")
                .unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn kettle_setpoint_write_then_heat() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::Kettle).unwrap();
        sim.write(&id, "trait.temperature.setpoint_c", Value::F32(80.0))
            .unwrap();
        sim.write(&id, "trait.cycle.start", Value::Void).unwrap();
        sim.tick(&id, 16_000).unwrap();
        let current = f32_val(&sim.read_value(&id, "trait.temperature.current_c").unwrap());
        assert!((current - 80.0).abs() < 0.01, "current={current}");
    }

    #[test]
    fn oven_heats_toward_setpoint_when_cycle_running() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::Oven).unwrap();
        sim.write(&id, "trait.program.program", Value::Enum("bake".into()))
            .unwrap();
        sim.write(&id, "trait.temperature.setpoint_c", Value::F32(180.0))
            .unwrap();
        sim.write(&id, "trait.cycle.start", Value::Void).unwrap();
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("running".into())
        );
        // 10 °C/s from 20 → 170 needs 15 s; use 16 s of sim time.
        sim.tick(&id, 16_000).unwrap();
        let current = f32_val(&sim.read_value(&id, "trait.temperature.current_c").unwrap());
        assert!(
            current >= 170.0,
            "current={current}, expected >= 170 after 16s at 10 C/s"
        );
        assert_eq!(
            sim.read_value(&id, "trait.program.program").unwrap(),
            Value::Enum("bake".into())
        );
    }

    #[test]
    fn air_fryer_heats_toward_setpoint_when_cycle_running() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::AirFryer).unwrap();
        sim.write(&id, "trait.program.program", Value::Enum("fries".into()))
            .unwrap();
        sim.write(&id, "trait.temperature.setpoint_c", Value::F32(200.0))
            .unwrap();
        sim.write(&id, "trait.cycle.start", Value::Void).unwrap();
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("running".into())
        );
        // 10 °C/s from 20 → 190 needs 17 s; use 18 s of sim time.
        sim.tick(&id, 18_000).unwrap();
        let current = f32_val(&sim.read_value(&id, "trait.temperature.current_c").unwrap());
        assert!(
            current >= 190.0,
            "current={current}, expected >= 190 after 18s at 10 C/s"
        );
        assert_eq!(
            sim.read_value(&id, "trait.program.program").unwrap(),
            Value::Enum("fries".into())
        );
    }

    #[test]
    fn coffee_boiler_heats_toward_target_when_cycle_running() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::CoffeeMachine).unwrap();
        sim.write(&id, "trait.program.program", Value::Enum("espresso".into()))
            .unwrap();
        sim.write(&id, "trait.cycle.start", Value::Void).unwrap();
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("running".into())
        );
        // 10 °C/s from 20 → 85 needs 6.5 s; use 7 s of sim time.
        sim.tick(&id, 7_000).unwrap();
        let boiler = f32_val(
            &sim.read_value(&id, "class.coffee_machine.boiler_c")
                .unwrap(),
        );
        assert!(
            boiler >= 85.0,
            "boiler_c={boiler}, expected >= 85 after 7s at 10 C/s"
        );
        assert_eq!(
            sim.read_value(&id, "trait.program.program").unwrap(),
            Value::Enum("espresso".into())
        );
    }

    #[test]
    fn microwave_cook_advances_elapsed_toward_cook_s() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::Microwave).unwrap();
        sim.write(&id, "class.microwave.cook_s", Value::DurationS(45))
            .unwrap();
        sim.write(
            &id,
            "class.microwave.power_level_percent",
            Value::Percent(70.0),
        )
        .unwrap();
        sim.write(&id, "trait.cycle.start", Value::Void).unwrap();
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("running".into())
        );
        assert_eq!(
            sim.read_value(&id, "trait.cycle.elapsed_s").unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&id, "trait.cycle.remaining_s").unwrap(),
            Value::DurationS(45)
        );

        sim.tick(&id, 20_000).unwrap();
        assert_eq!(
            sim.read_value(&id, "trait.cycle.elapsed_s").unwrap(),
            Value::DurationS(20)
        );
        assert_eq!(
            sim.read_value(&id, "trait.cycle.remaining_s").unwrap(),
            Value::DurationS(25)
        );
        let progress = f32_val(&sim.read_value(&id, "trait.cycle.progress_percent").unwrap());
        assert!(
            (progress - (20.0 / 45.0 * 100.0)).abs() < 0.5,
            "progress={progress}"
        );
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("running".into())
        );

        sim.tick(&id, 25_000).unwrap();
        assert_eq!(
            sim.read_value(&id, "trait.cycle.elapsed_s").unwrap(),
            Value::DurationS(45)
        );
        assert_eq!(
            sim.read_value(&id, "trait.cycle.remaining_s").unwrap(),
            Value::DurationS(0)
        );
        assert_eq!(
            sim.read_value(&id, "trait.cycle.cycle_state").unwrap(),
            Value::Enum("complete".into())
        );
        assert_eq!(
            sim.read_value(&id, "trait.power.power_state").unwrap(),
            Value::Enum("standby".into())
        );
        let done = f32_val(&sim.read_value(&id, "trait.cycle.progress_percent").unwrap());
        assert!((done - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn water_heater_thermal_port_read_write() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::WaterHeater).unwrap();
        assert_eq!(
            sim.read_value(&id, "class.water_heater.thermal_port_id")
                .unwrap(),
            Value::String("preheat".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.water_heater.thermal_port_direction")
                .unwrap(),
            Value::Enum("sink".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.water_heater.thermal_port_media")
                .unwrap(),
            Value::Enum("water".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.water_heater.thermal_port_max_power_w")
                .unwrap(),
            Value::F32(2_000.0)
        );
        assert_eq!(
            sim.read_value(&id, "class.water_heater.thermal_port_attached_reservoir_id")
                .unwrap(),
            Value::String(String::new())
        );
        sim.write(
            &id,
            "class.water_heater.thermal_port_attached_reservoir_id",
            Value::String("dhw-tank".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&id, "class.water_heater.thermal_port_attached_reservoir_id")
                .unwrap(),
            Value::String("dhw-tank".into())
        );
        let err = sim
            .write(
                &id,
                "class.water_heater.thermal_port_direction",
                Value::Enum("source".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn fridge_thermal_port_read_write() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::Fridge).unwrap();
        assert_eq!(
            sim.read_value(&id, "class.fridge.thermal_port_id").unwrap(),
            Value::String("condenser".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.fridge.thermal_port_direction")
                .unwrap(),
            Value::Enum("source".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.fridge.thermal_port_media")
                .unwrap(),
            Value::Enum("water".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.fridge.thermal_port_max_power_w")
                .unwrap(),
            Value::F32(120.0)
        );
        sim.write(
            &id,
            "class.fridge.thermal_port_attached_reservoir_id",
            Value::String("dhw-tank".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&id, "class.fridge.thermal_port_attached_reservoir_id")
                .unwrap(),
            Value::String("dhw-tank".into())
        );
    }

    #[test]
    fn hvac_thermal_port_read_write() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::Hvac).unwrap();
        assert_eq!(
            sim.read_value(&id, "class.hvac.thermal_port_id").unwrap(),
            Value::String("coil".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.hvac.thermal_port_direction")
                .unwrap(),
            Value::Enum("sink".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.hvac.thermal_port_media")
                .unwrap(),
            Value::Enum("water".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.hvac.thermal_port_max_power_w")
                .unwrap(),
            Value::F32(5_000.0)
        );
        assert_eq!(
            sim.read_value(&id, "class.hvac.thermal_port_attached_reservoir_id")
                .unwrap(),
            Value::String(String::new())
        );
        sim.write(
            &id,
            "class.hvac.thermal_port_attached_reservoir_id",
            Value::String("chw-buffer".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&id, "class.hvac.thermal_port_attached_reservoir_id")
                .unwrap(),
            Value::String("chw-buffer".into())
        );
        let err = sim
            .write(
                &id,
                "class.hvac.thermal_port_media",
                Value::Enum("air".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn dishwasher_thermal_port_read_write() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::Dishwasher).unwrap();
        assert_eq!(
            sim.read_value(&id, "class.dishwasher.thermal_port_id")
                .unwrap(),
            Value::String("inlet_preheat".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.dishwasher.thermal_port_direction")
                .unwrap(),
            Value::Enum("sink".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.dishwasher.thermal_port_media")
                .unwrap(),
            Value::Enum("water".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.dishwasher.thermal_port_max_power_w")
                .unwrap(),
            Value::F32(1_800.0)
        );
        assert_eq!(
            sim.read_value(&id, "class.dishwasher.thermal_port_attached_reservoir_id")
                .unwrap(),
            Value::String(String::new())
        );
        sim.write(
            &id,
            "class.dishwasher.thermal_port_attached_reservoir_id",
            Value::String("dhw-tank".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&id, "class.dishwasher.thermal_port_attached_reservoir_id")
                .unwrap(),
            Value::String("dhw-tank".into())
        );
        let err = sim
            .write(
                &id,
                "class.dishwasher.thermal_port_direction",
                Value::Enum("source".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }

    #[test]
    fn dryer_thermal_port_read_write() {
        let mut sim = Simulator::new();
        let id = sim.spawn(ApplianceClassId::Dryer).unwrap();
        assert_eq!(
            sim.read_value(&id, "class.dryer.thermal_port_id").unwrap(),
            Value::String("exhaust".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.dryer.thermal_port_direction")
                .unwrap(),
            Value::Enum("source".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.dryer.thermal_port_media")
                .unwrap(),
            Value::Enum("air".into())
        );
        assert_eq!(
            sim.read_value(&id, "class.dryer.thermal_port_max_power_w")
                .unwrap(),
            Value::F32(2_000.0)
        );
        assert_eq!(
            sim.read_value(&id, "class.dryer.thermal_port_attached_reservoir_id")
                .unwrap(),
            Value::String(String::new())
        );
        sim.write(
            &id,
            "class.dryer.thermal_port_attached_reservoir_id",
            Value::String("air-buffer".into()),
        )
        .unwrap();
        assert_eq!(
            sim.read_value(&id, "class.dryer.thermal_port_attached_reservoir_id")
                .unwrap(),
            Value::String("air-buffer".into())
        );
        let err = sim
            .write(
                &id,
                "class.dryer.thermal_port_direction",
                Value::Enum("sink".into()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::NotWritable);
    }
}
