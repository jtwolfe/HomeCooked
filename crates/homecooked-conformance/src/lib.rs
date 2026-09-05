//! Light end-to-end conformance smoke helpers (Stream 7).
//!
//! Integration entry point: `tests/smoke.rs` (`cargo test -p homecooked-conformance`).

use std::fmt;
use std::thread;
use std::time::Duration;

use homecooked_bridge::{
    BacnetBridge, BacnetObjectType, BacnetPropValue, BacnetProperty, BacnetRaw, Bridge, ForeignRaw,
    ForeignRef, MatterAttrValue, MatterBridge, MatterRaw, ModbusBridge, PointRef, ZigbeeAttrValue,
    ZigbeeBridge, ZigbeeRaw,
};
use homecooked_controller::{Controller, CottonOptions, CyclePhase, CycleState, WasherState};
use homecooked_core::DeviceId;
use homecooked_hal::ChannelId;
use homecooked_hub::{LabHub, LAB_KETTLE_ID};
use homecooked_procedure::{
    run, DeviceBindings, Procedure, DISHWASHER_DHW_PREHEAT_JSON, KETTLE_HEAT_80_JSON,
    WASH_THEN_DRY_JSON,
};
use homecooked_protocol::{Envelope, Payload, PingBody, WriteOp};
use homecooked_schema::{
    typical_capability, ApplianceClassId, QualifiedPointId, Value, TIER_A_CLASS_IDS,
    TIER_B_CLASS_IDS,
};
use homecooked_sim::Simulator;
use homecooked_thermal::{
    energy_kwh, PortRef, PowerBandW, ThermalPlant, TransferOffer, TransferReply, TransferTarget,
};
use homecooked_transport::{spawn_server, spawn_server_with_config, ServerConfig, TcpClient};

/// Named smoke scenario failure (printed by the suite runner).
#[derive(Debug)]
pub struct ScenarioError {
    pub scenario: &'static str,
    pub message: String,
}

impl ScenarioError {
    pub fn new(scenario: &'static str, message: impl Into<String>) -> Self {
        Self {
            scenario,
            message: message.into(),
        }
    }
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.scenario, self.message)
    }
}

impl std::error::Error for ScenarioError {}

type ScenarioResult = Result<(), ScenarioError>;
type ScenarioFn = fn() -> ScenarioResult;

fn err(scenario: &'static str, message: impl Into<String>) -> ScenarioError {
    ScenarioError::new(scenario, message)
}

