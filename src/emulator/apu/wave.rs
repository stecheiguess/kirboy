use super::channel::Channel;

pub struct Wave {}

impl Wave {}

impl Channel for Wave {
    fn on(&self) -> bool {
        true
    }
    fn read(&self, address: u16) -> u8 {
        0
    }

    fn write(&mut self, value: u8, address: u16) {}

    fn step(&mut self, m_cycles: u8) {}
}
