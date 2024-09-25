use blip_buf::BlipBuf;

use super::channel::{Channel, Envelope, Length};

pub struct Noise {
    on: bool,
    length: Length,
    envelope: Envelope,
    divisor_code: u8,
    shift: u8,
    lfsr: LFSR,
    blip: BlipBuf,
    timer: usize,
}

impl Noise {
    fn period(&self) -> usize {
        (self.divisor() as usize) << (self.shift as usize)
    }

    fn divisor(&self) -> usize {
        match self.divisor_code {
            0 => 8,
            1 => 16,
            2 => 32,
            3 => 48,
            4 => 64,
            5 => 80,
            6 => 96,
            7 => 112,
            _ => panic!("Invalid divisor code"),
        }
    }
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
            //nrx1
            0xff20 => {
                self.length.set(value as u16 & 0x3f);
                //self.length = value & 0x3F;
            }

            //nrx2
            0xff21 => self.envelope.write(value),

            //nrx3
            0xff22 => {
                self.divisor_code = value & 0x7;

                self.lfsr.set(value);

                self.shift = value >> 4;
            }

            0xff23 => {
                self.length.on = (value & 0x40 == 0x40);

                if value & 0x80 == 0x80 {
                    self.on = true;
                    self.length.trigger();
                }
            }

            _ => panic!("Invalid write for Noise"),
        }
    }

    fn on(&self) -> bool {
        self.on
    }

    fn step(&mut self, m_cycles: u8) {
        for _ in 0..(m_cycles * 4) {
            if self.timer >= self.period() {
                self.timer = 0;
            }
        }
    }
}

struct LFSR {
    lfsr: u16,
    shift: u8,
}

impl LFSR {
    pub fn new() -> Self {
        Self {
            lfsr: 0x0001,
            shift: 14,
        }
    }

    pub fn set(&mut self, value: u8) {
        self.shift = if value & 0x8 == 0x8 { 6 } else { 14 };
    }

    pub fn step(&mut self) -> bool {
        let old = self.lfsr;
        self.lfsr <<= 1;
        let bit = ((old >> self.shift) ^ (self.lfsr >> (self.shift))) & 0x1;

        self.lfsr |= bit;
        (old >> self.shift) & 0x0001 != 0
    }

    pub fn trigger(&mut self) {
        self.lfsr = 0x0001;
    }
}
