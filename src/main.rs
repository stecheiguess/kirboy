use std::env;
//use crate::gpu::GPU;
use emulator::mmu::MMU;
use emulator::registers::Registers;

// cpu
struct CPU {
    registers: Registers,
    memory: MMU,
    ime: bool,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            registers: Registers::new(),
            memory: MMU::new(),
            ime: false,
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
        println!("PROGRAM COUNTER: 0x{:04X}", self.registers.pc);
        let opcode = self.fetch();
        println!("Instruction {:2X}", opcode);
        match opcode {
            0x00 => {}

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

            0x02 => self.memory.write_byte(self.registers.a, self.registers.get_bc()),

            0x12 => self.memory.write_byte(self.registers.a, self.registers.get_de()),

            0x22 => {
                self.memory.write_byte(self.registers.a, self.registers.get_hl());
                println!("{hl:?}", hl = self.registers.get_hl());
                self.registers.set_hl(self.registers.get_hl() + 1)
            }

            0x32 => {
                self.memory.write_byte(self.registers.a, self.registers.get_hl());
                println!("{hl:04X}", hl = self.registers.get_hl());
                self.registers.set_hl(self.registers.get_hl() - 1)
            }

            0xf9 => {
                self.registers.sp = self.registers.get_hl();
            }

            // add 16 bit
            0x09 => self.add16(self.registers.get_bc()),
            0x19 => self.add16(self.registers.get_de()),
            0x29 => self.add16(self.registers.get_hl()),
            0x39 => self.add16(self.registers.sp),

            0xe8 => {
                self.registers.sp = self.add16e(self.registers.sp);
            }
            0xf8 => {
                let value = self.add16e(self.registers.sp);
                self.registers.set_hl(value)
            }

            // inc 16
            0x03 => self.registers.set_bc(self.registers.get_bc().wrapping_add(1)),
            0x13 => self.registers.set_de(self.registers.get_de().wrapping_add(1)),
            0x23 => self.registers.set_hl(self.registers.get_hl().wrapping_add(1)),
            0x33 => {
                self.registers.sp = self.registers.sp.wrapping_add(1);
            }

            // dec 16
            0x0b => self.registers.set_bc(self.registers.get_bc().wrapping_sub(1)),
            0x1b => self.registers.set_de(self.registers.get_de().wrapping_sub(1)),
            0x2b => self.registers.set_hl(self.registers.get_hl().wrapping_sub(1)),
            0x3b => {
                self.registers.sp = self.registers.sp.wrapping_sub(1);
            }

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
            0x70 => self.memory.write_byte(self.registers.b, self.registers.get_hl()),
            0x71 => self.memory.write_byte(self.registers.c, self.registers.get_hl()),
            0x72 => self.memory.write_byte(self.registers.d, self.registers.get_hl()),
            0x73 => self.memory.write_byte(self.registers.e, self.registers.get_hl()),
            0x74 => self.memory.write_byte(self.registers.h, self.registers.get_hl()),
            0x75 => self.memory.write_byte(self.registers.l, self.registers.get_hl()),
            0x77 => self.memory.write_byte(self.registers.a, self.registers.get_hl()),

            0x76 => {
                panic!("END");
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

            0x0a => {
                self.registers.a = self.memory.read_byte(self.registers.get_bc());
            }

            0x1a => {
                self.registers.a = self.memory.read_byte(self.registers.get_de());
            }

            0x2a => {
                self.registers.a = self.memory.read_byte(self.registers.get_hl());
                self.registers.set_hl(self.registers.get_hl() + 1)
            }

            0x3a => {
                self.registers.a = self.memory.read_byte(self.registers.get_hl());
                self.registers.set_hl(self.registers.get_hl() - 1)
            }

            // add
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

            // CB
            0xcb => self.execute_cb(),

            // a, n8
            0xc6 => {
                let byte = self.fetch();
                self.add(byte)
            }
            0xd6 => {
                let byte = self.fetch();
                self.sub(byte)
            }
            0xe6 => {
                let byte = self.fetch();
                self.and(byte)
            }
            0xf6 => {
                let byte = self.fetch();
                self.or(byte)
            }
            0xce => {
                let byte = self.fetch();
                self.adc(byte)
            }
            0xde => {
                let byte = self.fetch();
                self.sbc(byte)
            }
            0xee => {
                let byte = self.fetch();
                self.xor(byte)
            }
            0xfe => {
                let byte = self.fetch();
                self.cp(byte)
            }

            // random accumulator (a) stuff
            0x07 => {
                self.registers.a = self.rlc(self.registers.a);
                self.registers.f.zero = false;
            }

            0x17 => {
                self.registers.a = self.rl(self.registers.a);
                self.registers.f.zero = false;
            }

            // really complex daa.
            0x27 => {
                if !self.registers.f.subtract {
                    if self.registers.f.carry || self.registers.a > 0x99 {
                        self.registers.a = self.registers.a.wrapping_add(0x60);
                        self.registers.f.carry = true;
                    }
                    if self.registers.f.half_carry || self.registers.a & 0x0f > 0x09 {
                        self.registers.a = self.registers.a.wrapping_add(0x06);
                    }
                } else {
                    if self.registers.f.carry {
                        self.registers.a = self.registers.a.wrapping_sub(0x60);
                    }
                    if self.registers.f.half_carry {
                        self.registers.a = self.registers.a.wrapping_sub(0x06);
                    }
                }
                self.registers.f.zero = self.registers.a == 0;
                self.registers.f.half_carry = false;
            }

            0x0f => {
                self.registers.a = self.rrc(self.registers.a);
                self.registers.f.zero = false;
            }
            0x1f => {
                self.registers.a = self.rr(self.registers.a);
                self.registers.f.zero = false;
            }

            0x2f => {
                self.registers.a = !self.registers.a;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
            }

            // register stuff
            0x37 => {
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = true;
            }

            0x3f => {
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = !self.registers.f.carry;
            }

            // a, registers. [c]
            0xe0 => {
                let byte = self.fetch();
                self.memory.write_byte(self.registers.a, 0xff00 | (byte as u16))
            }

            0xf0 => {
                let byte = self.fetch();
                let value = self.memory.read_byte(0xff00 | (byte as u16));
                self.registers.a = value;
            }

            0xe2 => self.memory.write_byte(self.registers.a, 0xff00 | (self.registers.c as u16)),

            0xf2 => {
                self.registers.a = self.memory.read_byte(0xff00 | (self.registers.c as u16));
            }

            0xfa => {
                let word = self.fetch_word();
                self.registers.a = self.memory.read_byte(word);
            }

            0xea => {
                let word = self.fetch_word();
                self.memory.write_byte(self.registers.a, word)
            }

            // push
            0x08 => {
                let value = self.fetch_word();
                self.memory.write_word(value, self.registers.sp)
            }

            0xc5 => self.push(self.registers.get_bc()),
            0xd5 => self.push(self.registers.get_de()),
            0xe5 => self.push(self.registers.get_hl()),
            0xf5 => self.push(self.registers.get_af()),

            // pop
            0xc1 => {
                let value = self.pop();
                self.registers.set_bc(value)
            }
            0xd1 => {
                let value = self.pop();
                self.registers.set_de(value)
            }
            0xe1 => {
                let value = self.pop();
                self.registers.set_hl(value)
            }
            0xf1 => {
                let value = self.pop();
                self.registers.set_af(value)
            }

            // rst
            0xc4 => {
                if !self.registers.f.zero {
                    self.push(self.registers.pc + 2);
                    self.registers.pc = self.fetch_word();
                } else {
                    self.registers.pc += 2;
                }
            }
            0xd4 => {
                if !self.registers.f.carry {
                    self.push(self.registers.pc + 2);
                    self.registers.pc = self.fetch_word();
                } else {
                    self.registers.pc += 2;
                }
            }
            0xcc => {
                if self.registers.f.zero {
                    self.push(self.registers.pc + 2);
                    self.registers.pc = self.fetch_word();
                } else {
                    self.registers.pc += 2;
                }
            }
            0xdc => {
                if self.registers.f.carry {
                    self.push(self.registers.pc + 2);
                    self.registers.pc = self.fetch_word();
                } else {
                    self.registers.pc += 2;
                }
            }

            0xcd => {
                self.push(self.registers.pc + 2);
                self.registers.pc = self.fetch_word();
            }

            0xc7 => {
                self.push(self.registers.pc);
                self.registers.pc = 0x00;
            }

            0xd7 => {
                self.push(self.registers.pc);
                self.registers.pc = 0x10;
            }

            0xe7 => {
                self.push(self.registers.pc);
                self.registers.pc = 0x20;
            }

            0xf7 => {
                self.push(self.registers.pc);
                self.registers.pc = 0x30;
            }

            0xcf => {
                self.push(self.registers.pc);
                self.registers.pc = 0x08;
            }

            0xdf => {
                self.push(self.registers.pc);
                self.registers.pc = 0x18;
            }

            0xef => {
                self.push(self.registers.pc);
                self.registers.pc = 0x28;
            }

            0xff => {
                self.push(self.registers.pc);
                self.registers.pc = 0x38;
            }

            // return
            0xc0 => {
                if !self.registers.f.zero {
                    self.registers.pc = self.pop();
                }
            }

            0xd0 => {
                if !self.registers.f.carry {
                    self.registers.pc = self.pop();
                }
            }

            0xc8 => {
                if self.registers.f.zero {
                    self.registers.pc = self.pop();
                }
            }

            0xd8 => {
                if self.registers.f.carry {
                    self.registers.pc = self.pop();
                }
            }

            0xc9 => {
                self.registers.pc = self.pop();
            }

            // jump
            0x20 => {
                if !self.registers.f.zero {
                    self.jump()
                } else {
                    self.registers.pc += 1;
                }
            }

            0x30 => {
                if !self.registers.f.carry {
                    self.jump()
                } else {
                    self.registers.pc += 1;
                }
            }

            0x18 => self.jump(),

            0x28 => {
                if self.registers.f.zero {
                    self.jump()
                } else {
                    self.registers.pc += 1;
                }
            }

            0x38 => {
                if self.registers.f.carry {
                    self.jump()
                } else {
                    self.registers.pc += 1;
                }
            }

            0xc3 => {
                self.registers.pc = self.fetch_word();
            }
            0xc2 => {
                if !self.registers.f.zero {
                    self.registers.pc = self.fetch_word();
                } else {
                    self.registers.pc += 2;
                }
            }

            0xd2 => {
                if !self.registers.f.carry {
                    self.registers.pc = self.fetch_word();
                } else {
                    self.registers.pc += 2;
                }
            }

            0xca => {
                if self.registers.f.zero {
                    self.registers.pc = self.fetch_word();
                } else {
                    self.registers.pc += 2;
                }
            }

            0xda => {
                if self.registers.f.carry {
                    self.registers.pc = self.fetch_word();
                } else {
                    self.registers.pc += 2;
                }
            }

            0xe9 => {
                self.registers.pc = self.registers.get_hl();
            }

            // interrupts
            0xf3 => {
                self.ime = false;
            }

            0xfb => {
                self.ime = true;
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
            // rlc
            0x00 => {
                self.registers.b = self.rlc(self.registers.b);
            }
            0x01 => {
                self.registers.c = self.rlc(self.registers.c);
            }
            0x02 => {
                self.registers.d = self.rlc(self.registers.d);
            }
            0x03 => {
                self.registers.e = self.rlc(self.registers.e);
            }
            0x04 => {
                self.registers.h = self.rlc(self.registers.h);
            }
            0x05 => {
                self.registers.l = self.rlc(self.registers.l);
            }
            0x06 => {
                let value = self.rlc(self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0x07 => {
                self.registers.a = self.rlc(self.registers.a);
            }

            // rrc
            0x08 => {
                self.registers.b = self.rrc(self.registers.b);
            }
            0x09 => {
                self.registers.c = self.rrc(self.registers.c);
            }
            0x0a => {
                self.registers.d = self.rrc(self.registers.d);
            }
            0x0b => {
                self.registers.e = self.rrc(self.registers.e);
            }
            0x0c => {
                self.registers.h = self.rrc(self.registers.h);
            }
            0x0d => {
                self.registers.l = self.rrc(self.registers.l);
            }
            0x0e => {
                let value = self.rrc(self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0x0f => {
                self.registers.a = self.rrc(self.registers.a);
            }

            // rl
            0x10 => {
                self.registers.b = self.rl(self.registers.b);
            }
            0x11 => {
                self.registers.c = self.rl(self.registers.c);
            }
            0x12 => {
                self.registers.d = self.rl(self.registers.d);
            }
            0x13 => {
                self.registers.e = self.rl(self.registers.e);
            }
            0x14 => {
                self.registers.h = self.rl(self.registers.h);
            }
            0x15 => {
                self.registers.l = self.rl(self.registers.l);
            }
            0x16 => {
                let value = self.rl(self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0x17 => {
                self.registers.a = self.rl(self.registers.a);
            }

            // rr
            0x18 => {
                self.registers.b = self.rr(self.registers.b);
            }
            0x19 => {
                self.registers.c = self.rr(self.registers.c);
            }
            0x1a => {
                self.registers.d = self.rr(self.registers.d);
            }
            0x1b => {
                self.registers.e = self.rr(self.registers.e);
            }
            0x1c => {
                self.registers.h = self.rr(self.registers.h);
            }
            0x1d => {
                self.registers.l = self.rr(self.registers.l);
            }
            0x1e => {
                let value = self.rr(self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0x1f => {
                self.registers.a = self.rr(self.registers.a);
            }

            // sla
            0x20 => {
                self.registers.b = self.sla(self.registers.b);
            }
            0x21 => {
                self.registers.c = self.sla(self.registers.c);
            }
            0x22 => {
                self.registers.d = self.sla(self.registers.d);
            }
            0x23 => {
                self.registers.e = self.sla(self.registers.e);
            }
            0x24 => {
                self.registers.h = self.sla(self.registers.h);
            }
            0x25 => {
                self.registers.l = self.sla(self.registers.l);
            }
            0x26 => {
                let value = self.sla(self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0x27 => {
                self.registers.a = self.sla(self.registers.a);
            }

            // sra
            0x28 => {
                self.registers.b = self.sra(self.registers.b);
            }
            0x29 => {
                self.registers.c = self.sra(self.registers.c);
            }
            0x2a => {
                self.registers.d = self.sra(self.registers.d);
            }
            0x2b => {
                self.registers.e = self.sra(self.registers.e);
            }
            0x2c => {
                self.registers.h = self.sra(self.registers.h);
            }
            0x2d => {
                self.registers.l = self.sra(self.registers.l);
            }
            0x2e => {
                let value = self.sra(self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0x2f => {
                self.registers.a = self.sra(self.registers.a);
            }

            // swap
            0x30 => {
                self.registers.b = self.swap(self.registers.b);
            }
            0x31 => {
                self.registers.c = self.swap(self.registers.c);
            }
            0x32 => {
                self.registers.d = self.swap(self.registers.d);
            }
            0x33 => {
                self.registers.e = self.swap(self.registers.e);
            }
            0x34 => {
                self.registers.h = self.swap(self.registers.h);
            }
            0x35 => {
                self.registers.l = self.swap(self.registers.l);
            }
            0x36 => {
                let value = self.swap(self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0x37 => {
                self.registers.a = self.swap(self.registers.a);
            }

            // srl
            0x38 => {
                self.registers.b = self.srl(self.registers.b);
            }
            0x39 => {
                self.registers.c = self.srl(self.registers.c);
            }
            0x3a => {
                self.registers.d = self.srl(self.registers.d);
            }
            0x3b => {
                self.registers.e = self.srl(self.registers.e);
            }
            0x3c => {
                self.registers.h = self.srl(self.registers.h);
            }
            0x3d => {
                self.registers.l = self.srl(self.registers.l);
            }
            0x3e => {
                let value = self.srl(self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0x3f => {
                self.registers.a = self.srl(self.registers.a);
            }

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

            // set to 1
            0xc0 => {
                self.registers.b = self.set(0, self.registers.b);
            }
            0xc1 => {
                self.registers.c = self.set(0, self.registers.c);
            }
            0xc2 => {
                self.registers.d = self.set(0, self.registers.d);
            }
            0xc3 => {
                self.registers.e = self.set(0, self.registers.e);
            }
            0xc4 => {
                self.registers.h = self.set(0, self.registers.h);
            }
            0xc5 => {
                self.registers.l = self.set(0, self.registers.l);
            }
            0xc6 => {
                let value = self.set(0, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0xc7 => {
                self.registers.a = self.set(0, self.registers.a);
            }
            0xc8 => {
                self.registers.b = self.set(1, self.registers.b);
            }
            0xc9 => {
                self.registers.c = self.set(1, self.registers.c);
            }
            0xca => {
                self.registers.d = self.set(1, self.registers.d);
            }
            0xcb => {
                self.registers.e = self.set(1, self.registers.e);
            }
            0xcc => {
                self.registers.h = self.set(1, self.registers.h);
            }
            0xcd => {
                self.registers.l = self.set(1, self.registers.l);
            }
            0xce => {
                let value = self.set(1, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }
            0xcf => {
                self.registers.a = self.set(1, self.registers.a);
            }

            0xd0 => {
                self.registers.b = self.set(2, self.registers.b);
            }
            0xd1 => {
                self.registers.c = self.set(2, self.registers.c);
            }
            0xd2 => {
                self.registers.d = self.set(2, self.registers.d);
            }
            0xd3 => {
                self.registers.e = self.set(2, self.registers.e);
            }
            0xd4 => {
                self.registers.h = self.set(2, self.registers.h);
            }
            0xd5 => {
                self.registers.l = self.set(2, self.registers.l);
            }
            0xd6 => {
                let value = self.set(2, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0xd7 => {
                self.registers.a = self.set(2, self.registers.a);
            }
            0xd8 => {
                self.registers.b = self.set(3, self.registers.b);
            }
            0xd9 => {
                self.registers.c = self.set(3, self.registers.c);
            }
            0xda => {
                self.registers.d = self.set(3, self.registers.d);
            }
            0xdb => {
                self.registers.e = self.set(3, self.registers.e);
            }
            0xdc => {
                self.registers.h = self.set(3, self.registers.h);
            }
            0xdd => {
                self.registers.l = self.set(3, self.registers.l);
            }
            0xde => {
                let value = self.set(3, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }
            0xdf => {
                self.registers.a = self.set(3, self.registers.a);
            }
            0xe0 => {
                self.registers.b = self.set(4, self.registers.b);
            }
            0xe1 => {
                self.registers.c = self.set(4, self.registers.c);
            }
            0xe2 => {
                self.registers.d = self.set(4, self.registers.d);
            }
            0xe3 => {
                self.registers.e = self.set(4, self.registers.e);
            }
            0xe4 => {
                self.registers.h = self.set(4, self.registers.h);
            }
            0xe5 => {
                self.registers.l = self.set(4, self.registers.l);
            }
            0xe6 => {
                let value = self.set(4, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0xe7 => {
                self.registers.a = self.set(4, self.registers.a);
            }
            0xe8 => {
                self.registers.b = self.set(5, self.registers.b);
            }
            0xe9 => {
                self.registers.c = self.set(5, self.registers.c);
            }
            0xea => {
                self.registers.d = self.set(5, self.registers.d);
            }
            0xeb => {
                self.registers.e = self.set(5, self.registers.e);
            }
            0xec => {
                self.registers.h = self.set(5, self.registers.h);
            }
            0xed => {
                self.registers.l = self.set(5, self.registers.l);
            }
            0xee => {
                let value = self.set(5, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }
            0xef => {
                self.registers.a = self.set(5, self.registers.a);
            }

            0xf0 => {
                self.registers.b = self.set(6, self.registers.b);
            }
            0xf1 => {
                self.registers.c = self.set(6, self.registers.c);
            }
            0xf2 => {
                self.registers.d = self.set(6, self.registers.d);
            }
            0xf3 => {
                self.registers.e = self.set(6, self.registers.e);
            }
            0xf4 => {
                self.registers.h = self.set(6, self.registers.h);
            }
            0xf5 => {
                self.registers.l = self.set(6, self.registers.l);
            }
            0xf6 => {
                let value = self.set(6, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }

            0xf7 => {
                self.registers.a = self.set(6, self.registers.a);
            }
            0xf8 => {
                self.registers.b = self.set(7, self.registers.b);
            }
            0xf9 => {
                self.registers.c = self.set(7, self.registers.c);
            }
            0xfa => {
                self.registers.d = self.set(7, self.registers.d);
            }
            0xfb => {
                self.registers.e = self.set(7, self.registers.e);
            }
            0xfc => {
                self.registers.h = self.set(7, self.registers.h);
            }
            0xfd => {
                self.registers.l = self.set(7, self.registers.l);
            }
            0xfe => {
                let value = self.set(7, self.memory.read_byte(self.registers.get_hl()));
                self.memory.write_byte(value, self.registers.get_hl())
            }
            0xff => {
                self.registers.a = self.set(7, self.registers.a);
            }

            other => {}
        }
    }

    fn loope(&mut self) {
        while self.registers.pc != 0xffff {
            self.execute();
            println!("{:?}", self.registers);
        }
        /*for n in 1..100 {
            self.execute();
            println!("{:?}", self.registers);
        }*/
    }

    // bit wise CB operations

    fn bit(&mut self, bit: u8, register: u8) {
        println!("before: {}, after: {}", register, register >> bit);
        let val = (register >> bit) & 0x01;
        println!("{}", val);
        self.registers.f.zero = val == 0;
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

    fn add16(&mut self, value: u16) {
        let word = self.registers.get_hl();
        let (new_value, did_overflow) = word.overflowing_add(value);
        self.registers.set_hl(new_value);

        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        self.registers.f.half_carry = (word & 0x07ff) + (value & 0x07ff) > 0x07ff;
    }

    fn add16e(&mut self, register: u16) -> u16 {
        let byte = self.fetch() as i8 as i16 as u16;

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (register & 0x000f) + (byte & 0x000f) > 0x000f;
        self.registers.f.carry = (register & 0x00ff) + (byte & 0x00ff) > 0x00ff;
        register.wrapping_add(byte)
    }

    fn jump(&mut self) {
        let address = self.fetch() as i8;
        self.registers.pc = ((self.registers.pc as u32 as i32) + (address as i32)) as u16;
    }

    fn push(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        self.memory.write_word(value, self.registers.sp)
    }

    fn pop(&mut self) -> u16 {
        let result = self.memory.read_word(self.registers.sp);
        self.registers.sp += 2;
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
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    // Read boot-rom file
    let boot = std::fs::read("boot.bin").unwrap();

    let mut CPU = CPU::new();

    for (position, &byte) in boot.iter().enumerate() {
        //println!("{:X?}", byte);
        CPU.memory.write_byte(byte, position as u16);
    }

    CPU.loope();

    println!("{:?}", CPU.memory.read_word(CPU.registers.pc));

    //println!("{:?}", CPU.memory.ram)

    /*let bytes = std::fs::read("tetris.gb").unwrap();
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

    println!("{}", title);*/
}

/*    fn swap(&mut self, register: &mut u8) {
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
