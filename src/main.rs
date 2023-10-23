mod elf_parser;
mod error;
use crate::elf_parser::{program_header_parser, raw_section_header_parser, section_header_final};
use error::*;

fn main() -> Result<(), EmulatorError> {
    let data = std::fs::read("./a.out")?;

    let elf = elf_parser::elf_parser(&data);
    println!("{:?}\n\n", elf);

    let program_headers = program_header_parser(&data, &elf);
    println!("{}, {:?}\n\n", program_headers.len(), program_headers);

    let mut section_headers = raw_section_header_parser(&data, &elf);
    println!("{}, {:?}\n\n", section_headers.len(), section_headers);

    let headers_list = section_header_final(&data, &mut section_headers)?;
    for e in headers_list {
        println!("section: {}", e);
    }

    Ok(())
}
