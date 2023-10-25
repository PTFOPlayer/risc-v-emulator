mod elf_parser;
mod error;
mod instruction;
mod misc;
use std::{borrow::BorrowMut, cell::RefCell, thread::sleep, time::Duration};

use crate::{
    elf_parser::{extract_prog_bits, program_header_parser, raw_section_header_parser},
    instruction::get_instructions,
};
use error::*;


thread_local! {
    static REGISTERS: RefCell<[u64;32]> = RefCell::new([0;32]);
}

fn set_regs(reg: usize, val: u64) {
    REGISTERS.with(|x| {
        x.borrow_mut()[reg] = val;
    });
}

fn read_reg(reg: usize) -> u64 {
    REGISTERS.with(|x| x.borrow()[reg])
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

    for i in instructions {
        println!("{:?}", i);
    }

    Ok(())
}
