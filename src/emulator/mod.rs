use std::{ fs::{ self, File }, io::Read, path::PathBuf };

use cpu::CPU;
use joypad::Input;
use tao::keyboard::Key;

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
    save: PathBuf,
}

impl Emulator {
    pub fn new(rom_path: PathBuf) -> Box<Emulator> {
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
            cpu: CPU::new_wb(cartridge),
            save,
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

pub fn to_joypad(key: Key) -> Option<Input> {
    match key {
        Key::Character("w") => Some(Input::Up),
        Key::Character("a") => Some(Input::Left),
        Key::Character("s") => Some(Input::Down),
        Key::Character("d") => Some(Input::Right),
        Key::Character(",") => Some(Input::B),
        Key::Character(".") => Some(Input::A),
        Key::Shift => Some(Input::Select),
        Key::Enter => Some(Input::Start),
        _ => None,
    }
}



/*pub fn to_joypad_demo(key: Key) -> Option<Input> {
    match key.as_ref() {
        Key::Character("w") => Some(Input::Up),
        Key::Character("a") => Some(Input::Left),
        Key::Character("s") => Some(Input::Down),
        Key::Character("d") => Some(Input::Right),
        Key::Character(",") => Some(Input::B),
        Key::Character(".") => Some(Input::A),
        Key::Named(NamedKey::Shift) => Some(Input::Select),
        Key::Named(NamedKey::Enter) => Some(Input::Start),
        _ => None,
    }
}*/
