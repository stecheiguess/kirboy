mod mbc0;
mod mbc1;

pub trait MBC {
    //fn new() -> Self;

    fn read_rom(&self, address: u16) -> u8;

    fn write_rom(&mut self, value: u8, address: u16);

    fn read_ram(&self, address: u16) -> u8;

    fn write_ram(&mut self, value: u8, address: u16);
}

pub fn new(mbc_type: u8) -> Box<dyn MBC> {
    match mbc_type {
        0x00 => Box::new(mbc0::MBC0::new()),
        0x01..=0x03 => Box::new(mbc1::MBC1::new()),
        _ => { panic!("hafeifhe") }
    }
}
