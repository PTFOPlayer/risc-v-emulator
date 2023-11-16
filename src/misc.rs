use crate::{dram::Dram, read_reg, REGISTERS};

#[macro_export]
macro_rules! fast_transmute {
    (<$start:expr, u16>, $data: expr ) => {
        unsafe { std::mem::transmute::<[u8; 2], u16>([$data[$start + 0], $data[$start + 1]]) }
    };
    (<$start:expr, u32>, $data: expr ) => {
        unsafe {
            std::mem::transmute::<[u8; 4], u32>([
                $data[$start + 0],
                $data[$start + 1],
                $data[$start + 2],
                $data[$start + 3],
            ])
        }
    };
    (<$start:expr, u64>, $data: expr ) => {
        unsafe {
            std::mem::transmute::<[u8; 8], u64>([
                $data[$start + 0],
                $data[$start + 1],
                $data[$start + 2],
                $data[$start + 3],
                $data[$start + 4],
                $data[$start + 5],
                $data[$start + 6],
                $data[$start + 7],
            ])
        }
    };
    (<$start:expr, $t_type:tt @ $t_temp:tt>, $data: expr) => {
        unsafe {
            std::mem::transmute::<$t_temp, $t_type>(fast_transmute!(<$start, $t_temp>, $data))
        }
    }
}

#[inline(always)]
pub fn dbg_reg() {
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

pub fn dbg_stack(sp: usize, dbg_size: u64, dram: &mut Dram) {
    let sp = read_reg!(sp);
    for i in (sp..sp + dbg_size).step_by(8) {
        for j in 0..8 {
            print!("M{:02x}: 0x{:02x} ", i + j, dram.get_u32((i + j) as usize));
        }
        println!()
    }
}