/// (1) Every Tier-A class: typical_capability exists; sim spawn; describe → class id.
pub fn tier_a_catalog_sim_describe() -> ScenarioResult {
    const NAME: &str = "tier_a_catalog_sim_describe";
    if TIER_A_CLASS_IDS.len() != 25 {
        return Err(err(
            NAME,
            format!("expected 25 Tier-A ids, got {}", TIER_A_CLASS_IDS.len()),
        ));
    }

    let mut sim = Simulator::new();
    for &class_id in TIER_A_CLASS_IDS {
        let cap = typical_capability(class_id).ok_or_else(|| {
            err(
                NAME,
                format!("typical_capability missing for {}", class_id.as_str()),
            )
        })?;
        if cap.class_id != class_id {
            return Err(err(
                NAME,
                format!(
                    "capability class_id mismatch for {}: got {}",
                    class_id.as_str(),
                    cap.class_id.as_str()
                ),
            ));
        }

        let id = sim.spawn(class_id).map_err(|e| {
            err(
                NAME,
                format!("sim spawn failed for {}: {e}", class_id.as_str()),
            )
        })?;

        let describe = Envelope::request(
            Some(id.as_str().into()),
            Payload::Describe(homecooked_protocol::DescribeRequest { points: vec![] }),
        );
        let resp = sim.handle(describe);
        match resp.payload {
            Payload::DescribeOk(body) => {
                if body.capability.class_id != class_id {
                    return Err(err(
                        NAME,
                        format!(
                            "describe class_id for {}: got {}",
                            class_id.as_str(),
                            body.capability.class_id.as_str()
                        ),
                    ));
                }
            }
            other => {
                return Err(err(
                    NAME,
                    format!(
                        "describe for {} expected DescribeOk, got {other:?}",
                        class_id.as_str()
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Tier-B classes: typical_capability + sim spawn (thinner tables).
pub fn tier_b_catalog_sim_describe() -> ScenarioResult {
    const NAME: &str = "tier_b_catalog_sim_describe";
    if TIER_B_CLASS_IDS.len() != 31 {
        return Err(err(
            NAME,
            format!("expected 31 Tier-B ids, got {}", TIER_B_CLASS_IDS.len()),
        ));
    }

    let mut sim = Simulator::new();
    for &class_id in TIER_B_CLASS_IDS {
        let _cap = typical_capability(class_id).ok_or_else(|| {
            err(
                NAME,
                format!("typical_capability missing for {}", class_id.as_str()),
            )
        })?;
        let id = sim.spawn(class_id).map_err(|e| {
            err(
                NAME,
                format!("sim spawn failed for {}: {e}", class_id.as_str()),
            )
        })?;
        let describe = Envelope::request(
            Some(id.as_str().into()),
            Payload::Describe(homecooked_protocol::DescribeRequest { points: vec![] }),
        );
        let resp = sim.handle(describe);
        match resp.payload {
            Payload::DescribeOk(body) => {
                if body.capability.class_id != class_id {
                    return Err(err(
                        NAME,
                        format!(
                            "describe class_id for {}: got {}",
                            class_id.as_str(),
                            body.capability.class_id.as_str()
                        ),
                    ));
                }
            }
            other => {
                return Err(err(
                    NAME,
                    format!(
                        "describe for {} expected DescribeOk, got {other:?}",
                        class_id.as_str()
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// (2) Washer cotton path via controller reaches Done.
pub fn washer_cotton_controller() -> ScenarioResult {
    const NAME: &str = "washer_cotton_controller";
    let mut ctrl = Controller::washer_cotton_demo()
        .map_err(|e| err(NAME, format!("washer_cotton_demo: {e}")))?;

    let door = ChannelId::new("din.door_closed").map_err(|e| err(NAME, e.to_string()))?;
    ctrl.hal_mut()
        .inject(&door, true)
        .map_err(|e| err(NAME, format!("inject door: {e}")))?;

    let opts = CottonOptions {
        wash_temp_c: 40.0,
        spin_rpm: 800.0,
        target_fill_pa: 2500.0,
        wash_tumble_ticks: 2,
        spin_ticks: 2,
        rinse_tumble_ticks: 1,
    };
    ctrl.start_cotton(opts)
        .map_err(|e| err(NAME, format!("start_cotton: {e}")))?;
    ctrl.run_until_done(200)
        .map_err(|e| err(NAME, format!("run_until_done: {e}")))?;

    if ctrl.cycle_state() != CycleState::Complete {
        return Err(err(
            NAME,
            format!("cycle_state={:?}, expected Complete", ctrl.cycle_state()),
        ));
    }
    if ctrl.washer_state() != WasherState::Done {
        return Err(err(
            NAME,
            format!("washer_state={:?}, expected Done", ctrl.washer_state()),
        ));
    }
    if ctrl.phase() != CyclePhase::Complete {
        return Err(err(
            NAME,
            format!("phase={:?}, expected Complete", ctrl.phase()),
        ));
    }
    Ok(())
}

/// (3) Procedure kettle happy path via homecooked-procedure.
pub fn procedure_kettle_happy_path() -> ScenarioResult {
    const NAME: &str = "procedure_kettle_happy_path";
    let doc = Procedure::load_json(KETTLE_HEAT_80_JSON)
        .map_err(|e| err(NAME, format!("load procedure: {e}")))?;
    let mut sim = Simulator::new();
    let id = sim
        .spawn(ApplianceClassId::Kettle)
        .map_err(|e| err(NAME, format!("spawn kettle: {e}")))?;
    let bindings = DeviceBindings::new().bind("kettle", id.as_str());
    let result = run(&doc, &mut sim, &bindings);
    if !result.is_completed() {
        return Err(err(
            NAME,
            format!("expected completed, got {:?}", result.status),
        ));
    }
    let current = sim
        .read_value(&DeviceId::new(id.as_str()), "trait.temperature.current_c")
        .map_err(|e| err(NAME, format!("read current_c: {e}")))?;
    let c = current
        .as_f64()
        .ok_or_else(|| err(NAME, format!("current_c not numeric: {current:?}")))?;
    if c < 75.0 {
        return Err(err(NAME, format!("current_c={c}, expected >= 75")));
    }
    Ok(())
}

/// (3b) Multi-device wash-then-dry procedure via homecooked-procedure + sim.
pub fn procedure_wash_then_dry() -> ScenarioResult {
    const NAME: &str = "procedure_wash_then_dry";
    let doc = Procedure::load_json(WASH_THEN_DRY_JSON)
        .map_err(|e| err(NAME, format!("load procedure: {e}")))?;
    let mut sim = Simulator::new();
    let washer = sim
        .spawn(ApplianceClassId::Washer)
        .map_err(|e| err(NAME, format!("spawn washer: {e}")))?;
    let dryer = sim
        .spawn(ApplianceClassId::Dryer)
        .map_err(|e| err(NAME, format!("spawn dryer: {e}")))?;
    let bindings = DeviceBindings::new()
        .bind("washer", washer.as_str())
        .bind("dryer", dryer.as_str());
    let result = run(&doc, &mut sim, &bindings);
    if !result.is_completed() {
        return Err(err(
            NAME,
            format!("expected completed, got {:?}", result.status),
        ));
    }
    let washer_state = sim
        .read_value(&DeviceId::new(washer.as_str()), "trait.cycle.cycle_state")
        .map_err(|e| err(NAME, format!("read washer cycle_state: {e}")))?;
    let dryer_state = sim
        .read_value(&DeviceId::new(dryer.as_str()), "trait.cycle.cycle_state")
        .map_err(|e| err(NAME, format!("read dryer cycle_state: {e}")))?;
    if washer_state != Value::Enum("complete".into()) {
        return Err(err(
            NAME,
            format!("washer cycle_state={washer_state:?}, expected complete"),
        ));
    }
    if dryer_state != Value::Enum("complete".into()) {
        return Err(err(
            NAME,
            format!("dryer cycle_state={dryer_state:?}, expected complete"),
        ));
    }
    Ok(())
}

/// (4) Thermal fridge → DHW demo via homecooked-thermal.
pub fn thermal_fridge_dhw_demo() -> ScenarioResult {
    const NAME: &str = "thermal_fridge_dhw_demo";
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo()
        .map_err(|e| err(NAME, format!("demo plant: {e}")))?;

    if plant.get_port("fridge-kitchen", "condenser").is_none() {
        return Err(err(NAME, "missing fridge condenser port"));
    }
    let preheat = plant
        .get_port("water-heater-plant", "preheat")
        .ok_or_else(|| err(NAME, "missing water-heater preheat port"))?;
    if preheat.attached_reservoir_id.as_deref() != Some("dhw-tank") {
        return Err(err(
            NAME,
            format!(
                "preheat reservoir={:?}, expected dhw-tank",
                preheat.attached_reservoir_id
            ),
        ));
    }

    let start = plant
        .get_reservoir("dhw-tank")
        .and_then(|r| r.temp_c)
        .ok_or_else(|| err(NAME, "dhw-tank missing temp"))?;
    if (start - 35.0).abs() >= 1e-4 {
        return Err(err(NAME, format!("dhw start temp={start}, expected 35")));
    }

    let offer = TransferOffer::new(
        PortRef::new("fridge-kitchen", "condenser").map_err(|e| err(NAME, e.to_string()))?,
        TransferTarget::port("water-heater-plant", "preheat")
            .map_err(|e| err(NAME, e.to_string()))?,
        PowerBandW::new(80, 120).map_err(|e| err(NAME, e.to_string()))?,
        None,
        1,
    );
    match plant.negotiate(offer) {
        TransferReply::Accept(a) => {
            if a.accepted_power_w != 120 {
                return Err(err(
                    NAME,
                    format!("accepted_power_w={}, expected 120", a.accepted_power_w),
                ));
            }
        }
        other => return Err(err(NAME, format!("expected Accept, got {other:?}"))),
    }

    let results = plant
        .step(3_600.0)
        .map_err(|e| err(NAME, format!("step: {e}")))?;
    if results.len() != 1 {
        return Err(err(
            NAME,
            format!("expected 1 transfer result, got {}", results.len()),
        ));
    }
    if results[0].power_w != 120 {
        return Err(err(
            NAME,
            format!("power_w={}, expected 120", results[0].power_w),
        ));
    }
    let expected_kwh = energy_kwh(120, 3_600.0);
    if (results[0].energy_kwh - expected_kwh).abs() >= 1e-6 {
        return Err(err(
            NAME,
            format!(
                "energy_kwh={}, expected {expected_kwh}",
                results[0].energy_kwh
            ),
        ));
    }
    if (results[0].delta_temp_c - 1.2).abs() >= 1e-4 {
        return Err(err(
            NAME,
            format!("delta_temp_c={}, expected 1.2", results[0].delta_temp_c),
        ));
    }
    let end = plant
        .get_reservoir("dhw-tank")
        .and_then(|r| r.temp_c)
        .ok_or_else(|| err(NAME, "dhw-tank missing temp after step"))?;
    if (end - 36.2).abs() >= 1e-4 {
        return Err(err(NAME, format!("dhw end temp={end}, expected 36.2")));
    }
    Ok(())
}

/// (4b) Dual-path: fridge→DHW thermal transfer, then dishwasher preheat settings.
///
/// Procedures cannot call thermal APIs yet, so this scenario runs the plant
/// tick first, asserts DHW rose, then runs the dishwasher procedure leg
/// (`dishwasher_dhw_preheat`) which writes eco + wash_temp_c reflecting warm
/// inlet availability ([`docs/standard/thermal-plant.md`] §8).
pub fn thermal_then_dishwasher_preheat() -> ScenarioResult {
    const NAME: &str = "thermal_then_dishwasher_preheat";

    // --- Path A: thermal plant ---
    let mut plant = ThermalPlant::fridge_condenser_dhw_demo()
        .map_err(|e| err(NAME, format!("demo plant: {e}")))?;
    let start = plant
        .get_reservoir("dhw-tank")
        .and_then(|r| r.temp_c)
        .ok_or_else(|| err(NAME, "dhw-tank missing temp"))?;
    if (start - 35.0).abs() >= 1e-4 {
        return Err(err(NAME, format!("dhw start temp={start}, expected 35")));
    }

    let offer = TransferOffer::new(
        PortRef::new("fridge-kitchen", "condenser").map_err(|e| err(NAME, e.to_string()))?,
        TransferTarget::port("water-heater-plant", "preheat")
            .map_err(|e| err(NAME, e.to_string()))?,
        PowerBandW::new(80, 120).map_err(|e| err(NAME, e.to_string()))?,
        None,
        1,
    );
    match plant.negotiate(offer) {
        TransferReply::Accept(a) => {
            if a.accepted_power_w != 120 {
                return Err(err(
                    NAME,
                    format!("accepted_power_w={}, expected 120", a.accepted_power_w),
                ));
            }
        }
        other => return Err(err(NAME, format!("expected Accept, got {other:?}"))),
    }
    let results = plant
        .step(3_600.0)
        .map_err(|e| err(NAME, format!("step: {e}")))?;
    if results.len() != 1 || results[0].power_w != 120 {
        return Err(err(
            NAME,
            format!("transfer results={results:?}, expected one 120 W result"),
        ));
    }
    let dhw_end = plant
        .get_reservoir("dhw-tank")
        .and_then(|r| r.temp_c)
        .ok_or_else(|| err(NAME, "dhw-tank missing temp after step"))?;
    if (dhw_end - 36.2).abs() >= 1e-4 {
        return Err(err(
            NAME,
            format!("dhw end temp={dhw_end}, expected 36.2 after transfer"),
        ));
    }
    if dhw_end <= start {
        return Err(err(
            NAME,
            format!("dhw temp did not rise: start={start}, end={dhw_end}"),
        ));
    }

    // --- Path B: dishwasher procedure reflecting preheat available ---
    let doc = Procedure::load_json(DISHWASHER_DHW_PREHEAT_JSON)
        .map_err(|e| err(NAME, format!("load procedure: {e}")))?;
    let mut sim = Simulator::new();
    let dw = sim
        .spawn(ApplianceClassId::Dishwasher)
        .map_err(|e| err(NAME, format!("spawn dishwasher: {e}")))?;
    let bindings = DeviceBindings::new().bind("dishwasher", dw.as_str());
    let result = run(&doc, &mut sim, &bindings);
    if !result.is_completed() {
        return Err(err(
            NAME,
            format!(
                "dishwasher procedure expected completed, got {:?}",
                result.status
            ),
        ));
    }

    let program = sim
        .read_value(&DeviceId::new(dw.as_str()), "trait.program.program")
        .map_err(|e| err(NAME, format!("read program: {e}")))?;
    if program != Value::Enum("eco".into()) {
        return Err(err(
            NAME,
            format!("program={program:?}, expected eco (preheat-aware)"),
        ));
    }
    let wash_temp = sim
        .read_value(&DeviceId::new(dw.as_str()), "class.dishwasher.wash_temp_c")
        .map_err(|e| err(NAME, format!("read wash_temp_c: {e}")))?;
    if wash_temp != Value::F32(45.0) {
        return Err(err(
            NAME,
            format!("wash_temp_c={wash_temp:?}, expected 45.0 (preheat-aware)"),
        ));
    }
    Ok(())
}

/// (5) Modbus water_heater roundtrip via bridge.
pub fn modbus_water_heater_roundtrip() -> ScenarioResult {
    const NAME: &str = "modbus_water_heater_roundtrip";
    let mut bridge = ModbusBridge::water_heater_example()
        .map_err(|e| err(NAME, format!("water_heater_example: {e}")))?;

    let setpoint = PointRef::new("water-heater-plant", "trait.temperature.setpoint_c")
        .map_err(|e| err(NAME, e.to_string()))?;
    let holding =
        ForeignRef::holding("water-heater-plant", 0).map_err(|e| err(NAME, e.to_string()))?;
    let coil = ForeignRef::coil("water-heater-plant", 0).map_err(|e| err(NAME, e.to_string()))?;
    let power = PointRef::new("water-heater-plant", "trait.power.power_state")
        .map_err(|e| err(NAME, e.to_string()))?;

    // Foreign register → HomeCooked point (60.0 °C as 600 tenths)
    let translated = bridge
        .write_foreign(&holding, ForeignRaw::Register(600))
        .map_err(|e| err(NAME, format!("write_foreign holding: {e}")))?;
    if translated != Value::F32(60.0) {
        return Err(err(
            NAME,
            format!("translated holding write={translated:?}, expected F32(60)"),
        ));
    }
    let read_sp = bridge
        .read_point(&setpoint)
        .map_err(|e| err(NAME, format!("read setpoint: {e}")))?;
    if read_sp != Value::F32(60.0) {
        return Err(err(
            NAME,
            format!("setpoint after foreign write={read_sp:?}"),
        ));
    }

    // HomeCooked → Modbus register
    bridge
        .write_point(&setpoint, &Value::F32(42.0))
        .map_err(|e| err(NAME, format!("write setpoint: {e}")))?;
    if bridge.slave().get_holding(0) != 420 {
        return Err(err(
            NAME,
            format!(
                "holding after HC write={}, expected 420",
                bridge.slave().get_holding(0)
            ),
        ));
    }

    // Power coil roundtrip
    bridge
        .write_foreign(&coil, ForeignRaw::Coil(false))
        .map_err(|e| err(NAME, format!("write coil off: {e}")))?;
    let off = bridge
        .read_point(&power)
        .map_err(|e| err(NAME, format!("read power: {e}")))?;
    if off != Value::Enum("off".into()) {
        return Err(err(NAME, format!("power after coil false={off:?}")));
    }
    bridge
        .write_point(&power, &Value::Enum("on".into()))
        .map_err(|e| err(NAME, format!("write power on: {e}")))?;
    if !bridge.slave().get_coil(0) {
        return Err(err(NAME, "coil expected true after power on"));
    }
    Ok(())
}

/// (5b) Matter kettle roundtrip via mock fabric bridge.
pub fn matter_kettle_roundtrip() -> ScenarioResult {
    const NAME: &str = "matter_kettle_roundtrip";
    let mut bridge =
        MatterBridge::kettle_example().map_err(|e| err(NAME, format!("kettle_example: {e}")))?;

    let setpoint = PointRef::new("kettle-lab-1", "trait.temperature.setpoint_c")
        .map_err(|e| err(NAME, e.to_string()))?;
    let attr = ForeignRef::matter("kettle-lab-1", 1, 0x0201, 0x0012)
        .map_err(|e| err(NAME, e.to_string()))?;
    let onoff = ForeignRef::matter("kettle-lab-1", 1, 0x0006, 0x0000)
        .map_err(|e| err(NAME, e.to_string()))?;
    let power = PointRef::new("kettle-lab-1", "trait.power.power_state")
        .map_err(|e| err(NAME, e.to_string()))?;

    // Foreign attribute → HomeCooked point (60.0 °C as 6000 hundredths)
    let translated = bridge
        .write_foreign(&attr, ForeignRaw::Matter(MatterRaw::Int16(6000)))
        .map_err(|e| err(NAME, format!("write_foreign setpoint: {e}")))?;
    if translated != Value::F32(60.0) {
        return Err(err(
            NAME,
            format!("translated attr write={translated:?}, expected F32(60)"),
        ));
    }
    let read_sp = bridge
        .read_point(&setpoint)
        .map_err(|e| err(NAME, format!("read setpoint: {e}")))?;
    if read_sp != Value::F32(60.0) {
        return Err(err(
            NAME,
            format!("setpoint after foreign write={read_sp:?}"),
        ));
    }

    // HomeCooked → Matter attribute
    bridge
        .write_point(&setpoint, &Value::F32(42.0))
        .map_err(|e| err(NAME, format!("write setpoint: {e}")))?;
    let stored = bridge.attr_store().read(1, 0x0201, 0x0012);
    if stored != Some(MatterAttrValue::Int16(4200)) {
        return Err(err(
            NAME,
            format!("attr after HC write={stored:?}, expected Int16(4200)"),
        ));
    }

    // OnOff roundtrip
    bridge
        .write_foreign(&onoff, ForeignRaw::Matter(MatterRaw::Bool(false)))
        .map_err(|e| err(NAME, format!("write onoff off: {e}")))?;
    let off = bridge
        .read_point(&power)
        .map_err(|e| err(NAME, format!("read power: {e}")))?;
    if off != Value::Enum("off".into()) {
        return Err(err(NAME, format!("power after onoff false={off:?}")));
    }
    bridge
        .write_point(&power, &Value::Enum("on".into()))
        .map_err(|e| err(NAME, format!("write power on: {e}")))?;
    if bridge.attr_store().read(1, 0x0006, 0x0000) != Some(MatterAttrValue::Bool(true)) {
        return Err(err(NAME, "onoff expected true after power on"));
    }
    Ok(())
}

/// (5c) Zigbee kettle roundtrip via mock network bridge.
pub fn zigbee_kettle_roundtrip() -> ScenarioResult {
    const NAME: &str = "zigbee_kettle_roundtrip";
    let mut bridge =
        ZigbeeBridge::kettle_example().map_err(|e| err(NAME, format!("kettle_example: {e}")))?;

    let setpoint = PointRef::new("kettle-lab-1", "trait.temperature.setpoint_c")
        .map_err(|e| err(NAME, e.to_string()))?;
    let attr = ForeignRef::zigbee("kettle-lab-1", 1, 0x0201, 0x0012)
        .map_err(|e| err(NAME, e.to_string()))?;
    let onoff = ForeignRef::zigbee("kettle-lab-1", 1, 0x0006, 0x0000)
        .map_err(|e| err(NAME, e.to_string()))?;
    let power = PointRef::new("kettle-lab-1", "trait.power.power_state")
        .map_err(|e| err(NAME, e.to_string()))?;

    // Foreign attribute → HomeCooked point (60.0 °C as 6000 hundredths)
    let translated = bridge
        .write_foreign(&attr, ForeignRaw::Zigbee(ZigbeeRaw::Int16(6000)))
        .map_err(|e| err(NAME, format!("write_foreign setpoint: {e}")))?;
    if translated != Value::F32(60.0) {
        return Err(err(
            NAME,
            format!("translated attr write={translated:?}, expected F32(60)"),
        ));
    }
    let read_sp = bridge
        .read_point(&setpoint)
        .map_err(|e| err(NAME, format!("read setpoint: {e}")))?;
    if read_sp != Value::F32(60.0) {
        return Err(err(
            NAME,
            format!("setpoint after foreign write={read_sp:?}"),
        ));
    }

    // HomeCooked → Zigbee attribute
    bridge
        .write_point(&setpoint, &Value::F32(42.0))
        .map_err(|e| err(NAME, format!("write setpoint: {e}")))?;
    let stored = bridge.attr_store().read(1, 0x0201, 0x0012);
    if stored != Some(ZigbeeAttrValue::Int16(4200)) {
        return Err(err(
            NAME,
            format!("attr after HC write={stored:?}, expected Int16(4200)"),
        ));
    }

    // OnOff roundtrip
    bridge
        .write_foreign(&onoff, ForeignRaw::Zigbee(ZigbeeRaw::Bool(false)))
        .map_err(|e| err(NAME, format!("write onoff off: {e}")))?;
    let off = bridge
        .read_point(&power)
        .map_err(|e| err(NAME, format!("read power: {e}")))?;
    if off != Value::Enum("off".into()) {
        return Err(err(NAME, format!("power after onoff false={off:?}")));
    }
    bridge
        .write_point(&power, &Value::Enum("on".into()))
        .map_err(|e| err(NAME, format!("write power on: {e}")))?;
    if bridge.attr_store().read(1, 0x0006, 0x0000) != Some(ZigbeeAttrValue::Bool(true)) {
        return Err(err(NAME, "onoff expected true after power on"));
    }
    Ok(())
}

/// (5d) BACnet kettle roundtrip via mock device bridge.
pub fn bacnet_kettle_roundtrip() -> ScenarioResult {
    const NAME: &str = "bacnet_kettle_roundtrip";
    let mut bridge =
        BacnetBridge::kettle_example().map_err(|e| err(NAME, format!("kettle_example: {e}")))?;

    let setpoint = PointRef::new("kettle-lab-1", "trait.temperature.setpoint_c")
        .map_err(|e| err(NAME, e.to_string()))?;
    let prop = ForeignRef::bacnet(
        "kettle-lab-1",
        1,
        BacnetObjectType::AnalogValue,
        1,
        BacnetProperty::PresentValue,
    )
    .map_err(|e| err(NAME, e.to_string()))?;
    let bv = ForeignRef::bacnet(
        "kettle-lab-1",
        1,
        BacnetObjectType::BinaryValue,
        1,
        BacnetProperty::PresentValue,
    )
    .map_err(|e| err(NAME, e.to_string()))?;
    let power = PointRef::new("kettle-lab-1", "trait.power.power_state")
        .map_err(|e| err(NAME, e.to_string()))?;

    let translated = bridge
        .write_foreign(&prop, ForeignRaw::Bacnet(BacnetRaw::Int16(6000)))
        .map_err(|e| err(NAME, format!("write_foreign setpoint: {e}")))?;
    if translated != Value::F32(60.0) {
        return Err(err(
            NAME,
            format!("translated prop write={translated:?}, expected F32(60)"),
        ));
    }
    let read_sp = bridge
        .read_point(&setpoint)
        .map_err(|e| err(NAME, format!("read setpoint: {e}")))?;
    if read_sp != Value::F32(60.0) {
        return Err(err(
            NAME,
            format!("setpoint after foreign write={read_sp:?}"),
        ));
    }

    bridge
        .write_point(&setpoint, &Value::F32(42.0))
        .map_err(|e| err(NAME, format!("write setpoint: {e}")))?;
    let stored = bridge.prop_store().read(
        BacnetObjectType::AnalogValue,
        1,
        BacnetProperty::PresentValue,
    );
    if stored != Some(BacnetPropValue::Int16(4200)) {
        return Err(err(
            NAME,
            format!("prop after HC write={stored:?}, expected Int16(4200)"),
        ));
    }

    bridge
        .write_foreign(&bv, ForeignRaw::Bacnet(BacnetRaw::Bool(false)))
        .map_err(|e| err(NAME, format!("write bv off: {e}")))?;
    let off = bridge
        .read_point(&power)
        .map_err(|e| err(NAME, format!("read power: {e}")))?;
    if off != Value::Enum("off".into()) {
        return Err(err(NAME, format!("power after bv false={off:?}")));
    }
    bridge
        .write_point(&power, &Value::Enum("on".into()))
        .map_err(|e| err(NAME, format!("write power on: {e}")))?;
    if bridge.prop_store().read(
        BacnetObjectType::BinaryValue,
        1,
        BacnetProperty::PresentValue,
    ) != Some(BacnetPropValue::Bool(true))
    {
        return Err(err(NAME, "binary_value expected true after power on"));
    }
    Ok(())
}

fn qid(s: &str) -> Result<QualifiedPointId, ScenarioError> {
    QualifiedPointId::parse(s)
        .map_err(|e| err("tcp_kettle_discover_describe_read_write", e.to_string()))
}

/// (6) TCP discover/describe/read/write against sim kettle on ephemeral port.
pub fn tcp_kettle_discover_describe_read_write() -> ScenarioResult {
    const NAME: &str = "tcp_kettle_discover_describe_read_write";
    let mut sim = Simulator::new();
    let kettle_id = sim
        .spawn_named("kettle-conformance", ApplianceClassId::Kettle)
        .map_err(|e| err(NAME, format!("spawn: {e}")))?;
    if kettle_id.as_str() != "kettle-conformance" {
        return Err(err(
            NAME,
            format!(
                "device id={}, expected kettle-conformance",
                kettle_id.as_str()
            ),
        ));
    }

    let (addr, _shared, _server) =
        spawn_server("127.0.0.1:0", sim).map_err(|e| err(NAME, format!("spawn_server: {e}")))?;
    thread::sleep(Duration::from_millis(20));

    let mut client = TcpClient::connect(addr).map_err(|e| err(NAME, format!("connect: {e}")))?;

    let discovered = client
        .discover(Some(ApplianceClassId::Kettle), vec![])
        .map_err(|e| err(NAME, format!("discover: {e}")))?;
    if discovered.devices.len() != 1 {
        return Err(err(
            NAME,
            format!("discover count={}, expected 1", discovered.devices.len()),
        ));
    }
    if discovered.devices[0].device_id != "kettle-conformance" {
        return Err(err(
            NAME,
            format!("discover id={}", discovered.devices[0].device_id),
        ));
    }
    if discovered.devices[0].class_id != ApplianceClassId::Kettle {
        return Err(err(
            NAME,
            format!("discover class={:?}", discovered.devices[0].class_id),
        ));
    }

    let desc = client
        .describe("kettle-conformance", vec![])
        .map_err(|e| err(NAME, format!("describe: {e}")))?;
    if desc.capability.class_id != ApplianceClassId::Kettle {
        return Err(err(
            NAME,
            format!("describe class={:?}", desc.capability.class_id),
        ));
    }

    let read = client
        .read(
            "kettle-conformance",
            vec![
                qid("trait.temperature.setpoint_c")?,
                qid("trait.temperature.current_c")?,
            ],
        )
        .map_err(|e| err(NAME, format!("read: {e}")))?;
    if read.values.len() != 2 {
        return Err(err(NAME, format!("read len={}", read.values.len())));
    }
    if read.values[0].value != Some(Value::F32(100.0)) {
        return Err(err(
            NAME,
            format!("setpoint={:?}, expected 100", read.values[0].value),
        ));
    }
    if read.values[1].value != Some(Value::F32(20.0)) {
        return Err(err(
            NAME,
            format!("current={:?}, expected 20", read.values[1].value),
        ));
    }

    let write = client
        .write(
            "kettle-conformance",
            vec![WriteOp {
                id: qid("trait.temperature.setpoint_c")?,
                value: Value::F32(80.0),
            }],
        )
        .map_err(|e| err(NAME, format!("write: {e}")))?;
    if write.accepted.len() != 1 || write.accepted[0].value != Value::F32(80.0) {
        return Err(err(NAME, format!("write accepted={:?}", write.accepted)));
    }

    let read2 = client
        .read(
            "kettle-conformance",
            vec![qid("trait.temperature.setpoint_c")?],
        )
        .map_err(|e| err(NAME, format!("readback: {e}")))?;
    if read2.values[0].value != Some(Value::F32(80.0)) {
        return Err(err(
            NAME,
            format!("readback setpoint={:?}", read2.values[0].value),
        ));
    }
    Ok(())
}

/// (7) TCP PSK: good shared secret → describe + ping against sim kettle.
pub fn tcp_psk_good_secret_describe_ping() -> ScenarioResult {
    const NAME: &str = "tcp_psk_good_secret_describe_ping";
    let mut sim = Simulator::new();
    let kettle_id = sim
        .spawn_named("kettle-psk-conformance", ApplianceClassId::Kettle)
        .map_err(|e| err(NAME, format!("spawn: {e}")))?;
    if kettle_id.as_str() != "kettle-psk-conformance" {
        return Err(err(
            NAME,
            format!(
                "device id={}, expected kettle-psk-conformance",
                kettle_id.as_str()
            ),
        ));
    }

    let (addr, _shared, _server) = spawn_server_with_config(
        "127.0.0.1:0",
        sim,
        ServerConfig::with_psk("lab-conformance-psk"),
    )
    .map_err(|e| err(NAME, format!("spawn_server_with_config: {e}")))?;
    thread::sleep(Duration::from_millis(20));

    let mut client = TcpClient::connect_with_psk(addr, Some("lab-conformance-psk"))
        .map_err(|e| err(NAME, format!("connect_with_psk: {e}")))?;

    let desc = client
        .describe("kettle-psk-conformance", vec![])
        .map_err(|e| err(NAME, format!("describe: {e}")))?;
    if desc.capability.class_id != ApplianceClassId::Kettle {
        return Err(err(
            NAME,
            format!(
                "describe class={:?}, expected Kettle",
                desc.capability.class_id
            ),
        ));
    }

    let req = Envelope::new(Payload::Ping(PingBody {
        echo: Some("psk-conformance".into()),
    }));
    let resp = client
        .exchange(&req)
        .map_err(|e| err(NAME, format!("ping: {e}")))?;
    match resp.payload {
        Payload::Pong(p) => {
            if p.echo.as_deref() != Some("psk-conformance") {
                return Err(err(
                    NAME,
                    format!("pong echo={:?}, expected psk-conformance", p.echo),
                ));
            }
        }
        other => {
            return Err(err(NAME, format!("expected Pong, got {other:?}")));
        }
    }
    Ok(())
}

/// (8) Optional lab hub: spawn lab set, TCP discover ≥3 devices, describe one.
pub fn hub_lab_set_discover_describe() -> ScenarioResult {
    const NAME: &str = "hub_lab_set_discover_describe";
    let mut hub = LabHub::new();
    let set = hub
        .spawn_lab_set()
        .map_err(|e| err(NAME, format!("spawn_lab_set: {e}")))?;
    if hub.list().len() < 3 {
        return Err(err(
            NAME,
            format!("lab set size={}, expected ≥3", hub.list().len()),
        ));
    }

    let spawned = hub
        .serve("127.0.0.1:0")
        .map_err(|e| err(NAME, format!("serve: {e}")))?;
    thread::sleep(Duration::from_millis(20));

    let mut client =
        TcpClient::connect(spawned.addr()).map_err(|e| err(NAME, format!("connect: {e}")))?;

    let discovered = client
        .discover(None, vec![])
        .map_err(|e| err(NAME, format!("discover: {e}")))?;
    if discovered.devices.len() < 3 {
        return Err(err(
            NAME,
            format!("discover count={}, expected ≥3", discovered.devices.len()),
        ));
    }
    let ids: Vec<&str> = discovered
        .devices
        .iter()
        .map(|d| d.device_id.as_str())
        .collect();
    for expected in [
        set.kettle.as_str(),
        set.washer.as_str(),
        set.fridge.as_str(),
    ] {
        if !ids.contains(&expected) {
            return Err(err(
                NAME,
                format!("discover missing {expected}; got {ids:?}"),
            ));
        }
    }

    let desc = client
        .describe(LAB_KETTLE_ID, vec![])
        .map_err(|e| err(NAME, format!("describe: {e}")))?;
    if desc.capability.class_id != ApplianceClassId::Kettle {
        return Err(err(
            NAME,
            format!(
                "describe class={:?}, expected Kettle",
                desc.capability.class_id
            ),
        ));
    }
    Ok(())
}

/// Ordered smoke scenarios for the suite runner.
pub fn all_scenarios() -> &'static [(&'static str, ScenarioFn)] {
    &[
        ("tier_a_catalog_sim_describe", tier_a_catalog_sim_describe),
        ("tier_b_catalog_sim_describe", tier_b_catalog_sim_describe),
        ("washer_cotton_controller", washer_cotton_controller),
        ("procedure_kettle_happy_path", procedure_kettle_happy_path),
        ("procedure_wash_then_dry", procedure_wash_then_dry),
        ("thermal_fridge_dhw_demo", thermal_fridge_dhw_demo),
        (
            "thermal_then_dishwasher_preheat",
            thermal_then_dishwasher_preheat,
        ),
        (
            "modbus_water_heater_roundtrip",
            modbus_water_heater_roundtrip,
        ),
        ("matter_kettle_roundtrip", matter_kettle_roundtrip),
        ("zigbee_kettle_roundtrip", zigbee_kettle_roundtrip),
        ("bacnet_kettle_roundtrip", bacnet_kettle_roundtrip),
        (
            "tcp_kettle_discover_describe_read_write",
            tcp_kettle_discover_describe_read_write,
        ),
        (
            "tcp_psk_good_secret_describe_ping",
            tcp_psk_good_secret_describe_ping,
        ),
    ]
}

/// Run every scenario; return aggregated named failures (empty = pass).
pub fn run_all() -> Vec<String> {
    let mut failures = Vec::new();
    for (name, f) in all_scenarios() {
        if let Err(e) = f() {
            failures.push(format!("{name}: {}", e.message));
        }
    }
    failures
}
