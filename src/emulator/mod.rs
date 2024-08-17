use std::path::PathBuf;

use cpu::CPU;
use joypad::Input;

pub mod gpu;
pub mod joypad;
pub mod mmu;
pub mod registers;
pub mod timer;
pub mod cpu;
pub mod audio;
pub mod mbc;

pub struct Emulator {
    cpu: CPU,
    rom_path: PathBuf,
}

impl Emulator {
    pub fn new(file: PathBuf) -> Self {
        let rom_path = file.clone();
        let rom: Vec<u8> = std::fs::read(file).unwrap();

        let mut cpu = CPU::new_wb(rom);

        Self {
            cpu,
            rom_path,
        }
    }

    pub fn title(&self) -> String {
        self.cpu.mmu.cartridge.title()
    }
    // runs as many cycle counts before updating screen.
    pub fn update(&mut self) {
        let mut cycle_count: u16 = 0;
        while cycle_count < 17556 {
            cycle_count = cycle_count.wrapping_add(self.cpu.step() as u16);
        }
    }

    pub fn draw(&mut self, frame: &mut [u8]) {
        self.update();
        let screen = self.cpu.mmu.gpu.buffer;
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let rgba = match screen[i] {
                0 => [0xff, 0xff, 0xff, 0xff], // white
                1 => [0xcb, 0xcc, 0xcc, 0xff], // light gray
                2 => [0x77, 0x77, 0x77, 0xff], // dark gray
                3 => [0x00, 0x00, 0x00, 0xff], // black

                _ => [0x00, 0x00, 0x00, 0xff],
            };

            pixel.copy_from_slice(&rgba);
            //println!("{i:?}");
        }
    }

    pub fn key_up(&mut self, key: Input) {
        self.cpu.mmu.joypad.key_up(key)
    }

    pub fn key_down(&mut self, key: Input) {
        self.cpu.mmu.joypad.key_down(key)
    }

    pub fn load(&mut self, file: PathBuf) {}

    pub fn save(&self) {
        let data = self.cpu.mmu.cartridge.save_ram();
        if data.is_some() {
            // blah file save
        }
    }
}
