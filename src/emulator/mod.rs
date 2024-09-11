use std::{ fs::{ self, File }, io::Read, path::PathBuf };
use std::collections::HashMap;

use cpu::CPU;
use joypad::Input;
use tao::keyboard::Key;

use crate::config::{ Config, Color };

pub mod gpu;
pub mod joypad;
pub mod mmu;
pub mod registers;
pub mod timer;
pub mod cpu;
pub mod mbc;
pub mod apu;

pub struct Emulator {
    cpu: CPU,
    save: PathBuf,
    key_table: HashMap<Key<'static>, Input>,
    color: Color,
}

impl Emulator {
    pub fn new(rom_path: &PathBuf, conf: &Config) -> Box<Emulator> {
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
    // runs as many cycle counts before updating screen.
    fn update(&mut self) {
        let mut cycle_count: u16 = 0;
        while cycle_count < 17556 {
            cycle_count = cycle_count.wrapping_add(self.cpu.step() as u16);
        }
    }

    // draws to pixel buffer.
    pub fn draw(&mut self, frame: &mut [u8]) {
        self.update();
        let screen = self.cpu.mmu.gpu.buffer;

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let mut rgba: [u8; 4] = [0, 0, 0, 0xff];
            match screen[i] {
                0 => rgba[..3].copy_from_slice(&self.color.id0), // white
                1 => rgba[..3].copy_from_slice(&self.color.id1), // light gray
                2 => rgba[..3].copy_from_slice(&self.color.id2), // dark gray
                3 => rgba[..3].copy_from_slice(&self.color.id3), // black

                _ => (),
            }

            pixel.copy_from_slice(&rgba);
            //println!("{i:?}");
        }
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
}

// dumps save when exit.
impl Drop for Emulator {
    fn drop(&mut self) {
        let data = self.cpu.mmu.cartridge.save_ram();

        if data.is_some() {
            // blah file save
            let _ = fs::write(&self.save, data.unwrap());
        }
        println!("Saved");
    }
}
