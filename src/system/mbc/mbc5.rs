use crate::system::mbc::{ram_banks, rom_banks, MBC};

pub struct MBC5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize,
    ram_on: bool,
    rom_banks: usize,
    ram_banks: usize,
    battery: bool,
}

impl MBC5 {
    pub fn new(data: Vec<u8>) -> Self {
        let rom_banks = rom_banks(data[0x148]);
        let ram_banks = ram_banks(data[0x149]);

        //println!("{}", ram_banks);
        let battery = match data[0x147] {
            0x1b => true,
            0x1e => true,
            _ => false,
        };

        Self {
            rom: data,
            ram: vec![0; ram_banks * 0x2000],
            ram_on: false,
            rom_bank: 1,
            ram_bank: 0,
            rom_banks,
            ram_banks,
            battery,
        }
    }
}

impl MBC for MBC5 {
    fn write_rom(&mut self, value: u8, address: u16) {
        match address {
            0x0000..=0x1fff => {
                self.ram_on = (value & 0xf) == 0xa;
            }
            0x2000..=0x2fff => {
                self.rom_bank = ((self.rom_bank | 0xff) & (value as usize)) % self.rom_banks;
            }
            0x3000..=0x3fff => {
                self.rom_bank =
                    (self.rom_bank & (((value as usize) & (0x1 << 7)) | 0xff)) % self.rom_banks;
            }
            0x4000..=0x5fff => {
                self.ram_bank = ((value & 0x0f) as usize) % self.ram_banks;
            }
            _ => (),
        }
    }

    fn write_ram(&mut self, value: u8, address: u16) {
        match address {
            0xa000..=0xbfff => {
                if self.ram_on {
                    self.ram[(0x2000 * self.ram_bank) | ((address & 0x1fff) as usize)] = value;
                }
            }
            _ => panic!("Invalid RAM range"),
        }
    }

    fn read_rom(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => self.rom[address as usize],
            0x4000..=0x7fff => self.rom[(0x4000 * self.rom_bank) | ((address & 0x3fff) as usize)],
            _ => panic!("Invalid ROM range"),
        }
    }

    fn read_ram(&self, address: u16) -> u8 {
        match address {
            0xa000..=0xbfff => {
                if self.ram_on {
                    self.ram[(0x2000 * self.ram_bank) | ((address & 0x1fff) as usize)]
                } else {
                    0xff
                }
            }
            _ => panic!("Invalid RAM range"),
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
