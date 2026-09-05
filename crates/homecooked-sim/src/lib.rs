//! In-memory simulated HomeCooked devices driven by the catalog.

mod behavior;
mod defaults;
mod simulator;

pub use behavior::{KETTLE_HEAT_RATE_C_PER_S, WASHER_CYCLE_S};
pub use defaults::{seed_identity, seed_state, sim_capability};
pub use simulator::Simulator;

#[cfg(test)]
mod tests {
    use homecooked_schema::{ApplianceClassId, ErrorCode, Value, STATIC_CLASS_IDS};

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
        assert_eq!(sim.list().len(), 9);
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
}
