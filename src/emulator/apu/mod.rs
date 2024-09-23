mod channel;
mod noise;
mod square;
mod wave;

#[derive(Copy, Clone, Debug)]
pub struct APU {
    enable: bool,
    panning: u8,
    volume_left: u8,
    volume_right: u8,
}

impl APU {
    pub fn new() -> Self {
        Self {
            enable: false,
            /*ch1: false,
            ch2: false,
            ch3: false,
            ch4: false,*/
            panning: 0,
            volume_left: 0,
            volume_right: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            //0xff24 => { self.v }
            0xff25 => self.panning,
            0xff26 => (self.enable as u8) << 7,
            _ => panic!("Invalid read for APU"),
        }
    }

    pub fn write(&mut self, value: u8, address: u16) {
        match address {
            0xff24 => {
                self.volume_left = (value >> 4) & 0x8;
                self.volume_right = value & 0x8;
            }

            0xff25 => {
                self.panning = value;
            }

            0xff26 => {
                self.enable = ((value >> 7) & 0b1) != 0;
            }
            _ => panic!("Invalid write for APU"),
        }
    }
}
