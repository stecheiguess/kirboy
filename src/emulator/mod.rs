use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use cpu::CPU;
use joypad::Input;
use tao::keyboard::Key;

use crate::config::{Color, Config};

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
    key_table: HashMap<Key<'static>, Input>,
    color: Color,
}

impl Emulator {
    pub fn new(rom_path: &PathBuf, conf: Config) -> Box<Emulator> {
        let ram_path = rom_path.with_extension("sav");
        let rom: Vec<u8> = std::fs::read(rom_path).unwrap();

        let mut cartridge = mbc::new(rom);

        // load cartridge
        match std::fs::File::open(&ram_path) {
            // only if cart has ram file
            Ok(mut file) => {
                let mut data = vec![];
                match file.read_to_end(&mut data) {
                    Err(..) => panic!("Cannot Read Save File"),
                    Ok(..) => {
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
            key_table: conf.get_table(),
            color: conf.get_color(),
            //controls: conf.controls
        })
    }

    pub fn title(&self) -> String {
        self.cpu.mmu.cartridge.title()
    }

    pub fn step(&mut self) -> u8 {
        //thread::sleep(Duration::from_nanos(200));
        self.cpu.step()
    }

    pub fn updated(&mut self) -> bool {
        let updated = self.cpu.mmu.gpu.v_blank;
        self.cpu.mmu.gpu.v_blank = false;
        updated
    }
    // draws to pixel buffer.
    pub fn draw(&mut self) -> Vec<u8> {
        //self.update();
        let screen = self.cpu.mmu.gpu.buffer;

        let mut frame = Vec::new();
        for (&byte) in screen.iter() {
            let mut rgba: [u8; 4] = [0, 0, 0, 0xff];
            match byte {
                0 => rgba[..3].copy_from_slice(&self.color.id0), // white
                1 => rgba[..3].copy_from_slice(&self.color.id1), // light gray
                2 => rgba[..3].copy_from_slice(&self.color.id2), // dark gray
                3 => rgba[..3].copy_from_slice(&self.color.id3), // black

                _ => (),
            }

            frame.extend_from_slice(&rgba);
            //println!("{i:?}");
        }
        frame
    }

    pub fn key_up(&mut self, key: &Key) {
        let button = self.check_table(key);
        if button.is_some() {
            self.cpu.mmu.joypad.key_up(button.unwrap())
        }
    }

    pub fn key_down(&mut self, key: &Key) {
        let button = self.check_table(key);
        if button.is_some() {
            self.cpu.mmu.joypad.key_down(button.unwrap())
        }
    }

    fn check_table(&self, key: &Key) -> Option<Input> {
        //println!("checking key table");
        self.key_table.get(key).copied()
    }

    // changes to green just because
    pub fn green(&mut self) {
        self.color.id0 = [155, 188, 15];
        self.color.id1 = [139, 172, 15];
        self.color.id2 = [48, 98, 48];
        self.color.id3 = [15, 56, 15];
        println!("to green")
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
