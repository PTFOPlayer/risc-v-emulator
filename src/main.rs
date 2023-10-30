mod elf_parser;
mod error;
mod instruction;
mod misc;
use std::cell::RefCell;

use crate::{
    elf_parser::{extract_prog_bits, program_header_parser, raw_section_header_parser},
    instruction::{get_instructions, Instructions},
};
use error::*;

thread_local! {
    static REGISTERS: RefCell<[u64;32]> = RefCell::new([0;32]);
    static PC: RefCell<usize> = RefCell::new(0);
}

fn main() -> Result<(), EmulatorError> {
    let data = std::fs::read("./a.out")?;

    let elf = elf_parser::elf_parser(&data);
    println!("{:?}\n\n", elf);

    let program_headers = program_header_parser(&data, &elf);
    println!("{}, {:?}\n\n", program_headers.len(), program_headers);

    let mut section_headers = raw_section_header_parser(&data, &elf);
    section_headers.fill_names(&data)?;
    println!("{}, {:?}\n\n", section_headers.len(), section_headers);

    let text = section_headers.find_text_section().unwrap();
    println!("{:?}", text);

    let instructions = get_instructions(extract_prog_bits(&data, text)?);

    const U_TYPE_RD: u32 = 0b111110000000;
    const U_TYPE_IMM: u32 = !0b111111111111;

    const I_TYPE_RD: u32 = 0b111110000000;
    const I_TYPE_RS: u32 = 0b11111000000000000000;
    const I_TYPE_IMM: u32 = !0b11111111111111111111;

    const A0: usize = 10;
    const A1: usize = A0 + 1;
    const A2: usize = A1 + 1;
    const A3: usize = A2 + 1;
    const A4: usize = A3 + 1;
    const A5: usize = A4 + 1;
    const A6: usize = A5 + 1;
    const A7: usize = A6 + 1;

    for i in instructions {
        let raw = i.instruction_raw;
        let inst = &i.instruction;
        match inst {
            Instructions::Unknown => panic!("unknown instruction"),
            Instructions::Lui => {
                let rd = (raw & U_TYPE_RD) >> 7;
                let imm = raw & U_TYPE_IMM;
                set_reg(rd as usize, imm as u64);
                inc_pc();
            }
            Instructions::Auipc => {}
            Instructions::Addi => {
                let rd = (raw & I_TYPE_RD) >> 7;
                let rs = (raw & I_TYPE_RS) >> 15;
                let imm = (raw & I_TYPE_IMM) >> 20;
                set_reg(rd as usize, read_reg(rs as usize) + imm as u64);
                inc_pc();
            }
            Instructions::Slti => {}
            Instructions::Sltiu => {}
            Instructions::Xori => {}
            Instructions::Ori => {}
            Instructions::Andi => {}
            Instructions::Slli => {}
            Instructions::Srli => {}
            Instructions::Srai => {}
            Instructions::Add => {}
            Instructions::Sub => {}
            Instructions::Sll => {}
            Instructions::Slt => {}
            Instructions::Sltu => {}
            Instructions::Xor => {}
            Instructions::Srl => {}
            Instructions::Sra => {}
            Instructions::Or => {}
            Instructions::And => {}
            Instructions::Fence => {}
            Instructions::FenceI => {}
            Instructions::Ecall => match read_reg(A0) {
                1 => {

                }
                _ => {}
            },
            Instructions::Ebreak => {}
        };
        dbg_reg();
    }

    Ok(())
}

#[inline(always)]
fn inc_pc() {
    PC.with(|x| {
        *x.borrow_mut() += 4;
    });
}

#[inline(always)]
fn set_pc(pc: usize) {
    PC.with(|x| {
        *x.borrow_mut() += pc;
    });
}

#[inline(always)]
fn get_pc(pc: usize) -> usize {
    PC.with(|x| *x.borrow())
}

#[inline(always)]
fn set_reg(reg: usize, val: u64) {
    REGISTERS.with(|x| {
        x.borrow_mut()[reg] = val;
    });
}

#[inline(always)]
fn read_reg(reg: usize) -> u64 {
    REGISTERS.with(|x| x.borrow()[reg])
}

#[inline(always)]
fn dbg_reg() {
    REGISTERS.with(|x| {
        let temp = x.borrow();
        for i in (0..temp.len()).step_by(8) {
            for j in 0..8 {
                print!("x{}: {} \t", i + j, temp[i + j]);
            }
            println!();
        }
    })
}
