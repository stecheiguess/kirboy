pub struct Timer {
    pub period: u32,
    pub n: u32,
}

impl Timer {
    pub fn new(period: u32) -> Self {
        Self { period, n: 0x00 }
    }

    pub fn step(&mut self, cycles: u32) -> u32 {
        self.n += cycles;
        let rs = self.n / self.period;
        self.n = self.n % self.period;
        //println!("{}", rs);
        rs
    }
}
