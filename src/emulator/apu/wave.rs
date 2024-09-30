use blip_buf::BlipBuf;

use super::channel::{Channel, Envelope, Length};

pub struct Wave {
    pub length: Length,
    clock: u32,
    wave_ram: [u8; 16],
    on: bool,
    volume: u8,
    frequency: u16,
    ampl: i32,
    wave_index: usize,
    pub from: u32,
    pub blip: BlipBuf,
}

impl Wave {
    pub fn new(blip: BlipBuf) -> Self {
        Self {
            length: Length::new(256),
            clock: 0,
            wave_ram: [0; 16],
            on: false,
            volume: 0,
            frequency: 0,
            ampl: 0,
            wave_index: 0,
            from: 0,
            blip,
        }
    }

    fn period(&self) -> u32 {
        (2048 - self.frequency as u32) * 2
    }
}

impl Channel for Wave {
    fn on(&self) -> bool {
        self.on
    }
    fn read(&self, address: u16) -> u8 {
        match address {
            // nrx0
            0xff1a => (self.on as u8) << 7,

            0xff1c => (self.volume & 0x3) << 5,

            0xff1e => 0x80 | if self.length.on { 0x40 } else { 0 } | 0x3F,

            0xff30..=0xff3f => {
                if !self.on {
                    self.wave_ram[(address as usize & 0xF)]
                } else {
                    0xFF
                }
            }

            _ => panic!("Invalid read for Wave"),
        }
    }

    fn write(&mut self, value: u8, address: u16) {
        match address {
            //nrx0
            0xff1a => {
                self.on = ((value >> 7) & 0b1) != 0;
            }

            //nrx1
            0xff1b => {
                self.length.set(value as u16);
            }

            //nrx2
            0xff1c => {
                self.volume = (value >> 5) & 0x3;
            }

            //nrx3
            0xff1d => {
                self.frequency = (self.frequency & 0xff00) | (value as u16);
            }

            //nrx4
            0xff1e => {
                self.frequency = (self.frequency & 0xff) | ((value as u16 & 0x07) << 8);

                self.length.on = (value & 0x40 == 0x40);

                self.on &= self.length.active();

                // if set
                if value & 0x80 == 0x80 {
                    self.on = true;

                    self.length.trigger();
                }
            }

            0xff30..=0xff3f => {
                if !self.on {
                    self.wave_ram[(address as usize & 0xF)] = value;
                }
            }
            _ => panic!("Invalid write for Wave"),
        }
    }

    fn step(&mut self, t_cycles: u32) {
        let volume = match self.volume {
            0 => 4,
            1 => 0,
            2 => 1,
            3 => 2,
            _ => panic!("Invalid match for Wave Volume"),
        };

        for _ in 0..(t_cycles) {
            self.clock += 1;
            if self.clock >= self.period() {
                let sample = if self.wave_index & 0x1 == 0 {
                    self.wave_ram[self.wave_index >> 1] & 0xf
                } else {
                    self.wave_ram[self.wave_index >> 1] >> 4
                };

                let ampl = if self.on {
                    (sample >> volume) as i32
                } else {
                    0x00
                };

                self.from = self.from.wrapping_add(self.clock);

                let d = ampl - self.ampl;
                self.ampl = ampl;
                self.blip.add_delta(self.from, d);

                self.wave_index = (self.wave_index + 1) % 32;
                self.clock = 0;
            }
        }
    }
}