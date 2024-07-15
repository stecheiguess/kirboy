use std::env;

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

            /*0x20 => {
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
            }*/

            // ld 16 bit

            0x01 => {
                let word = self.fetch_word();
                self.registers.set_bc(word)
            }

            0x11 => {
                let word = self.fetch_word();
                self.registers.set_de(word)
            }

            0x21 => {
                let word = self.fetch_word();
                self.registers.set_hl(word)
            }

            0x31 => {
                self.registers.sp = self.fetch_word();
            }

            0x02 => { self.memory.write_byte(self.registers.a, self.registers.get_bc()) }

            0x12 => { self.memory.write_byte(self.registers.a, self.registers.get_de()) }

            0x22 => {
                self.memory.write_byte(self.registers.a, self.registers.get_hl());
                println!("{hl:?}", hl = self.registers.get_hl());
                self.registers.set_hl(self.registers.get_hl() + 1)
            }

            0x32 => {
                self.memory.write_byte(self.registers.a, self.registers.get_hl());
                println!("{hl:?}", hl = self.registers.get_hl());
                self.registers.set_hl(self.registers.get_hl() - 1)
            }

            // add 16 bit

            // inc
            0x04 => {
                self.registers.b = self.inc(self.registers.b);
            }
            0x14 => {
                self.registers.d = self.inc(self.registers.d);
            }
            0x24 => {
                self.registers.h = self.inc(self.registers.h);
            }
            0x34 => {
                let value = self.inc(self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl());
            }

            0x0c => {
                self.registers.c = self.inc(self.registers.c);
            }
            0x1c => {
                self.registers.e = self.inc(self.registers.e);
            }
            0x2c => {
                self.registers.l = self.inc(self.registers.l);
            }
            0x3c => {
                self.registers.a = self.inc(self.registers.a);
            }

            // dec
            0x05 => {
                self.registers.b = self.dec(self.registers.b);
            }
            0x15 => {
                self.registers.d = self.dec(self.registers.d);
            }
            0x25 => {
                self.registers.h = self.dec(self.registers.h);
            }
            0x35 => {
                let value = self.dec(self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl());
            }

            0x0d => {
                self.registers.c = self.dec(self.registers.c);
            }
            0x1d => {
                self.registers.e = self.dec(self.registers.e);
            }
            0x2d => {
                self.registers.l = self.dec(self.registers.l);
            }
            0x3d => {
                self.registers.a = self.dec(self.registers.a);
            }

            //ld d8

            0x06 => {
                self.registers.b = self.fetch();
            }
            0x16 => {
                self.registers.d = self.fetch();
            }
            0x26 => {
                self.registers.h = self.fetch();
            }
            0x36 => {
                let byte = self.fetch();
                self.memory.write_byte(byte, self.registers.get_hl())
            }

            0x0e => {
                self.registers.c = self.fetch();
            }
            0x1e => {
                self.registers.e = self.fetch();
            }
            0x2e => {
                self.registers.l = self.fetch();
            }
            0x3e => {
                self.registers.a = self.fetch();
            }

            // ld b
            0x40 => {
                self.registers.b = self.registers.b;
            }
            0x41 => {
                self.registers.b = self.registers.c;
            }
            0x42 => {
                self.registers.b = self.registers.d;
            }
            0x43 => {
                self.registers.b = self.registers.e;
            }
            0x44 => {
                self.registers.b = self.registers.h;
            }
            0x45 => {
                self.registers.b = self.registers.l;
            }
            0x46 => {
                self.registers.b = self.memory.read_byte(self.registers.get_hl());
            }
            0x47 => {
                self.registers.b = self.registers.a;
            }
            // ld c
            0x48 => {
                self.registers.c = self.registers.b;
            }
            0x49 => {
                self.registers.c = self.registers.c;
            }
            0x4a => {
                self.registers.c = self.registers.d;
            }
            0x4b => {
                self.registers.c = self.registers.e;
            }
            0x4c => {
                self.registers.c = self.registers.h;
            }
            0x4d => {
                self.registers.c = self.registers.l;
            }
            0x4e => {
                self.registers.c = self.memory.read_byte(self.registers.get_hl());
            }
            0x4f => {
                self.registers.c = self.registers.a;
            }
            // ld d
            0x50 => {
                self.registers.d = self.registers.b;
            }
            0x51 => {
                self.registers.d = self.registers.c;
            }
            0x52 => {
                self.registers.d = self.registers.d;
            }
            0x53 => {
                self.registers.d = self.registers.e;
            }
            0x54 => {
                self.registers.d = self.registers.h;
            }
            0x55 => {
                self.registers.d = self.registers.l;
            }
            0x56 => {
                self.registers.d = self.memory.read_byte(self.registers.get_hl());
            }
            0x57 => {
                self.registers.d = self.registers.a;
            }
            // ld e
            0x58 => {
                self.registers.e = self.registers.b;
            }
            0x59 => {
                self.registers.e = self.registers.c;
            }
            0x5a => {
                self.registers.e = self.registers.d;
            }
            0x5b => {
                self.registers.e = self.registers.e;
            }
            0x5c => {
                self.registers.e = self.registers.h;
            }
            0x5d => {
                self.registers.e = self.registers.l;
            }
            0x5e => {
                self.registers.e = self.memory.read_byte(self.registers.get_hl());
            }
            0x5f => {
                self.registers.e = self.registers.a;
            }
            // ld h
            0x60 => {
                self.registers.h = self.registers.b;
            }
            0x61 => {
                self.registers.h = self.registers.c;
            }
            0x62 => {
                self.registers.h = self.registers.d;
            }
            0x63 => {
                self.registers.h = self.registers.e;
            }
            0x64 => {
                self.registers.h = self.registers.h;
            }
            0x65 => {
                self.registers.h = self.registers.l;
            }
            0x66 => {
                self.registers.h = self.memory.read_byte(self.registers.get_hl());
            }
            0x67 => {
                self.registers.h = self.registers.a;
            }
            // ld l
            0x68 => {
                self.registers.l = self.registers.b;
            }
            0x69 => {
                self.registers.l = self.registers.c;
            }
            0x6a => {
                self.registers.l = self.registers.d;
            }
            0x6b => {
                self.registers.l = self.registers.e;
            }
            0x6c => {
                self.registers.l = self.registers.h;
            }
            0x6d => {
                self.registers.l = self.registers.l;
            }
            0x6e => {
                self.registers.l = self.memory.read_byte(self.registers.get_hl());
            }
            0x6f => {
                self.registers.l = self.registers.a;
            }

            // ld [hl]

            0x70 => { self.memory.write_byte(self.registers.b, self.registers.get_hl()) }
            0x71 => { self.memory.write_byte(self.registers.c, self.registers.get_hl()) }
            0x72 => { self.memory.write_byte(self.registers.d, self.registers.get_hl()) }
            0x73 => { self.memory.write_byte(self.registers.e, self.registers.get_hl()) }
            0x74 => { self.memory.write_byte(self.registers.h, self.registers.get_hl()) }
            0x75 => { self.memory.write_byte(self.registers.l, self.registers.get_hl()) }
            0x77 => { self.memory.write_byte(self.registers.a, self.registers.get_hl()) }

            0x76 => {
                //panic!("END");
            }

            // ld a
            0x78 => {
                self.registers.a = self.registers.b;
            }
            0x79 => {
                self.registers.a = self.registers.c;
            }
            0x7a => {
                self.registers.a = self.registers.d;
            }
            0x7b => {
                self.registers.a = self.registers.e;
            }
            0x7c => {
                self.registers.a = self.registers.h;
            }
            0x7d => {
                self.registers.a = self.registers.l;
            }
            0x7e => {
                self.registers.a = self.memory.read_byte(self.registers.get_hl());
            }
            0x7f => {
                self.registers.a = self.registers.a;
            }

            0x80 => self.add(self.registers.b),
            0x81 => self.add(self.registers.c),
            0x82 => self.add(self.registers.d),
            0x83 => self.add(self.registers.e),
            0x84 => self.add(self.registers.h),
            0x85 => self.add(self.registers.l),
            0x86 => self.add(self.memory.read_byte(self.registers.get_hl())),
            0x87 => self.add(self.registers.a),
            // add carry
            0x88 => self.adc(self.registers.b),
            0x89 => self.adc(self.registers.c),
            0x8a => self.adc(self.registers.d),
            0x8b => self.adc(self.registers.e),
            0x8c => self.adc(self.registers.h),
            0x8d => self.adc(self.registers.l),
            0x8e => self.adc(self.memory.read_byte(self.registers.get_hl())),
            0x8f => self.adc(self.registers.a),
            // sub
            0x90 => self.sub(self.registers.b),
            0x91 => self.sub(self.registers.c),
            0x92 => self.sub(self.registers.d),
            0x93 => self.sub(self.registers.e),
            0x94 => self.sub(self.registers.h),
            0x95 => self.sub(self.registers.l),
            0x96 => self.sub(self.memory.read_byte(self.registers.get_hl())),
            0x97 => self.sub(self.registers.a),
            // sub carry
            0x98 => self.sbc(self.registers.b),
            0x99 => self.sbc(self.registers.c),
            0x9a => self.sbc(self.registers.d),
            0x9b => self.sbc(self.registers.e),
            0x9c => self.sbc(self.registers.h),
            0x9d => self.sbc(self.registers.l),
            0x9e => self.sbc(self.memory.read_byte(self.registers.get_hl())),
            0x9f => self.sbc(self.registers.a),
            // and
            0xa0 => self.and(self.registers.b),
            0xa1 => self.and(self.registers.c),
            0xa2 => self.and(self.registers.d),
            0xa3 => self.and(self.registers.e),
            0xa4 => self.and(self.registers.h),
            0xa5 => self.and(self.registers.l),
            0xa6 => self.and(self.memory.read_byte(self.registers.get_hl())),
            0xa7 => self.and(self.registers.a),
            // xor
            0xa8 => self.xor(self.registers.b),
            0xa9 => self.xor(self.registers.c),
            0xaa => self.xor(self.registers.d),
            0xab => self.xor(self.registers.e),
            0xac => self.xor(self.registers.h),
            0xad => self.xor(self.registers.l),
            0xae => self.xor(self.memory.read_byte(self.registers.get_hl())),
            0xaf => self.xor(self.registers.a),
            // or
            0xb0 => self.or(self.registers.b),
            0xb1 => self.or(self.registers.c),
            0xb2 => self.or(self.registers.d),
            0xb3 => self.or(self.registers.e),
            0xb4 => self.or(self.registers.h),
            0xb5 => self.or(self.registers.l),
            0xb6 => self.or(self.memory.read_byte(self.registers.get_hl())),
            0xb7 => self.or(self.registers.a),
            // cp
            0xb8 => self.cp(self.registers.b),
            0xb9 => self.cp(self.registers.c),
            0xba => self.cp(self.registers.d),
            0xbb => self.cp(self.registers.e),
            0xbc => self.cp(self.registers.h),
            0xbd => self.cp(self.registers.l),
            0xbe => self.cp(self.memory.read_byte(self.registers.get_hl())),
            0xbf => self.cp(self.registers.a),

            0xcb => { self.execute_cb() }

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

    fn execute_cb(&mut self) {
        let opcode = self.fetch();
        println!("CB Opcode is: {:2X}", opcode);

        match opcode {
            // bit 0
            0x40 => self.bit(0, self.registers.b),
            0x41 => self.bit(0, self.registers.c),
            0x42 => self.bit(0, self.registers.d),
            0x43 => self.bit(0, self.registers.e),
            0x44 => self.bit(0, self.registers.h),
            0x45 => self.bit(0, self.registers.l),
            0x46 => self.bit(0, self.memory.read_byte(self.registers.get_hl())),
            0x47 => self.bit(0, self.registers.a),
            // bit 1
            0x48 => self.bit(1, self.registers.b),
            0x49 => self.bit(1, self.registers.c),
            0x4a => self.bit(1, self.registers.d),
            0x4b => self.bit(1, self.registers.e),
            0x4c => self.bit(1, self.registers.h),
            0x4d => self.bit(1, self.registers.l),
            0x4e => self.bit(1, self.memory.read_byte(self.registers.get_hl())),
            0x4f => self.bit(1, self.registers.a),
            // bit 2
            0x50 => self.bit(2, self.registers.b),
            0x51 => self.bit(2, self.registers.c),
            0x52 => self.bit(2, self.registers.d),
            0x53 => self.bit(2, self.registers.e),
            0x54 => self.bit(2, self.registers.h),
            0x55 => self.bit(2, self.registers.l),
            0x56 => self.bit(2, self.memory.read_byte(self.registers.get_hl())),
            0x57 => self.bit(2, self.registers.a),
            // bit 3
            0x58 => self.bit(3, self.registers.b),
            0x59 => self.bit(3, self.registers.c),
            0x5a => self.bit(3, self.registers.d),
            0x5b => self.bit(3, self.registers.e),
            0x5c => self.bit(3, self.registers.h),
            0x5d => self.bit(3, self.registers.l),
            0x5e => self.bit(3, self.memory.read_byte(self.registers.get_hl())),
            0x5f => self.bit(3, self.registers.a),
            // bit 4
            0x60 => self.bit(4, self.registers.b),
            0x61 => self.bit(4, self.registers.c),
            0x62 => self.bit(4, self.registers.d),
            0x63 => self.bit(4, self.registers.e),
            0x64 => self.bit(4, self.registers.h),
            0x65 => self.bit(4, self.registers.l),
            0x66 => self.bit(4, self.memory.read_byte(self.registers.get_hl())),
            0x67 => self.bit(4, self.registers.a),
            // bit 5
            0x68 => self.bit(5, self.registers.b),
            0x69 => self.bit(5, self.registers.c),
            0x6a => self.bit(5, self.registers.d),
            0x6b => self.bit(5, self.registers.e),
            0x6c => self.bit(5, self.registers.h),
            0x6d => self.bit(5, self.registers.l),
            0x6e => self.bit(5, self.memory.read_byte(self.registers.get_hl())),
            0x6f => self.bit(5, self.registers.a),
            // bit 6
            0x70 => self.bit(6, self.registers.b),
            0x71 => self.bit(6, self.registers.c),
            0x72 => self.bit(6, self.registers.d),
            0x73 => self.bit(6, self.registers.e),
            0x74 => self.bit(6, self.registers.h),
            0x75 => self.bit(6, self.registers.l),
            0x76 => self.bit(6, self.memory.read_byte(self.registers.get_hl())),
            0x77 => self.bit(6, self.registers.a),
            // bit 7
            0x78 => self.bit(7, self.registers.b),
            0x79 => self.bit(7, self.registers.c),
            0x7a => self.bit(7, self.registers.d),
            0x7b => self.bit(7, self.registers.e),
            0x7c => self.bit(7, self.registers.h),
            0x7d => self.bit(7, self.registers.l),
            0x7e => self.bit(7, self.memory.read_byte(self.registers.get_hl())),
            0x7f => self.bit(7, self.registers.a),

            // reset to 0

            0x80 => {
                self.registers.b = self.res(0, self.registers.b);
            }
            0x81 => {
                self.registers.c = self.res(0, self.registers.c);
            }
            0x82 => {
                self.registers.d = self.res(0, self.registers.d);
            }
            0x83 => {
                self.registers.e = self.res(0, self.registers.e);
            }
            0x84 => {
                self.registers.h = self.res(0, self.registers.h);
            }
            0x85 => {
                self.registers.l = self.res(0, self.registers.l);
            }
            0x86 => {
                let value = self.res(0, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0x87 => {
                self.registers.a = self.res(0, self.registers.a);
            }
            0x88 => {
                self.registers.b = self.res(1, self.registers.b);
            }
            0x89 => {
                self.registers.c = self.res(1, self.registers.c);
            }
            0x8a => {
                self.registers.d = self.res(1, self.registers.d);
            }
            0x8b => {
                self.registers.e = self.res(1, self.registers.e);
            }
            0x8c => {
                self.registers.h = self.res(1, self.registers.h);
            }
            0x8d => {
                self.registers.l = self.res(1, self.registers.l);
            }
            0x8e => {
                let value = self.res(1, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }
            0x8f => {
                self.registers.a = self.res(1, self.registers.a);
            }

            0x90 => {
                self.registers.b = self.res(2, self.registers.b);
            }
            0x91 => {
                self.registers.c = self.res(2, self.registers.c);
            }
            0x92 => {
                self.registers.d = self.res(2, self.registers.d);
            }
            0x93 => {
                self.registers.e = self.res(2, self.registers.e);
            }
            0x94 => {
                self.registers.h = self.res(2, self.registers.h);
            }
            0x95 => {
                self.registers.l = self.res(2, self.registers.l);
            }
            0x96 => {
                let value = self.res(2, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0x97 => {
                self.registers.a = self.res(2, self.registers.a);
            }
            0x98 => {
                self.registers.b = self.res(3, self.registers.b);
            }
            0x99 => {
                self.registers.c = self.res(3, self.registers.c);
            }
            0x9a => {
                self.registers.d = self.res(3, self.registers.d);
            }
            0x9b => {
                self.registers.e = self.res(3, self.registers.e);
            }
            0x9c => {
                self.registers.h = self.res(3, self.registers.h);
            }
            0x9d => {
                self.registers.l = self.res(3, self.registers.l);
            }
            0x9e => {
                let value = self.res(3, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }
            0x9f => {
                self.registers.a = self.res(3, self.registers.a);
            }
            0xa0 => {
                self.registers.b = self.res(4, self.registers.b);
            }
            0xa1 => {
                self.registers.c = self.res(4, self.registers.c);
            }
            0xa2 => {
                self.registers.d = self.res(4, self.registers.d);
            }
            0xa3 => {
                self.registers.e = self.res(4, self.registers.e);
            }
            0xa4 => {
                self.registers.h = self.res(4, self.registers.h);
            }
            0xa5 => {
                self.registers.l = self.res(4, self.registers.l);
            }
            0xa6 => {
                let value = self.res(4, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0xa7 => {
                self.registers.a = self.res(4, self.registers.a);
            }
            0xa8 => {
                self.registers.b = self.res(5, self.registers.b);
            }
            0xa9 => {
                self.registers.c = self.res(5, self.registers.c);
            }
            0xaa => {
                self.registers.d = self.res(5, self.registers.d);
            }
            0xab => {
                self.registers.e = self.res(5, self.registers.e);
            }
            0xac => {
                self.registers.h = self.res(5, self.registers.h);
            }
            0xad => {
                self.registers.l = self.res(5, self.registers.l);
            }
            0xae => {
                let value = self.res(5, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }
            0xaf => {
                self.registers.a = self.res(5, self.registers.a);
            }

            0xb0 => {
                self.registers.b = self.res(6, self.registers.b);
            }
            0xb1 => {
                self.registers.c = self.res(6, self.registers.c);
            }
            0xb2 => {
                self.registers.d = self.res(6, self.registers.d);
            }
            0xb3 => {
                self.registers.e = self.res(6, self.registers.e);
            }
            0xb4 => {
                self.registers.h = self.res(6, self.registers.h);
            }
            0xb5 => {
                self.registers.l = self.res(6, self.registers.l);
            }
            0xb6 => {
                let value = self.res(6, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0xb7 => {
                self.registers.a = self.res(6, self.registers.a);
            }
            0xb8 => {
                self.registers.b = self.res(7, self.registers.b);
            }
            0xb9 => {
                self.registers.c = self.res(7, self.registers.c);
            }
            0xba => {
                self.registers.d = self.res(7, self.registers.d);
            }
            0xbb => {
                self.registers.e = self.res(7, self.registers.e);
            }
            0xbc => {
                self.registers.h = self.res(7, self.registers.h);
            }
            0xbd => {
                self.registers.l = self.res(7, self.registers.l);
            }
            0xbe => {
                let value = self.res(7, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }
            0xbf => {
                self.registers.a = self.res(7, self.registers.a);
            }

            other => {}
        }
    }

    fn loope(&mut self) {
        while self.registers.pc != 0xffff {
            self.execute();
        }
    }

    // bit wise CB operations

    fn bit(&mut self, bit: u8, register: u8) {
        println!("before: {}, after: {}", register, register >> bit);
        let val = (register >> bit) & 0x01;
        println!("{}", val);
        self.registers.f.zero = val == 1;
        self.registers.f.half_carry = true;
        self.registers.f.subtract = false;
    }

    fn res(&mut self, bit: u8, register: u8) -> u8 {
        let mask: u8 = !(0x01 << bit);
        println!("{mask:b}");
        register & mask
    }

    fn set(&mut self, bit: u8, register: u8) -> u8 {
        let mask: u8 = 0x01 << bit;
        println!("{mask:b}");
        register | mask
    }

    // ALU
    fn add(&mut self, value: u8) {
        // add
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        //if the result is larger than 0xF than the addition caused a carry from the lower nibble to the upper nibble.
        self.registers.f.half_carry = (self.registers.a & 0xf) + (value & 0xf) > 0xf;
        self.registers.a = new_value;
    }

    fn adc(&mut self, value: u8) {
        // add carry
        let (intermediate, intermediate_overflow) = self.registers.a.overflowing_add(value);
        let (new_value, did_overflow) = intermediate.overflowing_add(self.registers.f.carry as u8);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow | intermediate_overflow;

        self.registers.f.half_carry =
            (self.registers.a & 0xf) + (value & 0xf) + (self.registers.f.carry as u8) > 0xf;
        self.registers.a = new_value;
    }

    fn sub(&mut self, value: u8) {
        // subtract
        let (new_value, did_overflow) = self.registers.a.overflowing_sub(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = did_overflow;

        // sees if lower nibble is greater, if is, then set half carry to true.
        self.registers.f.half_carry = self.registers.a & 0xf < value & 0xf;
        self.registers.a = new_value;
    }

    fn sbc(&mut self, value: u8) {
        // subtract carry
        let (intermediate, intermediate_overflow) = self.registers.a.overflowing_sub(value);
        let (new_value, did_overflow) = intermediate.overflowing_sub(self.registers.f.carry as u8);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = did_overflow | intermediate_overflow;

        // sees if lower nibble is greater, if is, then set half carry to true.
        self.registers.f.half_carry =
            self.registers.a & 0xf < (value & 0xf) + (self.registers.f.carry as u8);
        self.registers.a = new_value;
    }

    fn and(&mut self, value: u8) {
        // AND
        let new_value = self.registers.a & value;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = true;
        self.registers.a = new_value;
    }

    fn or(&mut self, value: u8) {
        // OR
        let new_value = self.registers.a | value;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;
        self.registers.a = new_value;
    }

    fn xor(&mut self, value: u8) {
        // XOR
        let new_value = self.registers.a ^ value;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;
        self.registers.a = new_value;
    }
    fn cp(&mut self, value: u8) {
        // compare
        self.registers.f.zero = self.registers.a == value;
        self.registers.f.subtract = true;
        self.registers.f.carry = self.registers.a < value;
        self.registers.f.half_carry = self.registers.a & 0xf < value & 0xf;
    }

    fn inc(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = value.overflowing_add(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (value & 0xf) == 0xf;
        new_value
    }

    fn dec(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = value.overflowing_sub(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (value & 0xf) == 0xf;
        new_value
    }

    fn add16(&mut self, value: u16) {}
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    /* 
    // Read boot-rom file
    let boot = std::fs::read("tetris.gb").unwrap();

    let mut CPU = CPU::new();

    for (position, &byte) in boot.iter().enumerate() {
        //println!("{:X?}", byte);
        CPU.memory.write_byte(byte, position as u16);
    }

    //println!("{:?}", CPU.memory.read_word(CPU.registers.pc));
    */

    // println!("{:?}", CPU.registers);

    //println!("{:?}", CPU.memory.ram)

    let bytes = std::fs::read("tetris.gb").unwrap();
    let mut CPU = CPU::new();

    let mut title = String::new();

    // grab title
    for &byte in &bytes[0x0134..0x0143] {
        title.push(byte as char);
    }
    println!("{}", title);

    for (position, &byte) in bytes.iter().enumerate() {
        //println!("{:X?}", byte);
        CPU.memory.write_byte(byte, (position as u16) + 0x4000);
    }

    println!("{:b}", CPU.memory.ram[0x41a2]);
    println!("{:b}", CPU.set(4, CPU.memory.ram[0x41a2]));

    //CPU.loope();

    println!("{}", title);
}
