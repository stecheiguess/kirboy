use std::env;
//use crate::gpu::GPU;
use emulator::cpu::CPU;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    // Read boot-rom file
    let boot = std::fs::read("boot.bin").unwrap();

    let mut CPU = CPU::new();

    for (position, &byte) in boot.iter().enumerate() {
        //println!("{:X?}", byte);
        CPU.mmu.write_byte(byte, position as u16);
    }

    println!("{:?}", CPU.registers);

    while !CPU.halted {
        CPU.cycle();

        //println!("{:?}", CPU.ime);
    }

    println!("{:?}", CPU.registers);

    //println!("{:?}", CPU.mmu.read_word(CPU.registers.pc));

    //println!("{:?}", CPU.mmu.ram)

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
        CPU.mmu.write_byte(byte, (position as u16) + 0x4000);
    }

    println!("{:b}", CPU.mmu.ram[0x41a2]);
    println!("{:b}", CPU.set(4, CPU.mmu.ram[0x41a2]));

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
