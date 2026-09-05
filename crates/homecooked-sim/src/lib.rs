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
}
