use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{
    fs::{self},
    io::Read,
    path::PathBuf,
};
use std::{thread, time};

use dirs::config_local_dir;

use crate::circular::Circular;
use crate::system::cpu::{CPUState, CPU};
use crate::system::joypad::Input;
use crate::system::mbc::{self, MBCError};

pub const CLOCK_FREQUENCY: u32 = 4_194_304;
pub const STEP_TIME: u32 = 12;
pub const STEP_CYCLES: u32 = (STEP_TIME as f64 / (1000_f64 / CLOCK_FREQUENCY as f64)) as u32;

pub enum EmulatorError {
    InvalidFileExtension,
    InvalidSave,
    InvalidType(u8),
    InvalidCGB,
}
pub struct Emulator {
    cpu: CPU,
    save: PathBuf,
    clock: u32,
    now: Instant,
    state_buffer: Circular<CPUState>,
}

impl Emulator {
    pub fn new(rom_path: &PathBuf) -> Result<Box<Emulator>, EmulatorError> {
        if rom_path.extension().unwrap().to_str().unwrap() != "gb" {
            return Err(EmulatorError::InvalidFileExtension);
        }
        let ram_path = rom_path.with_extension("sav");
        let rom: Vec<u8> = std::fs::read(rom_path).unwrap();

        let cartridge = match mbc::new(rom) {
            Ok(c) => c,
            Err(MBCError::CGB) => return Err(EmulatorError::InvalidCGB),
            Err(MBCError::MBCType(t)) => return Err(EmulatorError::InvalidType(t)),
            Err(MBCError::RAMLength) => return Err(EmulatorError::InvalidSave),
        };

        let save = ram_path.clone();

        Ok(Box::new(Emulator {
            cpu: CPU::new(cartridge),
            save,
            clock: 0,
            now: Instant::now(),
            state_buffer: Circular::new(500),
        }))
    }

    pub fn load_save(&mut self, rom_path: &PathBuf) -> Result<(), EmulatorError> {
        let ram_path = rom_path.with_extension("sav");

        match std::fs::File::open(&ram_path) {
            // only if cart has ram file
            Ok(mut file) => {
                let mut data = vec![];
                match file.read_to_end(&mut data) {
                    Err(_) => panic!("Cannot Read Save File"),
                    Ok(_) => match self.cpu.mmu.cartridge.load_ram(data) {
                        Ok(_) => Ok(()),
                        Err(_) => {
                            println!("HIHIHIHIHIHIH");
                            return Err(EmulatorError::InvalidSave);
                        }
                    },
                }
            }
            Err(..) => Ok(()),
        }
    }

    pub fn title(&self) -> String {
        self.cpu.mmu.cartridge.title()
    }

    pub fn step(&mut self) -> CPUState {
        // makes the emulator run at proper speed
        if self.clock > (STEP_CYCLES) {
            self.clock -= STEP_CYCLES;
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

        let cpu_state = self.cpu.step();

        self.state_buffer.push(cpu_state);

        let t_cycles = cpu_state.timing * 4;
        self.clock += t_cycles as u32;

        cpu_state
    }

    pub fn screen_updated(&mut self) -> bool {
        let updated = self.cpu.mmu.ppu.v_blank;
        self.cpu.mmu.ppu.v_blank = false;
        updated
    }

    pub fn screen(&self) -> Vec<u8> {
        self.cpu.mmu.ppu.buffer.to_vec()
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

    pub fn audio(&self) -> Arc<Mutex<Vec<(f32, f32)>>> {
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

    // sets the internal APU sample rate.
    pub fn sample(&mut self, sample: u32) {
        self.cpu.mmu.apu.sample(sample);
    }
}

// dumps save when exit.
impl Drop for Emulator {
    fn drop(&mut self) {
        let mut log_path = config_local_dir().unwrap();
        log_path.push("kirboy/log.txt");

        fs::write(&log_path, "");

        let mut file = OpenOptions::new().append(true).open(&log_path).unwrap();

        // Iterates through the circular queue and write each String to the file
        for i in self.state_buffer.iter() {
            file.write(i.display().as_bytes()).unwrap();
            file.write(b"\n").unwrap(); // Add newline after each string
        }

        //println!("{:?}", self.state_buffer);
        self.save();
    }
}
