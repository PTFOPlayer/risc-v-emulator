// opcode mask for type R: 0b11111110000000000111000001111111
// opcode mask for type I:                  0b111000001111111
// opcode mask for type S:                  0b111000001111111
// opcode mask for type B:                  0b111000001111111
// opcode mask for type U:                          0b1111111
// opcode mask for type J:                          0b1111111

#[repr(u32)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Instructions {
    Unknown = 0,
    //U type
    Lui = 0b0110111,
    Auipc = 0b0010111,
    //I type
    Addi = 0b000000000010011,
    Slti = 0b010000000010011,
    Sltiu = 0b011000000010011,
    Xori = 0b100000000010011,
    Ori = 0b110000000010011,
    Andi = 0b111000000010011,
    //R type
    Slli = 0b00000000000000000001000000010011,
    Srli = 0b00000000000000000101000000010011,
    Srai = 0b01000000000000000101000000010011,
    Add = 0b00000000000000000000000000110011,
    Sub = 0b01000000000000000000000000110011,
    Sll = 0b00000000000000000001000000110011,
    Slt = 0b00000000000000000010000000110011,
    Sltu = 0b00000000000000000011000000110011,
    Xor = 0b00000000000000000100000000110011,
    Srl = 0b00000000000000000101000000110011,
    Sra = 0b01000000000000000101000000110011,
    Or = 0b00000000000000000110000000110011,
    And = 0b00000000000000000111000000110011,
    //Other R type
    Fence = 0b00000000000000000000000000001111,
    FenceI = 0b00000000000000000001000000001111,
    //sys calls
    Ecall = 0b00000000000000000000000001110011,
    Ebreak = 0b00000000000100000000000001110011,
}

impl Instructions {
    fn parse(op: u32) -> Self {
        let u_type = match op & 0b1111111 {
            0b0110111 => Self::Lui,
            0b0010111 => Self::Auipc,
            _ => Self::Unknown,
        };

        let i_type = match op & 0b111000001111111 {
            0b000000000010011 => Self::Addi,
            0b010000000010011 => Self::Slti,
            0b011000000010011 => Self::Sltiu,
            0b100000000010011 => Self::Xori,
            0b110000000010011 => Self::Ori,
            0b111000000010011 => Self::Andi,
            _ => Self::Unknown,
        };

        // 0b11111110000000000111000001111111
        let r_type = match op & 0b11111110000000000111000001111111 {
            0b00000000000000000001000000010011 => Self::Slli,
            0b00000000000000000101000000010011 => Self::Srli,
            0b01000000000000000101000000010011 => Self::Srai,
            0b00000000000000000000000000110011 => Self::Add,
            0b01000000000000000000000000110011 => Self::Sub,
            0b00000000000000000001000000110011 => Self::Sll,
            0b00000000000000000010000000110011 => Self::Slt,
            0b00000000000000000011000000110011 => Self::Sltu,
            0b00000000000000000100000000110011 => Self::Xor,
            0b00000000000000000101000000110011 => Self::Srl,
            0b01000000000000000101000000110011 => Self::Sra,
            0b00000000000000000110000000110011 => Self::Or,
            0b00000000000000000111000000110011 => Self::And,
            _ => Self::Unknown,
        };

        let o_r_type = match op & 0b11111110000000000111000001111111 {
            0b00000000000000000000000000001111 => Self::Fence,
            0b00000000000000000001000000001111 => Self::FenceI,
            0b00000000000000000000000001110011 => Self::Ecall,
            0b00000000000100000000000001110011 => Self::Ebreak,
            _ => Self::Unknown,
        };

        match (u_type, i_type, r_type, o_r_type) {
            (Self::Unknown, Self::Unknown, Self::Unknown, a) => a,
            (Self::Unknown, Self::Unknown, a, Self::Unknown) => a,
            (Self::Unknown, a, Self::Unknown, Self::Unknown) => a,
            (a, Self::Unknown, Self::Unknown, Self::Unknown) => a,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    pub instruction_raw: u32,
    pub instruction: Instructions,
}

use crate::fast_transmute;

pub fn get_instructions(prog_bits: &[u8]) -> Vec<Instruction> {
    let mut instructions = vec![];
    for i in (0..prog_bits.len()).step_by(4) {
        let temp = fast_transmute!(<0, u32>, [prog_bits[i+0], prog_bits[i+1], prog_bits[i+2], prog_bits[i+3]]);
        instructions.push(Instruction { instruction_raw: temp, instruction: Instructions::parse(temp) });
    }

    return instructions;
}
