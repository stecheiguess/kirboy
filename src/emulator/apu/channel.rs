pub trait Channel {
    fn read(&self, address: u16) -> u8;

    fn write(&mut self, value: u8, address: u8);

    fn on() -> bool;
}
