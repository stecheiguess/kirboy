pub struct GPU {
    control: ControlRegister,
    ly: u8,
    lyc: u8,
    scy: u8,
    scx: u8,
    winy: u8,
    winx: u8,
    vram0: [u8; 0x800],
    vram1: [u8; 0x800],
    vram2: [u8; 0x800],
    tile_maps: [u8; 0x400],
    oam: [u8; 0xa0],
    bgp: u8,
    obp0: u8,
    obp1: u8,
}

#[derive(Copy, Clone, Debug)]
struct ControlRegister {
    lcd_enable: bool,
    window_tile_map: bool,
    window_enable: bool,
    tile_data: bool,
    bg_tile_map: bool,
    obj_size: bool,
    obj_enable: bool,
    // for DMG, bg and window display, for CGB, master priority
    bg_window_enable: bool,
}

impl std::convert::From<ControlRegister> for u8 {
    fn from(reg: ControlRegister) -> u8 {
        ((reg.lcd_enable as u8) << 7) |
            ((reg.window_tile_map as u8) << 6) |
            ((reg.window_enable as u8) << 5) |
            ((reg.tile_data as u8) << 4) |
            ((reg.bg_tile_map as u8) << 3) |
            ((reg.obj_size as u8) << 2) |
            ((reg.obj_enable as u8) << 1) |
            (reg.bg_window_enable as u8)
    }
}

impl std::convert::From<u8> for ControlRegister {
    fn from(byte: u8) -> ControlRegister {
        let lcd_enable = ((byte >> 7) & 0b1) != 0;
        let window_tile_map = ((byte >> 6) & 0b1) != 0;
        let window_enable = ((byte >> 5) & 0b1) != 0;
        let tile_data = ((byte >> 4) & 0b1) != 0;
        let bg_tile_map = ((byte >> 3) & 0b1) != 0;
        let obj_size = ((byte >> 2) & 0b1) != 0;
        let obj_enable = ((byte >> 1) & 0b1) != 0;
        let bg_window_enable = (byte & 0b1) != 0;

        ControlRegister {
            lcd_enable,
            window_tile_map,
            window_enable,
            tile_data,
            bg_tile_map,
            obj_size,
            obj_enable,
            bg_window_enable,
        }
    }
}

impl GPU {
    pub fn new() -> GPU {
        GPU {
            control: ControlRegister {
                lcd_enable: false,
                window_tile_map: false,
                window_enable: false,
                tile_data: false,
                bg_tile_map: false,
                obj_size: false,
                obj_enable: false,
                // for DMG, bg and window display, for CGB, master priority
                bg_window_enable: false,
            },
            ly: 0,
            lyc: 0,
            scy: 0,
            scx: 0,
            winy: 0,
            winx: 0,
            vram0: [0; 0x800],
            vram1: [0; 0x800],
            vram2: [0; 0x800],
            tile_maps: [0; 0x400],
            oam: [0; 0xa0],
            bgp: 0,
            obp0: 0,
            obp1: 0,
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0xff40 => u8::from(self.control),
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff45 => self.lyc,
            0xff47 => self.bgp,
            0xff48 => self.obp0,
            0xff49 => self.obp1,
            0xff4a => self.winy,
            0xff4b => self.winx,

            0x8000..=0x87ff => self.vram0[(address - 0x8000) as usize],
            0x8800..=0x8fff => self.vram1[(address - 0x8800) as usize],
            0x9000..=0x97ff => self.vram2[(address - 0x9000) as usize],
            0x9800..=0x9bff => self.tile_maps[(address - 0x9800) as usize],

            0xfe00..=0xfe9f => self.oam[(address - 0xfe00) as usize],
            _ => 0x0,
        }
    }

    pub fn write(&mut self, value: u8, address: u16) {
        match address {
            0xff40 => {
                self.control = ControlRegister::from(value);
            }
            0xff42 => {
                self.scy = value;
            }
            0xff43 => {
                self.scx = value;
            }
            //read only
            0xff44 => {}

            0xff45 => {
                self.lyc = value;
            }

            0xff47 => {
                self.bgp = value;
            }
            0xff48 => {
                self.obp0 = value;
            }

            0xff49 => {
                self.obp1 = value;
            }

            0xff4a => {
                self.winy = value;
            }
            0xff4b => {
                self.winx = value;
            }

            0x8000..=0x87ff => {
                self.vram0[(address - 0x8000) as usize] = value;
            }
            0x8800..=0x8fff => {
                self.vram1[(address - 0x8800) as usize] = value;
            }
            0x9000..=0x97ff => {
                self.vram2[(address - 0x9000) as usize] = value;
            }
            0x9800..=0x9bff => {
                self.tile_maps[(address - 0x9800) as usize] = value;
            }
            0xfe00..=0xfe9f => {
                self.oam[(address - 0xfe00) as usize] = value;
            }

            _ => {}
        }
    }
}
