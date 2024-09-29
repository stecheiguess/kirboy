use blip_buf::BlipBuf;

use super::channel::{Channel, Envelope, Length};

pub struct Square {
    on: bool,
    duty: u8,
    frequency: u16,
    clock: u32,
    duty_step: u8,
    //blip: BlipBuf,
    ampl: i32,
    sweep: Sweep,
    pub length: Length,
    pub envelope: Envelope,
    has_sweep: bool,
}

impl Square {
    pub fn new(has_sweep: bool) -> Self {
        Self {
            on: false,
            duty: 1,
            frequency: 0,
            clock: 2048,
            duty_step: 0,
            ampl: 0,
            sweep: Sweep::new(),
            length: Length::new(64),
            envelope: Envelope::new(),
            has_sweep,
        }
    }
    fn period(&self) -> u32 {
        (2048 - self.frequency as u32) * 4
    }

    fn duty_phase(&self) -> bool {
        (self.duty >> self.duty_step) & 0x01 != 0
    }

    fn sweep_calc_frequency(&mut self) -> u16 {
        let d = self.sweep.frequency >> self.sweep.shift;

        let new_frequency = if self.sweep.direction {
            self.sweep.frequency - d
        } else {
            self.sweep.frequency + d
        };

        if new_frequency > 2047 {
            self.on = false
        };

        new_frequency
    }

    pub fn sweep_step(&mut self) {
        if self.sweep.clock > 0 {
            self.sweep.clock -= 1
        }
        if self.sweep.clock == 0 {
            if self.sweep.period > 0 {
                self.sweep.clock = self.sweep.period
            } else {
                self.sweep.clock = 8
            }
        }

        if self.sweep.on && self.sweep.period > 0 {
            let new_frequency = self.sweep_calc_frequency();

            if new_frequency <= 2047 && self.sweep.shift > 0 {
                self.frequency = new_frequency;
                self.sweep.frequency = new_frequency;

                self.sweep_calc_frequency();
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
                match value >> 6 {
                    0 => self.duty = 0x1,
                    1 => self.duty = 0x3,
                    2 => self.duty = 0xF,
                    3 => self.duty = 0xFC,
                    _ => panic!("Invalid write for Square Duty"),
                }

                self.length.set(value as u16 & 0x3F);
                //self.clock = value & 0x3F
            }

            // nrx2
            0xff12 | 0xff17 => {
                self.envelope.write(value);
            }

            // nrx3
            0xff13 | 0xff18 => {
                self.frequency = (self.frequency & 0xff00) | (value as u16);
                //self.period();
            }
            // nrx4
            0xff14 | 0xff19 => {
                self.frequency = (self.frequency & 0xff) | ((value as u16 & 0x07) << 8);

                self.length.on = (value & 0x40 == 0x40);

                self.on &= self.length.active();

                // if set
                if value & 0x80 == 0x80 {
                    self.on = true;

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
        for _ in 0..(t_cycles) {
            self.clock += 1;
            if self.clock >= self.period() {
                let ampl = if self.duty_phase() { 99 } else { 0 };

                if ampl != self.ampl {
                    //self.blip.add_delta(self.from, ampl - self.ampl);
                    self.ampl = ampl
                }

                self.duty_step = (self.duty_step + 1) % 8;
                self.clock = 0;
            }
            //self.sample = 99
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
