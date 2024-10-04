use crate::emulator::mbc::MBC;

pub struct MBC2 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_on: bool,
    rom_bank: usize,

    battery: bool,
}

impl MBC2 {
    pub fn new(data: Vec<u8>) -> Self {
        let battery = match data[0x147] {
            0x06 => true,
            _ => false,
        };

        Self {
            rom: data,
            ram: vec![0; 512],
            ram_on: false,
            rom_bank: 1,
            battery,
        }
    }
}

impl MBC for MBC2 {
    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_on {
            return 0xff;
        }

        self.ram[(address as usize) & 0x1ff]
    }
    fn read_rom(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => self.rom[address as usize],
            0x4000..=0x7fff => self.rom[0x4000 * self.rom_bank + ((address as usize) & 0x3fff)],

            _ => {
                panic!("invalid read rom range")
            }
        }
    }
    fn write_ram(&mut self, value: u8, address: u16) {
        if !self.ram_on {
            return;
        }

        self.ram[(address as usize) & 0x1ff] = value | 0xf0;
    }

    fn write_rom(&mut self, value: u8, address: u16) {
        match address {
            // enable ram
            0x0000..=0x3fff => {
                if (address & 0x100) == 0 {
                    self.ram_on = (value & 0xf) == 0xa;
                } else {
                    if value == 0 {
                        self.rom_bank = 1;
                    } else {
                        self.rom_bank = value as usize;
                    }
                }
            }

            // mode switch {}
            _ => {}
        }
    }

    fn load_ram(&mut self, data: Vec<u8>) {
        if self.battery {
            self.ram = data;
        }
    }

    fn save_ram(&self) -> Option<Vec<u8>> {
        if self.battery {
            Some(self.ram.clone())
        } else {
            None
        }
    }
}
