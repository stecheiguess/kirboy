const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

const OAM_CYCLES: u16 = 80;
const DRAW_CYCLES: u16 = 172;
const HBLANK_CYCLES: u16 = 204;
const VBLANK_CYCLES: u16 = 456;

#[derive(Copy, Clone, Debug)]
struct LCDC {
    enable_lcd: bool,
    tile_map_window: bool,
    enable_window: bool,
    tile_area: bool,
    tile_map_bg: bool,
    obj_size: bool,
    enable_obj: bool,
    enable_bg_window: bool,
}
impl LCDC {
    fn get(&self) -> u8 {
        ((self.enable_lcd as u8) << 7)
            | ((self.tile_map_window as u8) << 6)
            | ((self.enable_window as u8) << 5)
            | ((self.tile_area as u8) << 4)
            | ((self.tile_map_bg as u8) << 3)
            | ((self.obj_size as u8) << 2)
            | ((self.enable_obj as u8) << 1)
            | (self.enable_bg_window as u8)
    }

    fn set(&mut self, byte: u8) {
        self.enable_lcd = ((byte >> 7) & 0b1) != 0;
        self.tile_map_window = ((byte >> 6) & 0b1) != 0;
        self.enable_window = ((byte >> 5) & 0b1) != 0;
        self.tile_area = ((byte >> 4) & 0b1) != 0;
        self.tile_map_bg = ((byte >> 3) & 0b1) != 0;
        self.obj_size = ((byte >> 2) & 0b1) != 0;
        self.enable_obj = ((byte >> 1) & 0b1) != 0;
        self.enable_bg_window = (byte & 0b1) != 0;
    }
}

#[derive(Copy, Clone, Debug)]
enum Mode {
    OAMScan = 2,
    Drawing = 3,
    HBlank = 0,
    VBlank = 1,
}

pub struct PPU {
    lcdc: LCDC,
    ly: u8,
    lyc: u8,
    scy: u8,
    scx: u8,
    winy: u8,
    winx: u8,
    bank_0: [u8; 0x800],
    bank_1: [u8; 0x800],
    bank_2: [u8; 0x800],
    map_0: [u8; 0x400],
    map_1: [u8; 0x400],
    oam: [[u8; 4]; 40],
    bgp: u8,
    obp_0: u8,
    obp_1: u8,
    int_lyc: bool,
    int_0: bool,
    int_1: bool,
    int_2: bool,
    mode: Mode,
    clock: u16,

    pub buffer: [u8; SCREEN_HEIGHT * SCREEN_WIDTH],

    pub interrupt_stat: bool,
    pub interrupt_vblank: bool,

    pub v_blank: bool,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            lcdc: LCDC {
                enable_lcd: false,
                tile_map_window: false,
                enable_window: false,
                tile_area: false,
                tile_map_bg: false,
                obj_size: false,
                enable_obj: false,
                // for DMG, bg and window display, for CGB, master priority
                enable_bg_window: false,
            },
            ly: 0,
            lyc: 0,
            scy: 0,
            scx: 0,
            winy: 0,
            winx: 0,
            bank_0: [0; 0x800],
            bank_1: [0; 0x800],
            bank_2: [0; 0x800],
            map_0: [0; 0x400],
            map_1: [0; 0x400],
            oam: [[0; 4]; 40],
            bgp: 0,
            obp_0: 0,
            obp_1: 0,
            mode: Mode::OAMScan,
            clock: 0,

            int_lyc: false,
            int_0: false,
            int_1: false,
            int_2: false,

            buffer: [1; SCREEN_HEIGHT * SCREEN_WIDTH],

            interrupt_stat: false,
            interrupt_vblank: false,

