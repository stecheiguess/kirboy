use crate::emulator::{apu::APU, joypad::Joypad, mbc::MBC, ppu::PPU, timer::Timer};

pub struct MMU {
    pub gpu: PPU,
    pub joypad: Joypad,
    pub inte: u8,
    pub intf: u8,
    ram: [u8; 0x10000],
    pub timer: Timer,
    pub cartridge: Box<dyn MBC>,
    pub apu: APU,
}

impl MMU {
    pub fn new(cartridge: Box<dyn MBC>) -> Self {
        Self {
            gpu: PPU::new(),
            joypad: Joypad::new(),
            timer: Timer::new(),
            inte: 0,
            intf: 0,
            ram: [0; 0x10000],
            cartridge,
            apu: APU::new(),
        }
    }

    // initializes gameboy state without needing boot rom.
    pub fn init(cartridge: Box<dyn MBC>) -> Self {
        let mut mmu = MMU::new(cartridge);
        mmu.write_byte(0x80, 0xff10);
        mmu.write_byte(0xbf, 0xff11);
        mmu.write_byte(0xf3, 0xff12);
        mmu.write_byte(0xbf, 0xff14);
        mmu.write_byte(0x3f, 0xff16);
        mmu.write_byte(0xbf, 0xff19);
        mmu.write_byte(0x7f, 0xff1a);
        mmu.write_byte(0xff, 0xff1b);
        mmu.write_byte(0x9f, 0xff1c);
        mmu.write_byte(0xff, 0xff1e);
        mmu.write_byte(0xff, 0xff20);
        mmu.write_byte(0xbf, 0xff23);
        mmu.write_byte(0x77, 0xff24);
        mmu.write_byte(0xf3, 0xff25);
        mmu.write_byte(0xf1, 0xff26);
        mmu.write_byte(0x91, 0xff40);
        mmu.write_byte(0xfc, 0xff47);
        mmu.write_byte(0xff, 0xff48);
        mmu.write_byte(0xff, 0xff49);
        mmu
    }

    pub fn step(&mut self, m_cycles: u8) {
        self.timer.step(m_cycles);
        self.intf |= (self.timer.interrupt as u8) << 2;
        self.timer.interrupt = false;

        self.intf |= (self.joypad.interrupt as u8) << 4;
        self.joypad.interrupt = false;

        self.gpu.step(m_cycles);
        self.intf |= self.gpu.interrupt_vblank as u8;
        self.gpu.interrupt_vblank = false;

        self.intf |= (self.gpu.interrupt_stat as u8) << 1;
        self.gpu.interrupt_stat = false;

        self.apu.step(m_cycles);
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7fff => self.cartridge.read_rom(address),
            0x8000..=0x9fff => self.gpu.read(address),
            0xa000..=0xbfff => self.cartridge.read_ram(address),
            0xfe00..=0xfe9f => self.gpu.read(address),
            0xff00 => self.joypad.read(),

            0xff10..=0xff3f => self.apu.read(address),

            0xff04..=0xff07 => self.timer.read(address),

            // oam dma transfer
            0xff46 => 0,

            0xff40..=0xff4b => self.gpu.read(address),

            0xff0f => 0xe0 | self.intf,

            0xffff => self.inte,

            other => {
                // println!("Address 0x{:02X} not yet implemented.", other);
                self.ram[address as usize]
            }
        }
    }

    pub fn write_byte(&mut self, value: u8, address: u16) {
        match address {
            0x0000..=0x7fff => self.cartridge.write_rom(value, address),
            0x8000..=0x9fff => self.gpu.write(value, address),
            0xa000..=0xbfff => self.cartridge.write_ram(value, address),
            0xfe00..=0xfe9f => self.gpu.write(value, address),
            0xff00 => self.joypad.write(value),

            0xff10..=0xff3f => self.apu.write(value, address),

            //oam dma transfer
            0xff46 => self.oam_dma(value),

            0xff40..=0xff4b => self.gpu.write(value, address),

            0xff04..=0xff07 => self.timer.write(value, address),

            0xff0f => {
                self.intf = value;
            }

            0xffff => {
                self.inte = value;
            }

            _ => {
                self.ram[address as usize] = value;
                // println!("Address 0x{:02X} not yet implemented.", other);
            }
        }
        //self.ram[address as usize] = value;
        /*match address {
            0x8000..=0x9fff => {}
        }*/
    }

    pub fn read_word(&self, address: u16) -> u16 {
        // little endian order of bits
        (self.read_byte(address) as u16) | ((self.read_byte(address + 1) as u16) << 8)
    }

    pub fn write_word(&mut self, value: u16, address: u16) {
        // write in little endian
        self.write_byte((value & 0x00ff) as u8, address);
        self.write_byte((value >> 8) as u8, address.wrapping_add(1));
    }

    fn oam_dma(&mut self, value: u8) {
        let base_addr = (value as u16) << 8;

        for index in 0..=0x9f {
            let value = self.read_byte(base_addr | index);
            self.write_byte(value, 0xfe00 | index);
        }
    }
}
