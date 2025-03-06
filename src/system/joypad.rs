pub struct Joypad {
    a: bool,
    b: bool,
    start: bool,
    select: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    direction: bool,
    action: bool,
    pub interrupt: bool,
}

#[derive(Copy, Clone)]
pub enum Input {
    Left,
    Right,
    Up,
    Down,
    A,
    B,
    Start,
    Select,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            a: false,
            b: false,
            start: false,
            select: false,
            up: false,
            down: false,
            left: false,
            right: false,
            direction: false,
            action: false,
            interrupt: false,
        }
    }

    pub fn write(&mut self, value: u8) {
        self.direction = ((value >> 4) & 0x1) == 0;
        self.action = ((value >> 5) & 0x1) == 0;
    }

    pub fn read(&self) -> u8 {
        if self.direction {
            0xc0 | ((!self.down as u8) << 3)
                | ((!self.up as u8) << 2)
                | ((!self.left as u8) << 1)
                | (!self.right as u8)
        } else if self.action {
            0xc0 | ((!self.start as u8) << 3)
                | ((!self.select as u8) << 2)
                | ((!self.b as u8) << 1)
                | (!self.a as u8)
        } else {
            0xc0
        }
    }

    pub fn key_down(&mut self, key: Input) {
        let old = self.read();
        match key {
            Input::Right => {
                self.right = true;
            }
            Input::Left => {
                self.left = true;
            }
            Input::Up => {
                self.up = true;
            }
            Input::Down => {
                self.down = true;
            }
            Input::A => {
                self.a = true;
            }
            Input::B => {
                self.b = true;
            }
            Input::Select => {
                self.select = true;
            }
            Input::Start => {
                self.start = true;
            }
        }
        let new = self.read();
        if old != new {
            self.interrupt = true;
        }
    }

    pub fn key_up(&mut self, key: Input) {
        let old = self.read();
        match key {
            Input::Right => {
                self.right = false;
            }
            Input::Left => {
                self.left = false;
            }
            Input::Up => {
                self.up = false;
            }
            Input::Down => {
                self.down = false;
            }
            Input::A => {
                self.a = false;
            }
            Input::B => {
                self.b = false;
            }
            Input::Select => {
                self.select = false;
            }
            Input::Start => {
                self.start = false;
            }
        }
        let new = self.read();
        if old != new {
            self.interrupt = true;
        }
    }
}
