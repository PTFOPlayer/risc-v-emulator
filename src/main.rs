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

const DEBUG: bool = false;

thread_local! {
    static REGISTERS: RefCell<[u64;32]> = RefCell::new([0;32]);
    static PC: RefCell<u32> = RefCell::new(0);
}

const DRAM_SIZE: usize = 64 * 1024 * 1024;

fn main() -> Result<(), EmulatorError> {
    let mut dram = vec![0u8; DRAM_SIZE];
    let data = std::fs::read("./a.out")?;

    let elf = elf_parser::elf_parser(&data);

    let program_headers = program_header_parser(&data, &elf);

    let mut section_headers = raw_section_header_parser(&data, &elf);
    section_headers.fill_names(&data)?;

    let text = section_headers.find_text_section().unwrap();

    if DEBUG {
        println!("{:?}\n\n", elf);
        println!("{}, {:?}\n\n", program_headers.len(), program_headers);
        for i in 0..section_headers.len() {
            println!("{:?}", section_headers.headers[i]);
        }
        println!("{:?}", text);
        println!("adddr: {:x?}", text.section_address);
    }

    let prog_bits = extract_prog_bits(&data, text)?;

    set_pc!(text.section_address);
    let mut i = text.section_address as usize;
    for e in prog_bits {
        dram[i] = *e;
        i += 1;
    }

    const RD: u32 = 0b111110000000;

    const U_TYPE_RD: u32 = 0b111110000000;
    const U_TYPE_IMM: u32 = !0b111111111111;

    const J_TYPE_BITS_12_19: i32 = 0xff000;
    // const J_TYPE_BIT_11: i32 = 0b10000000000000000000;
    // const J_TYPE_BIT_20: i32 = 0b10000000000000000000000000000000;
    // const J_TYPE_BITS_1_10: i32 = 0b01111111111100000000000000000000;

    const I_TYPE_RS: u32 = 0b11111000000000000000;
    const I_TYPE_IMM: u32 = !0b11111111111111111111;

    const ZERO: usize = 0;

    const A0: usize = 10;
    const A1: usize = A0 + 1;
    const A2: usize = A1 + 1;
    const A3: usize = A2 + 1;
    const A4: usize = A3 + 1;
    const A5: usize = A4 + 1;
    const A6: usize = A5 + 1;
    const A7: usize = A6 + 1;

    macro_rules! branch {
        ($raw: expr, $e: tt) => {
            let imm = ($raw as i32 >> 7 & 0x1E)
                | ($raw as i32 >> 22 & 0x3F << 5)
                | ($raw as i32 & 0x100 << 2)
                | (($raw as i32 >> 31 & 1) << 11);
            let imm = (imm << (32 - 12)) >> (32 - 12);
            let rs1 = $raw >> 15 & 0x1F;
            let rs2 = $raw >> 20 & 0x1F;
            if (read_reg!(rs1) as u32) $e (read_reg!(rs2) as u32) {
                let temp_pc = get_pc!() as i32;
                set_pc!(temp_pc.wrapping_add(imm).wrapping_sub(4));
            }
        };
        ($raw: expr, $e: tt, int) => {
            let imm = ($raw as i32 >> 7 & 0x1E)
                | ($raw as i32 >> 22 & 0x3F << 5)
                | ($raw as i32 & 0x100 << 2)
                | (($raw as i32 >> 31 & 1) << 11);
            let imm = (imm << (32 - 12)) >> (32 - 12);
            let rs1 = $raw >> 15 & 0x1F;
            let rs2 = $raw >> 20 & 0x1F;
            if (read_reg!(rs1) as i32) $e (read_reg!(rs2) as i32) {
                let temp_pc = get_pc!() as i32;
                set_pc!(temp_pc.wrapping_add(imm).wrapping_sub(4));
            }
        };
    }

    loop {
        let i = get_instructions(&dram, get_pc!());
        let raw = i.instruction_raw;
        let inst = &i.instruction;
        match inst {
            Instructions::Unknown => panic!("unknown instruction: {}", raw),
            Instructions::Lui => {
                let rd = (raw & U_TYPE_RD) >> 7;
                let imm = raw & U_TYPE_IMM;
                set_reg!(rd, imm);
            }
            Instructions::Auipc => {
                let rd = (raw & U_TYPE_RD) >> 7;
                let imm = (raw & U_TYPE_IMM) + get_pc!();
                set_reg!(rd, imm);
            }
            Instructions::Addi => {
                let rd = (raw & RD) >> 7;
                let rs = (raw & I_TYPE_RS) >> 15;
                let imm = (raw & I_TYPE_IMM) >> 20;
                set_reg!(rd, read_reg!(rs as usize) + imm as u64);
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
            Instructions::Ecall => match (read_reg!(A0), read_reg!(A7)) {
                (1, 64) => {
                    println!("call")
                }
                _ => {}
            },
            Instructions::Ebreak => {}
            Instructions::Jal => {
                let imm = ((raw as i32 >> (31 - 20)) & (1 << 20))
                    | ((raw as i32 >> (21 - 1)) & 0x7fe)
                    | ((raw as i32 >> (20 - 11)) & (1 << 11))
                    | (raw as i32 & J_TYPE_BITS_12_19);
                let imm = (imm << 11) >> 11;

                let temp_pc = get_pc!() as i32;
                let rd = (raw & RD) >> 7;
                set_reg!(rd, get_pc!() + 4);
                // println!("0x{:x}", temp_pc.wrapping_add(imm).wrapping_sub(4));
                set_pc!(temp_pc.wrapping_add(imm).wrapping_sub(4));
            }
            Instructions::Jalr => {}
            Instructions::Beq => {
                branch!(raw, ==);
                // let imm = (raw as i32 >> 7 & 0x1E)
                //     | (raw as i32 >> 22 & 0x3F << 5)
                //     | (raw as i32 & 0x100 << 2)
                //     | ((raw as i32 >> 31 & 1) << 11);
                // let imm = (imm << (32 - 12)) >> (32 - 12);
                // let rs1 = raw >> 15 & 0x1F;
                // let rs2 = raw >> 20 & 0x1F;
                // if read_reg!(rs1) == read_reg!(rs2) {
                //     let temp_pc = get_pc!() as i32;
                //     set_pc!(temp_pc.wrapping_add(imm).wrapping_sub(4));
                // }
            }
            Instructions::Bne => {
                branch!(raw, !=);
                // let imm = (raw as i32 >> 7 & 0x1E)
                //     | (raw as i32 >> 22 & 0x3F << 5)
                //     | (raw as i32 & 0x100 << 2)
                //     | ((raw as i32 >> 31 & 1) << 11);
                // let imm = (imm << (32 - 12)) >> (32 - 12);
                // let rs1 = raw >> 15 & 0x1F;
                // let rs2 = raw >> 20 & 0x1F;
                // if read_reg!(rs1) != read_reg!(rs2) {
                //     let temp_pc = get_pc!() as i32;
                //     set_pc!(temp_pc.wrapping_add(imm).wrapping_sub(4));
                // }
            }
            Instructions::Blt => {
                branch!(raw, <, int);
                // let imm = (raw as i32 >> 7 & 0x1E)
                //     | (raw as i32 >> 22 & 0x3F << 5)
                //     | (raw as i32 & 0x100 << 2)
                //     | ((raw as i32 >> 31 & 1) << 11);
                // let imm = (imm << (32 - 12)) >> (32 - 12);
                // let rs1 = raw >> 15 & 0x1F;
                // let rs2 = raw >> 20 & 0x1F;
                // if (read_reg!(rs1) as i32) < (read_reg!(rs2) as i32) {
                //     let temp_pc = get_pc!() as i32;
                //     set_pc!(temp_pc.wrapping_add(imm).wrapping_sub(4));
                // }
            }
            Instructions::Bge => {
                branch!(raw, >=, int);
                // let imm = (raw as i32 >> 7 & 0x1E)
                //     | (raw as i32 >> 22 & 0x3F << 5)
                //     | (raw as i32 & 0x100 << 2)
                //     | ((raw as i32 >> 31 & 1) << 11);
                // let imm = (imm << (32 - 12)) >> (32 - 12);
                // let rs1 = raw >> 15 & 0x1F;
                // let rs2 = raw >> 20 & 0x1F;
                // if (read_reg!(rs1) as i32) >= (read_reg!(rs2) as i32) {
                //     let temp_pc = get_pc!() as i32;
                //     set_pc!(temp_pc.wrapping_add(imm).wrapping_sub(4));
                // }
            }
            Instructions::Bltu => {

                branch!(raw, <);
                // let imm = (raw as i32 >> 7 & 0x1E)
                //     | (raw as i32 >> 22 & 0x3F << 5)
                //     | (raw as i32 & 0x100 << 2)
                //     | ((raw as i32 >> 31 & 1) << 11);
                // let imm = (imm << (32 - 12)) >> (32 - 12);
                // let rs1 = raw >> 15 & 0x1F;
                // let rs2 = raw >> 20 & 0x1F;
                // if (read_reg!(rs1) as u32) < (read_reg!(rs2) as u32) {
                //     let temp_pc = get_pc!() as i32;
                //     set_pc!(temp_pc.wrapping_add(imm).wrapping_sub(4));
                // }
            }
            Instructions::Bgeu => {
                branch!(raw, >=);
                // let imm = (raw as i32 >> 7 & 0x1E)
                //     | (raw as i32 >> 22 & 0x3F << 5)
                //     | (raw as i32 & 0x100 << 2)
                //     | ((raw as i32 >> 31 & 1) << 11);
                // let imm = (imm << (32 - 12)) >> (32 - 12);
                // let rs1 = raw >> 15 & 0x1F;
                // let rs2 = raw >> 20 & 0x1F;
                // if (read_reg!(rs1) as u32) >= (read_reg!(rs2) as u32) {
                //     let temp_pc = get_pc!() as i32;
                //     set_pc!(temp_pc.wrapping_add(imm).wrapping_sub(4));
                // }
            }
        };

        inc_pc!();

        dbg_reg();

        if read_reg!(ZERO) != 0 {
            set_reg!(ZERO, 0);
        };

        // if read_reg!(A0) == 10 {
        //     panic!("reg a0 test");
        // }

        if get_pc!() as u64 > text.section_address + text.section_size {
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
