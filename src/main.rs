mod elf_parser;
mod error;
mod instruction;
mod misc;
mod dram;
use std::cell::RefCell;

use crate::{
    elf_parser::{extract_prog_bits, program_header_parser, raw_section_header_parser},
    instruction::{get_instructions, Instructions}, dram::Dram,
};
use error::*;

const DEBUG: bool = true;

thread_local! {
    static REGISTERS: RefCell<[u64;32]> = RefCell::new([0;32]);
    static PC: RefCell<u32> = RefCell::new(0);
}

const DRAM_SIZE: usize = 256 * 1024 * 1024;

#[inline(always)]
fn __extract_branch(raw: u32) -> (i32, u32, u32) {
    let imm = (raw as i32 >> 7 & 0x1E)
        | (raw as i32 >> 22 & 0x3F << 5)
        | (raw as i32 & 0x100 << 2)
        | ((raw as i32 >> 31 & 1) << 11);
    let imm = (imm << (32 - 12)) >> (32 - 12);
    let rs1 = raw >> 15 & 0x1F;
    let rs2 = raw >> 20 & 0x1F;
    (imm, rs1, rs2)
}

macro_rules! branch {
    ($raw: expr, $e: tt) => {
        let (imm, rs1, rs2) = __extract_branch($raw as u32);
        if (read_reg!(rs1) as u32) $e (read_reg!(rs2) as u32) {
            let temp_pc = get_pc!() as i32;
            set_pc!(temp_pc.wrapping_add(imm).wrapping_sub(4));
        }
    };
    ($raw: expr, $e: tt, int) => {
        let (imm, rs1, rs2) = __extract_branch($raw as u32);
        if (read_reg!(rs1) as i32) $e (read_reg!(rs2) as i32) {
            let temp_pc = get_pc!() as i32;
            set_pc!(temp_pc.wrapping_add(imm).wrapping_sub(4));
        }
    };
}

macro_rules! rd {
    ($raw: expr) => {
        ($raw as u32 >> 7) & 0x1F
    };
}

macro_rules! rs1 {
    ($raw: expr) => {
        ($raw as u32 >> 15) & 0x1F
    };
}

macro_rules! rs2 {
    ($raw: expr) => {
        ($raw as u32 >> 20) & 0x1F
    };
}

macro_rules! imm {
    (I, $raw: expr) => {
        ($raw as u32 >> 20) & 0x7FF
    };
    (U, $raw: expr) => {
        ($raw as u32 >> 12) & 0x7FFFF
    };
}

fn main() -> Result<(), EmulatorError> {
    let mut dram = Dram::new_dram();
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

    set_pc!(text.section_address);
    let prog_bits = extract_prog_bits(&data, text)?;

    let mut i = text.section_address as usize;
    for e in prog_bits {
        dram.set_u8(i, *e);
        i += 1;
    }

    match section_headers.find_data_section() {
        Some(res) => {
            let mut i = res.section_address as usize;

            let start = res.section_offset as usize;
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

    const ZERO: usize = 0;

    const A0: usize = 10;
    const A1: usize = A0 + 1;
    const A2: usize = A1 + 1;
    const A3: usize = A2 + 1;
    const A4: usize = A3 + 1;
    const A5: usize = A4 + 1;
    const A6: usize = A5 + 1;
    const A7: usize = A6 + 1;

    loop {
        let i = get_instructions(&dram, get_pc!());
        let raw = i.instruction_raw;
        let inst = &i.instruction;
        println!("{:?}", inst);
        match inst {
            Instructions::Unknown => panic!("unknown instruction: {}", raw),
            Instructions::Lui => {
                set_reg!(rd!(raw), imm!(U, raw) << 12);
            }
            Instructions::Auipc => {
                set_reg!(rd!(raw), get_pc!() + (imm!(U, raw) << 12));
            }
            Instructions::Addi => {
                let rd = rd!(raw);
                let rs = rs1!(raw);
                set_reg!(rd, read_reg!(rs) + imm!(I, raw) as u64);
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
            Instructions::Slli => {}
            Instructions::Srli => {}
            Instructions::Srai => {}
            Instructions::Add => {
                let rd = rd!(raw);
                let rs1 = rs1!(raw);
                let rs2 = rs2!(raw);
                set_reg!(rd, read_reg!(rs1) + read_reg!(rs2));
            }
            Instructions::Sub => {
                let rd = rd!(raw);
                let rs1 = rs1!(raw);
                let rs2 = rs2!(raw);
                set_reg!(rd, read_reg!(rs1) - read_reg!(rs2));
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
                        println!("{}", s);
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

        dbg_reg();

        if read_reg!(ZERO) != 0 {
            set_reg!(ZERO, 0);
        };

        if get_pc!() as u64 >= text.section_address + text.section_size {
            panic!("end, pc: {:x}", get_pc!());
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

#[inline(always)]
fn dbg_reg() {
    REGISTERS.with(|x| {
        let temp = x.borrow();
        for i in (0..temp.len()).step_by(8) {
            for j in 0..8 {
                print!("x{:02}: 0x{:x} ", i + j, temp[i + j]);
            }
            println!();
        }
    })
}
