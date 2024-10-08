pub trait Channel {
    fn read(&self, address: u16) -> u8;

    fn write(&mut self, value: u8, address: u16);

    fn on(&self) -> bool;

    fn step(&mut self, t_cycles: u32);

    // fn trigger() ->
}

pub struct Envelope {
    pub initial_volume: u8,
    pub volume: u8,
    // 0 decrease over time, 1 increase
    pub direction: bool,
    // 0 disables.
    pub period: u8,
    pub clock: u8,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            initial_volume: 0,
            volume: 0,
            direction: false,
            period: 0,
            clock: 0,
        }
    }

    pub fn read(&self) -> u8 {
        (self.initial_volume & 0xf) << 4 | (self.direction as u8) << 3 | self.clock & 0x7
    }

    pub fn write(&mut self, value: u8) {
        self.initial_volume = value >> 4;
        self.direction = value & 0x8 == 0x8;
        self.period = value & 0x7;
        self.volume = self.initial_volume;
    }

    pub fn trigger(&mut self) {
        self.volume = self.initial_volume;
        self.clock = self.period;
    }

    pub fn step(&mut self) {
        if self.period != 0 {
            if self.clock > 0 {
                self.clock -= 1
            } else {
                self.clock = self.period;
                if (self.volume < 0xF && self.direction) || (self.volume > 0x0 && !self.direction) {
                    if self.direction {
                        self.volume += 1
                    } else {
                        self.volume -= 1
                    }
                }
            }
        }
    }
}

pub struct Length {
    pub clock: u16,
    pub on: bool,
    pub max: u16,
}

impl Length {
    pub fn new(max: u16) -> Self {
        Self {
            clock: 0,
            on: false,
            max,
        }
    }

    pub fn active(&self) -> bool {
        self.clock > 0
    }

    pub fn set(&mut self, clock: u16) {
        self.clock = self.max - clock;
    }

    pub fn step(&mut self) {
        if self.on && self.clock != 0 {
            self.clock -= 1;
            if self.clock == 0 {
                self.on = false
            }
        }
    }

    pub fn trigger(&mut self) {
        if self.clock == 0 {
            self.clock = self.max;
        }
    }
}
