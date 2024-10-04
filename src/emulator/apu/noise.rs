use blip_buf::BlipBuf;

use super::channel::{Channel, Envelope, Length};

pub struct Noise {
    on: bool,
    pub length: Length,
    pub envelope: Envelope,
    divisor_code: u8,
    shift: u8,
    lfsr: LFSR,
    clock: u32,
    pub from: u32,
    pub blip: BlipBuf,
    ampl: i32,
}

impl Noise {
    pub fn new(blip: BlipBuf) -> Self {
        Self {
            on: false,
            length: Length::new(64),
            envelope: Envelope::new(),
            divisor_code: 0,
            shift: 0,
            lfsr: LFSR::new(),
            clock: 0,
            from: 0,
            blip,
            ampl: 0,
        }
    }

    fn period(&self) -> u32 {
        (self.divisor() as u32) << (self.shift as u32)
    }

    fn divisor(&self) -> u32 {
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
            0xff20 => 0xff,
            0xff21 => self.envelope.read(),
            //0xff22 =>
            0xff23 => 0x80 | if self.length.on { 0x40 } else { 0 } | 0x3f,
            _ => 0xff, //panic!("Invalid read for Noise"),
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
                self.length.on = value & 0x40 == 0x40;

                if value & 0x80 == 0x80 {
                    self.on = true;
                    self.length.trigger();
                }
            }

            _ => (), //panic!("Invalid write for Noise"),
        }
    }

    fn on(&self) -> bool {
        self.on
    }

    fn step(&mut self, t_cycles: u32) {
        for _ in 0..(t_cycles) {
            if self.clock >= self.period() {
                self.on &= self.length.active();
                let ampl = if !self.on {
                    0x00
                } else if self.lfsr.step() {
                    self.envelope.volume as i32
                } else {
                    (self.envelope.volume as i32) * -1
                };

                self.from = self.from.wrapping_add(self.clock);

                let d = ampl - self.ampl;
                self.ampl = ampl;
                self.blip.add_delta(self.from, d);

                self.clock = 0;
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
