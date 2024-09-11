use super::channel::Channel;

pub struct Square {
    period: usize,
}

impl Channel for Square {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xff10 => { 0 }
            _ => panic!("Invalid read for Square"),
        }
    }

    fn write(&mut self, value: u8, address: u8) {
        match address {
            _ => panic!("Invalid write for Square"),
        }
    }

    fn on() -> bool {
        true
    }
}
