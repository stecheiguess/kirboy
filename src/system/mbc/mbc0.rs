use crate::system::mbc::MBC;

use super::MBCError;

pub struct MBC0 {
    rom: Vec<u8>,
}

impl MBC0 {
    pub fn new(data: Vec<u8>) -> Self {
        Self { rom: data }
    }
}

impl MBC for MBC0 {
    fn read_ram(&self, _address: u16) -> u8 {
        0
    }
    fn read_rom(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }
    fn write_ram(&mut self, _value: u8, _address: u16) {
        return;
    }
    fn write_rom(&mut self, _value: u8, _address: u16) {
        return;
    }

    fn load_ram(&mut self, _data: Vec<u8>) -> Result<(), MBCError> {
        Ok(())
    }

    fn save_ram(&self) -> Option<Vec<u8>> {
        None
    }
}
