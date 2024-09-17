use super::channel::Channel;

enum Duty {
    A = 0x1,
    B = 0x3,
    C = 0xF,
    D = 0xFC,
}
pub struct Square {
    duty: Duty,
    frequency: u16,
    period: usize,
}

impl Square {
    fn period(&mut self) {
        self.period = (2048 - self.frequency as usize) * 4
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
                    0 => self.duty = Duty::A,
                    1 => self.duty = Duty::B,
                    2 => self.duty = Duty::C,
                    3 => self.duty = Duty::D,
                    _ => panic!("Invalid write for Square Duty"),
                }
                //self.timer = value & 0x3F
            }

            // nrx2

            // nrx3
            0xff13 | 0xff18 => {
                self.frequency = (self.frequency & 0xff00) | (value as u16);
                self.period();
            }
            // nrx4
            0xff14 | 0xff19 => {
                self.frequency = (self.frequency & 0xff) | ((value as u16 & 0x07) << 8);
                self.period();
            }
            _ => panic!("Invalid write for Square"),
        }
    }

    fn on() -> bool {
        true
    }

    fn step(&mut self) {}
}
