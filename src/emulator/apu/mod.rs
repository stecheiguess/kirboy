use channel::Channel;
use noise::Noise;
use square::Square;
use wave::Wave;

mod channel;
mod noise;
mod square;
mod wave;

pub const CPU_FREQUENCY: u32 = 4_194_304;

//#[derive(Copy, Clone, Debug)]
pub struct APU {
    on: bool,
    sequencer: Sequencer,
    panning: u8,
    volume_left: u8,
    volume_right: u8,
    clock: u32,
    ch1: Square,
    ch2: Square,
    ch3: Wave,
    ch4: Noise,
}

impl APU {
    pub fn new() -> Self {
        Self {
            on: false,
            sequencer: Sequencer::new(),
            panning: 0,
            volume_left: 0,
            volume_right: 0,
            clock: 0,
            ch1: Square::new(true),
            ch2: Square::new(false),
            ch3: Wave::new(),
            ch4: Noise::new(),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            //0xff24 => { self.v }
            0xff25 => self.panning,
            0xff26 => (self.on as u8) << 7,
            _ => panic!("Invalid read for APU"),
        }
    }

    pub fn write(&mut self, value: u8, address: u16) {
        match address {
            0xff24 => {
                self.volume_left = (value >> 4) & 0x8;
                self.volume_right = value & 0x8;
            }

            0xff25 => {
                self.panning = value;
            }

            0xff26 => {
                self.on = ((value >> 7) & 0b1) != 0;
            }
            _ => panic!("Invalid write for APU"),
        }
    }

    pub fn step(&mut self, m_cycles: u8) {
        self.clock += (m_cycles as u32 * 4);
        if self.clock >= (CPU_FREQUENCY / 512) {
            self.ch1.step(self.clock);
            self.ch2.step(self.clock);
            self.ch3.step(self.clock);
            self.ch4.step(self.clock);

            let step = self.sequencer.step();

            match step {
                0 | 4 => {
                    // length counter step
                    self.ch1.length.step();
                    self.ch2.length.step();
                    self.ch3.length.step();
                    self.ch4.length.step();
                }

                2 | 6 => {
                    // sweep and length counter step
                    self.ch1.sweep_step();
                    self.ch1.length.step();
                    self.ch2.length.step();
                    self.ch3.length.step();
                    self.ch4.length.step();
                }

                7 => {
                    // volume envelope step
                    self.ch1.envelope.step();
                    self.ch2.envelope.step();
                    self.ch4.envelope.step();
                }
                _ => (),
            }

            self.clock = 0;
        }
    }
}

// frame sequencer
struct Sequencer {
    step: u8,
}

impl Sequencer {
    pub fn new() -> Self {
        Self { step: 0 }
    }

    pub fn step(&mut self) -> u8 {
        self.step += 1;
        if self.step >= 8 {
            self.step = 0
        }
        self.step
    }
}
