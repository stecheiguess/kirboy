#[derive(Debug)]
struct Registers {
    a: u8,
    f: FlagsRegister,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
    ime: bool,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            f: FlagsRegister {
                zero: false,
                subtract: false,
                half_carry: false,
                carry: false,
            },
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            pc: 0,
            sp: 0,
            ime: false,
        }
    }

    fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xff00) >> 8) as u8;
        self.c = (value & 0xff) as u8;
    }

    fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (u8::from(self.f) as u16)
    }

    fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xff00) >> 8) as u8;
        self.f = FlagsRegister::from((value & 0xff) as u8);
    }

    fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xff00) >> 8) as u8;
        self.e = (value & 0xff) as u8;
    }
    fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }
    fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xff00) >> 8) as u8;
        self.l = (value & 0xff) as u8;
    }
}

// Flag Register.

#[derive(Copy, Clone, Debug)]
struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

impl std::convert::From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> u8 {
        ((if flag.zero { 1 } else { 0 }) << ZERO_FLAG_BYTE_POSITION) |
            ((if flag.subtract { 1 } else { 0 }) << SUBTRACT_FLAG_BYTE_POSITION) |
            ((if flag.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_BYTE_POSITION) |
            ((if flag.carry { 1 } else { 0 }) << CARRY_FLAG_BYTE_POSITION)
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        let subtract = ((byte >> SUBTRACT_FLAG_BYTE_POSITION) & 0b1) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        let carry = ((byte >> CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;

        FlagsRegister {
            zero,
            subtract,
            half_carry,
            carry,
        }
    }
}
struct Memory {
    ram: [u8; 0x10000],
}

impl Memory {
    pub fn new() -> Memory {
        Memory { ram: [0; 0x10000] }
    }

    fn read_byte(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    fn write_byte(&mut self, value: u8, address: u16) {
        self.ram[address as usize] = value;
    }

    fn read_word(&self, address: u16) -> u16 {
        // little endian order of bits
        (self.read_byte(address) as u16) | ((self.read_byte(address + 1) as u16) << 8)
    }

    fn write_word(&mut self, value: u16, address: u16) {
        // write in little endian
        self.write_byte((value & 0x00ff) as u8, address);
        self.write_byte((value >> 8) as u8, address + 1);
    }
}

// cpu
struct CPU {
    registers: Registers,
    memory: Memory,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            registers: Registers::new(),
            memory: Memory::new(),
        }
    }

    fn fetch(&mut self) -> u8 {
        let byte = self.memory.read_byte(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        byte
    }

    fn fetch_word(&mut self) -> u16 {
        let word = self.memory.read_word(self.registers.pc);
        self.registers.pc += 2;
        word
    }

    fn execute(&mut self) {
        println!("PROGRAM COUNTER: {}", self.registers.pc);
        let opcode = self.fetch();
        println!("Instruction {:2X}", opcode);
        match opcode {
            0x00 => {}

            0x0c => {
                self.registers.c += 1;
            }

            0x0e => {
                self.registers.c = self.fetch();
            }

            0x20 => {
                //println!("{:?}", self.registers);
                if self.registers.f.zero == false {
                    //println!("{}", self.fetch())
                    //self.registers.pc =

                    if self.memory.read_byte(0xff50) == 0 {
                        println!("BOOT ROM:");
                        let new = (self.registers.pc + (self.fetch() as u16)) % 255;
                        self.registers.pc = new;
                    } else {
                        println!("{}", self.registers.pc.wrapping_add(self.fetch() as u16));
                    }
                    println!("PROGRAM COUNTER: {}", self.registers.pc);
                }
            }

            0x21 => {
                let word = self.fetch_word();
                self.registers.set_hl(word)
            }

            0x31 => {
                self.registers.sp = self.fetch_word();
            }

            0x32 => {
                self.memory.write_byte(self.registers.a, self.registers.get_hl());
                self.registers.set_hl(self.registers.get_hl() - 1)
            }

            0x3e => {
                self.registers.a = self.fetch();
            }

            0x76 => {
                panic!("END");
            }

            0x77 => { self.memory.write_byte(self.registers.a, self.registers.get_hl()) }

            0xaf => {
                self.registers.a ^= self.registers.a;
                self.registers.f.zero = true;
            }

            0xcb => {
                let opcode = self.fetch();
                println!("CB Opcode is: {:2X}", opcode);

                match opcode {
                    0x7c => { self.bit(7, self.registers.h) }
                    other => {}
                }
            }

            0xe0 => {
                let byte = self.fetch() as u16;
                self.memory.write_byte(self.registers.a, 0xff00 + byte)
            }

            0xe2 => { self.memory.write_byte(self.registers.a, 0xff00 + (self.registers.c as u16)) }

            0xfb => {
                self.registers.ime = true;
            }

            other => {
                println!("Instruction {:2X} is not implemented", other);
            }
        }
    }

    fn loope(&mut self) {
        while self.registers.pc != 0x008f {
            self.execute();
        }
    }

    // bit wise operations

    fn bit(&mut self, bit: u8, register: u8) {
        println!("before: {}, after: {}", register, register >> (bit - 1));
        let val = (register >> (bit - 1)) & 0x01;
        println!("{}", val);
        self.registers.f.zero = val == 1;
        self.registers.f.half_carry = true;
        self.registers.f.subtract = false;
    }
}

fn main() {
    // Read boot-rom file
    let boot = std::fs::read("boot.bin").unwrap();

    let mut CPU = CPU::new();

    for (position, &byte) in boot.iter().enumerate() {
        //println!("{:X?}", byte);
        CPU.memory.write_byte(byte, position as u16);
    }

    //println!("{:?}", CPU.memory.read_word(CPU.registers.pc));

    CPU.loope();

    println!("{:?}", CPU.registers)
    //println!("{:?}", CPU.memory.ram)

    /*
    let bytes = std::fs::read("tetris.gb").unwrap();

    let mut title = String::new();

    // grab title
    for &byte in &bytes[0x0134..0x0143] {
        title.push(byte as char);
    }

    println!("{}", title);*/
}
