use crate::gpu::GPU;
pub struct MMU {
    //gpu: GPU,
    pub ram: [u8; 0x10000],
}

impl MMU {
    pub fn new() -> MMU {
        MMU { ram: [0; 0x10000] }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    pub fn write_byte(&mut self, value: u8, address: u16) {
        self.ram[address as usize] = value;
        /*match address {
            0x8000..=0x9fff => {}
        }*/
    }

    pub fn read_word(&self, address: u16) -> u16 {
        // little endian order of bits
        (self.read_byte(address) as u16) | ((self.read_byte(address + 1) as u16) << 8)
    }

    pub fn write_word(&mut self, value: u16, address: u16) {
        // write in little endian
        self.write_byte((value & 0x00ff) as u8, address);
        self.write_byte((value >> 8) as u8, address + 1);
    }
}
