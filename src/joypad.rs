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
            0xc0 |
                ((!self.down as u8) << 3) |
                ((!self.up as u8) << 2) |
                ((!self.left as u8) << 1) |
                (!self.right as u8)
        } else if self.action {
            0xc0 |
                ((!self.start as u8) << 3) |
                ((!self.select as u8) << 2) |
                ((!self.b as u8) << 1) |
                (!self.a as u8)
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

/*

pub struct Joypad {
    row0: u8,
    row1: u8,
    data: u8,
    pub interrupt: bool,
}

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
            row0: 0x0f,
            row1: 0x0f,
            data: 0xff,
            interrupt: false,
        }
    }
    /* 
    pub fn write(&mut self, value: u8) {
        self.direction = ((value >> 4) & 0x1) == 0;
        self.action = ((value >> 5) & 0x1) == 0;
    }

    pub fn read(&self) -> u8 {
        if self.direction {
            0xc0 |
                ((!self.down as u8) << 3) |
                ((!self.up as u8) << 2) |
                ((!self.left as u8) << 1) |
                (!self.right as u8)
        } else if self.action {
            0xc0 |
                ((!self.start as u8) << 3) |
                ((!self.select as u8) << 2) |
                ((!self.b as u8) << 1) |
                (!self.a as u8)
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

*/
    pub fn read(&self) -> u8 {
        self.data
    }

    pub fn write(&mut self, value: u8) {
        self.data = (self.data & 0xcf) | (value & 0x30);
        self.update();
    }

    fn update(&mut self) {
        let old_values = self.data & 0xf;
        let mut new_values = 0xf;

        if (self.data & 0x10) == 0x00 {
            new_values &= self.row0;
        }
        if (self.data & 0x20) == 0x00 {
            new_values &= self.row1;
        }

        if old_values == 0xf && new_values != 0xf {
            self.interrupt = true;
        }

        self.data = (self.data & 0xf0) | new_values;
    }

    pub fn key_down(&mut self, key: Input) {
        match key {
            Input::Right => {
                self.row0 &= !(1 << 0);
            }
            Input::Left => {
                self.row0 &= !(1 << 1);
            }
            Input::Up => {
                self.row0 &= !(1 << 2);
            }
            Input::Down => {
                self.row0 &= !(1 << 3);
            }
            Input::A => {
                self.row1 &= !(1 << 0);
            }
            Input::B => {
                self.row1 &= !(1 << 1);
            }
            Input::Select => {
                self.row1 &= !(1 << 2);
            }
            Input::Start => {
                self.row1 &= !(1 << 3);
            }
        }
        self.update();
    }

    pub fn key_up(&mut self, key: Input) {
        match key {
            Input::Right => {
                self.row0 |= 1 << 0;
            }
            Input::Left => {
                self.row0 |= 1 << 1;
            }
            Input::Up => {
                self.row0 |= 1 << 2;
            }
            Input::Down => {
                self.row0 |= 1 << 3;
            }
            Input::A => {
                self.row1 |= 1 << 0;
            }
            Input::B => {
                self.row1 |= 1 << 1;
            }
            Input::Select => {
                self.row1 |= 1 << 2;
            }
            Input::Start => {
                self.row1 |= 1 << 3;
            }
        }
        self.update();
    }
}
*/
