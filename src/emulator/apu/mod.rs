use std::sync::{Arc, Mutex};

use blip_buf::BlipBuf;
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
    pub buffer: Arc<Mutex<Vec<(f32, f32)>>>,
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
            ch1: Square::new(create_blipbuf(), true),
            ch2: Square::new(create_blipbuf(), false),
            ch3: Wave::new(create_blipbuf()),
            ch4: Noise::new(create_blipbuf()),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            //0xff24 => { self.v }
            0xff10..=0xff14 => self.ch1.read(address),
            0xff16..=0xff19 => self.ch2.read(address),
            0xff1a..=0xff1e => self.ch3.read(address),
            0xff1f..=0xff23 => self.ch4.read(address),
            0xff24 => ((self.volume_right & 0x7) << 4) | (self.volume_left & 0x7),
            0xff25 => self.panning,
            0xff26 => (self.on as u8) << 7,
            0xff30..=0xff3f => self.ch3.read(address),
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, value: u8, address: u16) {
        match address {
            0xff10..=0xff14 => self.ch1.write(value, address),
            0xff16..=0xff19 => self.ch2.write(value, address),
            0xff1a..=0xff1e => self.ch3.write(value, address),
            0xff1f..=0xff23 => self.ch4.write(value, address),
            0xff24 => {
                self.volume_left = (value >> 4) & 0x7;
                self.volume_right = value & 0x7;
            }

            0xff25 => {
                self.panning = value;
            }

            0xff26 => {
                self.on = ((value >> 7) & 0b1) != 0;
            }
            0xff30..=0xff3f => self.ch3.write(value, address),
            _ => (), //panic!("Invalid write for APU"),
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

            self.ch1.blip.end_frame(self.clock);
            self.ch2.blip.end_frame(self.clock);
            self.ch3.blip.end_frame(self.clock);
            self.ch4.blip.end_frame(self.clock);

            self.ch1.from = self.ch1.from.wrapping_sub(self.clock);
            self.ch2.from = self.ch2.from.wrapping_sub(self.clock);
            self.ch3.from = self.ch3.from.wrapping_sub(self.clock);
            self.ch4.from = self.ch4.from.wrapping_sub(self.clock);

            self.mix();
            self.clock = 0;
        }
    }

    fn play(&mut self, l: &[f32], r: &[f32]) {
        assert_eq!(l.len(), r.len());
        let mut buffer = self.buffer.lock().unwrap();
        for (l, r) in l.iter().zip(r) {
            // Do not fill the buffer with more than 1 second of data
            // This speeds up the resync after the turning on and off the speed limiter
            if buffer.len() > 44100 {
                return;
            }
            buffer.push((*l, *r));
        }
    }

    fn mix(&mut self) {
        let sc1 = self.ch1.blip.samples_avail();
        let sc2 = self.ch2.blip.samples_avail();
        let sc3 = self.ch3.blip.samples_avail();
        let sc4 = self.ch4.blip.samples_avail();
        assert_eq!(sc1, sc2);
        assert_eq!(sc2, sc3);
        assert_eq!(sc3, sc4);

        let sample_count = sc1 as usize;

        let mut sum = 0;

        let left_vol = (self.volume_left as f32 / 7.0) * (1.0 / 15.0) * 0.25;
        let right_vol = (self.volume_right as f32 / 7.0) * (1.0 / 15.0) * 0.25;

        while sum < sample_count {
            let buf_l = &mut [0f32; 2048];
            let buf_r = &mut [0f32; 2048];
            let buf = &mut [0i16; 2048];

            let count1 = self.ch1.blip.read_samples(buf, false);
            for (i, v) in buf[..count1].iter().enumerate() {
                if self.panning & 0x10 == 0x10 {
                    buf_l[i] += *v as f32 * left_vol;
                }
                if self.panning & 0x01 == 0x01 {
                    buf_r[i] += *v as f32 * right_vol;
                }
            }

            let count2 = self.ch2.blip.read_samples(buf, false);
            for (i, v) in buf[..count2].iter().enumerate() {
                if self.panning & 0x20 == 0x20 {
                    buf_l[i] += *v as f32 * left_vol;
                }
                if self.panning & 0x02 == 0x02 {
                    buf_r[i] += *v as f32 * right_vol;
                }
            }

            // channel3 is the WaveChannel, that outputs samples with a 4x
            // increase in amplitude in order to avoid a loss of precision.
            let count3 = self.ch3.blip.read_samples(buf, false);
            for (i, v) in buf[..count3].iter().enumerate() {
                if self.panning & 0x40 == 0x40 {
                    buf_l[i] += ((*v as f32) / 4.0) * left_vol;
                }
                if self.panning & 0x04 == 0x04 {
                    buf_r[i] += ((*v as f32) / 4.0) * right_vol;
                }
            }

            let count4 = self.ch4.blip.read_samples(buf, false);
            for (i, v) in buf[..count4].iter().enumerate() {
                if self.panning & 0x80 == 0x80 {
                    buf_l[i] += *v as f32 * left_vol;
                }
                if self.panning & 0x08 == 0x08 {
                    buf_r[i] += *v as f32 * right_vol;
                }
            }

            debug_assert!(count1 == count2);
            debug_assert!(count1 == count3);
            debug_assert!(count1 == count4);

            self.play(&buf_l[..count1], &buf_r[..count1]);

            sum += count1;
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

pub fn create_blipbuf() -> BlipBuf {
    let mut blipbuf = BlipBuf::new(44100);
    blipbuf.set_rates(f64::from(CPU_FREQUENCY), f64::from(44100));
    blipbuf
}
