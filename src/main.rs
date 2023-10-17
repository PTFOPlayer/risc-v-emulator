mod error;
mod elf_parser;
use std::sync::{Arc, RwLock};

use error::*;
use lazy_static::lazy_static;

lazy_static! {
    static ref X: Arc<RwLock<[u32; 32]>> = Arc::new(RwLock::new([0u32; 32]));
}


fn main() -> Result<(), EmulatorError> {
    let data = std::fs::read("./a.out")?;

    let elf = elf_parser::elf_parser(&data);
    println!("{:?}", elf);

    Ok(())
}
