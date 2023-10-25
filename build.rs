use std::process::Command;

fn main() {
    let mut c = Command::new("riscv64-linux-gnu-as");
    c.args(&["-march=rv64i", "-o", "risc_test.o", "risc_test.s"]);

    match c.output() {
        Ok(_res) => println!(
            "cargo:warning={:?},{},{}",
            _res.status,
            String::from_utf8_lossy(&_res.stdout),
            String::from_utf8_lossy(&_res.stderr)
        ),
        Err(err) => {println!("cargo:warning={:?}", err)}
    }

    let mut c2 = Command::new("riscv64-linux-gnu-ld");
    c2.args(&["-o", "a.out", "risc_test.o"]);

    match c2.output() {
        Ok(_res) => println!(
            "cargo:warning={:?},{},{}",
            _res.status,
            String::from_utf8_lossy(&_res.stdout),
            String::from_utf8_lossy(&_res.stderr)
        ),
        Err(err) => {println!("cargo:warning={:?}", err)},
    }
}
