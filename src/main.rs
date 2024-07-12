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

#[derive(Copy, Clone)]
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

    fn execute(&mut self) {
        let opcode = self.fetch();
        match opcode {
            0x31 => {
                self.registers.sp == self.memory.read_word(self.registers.pc);
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
            // or
            // cp
            _ => (),
        }
    }

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
        let (new_value, did_overflow) = self.registers.a.overflowing_add(
            value + (self.registers.f.carry as u8)
        );
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;

        self.registers.f.half_carry = (self.registers.a & 0xf) + (value & 0xf) > 0xf;
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
        let (new_value, did_overflow) = self.registers.a.overflowing_sub(
            value - (self.registers.f.carry as u8)
        );
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = did_overflow;

        // sees if lower nibble is greater, if is, then set half carry to true.
        self.registers.f.half_carry = self.registers.a & 0xf < value & 0xf;
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

    // functions that change
    /*
    fn ld(&self, register: &mut u8) {
        *register = self.memory.read_byte(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
    }

    fn ld16(&self, register: &mut u16) {
        *register = self.memory.read_word(self.registers.pc);
        self.registers.pc += 2;
    }

    fn inc(&mut self, register: &mut u8) {
        *register = register.wrapping_add(1);
        self.registers.f.zero = *register == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (*register & 0xf) == 0;
    }

    // swaps lower nibble and higher nibble.
    fn swap(&mut self, register: &mut u8) {
        *register = (*register >> 4) | (*register << 4);
        self.registers.f.zero = *register == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
    }

    // arithmetic/logical left shift.
    fn sla(&mut self, register: &mut u8) {
        self.registers.f.carry = (*register >> 7) == 1;
        *register <<= 1;
        self.registers.f.zero = *register == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
    }

    //arithmetic right shift.
    fn sra(&mut self, register: &mut u8) {
        self.registers.f.carry = (*register & 0x1) == 0x1;
        *register = (*register & 0x80) | (*register >> 1);
        self.registers.f.zero = *register == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
    }
    //logical right shift.
    fn srl(&mut self, register: &mut u8) {
        self.registers.f.carry = (*register & 0x1) == 0x1;
        *register = *register >> 1;
        self.registers.f.zero = *register == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
    } */

    /*
        fn inc16(&mut self, register: &mut u16) {
            *register += 1;
        }

        fn dec(&mut self, register: &mut u8) {
            *register += 1;
        }

        fn dec16(&mut self, register: &mut u16) {
            *register += 1;
        }
    */

    // Implementing INC and DEC
}

/*use winit::{ event_loop::EventLoop, window::{ Window, WindowBuilder } };

fn test() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Wait;

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = winit::event_loop::ControlFlow::Exit;
            }
            _ => (),
        }
    });
}*/

fn main() {
    // Read boot-rom file
    let boot = std::fs::read("boot.bin").unwrap();

    let mut CPU = CPU::new();

    for (position, &byte) in boot.iter().enumerate() {
        println!("{:X?}", byte);
        CPU.memory.write_byte(byte, position as u16);
    }

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
