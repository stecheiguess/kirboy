use super::channel::{Channel, Envelope};

pub struct Noise {
    length_timer: u8,
    envelope: Envelope,
}

impl Channel for Noise {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xff20 => 0,
            0xff21 => self.envelope.read(),
            _ => panic!("Invalid read for Noise"),
        }
    }

    fn write(&mut self, value: u8, address: u16) {
        match address {
            0xff20 => {
                self.length_timer = value & 0x3F;
            }
            0xff21 => self.envelope.write(value),

            0xff23 => {}
            _ => panic!("Invalid write for Noise"),
        }
    }

    fn on(&self) -> bool {
        true
    }

    fn step(&mut self, m_cycles: u8) {}
}
