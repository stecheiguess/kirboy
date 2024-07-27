use crate::{ gpu::GPU, joypad::Joypad };
pub struct MMU {
    //gpu: GPU,
    //ram: [u8; 0x10000],
    rom0: [u8; 0x4000],
    rom1: [u8; 0x4000],
    xram: [u8; 0x2000],
    //gpu: GPU,
    joypad: Joypad,
    wram: [u8; 0x2000],
    inte: u8,
    intf: u8,
    hram: [u8; 0x007f],
}

impl MMU {
    pub fn new() -> MMU {
        MMU {
            rom0: [0; 0x4000],
            rom1: [0; 0x4000],
            xram: [0; 0x2000],
            //gpu: GPU,
            joypad: Joypad::new(),
            wram: [0; 0x2000],
            inte: 0,
            intf: 0,
            hram: [0; 0x007f],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => { self.rom0[address as usize] }
            0xff40 => { self.joypad.read() }
            other => {
                println!("Address 0x{:02X} not yet implemented.", other);
                0
            }
        }
    }

    pub fn write_byte(&mut self, value: u8, address: u16) {
        match address {
            0x0000..=0x3fff => {
                self.rom0[address as usize] = value;
            }
            0xff40 => { self.joypad.write(value) }
            other => {
                println!("Address 0x{:02X} not yet implemented.", other);
            }
        }
        //self.ram[address as usize] = value;
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
