use super::MBC;

pub struct MBC0 {
    rom: [u8; 0x8000],
}

impl MBC0 {
    pub fn new() -> Self {
        Self {
            rom: [0; 0x8000],
        }
    }
}

impl MBC for MBC0 {
    fn read_ram(&self, address: u16) -> u8 {
        0
    }

    fn read_rom(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }
    fn write_ram(&mut self, value: u8, address: u16) {
        return;
    }
    fn write_rom(&mut self, value: u8, address: u16) {
        return;
    }
}
