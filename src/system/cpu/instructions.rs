use fundsp::funutd::map3::Shift;

use super::registers::Register;

type ID = u8;
enum Opcode {
    RST,
    RET,
    JP,
    CALL,

    // All the ALU and shift operations.
    ALU(Operation, Operand),

    INC(Operand),
    DEC(Operand),

    NOP,
    STOP,

    DAA,

    BIT(Operand, Operand),
    RES(Operand, Operand),
    SET(Operand, Operand),

    PUSH(Operand),
    POP(Operand),

    DI,
    EI,

    CPL,
    CCF,

    RRA,
    RRCA,

    JR(Operand, Operand),
    LD(Operand, Operand),

    RLA,
    RLCA,

    SCF,

    HALT,

    LDH(Operand, Operand),

    CB,

    ADDHL(RP),
}

enum Operation {
    Logic(ID),
    Shift(ID),
}

enum Operand8 {
    Register(ID),
    DoubleRegister(ID),
    D8,
    A8,
}

enum Operand {
    D8,
    D16,
    A16,
    A8,
    E8,
    Bit(ID),
    Flag(ID),
    C,
    Register(ID),
    DoubleRegister(ID),
}

enum R {
    B = 0,
    C = 1,
    D = 2,
    E = 3,
    H = 4,
    L = 5,
    HL = 6,
    A = 7,
}

enum RP {
    BC = 0,
    DE = 1,
    HL = 2,
    SP = 3,
}

enum RP2 {
    BC = 0,
    DE = 1,
    HL = 2,
    AF = 3,
}

enum CC {
    NZ = 0,
    Z = 1,
    NC = 2,
    C = 3,
}

enum ALU {
    ADD = 0,
    ADC = 1,
    SUB = 2,
    SBC = 3,
    AND = 4,
    XOR = 5,
    OR = 6,
    CP = 7,
}

enum ROT {
    RLC = 0,
    RRC = 1,
    RL = 2,
    RR = 3,
    SLA = 4,
    SRA = 5,
    SWAP = 6,
    SRL = 7,
}

pub fn to_instruction(byte: u8) -> Opcode {
    let x = byte >> 6;
    let y = (byte >> 3) & 0x07;
    let z = byte & 0x07;
    let q = y & 0x01;
    let p = (y >> 1) & 0x01;

    match x {
        0 => match z {
            0 => {}
            1 => {
                if q == 1 {
                } else {
                }
            }
            2 => {}
            3 => {}
            4 => {}
            5 => {}
            6 => {}
            7 => {}
            _ => panic!("z is not valid."),
        },
        1 => {}
        2 => {}
        3 => match z {
            0 => {}
            1 => {}
            2 => {}
            3 => {}
            4 => {}
            5 => {}
            6 => {}
            7 => {}
            _ => panic!("x is not valid."),
        },
        _ => panic!("x is not valid."),
    };

    match byte {
        0x00 => Opcode::NOP,
        0x10 => Opcode::STOP,

        0x01 | 0x11 | 0x21 | 0x31 => {
            let dest = (byte >> 4) & 0x03;
            Opcode::LD(Operand::DoubleRegister(dest), Operand::D16)
        }

        0x02 | 0x12 | 0x22 | 0x32 => Opcode::STOP,

        0x40..=0x7f => {
            let source = byte & 0x07;
            let dest = (byte >> 3) & 0x07;
            Opcode::LD(Operand::Register(dest), Operand::Register(source))
        }
        0x80..=0xbf => {
            let register = byte & 0x07;
            let operation = (byte >> 3) & 0x07;
            Opcode::ALU(Operation::Logic(operation), Operand::Register(register))
        }
        0xc0..=0xff => Opcode::STOP,
    }
}

pub fn to_instruction_cb(byte: u8) -> Opcode {
    match byte {
        0x00..=0x3f => {
            let register = byte & 0x07;
            let operation = (byte >> 3) & 0x07;
            Opcode::ALU(Operation::Shift(operation), Operand::Register(register))
        }
        0x40..=0x7f => {
            let register = byte & 0x07;
            let bit = (byte >> 3) & 0x07;
            Opcode::BIT(Operand::Bit(bit), Operand::Register(register))
        }

        0x80..=0xbf => {
            let register = byte & 0x07;
            let bit = (byte >> 3) & 0x07;
            Opcode::RES(Operand::Bit(bit), Operand::Register(register))
        }

        0xc0..=0xff => {
            let register = byte & 0x07;
            let bit = (byte >> 3) & 0x07;
            Opcode::SET(Operand::Bit(bit), Operand::Register(register))
        }
    }
}
