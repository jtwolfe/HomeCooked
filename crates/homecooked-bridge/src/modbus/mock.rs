//! In-memory Modbus slave: holding/input registers and coils/discretes.

use std::collections::HashMap;

use super::map::{ForeignBits, ModbusMap, RegisterKind};
use crate::error::Error;

/// In-memory Modbus slave (register/coil store; TCP lab frames over this).
#[derive(Debug, Clone, PartialEq)]
pub struct ModbusSlave {
    pub slave_id: u8,
    holding: HashMap<u16, u16>,
    input: HashMap<u16, u16>,
    coils: HashMap<u16, bool>,
    discrete: HashMap<u16, bool>,
}

impl ModbusSlave {
    pub fn new(slave_id: u8) -> Self {
        Self {
            slave_id,
            holding: HashMap::new(),
            input: HashMap::new(),
            coils: HashMap::new(),
            discrete: HashMap::new(),
        }
    }

    /// Allocate addresses from a map and apply `initial_raw` seeds.
    pub fn from_map(map: &ModbusMap) -> Self {
        let mut slave = Self::new(map.slave_id);
        for entry in &map.entries {
            let raw = entry.initial_raw.unwrap_or(0);
            match entry.kind {
                RegisterKind::Holding => {
                    slave.holding.insert(entry.address, raw);
                }
                RegisterKind::Input => {
                    slave.input.insert(entry.address, raw);
                }
                RegisterKind::Coil => {
                    slave.coils.insert(entry.address, raw != 0);
                }
                RegisterKind::Discrete => {
                    slave.discrete.insert(entry.address, raw != 0);
                }
            }
        }
        slave
    }

    pub fn get_holding(&self, address: u16) -> u16 {
        self.holding.get(&address).copied().unwrap_or(0)
    }

    pub fn set_holding(&mut self, address: u16, value: u16) {
        self.holding.insert(address, value);
    }

    pub fn get_input(&self, address: u16) -> u16 {
        self.input.get(&address).copied().unwrap_or(0)
    }

    pub fn set_input(&mut self, address: u16, value: u16) {
        self.input.insert(address, value);
    }

    pub fn get_coil(&self, address: u16) -> bool {
        self.coils.get(&address).copied().unwrap_or(false)
    }

    pub fn set_coil(&mut self, address: u16, value: bool) {
        self.coils.insert(address, value);
    }

    pub fn get_discrete(&self, address: u16) -> bool {
        self.discrete.get(&address).copied().unwrap_or(false)
    }

    pub fn set_discrete(&mut self, address: u16, value: bool) {
        self.discrete.insert(address, value);
    }

    pub fn read_bits(&self, kind: RegisterKind, address: u16) -> ForeignBits {
        match kind {
            RegisterKind::Holding => ForeignBits::Register(self.get_holding(address)),
            RegisterKind::Input => ForeignBits::Register(self.get_input(address)),
            RegisterKind::Coil => ForeignBits::Coil(self.get_coil(address)),
            RegisterKind::Discrete => ForeignBits::Coil(self.get_discrete(address)),
        }
    }

    pub fn write_bits(
        &mut self,
        kind: RegisterKind,
        address: u16,
        bits: ForeignBits,
    ) -> Result<(), Error> {
        match (kind, bits) {
            (RegisterKind::Holding, ForeignBits::Register(v)) => self.set_holding(address, v),
            (RegisterKind::Input, ForeignBits::Register(v)) => self.set_input(address, v),
            (RegisterKind::Coil, ForeignBits::Coil(v)) => self.set_coil(address, v),
            (RegisterKind::Discrete, ForeignBits::Coil(v)) => self.set_discrete(address, v),
            (kind, ForeignBits::Register(_)) => {
                return Err(Error::InvalidRaw {
                    detail: format!("register payload is not valid for {kind}"),
                });
            }
            (kind, ForeignBits::Coil(_)) => {
                return Err(Error::InvalidRaw {
                    detail: format!("coil payload is not valid for {kind}"),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modbus::ModbusMap;

    #[test]
    fn seeds_from_water_heater_map() {
        let map = ModbusMap::water_heater_example().unwrap();
        let slave = ModbusSlave::from_map(&map);
        assert_eq!(slave.slave_id, 1);
        assert_eq!(slave.get_holding(0), 550);
        assert_eq!(slave.get_holding(1), 480);
        assert!(slave.get_coil(0));
        assert_eq!(slave.get_holding(99), 0);
    }
}
