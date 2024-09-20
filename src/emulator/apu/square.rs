use blip_buf::BlipBuf;

use super::channel::Channel;

pub struct Square {
    on: bool,
    duty: u8,
    frequency: u16,
    timer: usize,
    duty_step: u8,
    // sample: usize,
    blip: BlipBuf,
    ampl: i32,
    from: u32,
}

impl Square {
    fn period(&self) -> usize {
        (2048 - self.frequency as usize) * 4
    }

    fn duty_phase(&self) -> bool {
        (self.duty >> self.duty_step) & 0x01 != 0
    }
}

impl Channel for Square {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xff10 => 0,
            _ => panic!("Invalid read for Square"),
        }
    }

    fn write(&mut self, value: u8, address: u16) {
        match address {
            // nrx1
            0xff11 | 0xff16 => {
                match value >> 6 {
                    0 => self.duty = 0x1,
                    1 => self.duty = 0x3,
                    2 => self.duty = 0xF,
                    3 => self.duty = 0xFC,
                    _ => panic!("Invalid write for Square Duty"),
                }
                //self.timer = value & 0x3F
            }

            // nrx2

            // nrx3
            0xff13 | 0xff18 => {
                self.frequency = (self.frequency & 0xff00) | (value as u16);
                //self.period();
            }
            // nrx4
            0xff14 | 0xff19 => {
                self.frequency = (self.frequency & 0xff) | ((value as u16 & 0x07) << 8);
                self.on = value >> 7 != 0;

                // self.period();
            }
            _ => panic!("Invalid write for Square"),
        }
    }

    fn on() -> bool {
        true
    }

    fn step(&mut self, m_cycles: u8) {
        for _ in 0..(m_cycles * 4) {
            self.timer += 1;
            if self.timer >= self.period() {
                let ampl = if self.duty_phase() { 99 } else { 0 };

                if ampl != self.ampl {
                    self.from = self.from.wrapping_add(self.period() as u32);
                    self.blip.add_delta(self.from, ampl - self.ampl);
                }
                self.duty_step = (self.duty_step + 1) % 8;
                self.timer = 0;
            }
            //self.sample = 99
        }
    }
}
