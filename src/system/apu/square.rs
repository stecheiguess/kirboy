use blip_buf::BlipBuf;

use super::{
    channel::{Channel, Envelope, Length},
    timer::Timer,
};

pub struct Square {
    on: bool,
    dac: bool,
    duty: u8,
    frequency: u16,
    timer: Timer,
    duty_step: u8,
    //blip: BlipBuf,
    sweep: Sweep,
    pub length: Length,
    pub envelope: Envelope,
    has_sweep: bool,
    pub blip: BlipBuf,
    pub from: u32,
    ampl: i32,
}

impl Square {
    pub fn new(blip: BlipBuf, has_sweep: bool) -> Self {
        Self {
            on: false,
            dac: false,
            duty: 1,
            frequency: 0,
            timer: Timer::new(8192),
            duty_step: 0,
            sweep: Sweep::new(),
            length: Length::new(64),
            envelope: Envelope::new(),
            has_sweep,
            blip,
            from: 0,
            ampl: 0,
        }
    }
    fn period(&self) -> u32 {
        (2048 - self.frequency as u32) * 4
    }

    fn duty_phase(&self) -> bool {
        let duty = match self.duty {
            0 => 0b00000001,
            1 => 0b00000011,
            2 => 0b00001111,
            3 => 0b11111100,
            _ => panic!(),
        };
        (duty >> self.duty_step) & 0x01 != 0
    }

    fn sweep_calc_frequency(&mut self) -> u16 {
        let d = self.sweep.frequency >> self.sweep.shift;

        let new_frequency = if self.sweep.direction {
            self.sweep.frequency - d
        } else {
            self.sweep.frequency + d
        };

        if new_frequency >= 2048 {
            self.on = false
        };

        new_frequency
    }

    pub fn sweep_step(&mut self) {
        if self.sweep.clock > 1 {
            self.sweep.clock -= 1
        } else {
            if self.sweep.period > 0 {
                self.sweep.clock = self.sweep.period;

                if self.sweep.on {
                    let new_frequency = self.sweep_calc_frequency();

                    if new_frequency <= 2047 {
                        if self.sweep.shift != 0 {
                            self.frequency = new_frequency;
                            self.sweep.frequency = new_frequency;
                        }
                        self.sweep_calc_frequency();
                    }
                }
            } else {
                self.sweep.clock = 8
            }
        }
    }

    fn sweep_trigger(&mut self) {
        self.sweep.frequency = self.frequency;
        self.sweep.clock = if self.sweep.period > 0 {
            self.sweep.period
        } else {
            8
        };
        self.sweep.on = self.sweep.period > 0 || self.sweep.shift > 0;
        if self.sweep.shift > 0 {
            self.sweep_calc_frequency();
        }
    }
}

impl Channel for Square {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xff10 => self.sweep.read(),

            0xFF11 | 0xFF16 => ((self.duty & 3) << 6) | 0x3F,

            0xFF12 | 0xFF17 => self.envelope.read(),

            0xFF13 | 0xFF18 => 0xFF,

            0xFF14 | 0xFF19 => 0x80 | if self.length.on { 0x40 } else { 0 } | 0x3F,

            _ => panic!("Invalid read for Square"),
        }
    }

    fn write(&mut self, value: u8, address: u16) {
        match address {
            // nrx0
            0xff10 => {
                self.sweep.write(value);
            }
            // nrx1
            0xff11 | 0xff16 => {
                self.duty = value >> 6;

                self.length.set(value as u16 & 0x3F);
                //self.clock = value & 0x3F
            }

            // nrx2
            0xff12 | 0xff17 => {
                self.dac = value & 0xf8 != 0;
                self.envelope.write(value);
            }

            // nrx3
            0xff13 | 0xff18 => {
                self.frequency = (self.frequency & 0xff00) | (value as u16);
                self.timer.period = self.period();
                //self.period();
            }
            // nrx4
            0xff14 | 0xff19 => {
                self.frequency = (self.frequency & 0xff) | ((value as u16 & 0x07) << 8);
                self.timer.period = self.period();

                self.length.on = value & 0x40 == 0x40;

                self.on &= self.length.active();

                // if set
                if value & 0x80 == 0x80 {
                    if self.dac {
                        self.on = true;
                    }

                    self.length.trigger();

                    if self.has_sweep {
                        self.sweep_trigger();
                    }

                    self.envelope.trigger();
                }

                // self.period();

                //self.sweep_trigger();
            }
            _ => panic!("Invalid write for Square"),
        }
    }
    fn on(&self) -> bool {
        self.on
    }

    fn step(&mut self, t_cycles: u32) {
        for _ in 0..self.timer.step(t_cycles) {
            self.on &= self.length.active();
            let ampl = if !self.on {
                0x00
            } else if self.duty_phase() {
                self.envelope.volume as i32
            } else {
                (self.envelope.volume as i32) * -1
            };

            self.from = self.from.wrapping_add(self.timer.period);

            let d = ampl - self.ampl;
            self.ampl = ampl;
            self.blip.add_delta(self.from, d);

            self.duty_step = (self.duty_step + 1) % 8;
        }
    }
}

pub struct Sweep {
    pub period: u8,
    // 0 = addition, 1 = subtraction
    pub direction: bool,
    pub shift: u8,
    pub on: bool,
    pub clock: u8,
    pub frequency: u16,
}

// Frequency Sweep Module
impl Sweep {
    pub fn new() -> Self {
        Self {
            period: 0,
            direction: false,
            shift: 0,
            on: false,
            clock: 0,
            frequency: 0,
        }
    }

    pub fn read(&self) -> u8 {
        (self.period & 0x7) << 4 | (self.direction as u8) << 3 | self.shift & 0x7
    }

    pub fn write(&mut self, value: u8) {
        self.period = value >> 4;
        self.direction = value & 0x8 == 0x8;
        self.shift = value & 0x7;
    }
}
