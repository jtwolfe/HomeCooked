//! [`ModbusBridge`]: map + in-memory slave + HomeCooked backend.

use homecooked_schema::Value;

use super::map::{ForeignBits, ModbusEntry, ModbusMap, RegisterKind};
use super::mock::ModbusSlave;
use crate::backend::{MemoryBackend, PointBackend};
use crate::bridge::{Bridge, ForeignRaw, ForeignRef, PointRef};
use crate::error::Error;

/// Modbus adapter with an in-memory slave (TCP lab is a separate module).
#[derive(Debug, Clone)]
pub struct ModbusBridge<B> {
    map: ModbusMap,
    slave: ModbusSlave,
    backend: B,
}

impl ModbusBridge<MemoryBackend> {
    pub fn with_memory(map: ModbusMap) -> Result<Self, Error> {
        Self::new(map, MemoryBackend::new())
    }

    pub fn water_heater_example() -> Result<Self, Error> {
        Self::with_memory(ModbusMap::water_heater_example()?)
    }
}

impl<B: PointBackend> ModbusBridge<B> {
    pub fn new(map: ModbusMap, mut backend: B) -> Result<Self, Error> {
        map.validate()?;
        let slave = ModbusSlave::from_map(&map);
        for entry in &map.entries {
            let bits = slave.read_bits(entry.kind, entry.address);
            let value = entry.decode_bits(bits)?;
            let point = PointRef::new(&map.device_id, &entry.point)?;
            backend.set(&point, value)?;
        }
        Ok(Self {
            map,
            slave,
            backend,
        })
    }

    pub fn map(&self) -> &ModbusMap {
        &self.map
    }

    pub fn slave(&self) -> &ModbusSlave {
        &self.slave
    }

    pub fn slave_mut(&mut self) -> &mut ModbusSlave {
        &mut self.slave
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    fn require_device(&self, device_id: &str) -> Result<(), Error> {
        if device_id != self.map.device_id {
            return Err(Error::DeviceMismatch {
                expected: self.map.device_id.clone(),
                actual: device_id.to_string(),
            });
        }
        Ok(())
    }

    fn entry_for_point(&self, point: &PointRef) -> Result<&ModbusEntry, Error> {
        self.require_device(&point.device_id)?;
        self.map
            .entry_for_point(&point.point_id)
            .ok_or_else(|| Error::UnmappedPoint {
                device_id: point.device_id.clone(),
                point_id: point.point_id.clone(),
            })
    }

    fn entry_for_foreign(&self, foreign: &ForeignRef) -> Result<&ModbusEntry, Error> {
        self.require_device(&foreign.device_id)?;
        let (kind, address) = foreign.as_modbus().ok_or_else(|| Error::LocatorMismatch {
            expected: "modbus",
            locator: foreign.locator.clone(),
        })?;
        self.map
            .entry_for_address(kind, address)
            .ok_or(Error::UnmappedAddress { kind, address })
    }

    fn apply_bits(&mut self, entry: &ModbusEntry, bits: ForeignBits) -> Result<Value, Error> {
        self.slave.write_bits(entry.kind, entry.address, bits)?;
        let value = entry.decode_bits(bits)?;
        let point = PointRef::new(&self.map.device_id, &entry.point)?;
        self.backend.set(&point, value.clone())?;
        Ok(value)
    }

    fn raw_to_bits(entry: &ModbusEntry, raw: ForeignRaw) -> Result<ForeignBits, Error> {
        match (entry.kind, raw) {
            (RegisterKind::Holding | RegisterKind::Input, ForeignRaw::Register(v)) => {
                Ok(ForeignBits::Register(v))
            }
            (RegisterKind::Coil | RegisterKind::Discrete, ForeignRaw::Coil(v)) => {
                Ok(ForeignBits::Coil(v))
            }
            (kind, ForeignRaw::Register(_)) => Err(Error::InvalidRaw {
                detail: format!("register raw is not valid for {kind}"),
            }),
            (kind, ForeignRaw::Coil(_)) => Err(Error::InvalidRaw {
                detail: format!("coil raw is not valid for {kind}"),
            }),
            (_, ForeignRaw::Matter(_))
            | (_, ForeignRaw::Zigbee(_))
            | (_, ForeignRaw::Bacnet(_)) => Err(Error::InvalidRaw {
                detail: "cluster attribute raw is not valid for modbus bridge".into(),
            }),
        }
    }
}

impl<B: PointBackend> Bridge for ModbusBridge<B> {
    fn fabric(&self) -> &'static str {
        "modbus"
    }

    fn read_point(&self, point: &PointRef) -> Result<Value, Error> {
        let entry = self.entry_for_point(point)?;
        let bits = self.slave.read_bits(entry.kind, entry.address);
        entry.decode_bits(bits)
    }

    fn write_point(&mut self, point: &PointRef, value: &Value) -> Result<(), Error> {
        let entry = self.entry_for_point(point)?.clone();
        if !entry.access.is_writable() {
            return Err(Error::NotWritable {
                point_id: entry.point.clone(),
            });
        }
        let bits = entry.encode_value(value)?;
        self.apply_bits(&entry, bits)?;
        Ok(())
    }

