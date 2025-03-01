use crate::emulator::mbc::{ram_banks, rom_banks, MBC};

pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_on: bool,
    rom_bank: usize,
    ram_bank: usize,
    rom_banks: usize,
    ram_banks: usize,
    mode: bool, // 1 = advanced, 0 = simple
    battery: bool,
}

impl MBC1 {
    pub fn new(data: Vec<u8>) -> Self {
        let rom_banks = rom_banks(data[0x148]);
        let ram_banks = ram_banks(data[0x149]);

        //println!("{}", ram_banks);
        let battery = match data[0x147] {
            0x03 => true,
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
            mode: false,
            battery,
        }
    }

    // function for zero bank calculation
    fn zero_bank(&self) -> usize {
        match self.rom_banks {
            0..=32 => 0,
            64 => (self.ram_bank & 0x1) << 5,
            128 => (self.ram_bank & 0x3) << 5,
            _ => {
                panic!("invalid zero bank")
            }
        }
    }

    // function for high bank calculation
    fn high_bank(&self) -> usize {
        match self.rom_banks {
            0..=32 => (self.rom_bank & 0x1f),
            64 => (self.rom_bank & 0x1f) | ((self.ram_bank & 0x1) << 5),
            128 => (self.rom_bank & 0x1f) | ((self.ram_bank & 0x3) << 5),
            _ => {
                panic!("invalid high bank")
            }
        }
    }
}

impl MBC for MBC1 {
    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_on {
            return 0xff;
        }

        let ram_address = if self.mode {
            (0x2000 * self.ram_bank) | ((address & 0x1fff) as usize)
        } else {
            (address & 0x1fff) as usize
        };

        //println!("{}", ram_address);
        self.ram[ram_address]
    }

    fn read_rom(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => {
                if self.mode {
                    self.rom[0x4000 * self.zero_bank() + (address as usize)]
                } else {
                    self.rom[address as usize]
                }
            }
            0x4000..=0x7fff => self.rom[0x4000 * self.high_bank() + ((address as usize) & 0x3fff)],

            _ => {
                panic!("invalid read rom range")
            }
        }
    }

    fn write_ram(&mut self, value: u8, address: u16) {
        if !self.ram_on {
            return;
        }

        let ram_address = if self.mode {
            self.ram_bank * 0x2000 | address as usize & 0x1fff
        } else {
            address as usize & 0x1fff
        };

        if ram_address < self.ram.len() {
            self.ram[ram_address] = value;
        }
    }

    fn write_rom(&mut self, value: u8, address: u16) {
        match address {
            // enable ram
            0x0000..=0x1fff => {
                self.ram_on = (value & 0xf) == 0xa;
            }

            // setting rom bank
            0x2000..=0x3fff => {
                // mask for determining bank https://hacktix.github.io/GBEDG/mbcs/mbc1/

                if value == 0 {
                    self.rom_bank = 1;
                } else {
                    let mask = (self.rom_banks - 1) & 0x1f;
                    self.rom_bank = mask & (value as usize);
                }
            }

            // setting ram bank
            0x4000..=0x5fff => {
                self.ram_bank = (value & 0x11) as usize;

                // also rom bank - to determine banks past 32.
                if self.rom_banks > 32 {
                    self.rom_bank = match self.rom_banks {
                        64 => (self.rom_bank & 0x1f) | (((value as usize) & 0x1) << 5),
                        128 => (self.rom_bank & 0x1f) | (((value as usize) & 0x3) << 5),
                        _ => {
                            panic!("cannot set rom bank")
                        }
                    };
                }
            }

            0x6000..=0x7fff => {
                self.mode = (value & 0x01) == 1;
            }

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
