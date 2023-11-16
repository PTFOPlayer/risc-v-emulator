mod dram;
mod elf_parser;
mod error;
mod instruction;
mod misc;
use crate::{instruction::instruction::{get_instructions, Instructions}, misc::{dbg_reg, dbg_stack}};
use std::{cell::RefCell, mem::transmute, process::exit};

use crate::{
    dram::Dram,
    elf_parser::{extract_prog_bits, program_header_parser, raw_section_header_parser},
};
use error::*;

const DEBUG: bool = false;
const P_INST: bool = false;
const P_PC: bool = false;
const P_REG: bool = false;
const P_STACK: bool = false;
const P_STACK_SIZE: u64 = 64;

thread_local! {
    pub static REGISTERS: RefCell<[u64;32]> = RefCell::new([0;32]);
    pub static PC: RefCell<u32> = RefCell::new(0);
}

//stack pointer register index
const SP: usize = 2;
// global pointer register index
const GP: usize = 3;

// zero register index
const ZERO: usize = 0;

// A registers indexes
const A0: usize = 10;
const A1: usize = A0 + 1;
const A2: usize = A1 + 1;
const A3: usize = A2 + 1;
const A4: usize = A3 + 1;
const A5: usize = A4 + 1;
const A6: usize = A5 + 1;
const A7: usize = A6 + 1;

