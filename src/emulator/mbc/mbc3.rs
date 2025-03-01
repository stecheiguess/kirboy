use std::time;

use crate::emulator::mbc::{ram_banks, rom_banks, MBC};

pub struct MBC3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_on: bool,
    rom_bank: usize,
    ram_bank: usize,
    rom_banks: usize,
    ram_banks: usize,

    battery: bool,

    rtc_select: bool,

    rtc: RTC,
}

impl MBC3 {
    pub fn new(data: Vec<u8>) -> Self {
        let rom_banks = rom_banks(data[0x148]);
        let ram_banks = ram_banks(data[0x149]);

        //println!("{}", ram_banks);
        let (has_rtc, battery) = match data[0x147] {
            0x0f => (true, false),
            0x10 => (true, true),
            0x13 => (false, true),
            _ => (false, false),
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

            rtc_select: false,

            rtc: RTC::new(has_rtc),
        }
    }
}

impl MBC for MBC3 {
    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_on {
            return 0xff;
        }

        if self.rtc_select {
            self.rtc.read()
        } else if self.ram_bank < self.ram_banks {
            let ram_address = (0x2000 * self.ram_bank) | ((address & 0x1fff) as usize);

            self.ram[ram_address]
        } else {
            0xff
        }
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

        if self.rtc_select {
            self.rtc.write(value);
        } else if self.ram_bank < self.ram_banks {
            let ram_address = (0x2000 * self.ram_bank) | ((address & 0x1fff) as usize);

            if ram_address < self.ram.len() {
                self.ram[ram_address] = value;
            }
        }
    }

    fn write_rom(&mut self, value: u8, address: u16) {
        match address {
            // enable ram
            0x0000..=0x1fff => {
                self.ram_on = (value & 0xf) == 0xa;
            }

            // rom bank
            0x2000..=0x3fff => {
                // mask for determining bank https://hacktix.github.io/GBEDG/mbcs/mbc1/

                if value == 0 {
                    self.rom_bank = 1;
                } else {
                    let mask = 0x7f;
                    self.rom_bank = mask & (value as usize);
                }
            }

            // ram bank
            0x4000..=0x5fff => match value {
                0x00..=0x03 => {
                    self.ram_bank = value as usize;
                    self.rtc_select = false
                }
                0x08..=0x0c => {
                    self.rtc.select(value);
                    self.rtc_select = true
                }
                _ => (),
            },

            0x6000..=0x7fff => {
                self.rtc.latch();
            }

            // mode switch {}
            _ => {}
        }
    }

    fn load_ram(&mut self, data: Vec<u8>) {
        if self.battery {
            self.rtc.load(data[0..8].to_vec());
            self.ram = data[8..].to_vec();
        }
    }

    fn save_ram(&self) -> Option<Vec<u8>> {
        if self.battery {
            Some({
                let mut t = self.rtc.save();
                t.append(&mut self.ram.clone());
                t
            })
        } else {
            None
        }
    }
}

struct RTC {
    enabled: bool,
    ram: [u8; 5],
    latch: [u8; 5],
    address: usize,
    start: u64,
}

impl RTC {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ram: [0; 5],
            latch: [0; 5],
            address: 0,
            start: 0,
        }
    }

    //loads the starting time stamp from the save file.
    pub fn load(&mut self, saved_start: Vec<u8>) {
        self.start = {
            let mut b: [u8; 8] = Default::default();
            b.copy_from_slice(&saved_start);
            u64::from_be_bytes(b)
        };
    }

    // returns the new starting time stamp for the next initialization of the save file.
    pub fn save(&self) -> Vec<u8> {
        if self.enabled {
            self.start.to_be_bytes().to_vec()
        } else {
            vec![0; 8]
        }
    }

    pub fn read(&self) -> u8 {
        self.latch[self.address]
    }

    pub fn write(&mut self, value: u8) {
        self.calculate();
        let mask = match self.address {
            0 | 1 => 0x3F,
            2 => 0x1F,
            4 => 0xC1,
            _ => 0xFF,
        };

        let new_value = mask & value;

        self.latch[self.address] = new_value;
        self.ram[self.address] = new_value;
        self.start = self.check();
    }

    pub fn select(&mut self, value: u8) {
        self.address = value as usize & 0x7;
    }

    // calculates the time elapsed into
    pub fn calculate(&mut self) {
        if self.ram[4] & 0x40 == 0x40 {
            return;
        }

        if self.check() == self.start {
            return;
        }

        let time_start = time::UNIX_EPOCH + time::Duration::from_secs(self.start);

        let difference = time::SystemTime::now()
            .duration_since(time_start)
            .unwrap()
            .as_secs();

        self.ram[0] = (difference % 60) as u8;
        self.ram[1] = ((difference / 60) % 60) as u8;
        self.ram[2] = ((difference / 3600) % 24) as u8;

        let days = difference / (3600 * 24);
        self.ram[3] = days as u8;
        self.ram[4] = (self.ram[4] & 0xfe) | (((days >> 8) & 0x01) as u8);

        if days >= 512 {
            self.ram[4] |= 0x80; // carry
            self.start = self.check()
        }
    }

    pub fn check(&self) -> u64 {
        let mut difference = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
            Ok(t) => t.as_secs(),
            Err(_) => {
                panic!("System clock is set to a time before the unix epoch (1970-01-01)")
            }
        };
        difference -= self.ram[0] as u64;
        difference -= (self.ram[1] as u64) * 60;
        difference -= (self.ram[2] as u64) * 3600;
        let days = ((self.ram[4] as u64 & 0x1) << 8) | (self.ram[3] as u64);
        difference -= days * 3600 * 24;

        return difference;
    }

    // saves the new calculated state into latch.
    pub fn latch(&mut self) {
        self.calculate();
        self.latch.clone_from_slice(&self.ram);
    }
}
