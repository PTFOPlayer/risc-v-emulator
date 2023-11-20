#[allow(unused_unsafe)]
mod dram;
mod elf_parser;
mod error;
mod instruction;
mod misc;
use crate::{
    dram::DRAM_SIZE,
    instruction::instruction::{execute_32, get_instructions},
    misc::{dbg_reg, dbg_stack},
};
use std::{cell::RefCell, mem::transmute, process::exit};

use crate::{
    dram::Dram,
    elf_parser::{extract_prog_bits, program_header_parser, raw_section_header_parser},
};
use error::*;

const DEBUG: bool = false;
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

    let program_headers: Vec<elf_parser::ProgramHeader> = program_header_parser(&data, &elf);

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
    set_reg!(SP, DRAM_SIZE);
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
        let raw = get_instructions(&dram, get_pc!());

        execute_32(raw, &mut dram);

        if DEBUG || P_PC {
            println!("PC: {:x?}", get_pc!());
        }

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
    ($raw: expr) => {
        crate::PC.with(|x| *x.borrow_mut() += $raw);
    };
}

#[macro_export]
macro_rules! set_pc {
    ($pc: expr) => {
        crate::PC.with(|x| *x.borrow_mut() = $pc as u32);
    };
}

#[macro_export]
macro_rules! get_pc {
    () => {
        crate::PC.with(|x| *x.borrow())
    };
}

#[macro_export]
macro_rules! set_reg {
    ($reg: expr, $val: expr) => {
        crate::REGISTERS
            .with(|x| x.borrow_mut()[$reg as usize] = unsafe { transmute($val as i64) });
    };
}

#[macro_export]
macro_rules! read_reg {
    ($reg: expr) => {
        crate::REGISTERS.with(|x| x.borrow()[$reg as usize] as u64)
    };
}