    fn read_foreign(&self, foreign: &ForeignRef) -> Result<Value, Error> {
        let entry = self.entry_for_foreign(foreign)?;
        let bits = self.slave.read_bits(entry.kind, entry.address);
        entry.decode_bits(bits)
    }

    fn write_foreign(&mut self, foreign: &ForeignRef, raw: ForeignRaw) -> Result<Value, Error> {
        let entry = self.entry_for_foreign(foreign)?.clone();
        let bits = Self::raw_to_bits(&entry, raw)?;
        self.apply_bits(&entry, bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heater() -> ModbusBridge<MemoryBackend> {
        ModbusBridge::water_heater_example().unwrap()
    }

    fn point(id: &str) -> PointRef {
        PointRef::new("water-heater-plant", id).unwrap()
    }

    #[test]
    fn seeds_backend_from_initial_raw() {
        let bridge = heater();
        assert_eq!(bridge.fabric(), "modbus");
        assert_eq!(
            bridge
                .backend()
                .get_value("water-heater-plant", "trait.temperature.setpoint_c"),
            Some(&Value::F32(55.0))
        );
        assert_eq!(
            bridge
                .backend()
                .get_value("water-heater-plant", "trait.temperature.current_c"),
            Some(&Value::F32(48.0))
        );
        assert_eq!(
            bridge
                .backend()
                .get_value("water-heater-plant", "trait.power.power_state"),
            Some(&Value::Enum("on".into()))
        );
    }

    #[test]
    fn foreign_register_write_updates_homecooked_backend() {
        let mut bridge = heater();
        let foreign = ForeignRef::holding("water-heater-plant", 0).unwrap();
        let value = bridge
            .write_foreign(&foreign, ForeignRaw::Register(620))
            .unwrap();
        assert_eq!(value, Value::F32(62.0));
        assert_eq!(
            bridge
                .backend()
                .get_value("water-heater-plant", "trait.temperature.setpoint_c"),
            Some(&Value::F32(62.0))
        );
        assert_eq!(
            bridge
                .read_point(&point("trait.temperature.setpoint_c"))
                .unwrap(),
            Value::F32(62.0)
        );
    }

    #[test]
    fn foreign_coil_write_updates_power_state() {
        let mut bridge = heater();
        let foreign = ForeignRef::coil("water-heater-plant", 0).unwrap();
        let value = bridge
            .write_foreign(&foreign, ForeignRaw::Coil(false))
            .unwrap();
        assert_eq!(value, Value::Enum("off".into()));
        assert_eq!(
            bridge
                .backend()
                .get_value("water-heater-plant", "trait.power.power_state"),
            Some(&Value::Enum("off".into()))
        );
        assert!(!bridge.slave().get_coil(0));
    }

    #[test]
    fn homecooked_write_updates_register() {
        let mut bridge = heater();
        bridge
            .write_point(&point("trait.temperature.setpoint_c"), &Value::F32(44.5))
            .unwrap();
        assert_eq!(bridge.slave().get_holding(0), 445);
        assert_eq!(
            bridge
                .backend()
                .get_value("water-heater-plant", "trait.temperature.setpoint_c"),
            Some(&Value::F32(44.5))
        );
    }

    #[test]
    fn homecooked_write_updates_coil() {
        let mut bridge = heater();
        bridge
            .write_point(
                &point("trait.power.power_state"),
                &Value::Enum("off".into()),
            )
            .unwrap();
        assert!(!bridge.slave().get_coil(0));
        bridge
            .write_point(&point("trait.power.power_state"), &Value::Enum("on".into()))
            .unwrap();
        assert!(bridge.slave().get_coil(0));
    }

    #[test]
    fn current_temp_is_read_only_from_homecooked() {
        let mut bridge = heater();
        let err = bridge
            .write_point(&point("trait.temperature.current_c"), &Value::F32(50.0))
            .unwrap_err();
        assert!(matches!(err, Error::NotWritable { .. }));
        assert_eq!(bridge.slave().get_holding(1), 480);

        let foreign = ForeignRef::holding("water-heater-plant", 1).unwrap();
        bridge
            .write_foreign(&foreign, ForeignRaw::Register(512))
            .unwrap();
        assert_eq!(
            bridge
                .read_point(&point("trait.temperature.current_c"))
                .unwrap(),
            Value::F32(51.2)
        );
    }

    #[test]
    fn rejects_unmapped_and_wrong_device() {
        let mut bridge = heater();
        let other = PointRef::new("other-device", "trait.temperature.setpoint_c").unwrap();
        assert!(matches!(
            bridge.read_point(&other),
            Err(Error::DeviceMismatch { .. })
        ));
        let unknown = point("trait.energy.power_w");
        assert!(matches!(
            bridge.read_point(&unknown),
            Err(Error::UnmappedPoint { .. })
        ));
        let missing = ForeignRef::holding("water-heater-plant", 99).unwrap();
        assert!(matches!(
            bridge.write_foreign(&missing, ForeignRaw::Register(1)),
            Err(Error::UnmappedAddress { .. })
        ));
    }
}
