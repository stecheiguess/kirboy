use std::{collections::HashMap, hash::Hash};

// TODO: Rewrite the entire class. Use Enums, for more self describing code, instead of an OOP approach, as I have enough of it in the code honestly.

// Flag Register.

#[derive(Copy, Clone, Debug)]
pub struct FlagsRegister {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

impl FlagsRegister {
    fn set(&mut self, v: u8) {
        self.zero = ((v >> ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        self.subtract = ((v >> SUBTRACT_FLAG_BYTE_POSITION) & 0b1) != 0;
        self.half_carry = ((v >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        self.carry = ((v >> CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
    }

    fn get(&self) -> u8 {
        ((if self.zero { 1 } else { 0 }) << ZERO_FLAG_BYTE_POSITION)
            | ((if self.subtract { 1 } else { 0 }) << SUBTRACT_FLAG_BYTE_POSITION)
            | ((if self.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_BYTE_POSITION)
            | ((if self.carry { 1 } else { 0 }) << CARRY_FLAG_BYTE_POSITION)
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

pub enum DoubleRegister {
    BC,
    DE,
    AF,
    HL,
}

impl DoubleRegister {
    pub fn to_tuple(d: DoubleRegister) -> (Register, Register) {
        match d {
            DoubleRegister::AF => (Register::A, Register::F),
            DoubleRegister::BC => (Register::B, Register::C),
            DoubleRegister::DE => (Register::D, Register::E),
            DoubleRegister::HL => (Register::H, Register::L),
        }
    }
}

impl Register {
    pub fn from_index(i: u8) -> Self {
        match i {
            0 => Register::B,
            1 => Register::C,
            2 => Register::D,
            3 => Register::E,
            4 => Register::H,
            5 => Register::L,
            7 => Register::A,
            _ => panic!("index cannot be matched to a single bit register."),
        }
    }
}

#[derive(Debug)]
pub struct Registers {
    data: HashMap<Register, u8>,
    pub f: FlagsRegister,
}
impl Registers {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            f: FlagsRegister {
                zero: false,
                subtract: false,
                half_carry: false,
                carry: false,
            },
        }
    }

    pub fn init() -> Self {
        let mut r = Registers::new();
        r.f.zero = true;
        r.f.carry = true;
        r.f.half_carry = true;
        r.set(Register::A, 0x01);
        r.set(Register::B, 0x00);
        r.set(Register::C, 0x13);
        r.set(Register::D, 0x00);
        r.set(Register::E, 0xd8);
        r.set(Register::H, 0x01);
        r.set(Register::L, 0x4d);

        r
    }
    pub fn get(&self, r: Register) -> u8 {
        match r {
            Register::F => self.f.get(),
            _ => {
                let v = self.data.get(&r).unwrap();
                *v
            }
        }
    }

    pub fn set(&mut self, r: Register, v: u8) {
        match r {
            Register::F => self.f.set(v),
            _ => {
                self.data.insert(r, v);
            }
        }
    }

    pub fn get_16(&self, d: DoubleRegister) -> u16 {
        let (r1, r2) = DoubleRegister::to_tuple(d);
        ((self.get(r1) as u16) << 8) | (self.get(r2) as u16)
    }
    pub fn set_16(&mut self, d: DoubleRegister, v: u16) {
        let (r1, r2) = DoubleRegister::to_tuple(d);
        self.set(r1, (v >> 8) as u8);
        self.set(r2, (v & 0xff) as u8)
    }
}
