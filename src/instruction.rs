// opcode mask for type R: 0b11111110000000000111000001111111
// opcode mask for type I:                  0b111000001111111
// opcode mask for type S:                  0b111000001111111
// opcode mask for type B:                  0b111000001111111
// opcode mask for type U:                          0b1111111
// opcode mask for type J:                          0b1111111

#[repr(u32)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Instructions {
    Unknown,
    //U type
    Lui,
    Auipc,
    //I type
    Addi,
    Slti,
    Sltiu,
    Xori,
    Ori,
    Andi,
    //R type
    Slli,
    Srli,
    Srai,
    Add,
    Sub,
    Sll,
    Slt,
    Sltu,
    Xor,
    Srl,
    Sra,
    Or,
    And,
    //Other R type
    Fence,
    FenceI,
    //sys calls
    Ecall,
    Ebreak,
    // jump / J type
    Jal,
    Jalr,
    Beq,
    Bne,
    Blt,
    Bge,
    Bltu,
    Bgeu,
}

impl Instructions {
    fn parse(op: u32) -> Self {
        let instruction_type = op & 0x7F;
        match instruction_type {
            // shifts
            0b00000000000000000001000000010011 => Self::Slli,
            0b00000000000000000101000000010011 => Self::Srli,
            0b01000000000000000101000000010011 => Self::Srai,
            // u_type
            0b0110111 => Self::Lui,
            0b0010111 => Self::Auipc,
            // j_type
            0b1101111 => Self::Jal,
            0b1100111 => Self::Jalr,
            // i_type
            0b0010011 => {
                let funct = op >> 12 & 0x7;
                match funct {
                    0b000 => Self::Addi,
                    0b010 => Self::Slti,
                    0b011 => Self::Sltiu,
                    0b100 => Self::Xori,
                    0b110 => Self::Ori,
                    0b111 => Self::Andi,
                    _ => Self::Unknown,
                }
            }
            // r_type
            0b0110011 => {
                let funct3 = op >> 12 & 0x7;
                let funct7 = op >> 24 & 0x7F;
                match (funct7, funct3) {
                    (0b0000000, 000) => Self::Add,
                    (0b0100000, 000) => Self::Sub,
                    (0b0000000, 001) => Self::Sll,
                    (0b0000000, 010) => Self::Slt,
                    (0b0000000, 011) => Self::Sltu,
                    (0b0000000, 100) => Self::Xor,
                    (0b0000000, 101) => Self::Srl,
                    (0b0100000, 101) => Self::Sra,
                    (0b0000000, 110) => Self::Or,
                    (0b0000000, 111) => Self::And,
                    (_, _) => Self::Unknown,
                }
            }
            // b_type
            0b1100011 => {
                let funct3 = op >> 12 & 0x7;
                match funct3 {
                    000 => Self::Beq,
                    001 => Self::Bne,
                    100 => Self::Blt,
                    101 => Self::Bge,
                    110 => Self::Bltu,
                    111 => Self::Bgeu,
                    _ => Self::Unknown,
                }
            }
            // fence
            0b00000000000000000000000000001111 => Self::Fence,
            0b00000000000000000001000000001111 => Self::FenceI,
            //calls
            0b00000000000000000000000001110011 => Self::Ecall,
            0b00000000000100000000000001110011 => Self::Ebreak,
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

pub fn get_instructions(prog_bits: &[u8], pc: u32) -> Instruction {
    let pc = pc as usize;
    let temp = fast_transmute!(<0, u32>, [prog_bits[pc+0], prog_bits[pc+1], prog_bits[pc+2], prog_bits[pc+3]]);
    Instruction {
        instruction_raw: temp,
        instruction: Instructions::parse(temp),
    }
}