fn main() -> Result<(), EmulatorError> {
    let data = std::fs::read("./test_asm/a.out")?;

    let elf = elf_parser::elf_parser(&data);

    let program_headers = program_header_parser(&data, &elf);

    let mut section_headers = raw_section_header_parser(&data, &elf);
    section_headers.fill_names(&data)?;

    let text = section_headers.find_text_section()?;

    if DEBUG {
        println!("{:?}\n\n", elf);
        println!("{}, {:?}\n\n", program_headers.len(), program_headers);
        for i in 0..section_headers.len() {
            println!("{:?}", section_headers.headers[i]);
        }
        println!("{:?}", text);
        println!("text addr: {:x?}", text.section_address);
    }

    // creating dram
    let mut dram = Dram::new_dram();
    // setting starting PC
    set_pc!(text.section_address);
    let prog_bits = extract_prog_bits(&data, text)?;

    // copying program_bits into dram
    let mut i = text.section_address as usize;
    for e in prog_bits {
        dram.set_u8(i, *e);
        i += 1;
    }

    // processing data section
    match section_headers.find_data_section() {
        Some(res) => {
            let mut i = res.section_address as usize;

            let start = res.section_offset as usize;
            // setting up global pointer ( start of data section )
            set_reg!(GP, i);
            let end = start + res.section_size as usize;
            for e in &data[start..end] {
                dram.set_u8(i, *e);
                i += 1;
            }
            if DEBUG {
                println!("data addr: {:x}", res.section_address);
                println!("{:?}", &data[start..end]);
            }
        }
        None => {}
    }

    loop {
        let i = get_instructions(&dram, get_pc!());
        let raw = i.instruction_raw;
        let inst = &i.instruction;
        if DEBUG || P_INST {
            println!("Inst: {:?}", inst);
        }
        if DEBUG || P_PC {
            println!("PC: {:x?}", get_pc!());
        }
        match inst {
            Instructions::Unknown => println!("unknown instruction: {:x}", raw),
            Instructions::Lui => {
                set_reg!(rd!(raw), imm!(U, raw) << 12);
            }
            Instructions::Auipc => {
                set_reg!(
                    rd!(raw),
                    (get_pc!() as i64).wrapping_add((imm!(U, raw) << 12) as i64)
                );
            }
            Instructions::Addi => {
                let rd = rd!(raw);
                let rs = rs1!(raw);
                set_reg!(rd, (read_reg!(rs) as i64).wrapping_add(imm!(I, raw) as i64));
            }
            Instructions::Slti => {
                let rd = rd!(raw);
                let rs = rs1!(raw);

                set_reg!(rd, (read_reg!(rs) as i64) << imm!(I, raw));
            }
            Instructions::Sltiu => {
                let rd = rd!(raw);
                let rs = rs1!(raw);

                set_reg!(rd, (read_reg!(rs) as u64) << imm!(I, raw));
            }
            Instructions::Xori => {
                let rd = rd!(raw);
                let rs = rs1!(raw);
                set_reg!(rd, read_reg!(rs) ^ imm!(I, raw) as u64);
            }
            Instructions::Ori => {
                let rd = rd!(raw);
                let rs = rs1!(raw);
                set_reg!(rd, read_reg!(rs) | imm!(I, raw) as u64);
            }
            Instructions::Andi => {
                let rd = rd!(raw);
                let rs = rs1!(raw);
                set_reg!(rd, read_reg!(rs) & imm!(I, raw) as u64);
            }
            Instructions::Lb => {
                let rd = rd!(raw);
                let rs = read_reg!(rs1!(raw));
                let imm = imm!(I, raw);
                let data = unsafe {
                    transmute::<u8, i8>(dram.get_u8(rs.wrapping_add(imm as u64) as usize)) as i64
                };
                set_reg!(rd, data);
            }
            Instructions::Lh => {
                let rd = rd!(raw);
                let rs = read_reg!(rs1!(raw));
                let imm = imm!(I, raw);
                let data = unsafe {
                    transmute::<u16, i16>(dram.get_u16(rs.wrapping_add(imm as u64) as usize))
                };
                set_reg!(rd, data);
            }
            Instructions::Lw => {
                let rd = rd!(raw);
                let rs = read_reg!(rs1!(raw));
                let imm = imm!(I, raw);
                let data = unsafe {
                    transmute::<u32, i32>(dram.get_u32(rs.wrapping_add(imm as u64) as usize))
                };
                set_reg!(rd, data);
            }
            Instructions::Lbu => {
                let rd = rd!(raw);
                let rs = read_reg!(rs1!(raw));
                let imm = imm!(I, raw);
                let addr = (rs.wrapping_add(imm as u64)) as usize;
                set_reg!(rd, dram.get_u8(addr));
            }
            Instructions::Lhu => {
                let rd = rd!(raw);
                let rs = read_reg!(rs1!(raw));
                let imm = imm!(I, raw);
                let addr = (rs.wrapping_add(imm as u64)) as usize;
                set_reg!(rd, dram.get_u16(addr));
            }
            Instructions::Sb => {
                let rs1 = read_reg!(rs1!(raw));
                let rs2 = read_reg!(rs2!(raw));
                let imm = imm!(S, raw);
                let addr = (rs1 + imm as u64) as usize;
                let val = (rs2 & 0xFF) as u8;
                dram.set_u8(addr, val);
            }
            Instructions::Sh => {
                let rs1 = read_reg!(rs1!(raw));
                let rs2 = read_reg!(rs2!(raw));
                let imm = imm!(S, raw);
                let addr = (rs1 + imm as u64) as usize;
                let val = (rs2 & 0xFFFF) as u16;
                dram.set_u16(addr, val);
            }
            Instructions::Sw => {
                let rs1 = read_reg!(rs1!(raw));
                let rs2 = read_reg!(rs2!(raw));
                let imm = imm!(S, raw);
                let addr = (rs1 + imm as u64) as usize;
                let val = (rs2 & 0xFFFFFFFF) as u32;
                dram.set_u32(addr, val);
            }
            Instructions::Slli => {}
            Instructions::Srli => {}
            Instructions::Srai => {}
            Instructions::Add => {
                let rd = rd!(raw);
                let rs1 = rs1!(raw);
                let rs2 = rs2!(raw);
                set_reg!(rd, read_reg!(rs1).wrapping_add(read_reg!(rs2)));
            }
            Instructions::Sub => {
                let rd = rd!(raw);
                let rs1 = rs1!(raw);
                let rs2 = rs2!(raw);
                set_reg!(rd, read_reg!(rs1).wrapping_sub(read_reg!(rs2)));
            }
            Instructions::Sll => {
                let rd = rd!(raw);
                let rs1 = rs1!(raw);
                let rs2 = rs2!(raw);
                set_reg!(rd, read_reg!(rs1) << read_reg!(rs2));
            }
            Instructions::Slt => {
                let rd = rd!(raw);
                let rs1 = rs1!(raw);
                let rs2 = rs2!(raw);
                set_reg!(rd, (read_reg!(rs1) as i64) < (read_reg!(rs2) as i64));
            }
            Instructions::Sltu => {
                let rd = rd!(raw);
                let rs1 = rs1!(raw);
                let rs2 = rs2!(raw);
                set_reg!(rd, read_reg!(rs1) < read_reg!(rs2));
            }
            Instructions::Xor => {
                let rd = rd!(raw);
                let rs1 = rs1!(raw);
                let rs2 = rs2!(raw);
                set_reg!(rd, read_reg!(rs1) ^ read_reg!(rs2));
            }
            Instructions::Srl => {
                let rd = rd!(raw);
                let rs1 = rs1!(raw);
                let rs2 = rs2!(raw);
                set_reg!(rd, read_reg!(rs1) >> read_reg!(rs2));
            }
            Instructions::Sra => {
                let rd = rd!(raw);
                let rs1 = rs1!(raw);
                let rs2 = rs2!(raw);
                set_reg!(rd, (read_reg!(rs1) as i64) >> (read_reg!(rs2) as i64));
            }
            Instructions::Or => {
                let rd = rd!(raw);
                let rs1 = rs1!(raw);
                let rs2 = rs2!(raw);
                set_reg!(rd, read_reg!(rs1) | read_reg!(rs2));
            }
            Instructions::And => {
                let rd = rd!(raw);
                let rs1 = rs1!(raw);
                let rs2 = rs2!(raw);
                set_reg!(rd, read_reg!(rs1) & read_reg!(rs2));
            }
            Instructions::Fence => {}
            Instructions::FenceI => {}
            Instructions::Ecall => match (read_reg!(A0), read_reg!(A7)) {
                (1, 64) => {
                    let addr = read_reg!(A1) as usize;
                    let len = read_reg!(A2) as usize;
                    let s = String::from_utf8_lossy(&dram[addr..addr + len]);
                    if DEBUG {
                        println!("{:x}", addr);
                        println!("ecall: print\n{:?}", s);
                    } else {
                        print!("{}", s);
                    }
                }
                _ => {}
            },
            Instructions::Ebreak => {}
            Instructions::Jal => {
                let imm = ((raw as i32 >> 11) & (1 << 20))
                    | ((raw as i32 >> 20) & 0x7FE)
                    | ((raw as i32 >> 9) & (1 << 11))
                    | (raw as i32 & 0xFF000);
                let imm = (imm << 11) >> 11;

                let temp_pc = get_pc!() as i32;
                let rd = rd!(raw);
                set_reg!(rd, get_pc!() + 4);
                set_pc!(temp_pc.wrapping_add(imm).wrapping_sub(4));
            }
            Instructions::Jalr => {}
            Instructions::Beq => {
                branch!(raw, ==);
            }
            Instructions::Bne => {
                branch!(raw, !=);
            }
            Instructions::Blt => {
                branch!(raw, <, int);
            }
            Instructions::Bge => {
                branch!(raw, >=, int);
            }
            Instructions::Bltu => {
                branch!(raw, <);
            }
            Instructions::Bgeu => {
                branch!(raw, >=);
            }
        };

        inc_pc!();
        if DEBUG || P_REG {
            dbg_reg();
        }
        if DEBUG || P_STACK {
            dbg_stack(SP, P_STACK_SIZE, &mut dram);
        }

        if read_reg!(ZERO) != 0 {
            set_reg!(ZERO, 0);
        };

        if get_pc!() as u64 >= text.section_address + text.section_size {
            println!("end");
            println!("pc: {:x}", get_pc!());
            println!("sp: {:x}", read_reg!(SP));
            exit(0);
        }

    }
}

#[macro_export]
macro_rules! inc_pc {
    () => {
        PC.with(|x| {
            *x.borrow_mut() += 4;
        });
    };
}

#[macro_export]
macro_rules! set_pc {
    ($pc: expr) => {
        PC.with(|x| {
            *x.borrow_mut() = $pc as u32;
        });
    };
}

#[macro_export]
macro_rules! get_pc {
    () => {
        PC.with(|x| *x.borrow())
    };
}

#[macro_export]
macro_rules! set_reg {
    ($reg: expr, $val: expr) => {
        REGISTERS.with(|x| {
            x.borrow_mut()[$reg as usize] = $val as u64;
        });
    };
}

#[macro_export]
macro_rules! read_reg {
    ($reg: expr) => {
        REGISTERS.with(|x| x.borrow()[$reg as usize])
    };
}