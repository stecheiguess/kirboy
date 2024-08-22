use crate::emulator::mbc::{ ram_banks, rom_banks, MBC };

pub struct MBC3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_on: bool,
    rom_bank: usize,
    ram_bank: usize,
    rom_banks: usize,
    ram_banks: usize,

    battery: bool,
    rtc: bool,

    s: u8,
    m: u8,
    h: u8,
    dl: u8,
    dh: u8,
}

impl MBC3 {
    pub fn new(data: Vec<u8>) -> Self {
        let rom_banks = rom_banks(data[0x148]);
        let ram_banks = ram_banks(data[0x149]);

        println!("{}", ram_banks);
        let (rtc, battery) = match data[0x147] {
            0x0f => (true, false),
            0x10 => (true, true),
            0x13 => (false, true),
            _ => (false, false),
        };

        Self {
            rom: data,
            ram: vec![0; ram_banks* 0x2000],
            ram_on: false,
            rom_bank: 1,
            ram_bank: 0,
            rom_banks,
            ram_banks,

            battery,
            rtc,

            s: 0,
            m: 0,
            h: 0,
            dl: 0,
            dh: 0,
        }
    }
}

impl MBC for MBC3 {
    fn read_ram(&self, address: u16) -> u8 {
        if !self.ram_on {
            return 0xff;
        }

        let ram_address = (0x2000 * self.ram_bank) | ((address & 0x1fff) as usize);

        self.ram[ram_address]
    }

    fn read_rom(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => { self.rom[address as usize] }
            0x4000..=0x7fff => { self.rom[0x4000 * self.ram_bank + ((address as usize) & 0x3fff)] }

            _ => { panic!("invalid read rom range") }
        }
    }
    fn write_ram(&mut self, value: u8, address: u16) {
        if !self.ram_on {
            return;
        }

        let ram_address = if self.mode {
            (0x2000 * self.ram_bank) | ((address & 0x1fff) as usize)
        } else {
            (address & 0x1fff) as usize
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
            0x4000..=0x5fff => {
                match value {
                    0x00..=0x03 => {
                        self.ram_bank = value;
                    }
                    _ => (),
                }
            }

            0x6000..=0x7fff => {}

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
        if self.battery { Some(self.ram.clone()) } else { None }
    }
}
