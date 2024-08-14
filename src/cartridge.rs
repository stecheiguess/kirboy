use std::{ fs::File, io::{ Read, SeekFrom, Seek }, vec };

//use crate::mbc::MBC;

pub struct Cartridge {
    //mbc: dyn MBC,
}

impl Cartridge {
    pub fn new(file: &str) -> Self {
        let mut rom = File::open(file).unwrap();
        rom.seek(SeekFrom::Start(0x100));

        let mut header: [u8; 0x50] = [0; 0x50];

        rom.read_exact(&mut header);

        let mbc_type = header[0x47];

        let mbc = println!("{:02X?}", header);

        Self {}
    }
}
