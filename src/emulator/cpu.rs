use crate::emulator::{
    mbc::MBC, mmu::MMU, registers::DoubleRegister, registers::Register, registers::Registers,
};

// cpu

// enum for the interrupt instruction handling
enum Interrupt {
    OFF,
    EXECUTE,
    QUEUED,
}

pub struct CPU {
    pub registers: Registers, // Register Class
    pub mmu: MMU,             // MMU Class for all the components.
    pub ime: bool,            // The IME Flag.
    pub halted: bool,         // Sees if the CPU is halted.
    di: Interrupt,            // disable interrupt
    ei: Interrupt,            // enable interrrupt.
    pc: u16,                  // Program Counter
    sp: u16,                  // Stack Pointer
}

impl CPU {
    pub fn new(cartridge: Box<dyn MBC>, boot: bool) -> Self {
        Self {
            registers: Registers::init(),
            mmu: MMU::init(cartridge),
            ime: false,
            halted: false,
            di: Interrupt::OFF,
            ei: Interrupt::OFF,
            pc: 0x100,
            sp: 0xFFFE,
        }
    }

    fn fetch(&mut self) -> u8 {
        let byte = self.mmu.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    fn fetch_word(&mut self) -> u16 {
        let word = self.mmu.read_word(self.pc);
        self.pc = self.pc.wrapping_add(2);
        word
    }

    fn handle_interrupt(&mut self) -> bool {
        let interrupts = self.mmu.inte & self.mmu.intf;

        if self.ime == false && self.halted == false {
            return false;
        }

        if interrupts == 0 {
            return false;
        }

        self.halted = false;
        if self.ime == false {
            return false;
        }

        self.ime = false;
        let interrupt = interrupts.trailing_zeros();
        // disable interrupt
        self.mmu.intf = self.mmu.intf & !(1 << interrupt);
        self.push(self.pc);
        self.pc = 0x40 | ((interrupt as u16) << 3);
        return true;
    }

    fn update_interrupt(&mut self) {
        //account for delay of DI and EI.
        self.di = match self.di {
            Interrupt::QUEUED => Interrupt::EXECUTE,
            Interrupt::EXECUTE => {
                self.ime = false;
                Interrupt::OFF
            }
            Interrupt::OFF => Interrupt::OFF,
        };

        self.ei = match self.ei {
            Interrupt::QUEUED => Interrupt::EXECUTE,
            Interrupt::EXECUTE => {
                self.ime = true;
                Interrupt::OFF
            }
            Interrupt::OFF => Interrupt::OFF,
        };
    }

    pub fn step(&mut self) -> u8 {
        self.update_interrupt();

        let m_cycles = if !self.handle_interrupt() {
            if self.halted {
                1
            } else {
                self.execute()
            }
        } else {
            4
        };

        self.mmu.step(m_cycles);
        m_cycles
    }

    fn execute(&mut self) -> u8 {
        //println!("PROGRAM COUNTER: 0x{:04X}", self.pc);
        let opcode = self.fetch();
        //println!("Instruction {:2X}", opcode);
        //println!("{:?}", self.registers);
        match opcode {
            0x00 | 0x10 => 1,

            // ld 16 bit
            0x01 | 0x11 | 0x21 | 0x31 => {
                let i = (opcode >> 4) & 0x03;
                let word = self.fetch_word();
                self.set_rg_16(i, opcode, word);
                3
            }

            0xf9 => {
                self.sp = self.registers.get_16(DoubleRegister::HL);
                2
            }

            // ld 16, a
            0x02 => {
                self.mmu.write_byte(
                    self.registers.get(Register::A),
                    self.registers.get_16(DoubleRegister::BC),
                );
                2
            }

            0x12 => {
                self.mmu.write_byte(
                    self.registers.get(Register::A),
                    self.registers.get_16(DoubleRegister::DE),
                );
                2
            }

            0x22 => {
                let v = self.registers.get_16(DoubleRegister::HL);
                self.mmu.write_byte(self.registers.get(Register::A), v);
                self.registers.set_16(DoubleRegister::HL, v + 1);
                2
            }

            0x32 => {
                let v = self.registers.get_16(DoubleRegister::HL);
                self.mmu.write_byte(self.registers.get(Register::A), v);
                self.registers.set_16(DoubleRegister::HL, v - 1);

                2
            }

            // add 16 bit
            0x09 | 0x19 | 0x29 | 0x39 => {
                let i = (opcode >> 4) & 0x03;
                self.add16(self.get_rg_16(i, opcode));
                2
            }

            0xe8 => {
                self.sp = self.add16e(self.sp);
                4
            }
            0xf8 => {
                let v = self.add16e(self.sp);
                self.registers.set_16(DoubleRegister::HL, v);
                3
            }

            // inc 16
            0x03 | 0x13 | 0x23 | 0x33 => {
                let i = (opcode >> 4) & 0x03;
                self.set_rg_16(i, opcode, self.get_rg_16(i, opcode).wrapping_add(1));
                2
            }

            // dec 16
            0x0b | 0x1b | 0x2b | 0x3b => {
                let i = (opcode >> 4) & 0x03;
                self.set_rg_16(i, opcode, self.get_rg_16(i, opcode).wrapping_sub(1));
                2
            }

            // inc
            0x04 | 0x14 | 0x24 | 0x34 | 0x0c | 0x1c | 0x2c | 0x3c => {
                let i = (opcode >> 3) & 0x7;
                self.inc(i);
                match i {
                    6 => 3,
                    _ => 1,
                }
            }

            // dec
            0x05 | 0x15 | 0x25 | 0x35 | 0x0d | 0x1d | 0x2d | 0x3d => {
                let i = (opcode >> 3) & 0x7;
                self.dec(i);
                match i {
                    6 => 3,
                    _ => 1,
                }
            }

            //ld d8
            0x06 | 0x16 | 0x26 | 0x36 | 0x0e | 0x1e | 0x2e | 0x3e => {
                let i = (opcode >> 3) & 0x7;
                let v = self.fetch();
                self.set_rg(i, v);
                match i {
                    6 => 3,
                    _ => 2,
                }
            }

            // halted
            0x76 => {
                self.halted = true;
                1
            }

            // LD
            0x40..=0x7f => {
                let from = opcode & 0x07;
                let dest = (opcode >> 3) & 0x07;
                self.set_rg(dest, self.get_rg(from));
                match dest {
                    6 => 2,
                    _ => match from {
                        6 => 2,
                        _ => 1,
                    },
                }
            }

            0x0a => {
                self.registers.set(
                    Register::A,
                    self.mmu
                        .read_byte(self.registers.get_16(DoubleRegister::BC)),
                );

                2
            }

            0x1a => {
                self.registers.set(
                    Register::A,
                    self.mmu
                        .read_byte(self.registers.get_16(DoubleRegister::DE)),
                );
                2
            }

            0x2a => {
                let v = self.registers.get_16(DoubleRegister::HL);
                self.registers.set(Register::A, self.mmu.read_byte(v));
                self.registers.set_16(DoubleRegister::HL, v + 1);
                2
            }

            0x3a => {
                let v = self.registers.get_16(DoubleRegister::HL);
                self.registers.set(Register::A, self.mmu.read_byte(v));
                self.registers.set_16(DoubleRegister::HL, v - 1);
                2
            }

            // operations.
            0x80..=0xbf => {
                let register_index = opcode & 0x07;
                let operation_index = (opcode >> 3) & 0x07;

                // assigns the operation as a first class object
                let operation = self.get_operation(operation_index);

                // calls the operation
                operation(self, self.get_rg(register_index));

                match register_index {
                    6 => 2,
                    _ => 1,
                }
            }

            // a, n8
            0xc6 | 0xd6 | 0xe6 | 0xf6 | 0xce | 0xde | 0xee | 0xfe => {
                let byte = self.fetch();
                let i = (opcode >> 3) & 0x7;
                let operation = self.get_operation(i);
                operation(self, byte);
                2
            }

            // CB
            0xcb => self.execute_cb(),

            // random accumulator (a) stuff
            0x07 => {
                let a = self.registers.get(Register::A);
                let v = self.rlc(a);
                self.registers.set(Register::A, v);
                self.registers.f.zero = false;
                1
            }

            0x17 => {
                let a = self.registers.get(Register::A);
                let v = self.rl(a);
                self.registers.set(Register::A, v);
                self.registers.f.zero = false;
                1
            }

            // really complex daa.
            0x27 => {
                let mut a = self.registers.get(Register::A);
                if !self.registers.f.subtract {
                    if self.registers.f.carry || a > 0x99 {
                        a = a.wrapping_add(0x60);
                        self.registers.set(Register::A, a);
                        self.registers.f.carry = true;
                    }
                    if self.registers.f.half_carry || a & 0x0f > 0x09 {
                        a = a.wrapping_add(0x06);
                        self.registers.set(Register::A, a);
                    }
                } else {
                    if self.registers.f.carry {
                        a = a.wrapping_sub(0x60);
                        self.registers.set(Register::A, a);
                    }
                    if self.registers.f.half_carry {
                        a = a.wrapping_sub(0x06);
                        self.registers.set(Register::A, a);
                    }
                }
                self.registers.f.zero = self.registers.get(Register::A) == 0;
                self.registers.f.half_carry = false;
                1
            }

            0x0f => {
                let a = self.registers.get(Register::A);
                let v = self.rrc(a);
                self.registers.set(Register::A, v);
                self.registers.f.zero = false;
                1
            }
            0x1f => {
                let a = self.registers.get(Register::A);
                let v = self.rr(a);
                self.registers.set(Register::A, v);
                self.registers.f.zero = false;
                1
            }

            0x2f => {
                let a = self.registers.get(Register::A);
                self.registers.set(Register::A, !a);
                self.registers.f.subtract = true;
                self.registers.f.half_carry = true;
                1
            }

            // register stuff
            0x37 => {
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = true;
                1
            }

            0x3f => {
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = !self.registers.f.carry;
                1
            }

            // a, registers. [c]
            0xe0 => {
                let byte = self.fetch();
                self.mmu
                    .write_byte(self.registers.get(Register::A), 0xff00 | (byte as u16));
                3
            }

            0xf0 => {
                let address = 0xff00 | (self.fetch() as u16);
                self.registers.set(Register::A, self.mmu.read_byte(address));
                3
            }

            0xe2 => {
                self.mmu.write_byte(
                    self.registers.get(Register::A),
                    0xff00 | (self.registers.get(Register::C) as u16),
                );
                2
            }

            0xf2 => {
                let c = self.registers.get(Register::C);
                self.registers
                    .set(Register::A, self.mmu.read_byte(0xff00 | (c as u16)));
                2
            }

            0xfa => {
                let word = self.fetch_word();
                self.registers.set(Register::A, self.mmu.read_byte(word));
                4
            }

            0xea => {
                let word = self.fetch_word();
                self.mmu.write_byte(self.registers.get(Register::A), word);
                4
            }

            // push

            // this fucking opcode got my value and address turned around
            0x08 => {
                let address = self.fetch_word();
                self.mmu.write_word(self.sp, address);
                5
            }

            0xc5 | 0xd5 | 0xe5 | 0xf5 => {
                let i = (opcode >> 4) & 0x03;
                self.push(self.get_rg_16(i, opcode));
                4
            }

            // pop
            0xc1 | 0xd1 | 0xe1 | 0xf1 => {
                let i = (opcode >> 4) & 0x03;
                let value = self.pop();
                self.set_rg_16(i, opcode, value);
                3
            }

            // rst
            0xc4 | 0xd4 | 0xcc | 0xdc => {
                let i = (opcode >> 3) & 0x03;
                if self.get_flag(i) {
                    self.push(self.pc + 2);
                    self.pc = self.fetch_word();
                    6
                } else {
                    self.pc = self.pc.wrapping_add(2);
                    3
                }
            }

            0xcd => {
                self.push(self.pc + 2);
                self.pc = self.fetch_word();
                6
            }

            0xc7 | 0xd7 | 0xe7 | 0xf7 | 0xcf | 0xdf | 0xef | 0xff => {
                let i = (opcode >> 3) & 0x7;
                self.push(self.pc);
                self.pc = (i << 3) as u16;
                4
            }

            // return
            0xc0 | 0xd0 | 0xc8 | 0xd8 => {
                let i = (opcode >> 3) & 0x03;
                if self.get_flag(i) {
                    self.pc = self.pop();
                    5
                } else {
                    2
                }
            }

            0xc9 => {
                self.pc = self.pop();
                4
            }

            // jump
            0x20 | 0x30 | 0x28 | 0x38 => {
                let i = (opcode >> 3) & 0x03;
                if self.get_flag(i) {
                    self.jump();
                    3
                } else {
                    self.pc = self.pc.wrapping_add(1);
                    2
                }
            }

            0x18 => {
                self.jump();
                3
            }

            0xc2 | 0xd2 | 0xca | 0xda => {
                let i = (opcode >> 3) & 0x03;
                if self.get_flag(i) {
                    self.pc = self.fetch_word();
                    4
                } else {
                    self.pc = self.pc.wrapping_add(2);
                    3
                }
            }

            0xc3 => {
                self.pc = self.fetch_word();
                4
            }

            0xe9 => {
                self.pc = self.registers.get_16(DoubleRegister::HL);

                1
            }

            // interrupts
            0xf3 => {
                self.di = Interrupt::QUEUED;
                1
            }

            0xfb => {
                self.ei = Interrupt::QUEUED;
                1
            }

            0xd9 => {
                self.pc = self.pop();
                self.ei = Interrupt::EXECUTE;
                4
            }

            _ => {
                //println!("Instruction {:2X} is not implemented", other);
                1
            }
        }
    }

    fn execute_cb(&mut self) -> u8 {
        let opcode = self.fetch();
        //println!("CB Opcode is: {:2X}", opcode);
        let register_index = opcode & 0x07;
        match opcode {
            // rlc
            0x00..=0x07 => {
                let v: u8 = self.rlc(self.get_rg(register_index));
                self.set_rg(register_index, v);
                match register_index {
                    6 => 4,
                    _ => 2,
                }
            }

            // rrc
            0x08..=0x0f => {
                let v: u8 = self.rrc(self.get_rg(register_index));
                self.set_rg(register_index, v);
                match register_index {
                    6 => 4,
                    _ => 2,
                }
            }

            // rl
            0x10..=0x17 => {
                let v: u8 = self.rl(self.get_rg(register_index));
                self.set_rg(register_index, v);
                match register_index {
                    6 => 4,
                    _ => 2,
                }
            }

            // rr
            0x18..=0x1f => {
                let v: u8 = self.rr(self.get_rg(register_index));
                self.set_rg(register_index, v);
                match register_index {
                    6 => 4,
                    _ => 2,
                }
            }

            // sla
            0x20..=0x27 => {
                let v: u8 = self.sla(self.get_rg(register_index));
                self.set_rg(register_index, v);
                match register_index {
                    6 => 4,
                    _ => 2,
                }
            }

            // sra
            0x28..=0x2f => {
                let v: u8 = self.sra(self.get_rg(register_index));
                self.set_rg(register_index, v);
                match register_index {
                    6 => 4,
                    _ => 2,
                }
            }

            // swap
            0x30..=0x37 => {
                let v: u8 = self.swap(self.get_rg(register_index));
                self.set_rg(register_index, v);
                match register_index {
                    6 => 4,
                    _ => 2,
                }
            }

            // srl
            0x38..=0x3f => {
                let v: u8 = self.srl(self.get_rg(register_index));
                self.set_rg(register_index, v);
                match register_index {
                    6 => 4,
                    _ => 2,
                }
            }

            // bit
            0x40..=0x7f => {
                let bit = (opcode >> 3) & 0x07;
                self.bit(bit, self.get_rg(register_index));
                match register_index {
                    6 => 3,
                    _ => 2,
                }
            }

            // reset to 0
            0x80..=0xbf => {
                let bit = (opcode >> 3) & 0x07;
                let v = self.res(bit, self.get_rg(register_index));
                self.set_rg(register_index, v);
                match register_index {
                    6 => 4,
                    _ => 2,
                }
            }
            // set to 1
            0xc0..=0xff => {
                let bit = (opcode >> 3) & 0x07;
                let v = self.set(bit, self.get_rg(register_index));
                self.set_rg(register_index, v);
                match register_index {
                    6 => 4,
                    _ => 2,
                }
            }

            other => 2,
        }
    }

    // bit wise CB operations

    fn bit(&mut self, bit: u8, register: u8) {
        let val = (register >> bit) & 0x01;
        //println!("{}", val);
        self.registers.f.zero = val == 0;
        self.registers.f.half_carry = true;
        self.registers.f.subtract = false;
    }

    fn res(&mut self, bit: u8, value: u8) -> u8 {
        let mask: u8 = !(0x01 << bit);
        //println!("{mask:b}");
        value & mask
    }

    fn set(&mut self, bit: u8, value: u8) -> u8 {
        let mask: u8 = 0x01 << bit;
        //println!("{mask:b}");
        value | mask
    }

    // ALU
    fn add(&mut self, value: u8) {
        // add
        let a: u8 = self.registers.get(Register::A);

        let (new_value, did_overflow) = a.overflowing_add(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        //if the result is larger than 0xF than the addition caused a carry from the lower nibble to the upper nibble.
        self.registers.f.half_carry = (a & 0xf) + (value & 0xf) > 0xf;
        self.registers.set(Register::A, new_value);
    }

    fn adc(&mut self, value: u8) {
        // add carry

        let a: u8 = self.registers.get(Register::A);

        let c = self.registers.f.carry as u8;

        let new_value = a.wrapping_add(value).wrapping_add(c);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = (a as u16) + (value as u16) + (c as u16) > 0xff;
        self.registers.f.half_carry = (a & 0x0f) + (value & 0x0f) + c > 0x0f;

        self.registers.set(Register::A, new_value);
    }

    fn sub(&mut self, value: u8) {
        // subtract

        let a: u8 = self.registers.get(Register::A);

        let (new_value, did_overflow) = a.overflowing_sub(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = did_overflow;

        // sees if lower nibble is greater, if is, then set half carry to true.
        self.registers.f.half_carry = a & 0xf < value & 0xf;
        self.registers.set(Register::A, new_value);
    }

    fn sbc(&mut self, value: u8) {
        // subtract carry

        let a: u8 = self.registers.get(Register::A);

        let c = self.registers.f.carry as u8;

        let new_value = a.wrapping_sub(value).wrapping_sub(c);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = (a as u16) < (value as u16) + (c as u16);
        self.registers.f.half_carry = a & 0x0f < (value & 0x0f) + c;

        self.registers.set(Register::A, new_value);
    }

    fn and(&mut self, value: u8) {
        // AND

        let a: u8 = self.registers.get(Register::A);

        let new_value = a & value;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = true;
        self.registers.set(Register::A, new_value);
    }

    fn or(&mut self, value: u8) {
        // OR

        let a: u8 = self.registers.get(Register::A);

        let new_value = a | value;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;
        self.registers.set(Register::A, new_value);
    }

    fn xor(&mut self, value: u8) {
        // XOR

        let a: u8 = self.registers.get(Register::A);

        let new_value = a ^ value;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;
        self.registers.set(Register::A, new_value);
    }
    fn cp(&mut self, value: u8) {
        // compare

        let a: u8 = self.registers.get(Register::A);

        self.registers.f.zero = a == value;
        self.registers.f.subtract = true;
        self.registers.f.carry = a < value;
        self.registers.f.half_carry = a & 0xf < value & 0xf;
    }

    fn inc(&mut self, i: u8) {
        let v = self.get_rg(i);
        let new_value = v.wrapping_add(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (v & 0xf) + 1 > 0xf;
        self.set_rg(i, new_value);
    }

    fn dec(&mut self, i: u8) {
        let v = self.get_rg(i);
        let new_value = v.wrapping_sub(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (v & 0xf) == 0;
        self.set_rg(i, new_value);
    }

    fn add16(&mut self, value: u16) {
        let word = self.registers.get_16(DoubleRegister::HL);
        let new_value = word.wrapping_add(value);

        self.registers.f.subtract = false;
        self.registers.f.carry = word > 0xffff - value;
        self.registers.f.half_carry = (word & 0x0fff) + (value & 0x0fff) > 0x0fff;

        self.registers.set_16(DoubleRegister::HL, new_value);
    }

    fn add16e(&mut self, register: u16) -> u16 {
        let byte = self.fetch() as i8 as i16 as u16;

        let new_value = register.wrapping_add(byte);

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (register & 0x000f) + (byte & 0x000f) > 0x000f;
        self.registers.f.carry = (register & 0x00ff) + (byte & 0x00ff) > 0x00ff;

        new_value
    }

    fn jump(&mut self) {
        let address = self.fetch() as i8;
        self.pc = ((self.pc as u32 as i32) + (address as i32)) as u16;
    }

    fn push(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(2);
        self.mmu.write_word(value, self.sp)
    }

    fn pop(&mut self) -> u16 {
        let result = self.mmu.read_word(self.sp);
        self.sp = self.sp.wrapping_add(2);
        result
    }

    fn swap(&mut self, value: u8) -> u8 {
        let new_value = (value >> 4) | (value << 4);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
        new_value
    }

    // arithmetic/logical left shift.
    fn sla(&mut self, value: u8) -> u8 {
        self.registers.f.carry = (value >> 7) == 1;
        let new_value = value << 1;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        new_value
    }

    //arithmetic right shift.
    fn sra(&mut self, value: u8) -> u8 {
        self.registers.f.carry = (value & 0x1) == 0x1;
        let new_value = (value & 0x80) | (value >> 1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        new_value
    }
    //logical right shift.
    fn srl(&mut self, value: u8) -> u8 {
        self.registers.f.carry = (value & 0x1) == 0x1;
        let new_value = value >> 1;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        new_value
    }

    // rotate left carry
    fn rlc(&mut self, value: u8) -> u8 {
        let new_value = (value << 1) | (value >> 7);
        self.registers.f.carry = (value >> 7) == 1;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        new_value
    }

    // rotate left

    fn rl(&mut self, value: u8) -> u8 {
        let new_value = (value << 1) | (self.registers.f.carry as u8);
        self.registers.f.carry = (value >> 7) == 1;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        new_value
    }

    // rotate right carry

    fn rrc(&mut self, value: u8) -> u8 {
        let new_value = (value >> 1) | (value << 7);
        self.registers.f.carry = (value & 0x01) == 1;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        new_value
    }

    // rotate right

    fn rr(&mut self, value: u8) -> u8 {
        let new_value = (value >> 1) | ((self.registers.f.carry as u8) << 7);
        self.registers.f.carry = (value & 0x01) == 1;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        new_value
    }

    fn get_rg(&self, i: u8) -> u8 {
        match i {
            6 => self
                .mmu
                .read_byte(self.registers.get_16(DoubleRegister::HL)),
            _ => self.registers.get(Register::from_index(i)),
        }
    }

    fn set_rg(&mut self, i: u8, v: u8) {
        match i {
            6 => self
                .mmu
                .write_byte(v, self.registers.get_16(DoubleRegister::HL)),
            _ => self.registers.set(Register::from_index(i), v),
        }
    }

    fn get_rg_16(&self, i: u8, opcode: u8) -> u16 {
        match i {
            0 => self.registers.get_16(DoubleRegister::BC),
            1 => self.registers.get_16(DoubleRegister::DE),
            2 => self.registers.get_16(DoubleRegister::HL),
            3 => match opcode {
                0x00..=0x3f => self.sp,
                _ => self.registers.get_16(DoubleRegister::AF),
            },
            _ => panic!("double register index not valid."),
        }
    }

    fn set_rg_16(&mut self, i: u8, opcode: u8, v: u16) {
        match i {
            0 => self.registers.set_16(DoubleRegister::BC, v),
            1 => self.registers.set_16(DoubleRegister::DE, v),
            2 => self.registers.set_16(DoubleRegister::HL, v),
            3 => match opcode {
                0x00..=0x3f => {
                    self.sp = v;
                }
                _ => self.registers.set_16(DoubleRegister::AF, v),
            },
            _ => panic!("double register index not valid."),
        }
    }

    // returns the operation depending on the index.
    fn get_operation(&self, i: u8) -> fn(&mut CPU, u8) {
        match i {
            0 => CPU::add,
            1 => CPU::adc,
            2 => CPU::sub,
            3 => CPU::sbc,
            4 => CPU::and,
            5 => CPU::xor,
            6 => CPU::or,
            7 => CPU::cp,
            _ => panic!("Operation index does not exist"),
        }
    }

    fn get_flag(&self, i: u8) -> bool {
        match i {
            0 => !self.registers.f.zero,
            1 => self.registers.f.zero,
            2 => !self.registers.f.carry,
            3 => self.registers.f.carry,
            _ => panic!("Flag index does not exist"),
        }
    }
}
