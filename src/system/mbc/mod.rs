mod mbc0;
mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;

const TITLE_LENGTH: usize = 11;

// MBC class that provides the abstract functions for MBC1, MBC2, 3 and 5.
pub trait MBC: Send {
    fn read_rom(&self, address: u16) -> u8;

    fn write_rom(&mut self, value: u8, address: u16);

    fn read_ram(&self, address: u16) -> u8;

    fn write_ram(&mut self, value: u8, address: u16);

    // loads the RAM buffer into the MBC class.
    fn load_ram(&mut self, data: Vec<u8>);

    // retrieves the RAM buffer, if battery is true. Else, it just returns None.
    fn save_ram(&self) -> Option<Vec<u8>>;

    // retrieves the title from the cartridge itself.
    fn title(&self) -> String {
        let mut title = String::with_capacity(TITLE_LENGTH);

        for i in 0..TITLE_LENGTH {
            let char = self.read_rom(0x134 + (i as u16));
            if char == 0 {
                break;
            }
            title.push(char as char);
        }

        title
    }
}

pub fn new(data: Vec<u8>) -> Box<dyn MBC> {
    let mbc_type = data[0x147];

    // ensures that all cartridges are compatible with DMG, instead of CGB.
    if data[0x143] == 0xC0 {
        panic!("This cartridge is only compatible with CGB.")
    }

    // prints the name of the cartridge type into the console log. Also additional check to see if type is valid.
    name(mbc_type);

    // matches the MBC type with the corresponding class.
    match mbc_type {
        0x00 => Box::new(mbc0::MBC0::new(data)),
        0x01..=0x03 => Box::new(mbc1::MBC1::new(data)),
        0x05..=0x06 => Box::new(mbc2::MBC2::new(data)),
        0x0f..=0x13 => Box::new(mbc3::MBC3::new(data)),
        0x19..=0x1e => Box::new(mbc5::MBC5::new(data)),
        _ => {
            panic!("MBC 0x{:02X} not implemented.", mbc_type)
        }
    }
}

pub fn rom_banks(value: u8) -> usize {
    // problem abstraction of the ROM Banks calculation by utilizing bit wise shifts.
    match value {
        0..=8 => 2 << value,
        _ => 0,
    }
}

pub fn ram_banks(value: u8) -> usize {
    match value {
        2 => 1,
        3 => 4,
        4 => 16,
        5 => 8,
        _ => 0,
    }
}

pub fn name(mbc_type: u8) {
    let name = match mbc_type {
        0x00 => "ROM ONLY",
        0x01 => "MBC1",
        0x02 => "MBC1+RAM",
        0x03 => "MBC1+RAM+BATTERY",
        0x05 => "MBC2",
        0x06 => "MBC2+BATTERY",
        0x08 => "ROM+RAM",
        0x09 => "ROM+RAM+BATTERY",
        0x0b => "MMM01",
        0x0c => "MMM01+RAM",
        0x0d => "MMM01+RAM+BATTERY",
        0x0f => "MBC3+TIMER+BATTERY",
        0x10 => "MBC3+TIMER+RAM+BATTERY",
        0x11 => "MBC3",
        0x12 => "MBC3+RAM",
        0x13 => "MBC3+RAM+BATTERY",
        0x19 => "MBC5",
        0x1a => "MBC5+RAM",
        0x1b => "MBC5+RAM+BATTERY",
        0x1c => "MBC5+RUMBLE",
        0x1d => "MBC5+RUMBLE+RAM",
        0x1e => "MBC5+RUMBLE+RAM+BATTERY",
        0x20 => "MBC6",
        0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
        0xfc => "POCKET CAMERA",
        0xfd => "BANDAI TAMA5",
        0xfe => "HuC3",
        0xff => "HuC1+RAM+BATTERY",
        _ => panic!("MBC does not exist."),
    };

    println!("{:02X}: {}", mbc_type, name);
}
