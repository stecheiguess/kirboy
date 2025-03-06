use core::panic;

pub struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    enabled: bool,
    clock_select: u8,
    pub interrupt: bool,
    clock: u16,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            enabled: false,
            clock_select: 0,
            interrupt: false,
            clock: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xff04 => ((self.div & 0xff00) >> 8) as u8,
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => ((self.enabled as u8) << 2) | self.clock_select,
            _ => {
                panic!("Invalid read for Timer")
            }
        }
    }

    pub fn write(&mut self, value: u8, address: u16) {
        match address {
            0xff04 => {
                self.div = 0;
            }
            0xff05 => {
                self.tima = value;
            }
            0xff06 => {
                self.tma = value;
            }
            0xff07 => {
                self.enabled = (value >> 2) != 0;
                self.clock_select = value & 0x03;
            }
            _ => {
                panic!("Invalid write for Timer")
            }
        }
    }

    pub fn step(&mut self, m_cycles: u8) {
        // TO DO - Div register reset on overflow

        //println!("{}", self.div);
        self.div = self.div.wrapping_add((m_cycles * 4) as u16);

        if self.enabled {
            self.clock = self.clock.wrapping_add(m_cycles as u16);
            let freq = match self.clock_select {
                0 => 256,
                1 => 4,
                2 => 16,
                3 => 64,
                _ => {
                    panic!("invalid")
                }
            };

            while self.clock >= freq {
                let (value, did_overflow) = self.tima.overflowing_add(1);

                if did_overflow {
                    self.interrupt = true;
                    self.tima = self.tma;
                } else {
                    self.tima = value;
                }
                self.clock -= freq;
            }
        }
    }
}
