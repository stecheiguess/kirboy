use super::channel::{Channel, Envelope, Length};

pub struct Wave {
    length: Length,
    envelope: Envelope,
    wave_ram: [u8; 32],
    on: bool,
}

impl Wave {
    pub fn new() {}
}

impl Channel for Wave {
    fn on(&self) -> bool {
        self.on
    }
    fn read(&self, address: u16) -> u8 {
        match address {
            0xff30..=0xff3f => {
                if !self.on {
                    self.wave_ram[(address as usize & 0xF) << 1] << 4
                        | 0x7 & self.wave_ram[((address as usize & 0xF) << 1) + 1]
                } else {
                    0xFF
                }
            }

            _ => panic!("Invalid read for Wave"),
        }
    }

    fn write(&mut self, value: u8, address: u16) {
        match address {
            0xff1b => {
                self.length.set(value as u16);
            }
            0xff30..=0xff3f => {
                if !self.on {
                    self.wave_ram[(address as usize & 0xF) << 1] = value >> 4;
                    self.wave_ram[((address as usize & 0xF) << 1) + 1] = value & 0xF;
                }
            }
            _ => panic!("Invalid write for Wave"),
        }
    }

    fn step(&mut self, m_cycles: u8) {}
}
