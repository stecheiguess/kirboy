mod mbc0;
mod mbc1;

const TITLE_LENGTH: usize = 11;

pub trait MBC {
    fn read_rom(&self, address: u16) -> u8;

    fn write_rom(&mut self, value: u8, address: u16);

    fn read_ram(&self, address: u16) -> u8;

    fn write_ram(&mut self, value: u8, address: u16);

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
    match mbc_type {
        0x00 => Box::new(mbc0::MBC0::new(data)),
        //0x01..=0x03 => Box::new(mbc1::MBC1::new()),
        _ => { panic!("MBC 0x{:02X} not implemented", mbc_type) }
    }
}
