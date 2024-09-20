pub trait Channel {
    fn read(&self, address: u16) -> u8;

    fn write(&mut self, value: u8, address: u16);

    fn on() -> bool;

    fn step(&mut self, m_cycles: u8);

    // fn trigger() ->
}

pub struct Envelope {
    pub volume: u8,
    // 0 decrease over time, 1 increase
    pub direction: bool,
    // 0 disables.
    pub sweep_pace: u8,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            volume: 0,
            direction: false,
            sweep_pace: 0,
        }
    }

    pub fn read(&self) -> u8 {
        (self.volume & 0xf) << 4 | (self.direction as u8) << 3 | self.sweep_pace & 0x7
    }

    pub fn write(&mut self, value: u8) {
        self.volume = value >> 4;
        self.direction = value & 0x8 == 0x8;
        self.sweep_pace = value & 0x7;
    }
}

pub struct LengthCounter {
    counter: u16,
}

impl LengthCounter {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    fn step(&mut self) {
        self.counter -= 1;
        if self.counter == 0 {}
    }
}