            v_blank: false,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xff40 => self.lcdc.get(),
            0xff41 => {
                0x80 | ((self.int_lyc as u8) << 6)
                    | ((self.int_2 as u8) << 5)
                    | ((self.int_1 as u8) << 4)
                    | ((self.int_0 as u8) << 3)
                    | (self.mode as u8)
            }
            0xff42 => self.scy,
            0xff43 => self.scx,
            0xff44 => self.ly,
            0xff45 => self.lyc,
            0xff47 => self.bgp,
            0xff48 => self.obp_0,
            0xff49 => self.obp_1,
            0xff4a => self.winy,
            0xff4b => self.winx,

            0x8000..=0x87ff => self.bank_0[(address - 0x8000) as usize],
            0x8800..=0x8fff => self.bank_1[(address - 0x8800) as usize],
            0x9000..=0x97ff => self.bank_2[(address - 0x9000) as usize],
            0x9800..=0x9bff => self.map_0[(address - 0x9800) as usize],
            0x9c00..=0x9fff => self.map_1[(address - 0x9c00) as usize],

            0xfe00..=0xfe9f => {
                self.oam[((address - 0xfe00) / 4) as usize][((address - 0xfe00) % 4) as usize]
            }

            _ => panic!("Invalid read for PPU"),
        }
    }

    pub fn write(&mut self, value: u8, address: u16) {
        match address {
            0xff40 => {
                let prev_lcd_state = self.lcdc.enable_lcd;
                self.lcdc.set(value);

                if prev_lcd_state && !self.lcdc.enable_lcd {
                    self.clock = 0;
                    self.ly = 0;
                    self.mode = Mode::HBlank;
                }
            }

            0xff41 => {
                self.int_lyc = ((value >> 6) & 0b1) != 0;
                self.int_2 = ((value >> 5) & 0b1) != 0;
                self.int_1 = ((value >> 4) & 0b1) != 0;
                self.int_0 = ((value >> 3) & 0b1) != 0;
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
                if self.ly == self.lyc && self.int_lyc {
                    self.interrupt_stat = true;
                }
            }

            0xff47 => {
                self.bgp = value;
            }
            0xff48 => {
                self.obp_0 = value;
            }

            0xff49 => {
                self.obp_1 = value;
            }

            0xff4a => {
                self.winy = value;
            }
            0xff4b => {
                self.winx = value;
            }

            0x8000..=0x87ff => {
                self.bank_0[(address - 0x8000) as usize] = value;
            }
            0x8800..=0x8fff => {
                self.bank_1[(address - 0x8800) as usize] = value;
            }
            0x9000..=0x97ff => {
                self.bank_2[(address - 0x9000) as usize] = value;
            }
            0x9800..=0x9bff => {
                self.map_0[(address - 0x9800) as usize] = value;
            }
            0x9c00..=0x9fff => {
                self.map_1[(address - 0x9c00) as usize] = value;
            }
            0xfe00..=0xfe9f => {
                self.oam[((address - 0xfe00) / 4) as usize][((address - 0xfe00) % 4) as usize] =
                    value;
            }

            _ => panic!("Invalid write for PPU"),
        }
    }

    fn draw_bg_line(&mut self) {
        if !self.lcdc.enable_bg_window {
            return;
        }

        let y = self.scy.wrapping_add(self.ly);
        let tile_map_row = y / 8;
        let y_in_tile = y % 8;

        let bg_addr = if self.lcdc.tile_map_bg {
            0x9c00
        } else {
            0x9800
        };

        for pixel_index in 0..SCREEN_WIDTH {
            let x = self.scx.wrapping_add(pixel_index as u8);
            let tile_map_col = x / 8;
            let x_in_tile = x % 8;

            let tile_index =
                self.read(bg_addr + ((tile_map_row as u16) << 5) + (tile_map_col as u16));
            let tile_addr = if self.lcdc.tile_area {
                0x8000 + (tile_index as u16) * 16
            } else {
                0x8800 + (((tile_index as i8 as i16) + 128) as u16) * 16
            };

            let low = (self.read(tile_addr + ((y_in_tile * 2) as u16)) >> (7 - x_in_tile)) & 0x1;
            let high =
                (self.read(tile_addr + ((y_in_tile * 2 + 1) as u16)) >> (7 - x_in_tile)) & 0x1;

            let pixel_color = (high << 1) | low;

            let pixel_id = (self.bgp >> (pixel_color * 2)) & 0x03;

            self.buffer[(self.ly as usize) * SCREEN_WIDTH + pixel_index] = pixel_id;
        }
    }

    fn draw_window_line(&mut self) {
        if !self.lcdc.enable_window {
            return;
        }

        if self.winy <= self.ly {
            let y = self.ly - self.winy;
            let tile_map_row = y / 8;
            let y_in_tile = y % 8;
            let win_addr = if self.lcdc.tile_map_window {
                0x9c00
            } else {
                0x9800
            };

            for pixel_index in 0..SCREEN_WIDTH {
                let x = ((pixel_index as i32) - ((self.winx as i32) - 7)) as u8;
                let tile_map_col = x / 8;
                let x_in_tile = x % 8;

                let tile_index =
                    self.read(win_addr + ((tile_map_row as u16) << 5) + (tile_map_col as u16));

                let tile_addr = if self.lcdc.tile_area {
                    // for unsigned order, just converts it to a u16 and add normally.
                    0x8000 + (tile_index as u16) * 16
                } else {
                    // for signed order, uses twos compliment to convert the ranges 128-255 to -128-1.
                    (0x9000u16 as i16 + (tile_index as i16) * 16) as u16
                };

                let low =
                    (self.read(tile_addr + ((y_in_tile * 2) as u16)) >> (7 - x_in_tile)) & 0x1;
                let high =
                    (self.read(tile_addr + ((y_in_tile * 2 + 1) as u16)) >> (7 - x_in_tile)) & 0x1;

                let pixel_color = (high << 1) | low;

                let pixel_id = (self.bgp >> (pixel_color * 2)) & 0x03;

                //if pixel_id > 0 {
                // panic!("DF");
                //}

                self.buffer[(self.ly as usize) * SCREEN_WIDTH + pixel_index] = pixel_id;
            }
        }
    }

    /* fn oam_scan(&mut self) {
        if !self.control.enable_obj {
            return;
        }
        //let mut buffer = [(0, 0, 0); 10];
        let sprite_size = if self.control.obj_size { 16 } else { 8 };

        let line = self.ly + 16;

        let mut buffer_index = 0;

        for sprite_index in 0..40 {
            if buffer_index > 9 {
                break;
            }

            let y = self.oam[sprite_index][0];
            let x = self.oam[sprite_index][1];
            //println!("{:?}", self.oam[sprite_index]);
            println!("y: {}, sprite_size: {}, line: {}", y, sprite_size, line);
            if y + sprite_size > line && line >= y {
                self.oam_buffer[buffer_index] = self.oam[sprite_index];
                buffer_index += 1;
            }
        }
    } */

    fn draw_sprite_line(&mut self) {
        if !self.lcdc.enable_obj {
            return;
        }

        for sprite_index in 0..40 {
            let y = (self.oam[sprite_index][0] as i32) - 16;
            let x = (self.oam[sprite_index][1] as i32) - 8;
            let sprite_attr = self.oam[sprite_index][3];
            let sprite_size = if self.lcdc.obj_size { 16 } else { 8 };

            let x_flip = ((sprite_attr >> 5) & 0x1) == 1;
            let y_flip = ((sprite_attr >> 6) & 0x1) == 1;

            let line = self.ly as i32;
            if line >= y && line < y + sprite_size {
                let y_in_sprite = if y_flip {
                    (sprite_size as u8) - ((line - y) as u8) - 1
                } else {
                    (line - y) as u8
                };

                let tile_index = if sprite_size == 16 {
                    if y_in_sprite < 8 {
                        self.oam[sprite_index][2] & 0xfe
                    } else {
                        self.oam[sprite_index][2] & 0xff
                    }
                } else {
                    self.oam[sprite_index][2]
                };

                let tile_addr = 0x8000 + (tile_index as u16) * 16;

                let palette = (sprite_attr >> 4) & 0x1;

                for x_in_sprite in 0..8 {
                    //println!("{}", x);
                    //println!("{}", x_in_sprite);

                    if x + x_in_sprite >= 0 {
                        let a = if x_flip { 7 - x_in_sprite } else { x_in_sprite };

                        let low =
                            (self.read(tile_addr + ((y_in_sprite * 2) as u16)) >> (7 - a)) & 0x1;
                        let high = (self.read(tile_addr + ((y_in_sprite * 2 + 1) as u16))
                            >> (7 - a))
                            & 0x1;

                        let pixel_color = (high << 1) | low;

                        let pixel_id = match palette {
                            0 => (self.obp_0 >> (pixel_color * 2)) & 0x03,
                            1 => (self.obp_1 >> (pixel_color * 2)) & 0x03,
                            _ => 0,
                        };

                        if pixel_color != 0 {
                            self.buffer[((self.ly as u16) * (SCREEN_WIDTH as u16)
                                + ((x + x_in_sprite) as u16))
                                as usize] = pixel_id;
                        }
                    }
                }
            }
        }
    }

    /*pub fn oam_scan(&mut self) {
        let mut buffer = [(0, 0, 0); 10];
        let sprite_size = if self.control.obj_size { 16 } else { 8 };

        let mut buffer_index = 0;

        for i in 0..self.oam.len() {
            if buffer_index > 9 {
                break;
            }
            let y_position = (self.oam[i][0] as i32) - 16;
            let x_position = (self.oam[i][1] as i32) - 8;
            if y_position + sprite_size > self.ly && self.ly >= y_position && x_position > 0 {
                buffer[buffer_index] = (y_position, x_position, i);
                buffer_index += 1;
            }
        }
    }*/

    pub fn draw_line(&mut self) {
        self.draw_bg_line();
        self.draw_window_line();
        self.draw_sprite_line();
    }

    pub fn step(&mut self, m_cycles: u8) {
        if !self.lcdc.enable_lcd {
            return;
        }

        self.clock += (m_cycles * 4) as u16;
        // add cycle to clock as t cycles
        match self.mode {
            // mode 2
            Mode::OAMScan => {
                if self.clock >= OAM_CYCLES {
                    self.mode = Mode::Drawing;
                    self.clock %= OAM_CYCLES;
                }

                if self.int_2 {
                    self.interrupt_stat = true;
                }
            }
            // mode 3
            Mode::Drawing => {
                if self.clock >= DRAW_CYCLES {
                    self.mode = Mode::HBlank;
                    self.clock %= DRAW_CYCLES;
                    self.draw_line();
                }
            }
            // mode 0
            Mode::HBlank => {
                if self.clock >= HBLANK_CYCLES {
                    self.ly += 1;
                    self.clock %= HBLANK_CYCLES;

                    if self.ly == self.lyc && self.int_lyc {
                        self.interrupt_stat = true;
                    }

                    if self.ly > 143 {
                        self.mode = Mode::VBlank;
                        self.interrupt_vblank = true;
                        self.v_blank = true;
                    } else {
                        self.mode = Mode::OAMScan;
                    }

                    if self.int_0 {
                        self.interrupt_stat = true;
                    }
                }
            }

            // mode 1
            Mode::VBlank => {
                if self.clock >= VBLANK_CYCLES {
                    self.ly += 1;
                    self.clock %= VBLANK_CYCLES;

                    if self.ly > 153 {
                        self.ly = 0;
                        self.mode = Mode::OAMScan;
                    }
                }

                if self.int_1 {
                    self.interrupt_stat = true;
                }
            }
        }
    }
}
