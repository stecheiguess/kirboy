use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{
    fs::{self},
    io::Read,
    path::PathBuf,
};
use std::{thread, time};

use cpu::CPU;
use joypad::Input;

pub mod apu;
pub mod cpu;
pub mod gpu;
pub mod joypad;
pub mod mbc;
pub mod mmu;
pub mod registers;
pub mod timer;

pub const CLOCK_FREQUENCY: u32 = 4_194_304;
pub const STEP_TIME: u32 = 16;
pub const STEP_CYCLES: u32 = (STEP_TIME as f64 / (1000_f64 / CLOCK_FREQUENCY as f64)) as u32;

pub struct Emulator {
    cpu: CPU,
    save: PathBuf,
    clock: u32,
    now: Instant,
    speed: u32,
}

impl Emulator {
    pub fn new(rom_path: &PathBuf) -> Box<Emulator> {
        let ram_path = rom_path.with_extension("sav");
        let rom: Vec<u8> = std::fs::read(rom_path).unwrap();

        let mut cartridge = mbc::new(rom);

        // load cartridge
        match std::fs::File::open(&ram_path) {
            // only if cart has ram file
            Ok(mut file) => {
                let mut data = vec![];
                match file.read_to_end(&mut data) {
                    Err(_) => panic!("Cannot Read Save File"),
                    Ok(_) => {
                        cartridge.load_ram(data);
                    }
                }
            }
            Err(..) => {}
        }

        let save = ram_path.clone();

        Box::new(Emulator {
            cpu: CPU::new(cartridge, false),
            save,
            clock: 0,
            now: Instant::now(), //controls: conf.controls
            speed: 1,
        })
    }

    pub fn title(&self) -> String {
        self.cpu.mmu.cartridge.title()
    }

    pub fn step(&mut self) -> u8 {
        // makes the emulator run at proper speed
        if self.clock > (STEP_CYCLES * self.speed) {
            self.clock -= STEP_CYCLES * self.speed;
            let now = time::Instant::now();
            let d = now.duration_since(self.now);
            let s = STEP_TIME.saturating_sub(d.as_millis() as u32) as u64;
            thread::sleep(time::Duration::from_millis(s));
            self.now = self
                .now
                .checked_add(time::Duration::from_millis(u64::from(STEP_TIME)))
                .unwrap();

            // If now is after the just updated target frame time, reset to
            // avoid drift.
            if now.checked_duration_since(self.now).is_some() {
                self.now = now;
            }
        }
        let cycles = self.cpu.step() * 4;
        self.clock += cycles as u32;
        cycles
    }

    pub fn updated(&mut self) -> bool {
        let updated = self.cpu.mmu.gpu.v_blank;
        self.cpu.mmu.gpu.v_blank = false;
        updated
    }

    pub fn screen(&self) -> Vec<u8> {
        self.cpu.mmu.gpu.buffer.to_vec()
    }
    // draws to pixel buffer.
    /*pub fn draw(&mut self, config: &Config) -> Vec<u8> {
        //self.update();
        let screen = self.cpu.mmu.gpu.buffer;

        let mut frame = Vec::new();
        for (&byte) in screen.iter() {
            let mut rgba: [u8; 4] = [0, 0, 0, 0xff];
            match byte {
                0 => rgba[..3].copy_from_slice(&config.color.id0), // white
                1 => rgba[..3].copy_from_slice(&config.color.id1), // light gray
                2 => rgba[..3].copy_from_slice(&config.color.id2), // dark gray
                3 => rgba[..3].copy_from_slice(&config.color.id3), // black

                _ => (),
            }

            frame.extend_from_slice(&rgba);
            //println!("{i:?}");
        }
        frame
    }*/

    pub fn key_up(&mut self, key: Option<Input>) {
        if key.is_some() {
            self.cpu.mmu.joypad.key_up(key.unwrap())
        }
    }

    pub fn key_down(&mut self, key: Option<Input>) {
        if key.is_some() {
            self.cpu.mmu.joypad.key_down(key.unwrap())
        }
    }

    /*fn check_table(&self, key: &Key) -> Option<Input> {
        //println!("checking key table");
        self.key_table.get(key).copied()
    }*/

    // changes to green just because
    /*pub fn green(&mut self) {
        self.color.id0 = [155, 188, 15];
        self.color.id1 = [139, 172, 15];
        self.color.id2 = [48, 98, 48];
        self.color.id3 = [15, 56, 15];
        println!("to green")
    }*/

    pub fn audio_buffer(&self) -> Arc<Mutex<Vec<(f32, f32)>>> {
        self.cpu.mmu.apu.buffer.clone()
    }

    pub fn save(&mut self) {
        let data = self.cpu.mmu.cartridge.save_ram();

        if data.is_some() {
            // blah file save
            let _ = fs::write(&self.save, data.unwrap());
        }
        println!("Saved");
    }
}

// dumps save when exit.
impl Drop for Emulator {
    fn drop(&mut self) {
        self.save();
    }
}
