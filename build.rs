use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=./test_asm/risc_test.s");

    let mut c = Command::new("riscv64-linux-gnu-as");
    c.args(&["-march=rv64im", "-o", "./test_asm/risc_test.o", "./test_asm/risc_test.s"]);

    match c.output() {
        Ok(_res) => {
            if _res.status.success() {
                println!(
                    "cargo:warning={},{}",
                    String::from_utf8_lossy(&_res.stdout),
                    String::from_utf8_lossy(&_res.stderr)
                )
            } else {
                panic!(
                    "cargo:panic={},{}",
                    String::from_utf8_lossy(&_res.stdout),
                    String::from_utf8_lossy(&_res.stderr)
                )
            }
        }
        Err(err) => {
            println!("cargo:warning={:?}", err)
        }
    }

    let mut c2 = Command::new("riscv64-linux-gnu-ld");
    c2.args(&["-o", "./test_asm/a.out", "./test_asm/risc_test.o"]);

    match c2.output() {
        Ok(_res) => {
            if _res.status.success() {
                println!(
                    "cargo:warning={},{}",
                    String::from_utf8_lossy(&_res.stdout),
                    String::from_utf8_lossy(&_res.stderr)
                )
            } else {
                panic!(
                    "cargo:panic={},{}",
                    String::from_utf8_lossy(&_res.stdout),
                    String::from_utf8_lossy(&_res.stderr)
                )
            }
        }
        Err(err) => {
            println!("cargo:warning={:?}", err)
        }
    }
}
