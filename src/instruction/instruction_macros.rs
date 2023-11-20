#[inline(always)]
pub fn __extract_branch(raw: u32) -> (i32, u32, u32) {
    let imm = ((raw & 0x80000000u32) >> 19)
        | ((raw & 0x80) << 4)
        | ((raw >> 20) & 0x7e0)
        | ((raw >> 7) & 0x1e);
    let imm = imm as i32;
    let imm = (imm << (31 - 12)) >> (31 - 12);
    let rs1 = raw >> 15 & 0x1F;
    let rs2 = raw >> 20 & 0x1F;

    (imm, rs1, rs2)
}

#[macro_export]
macro_rules! branch {
    ($raw: expr, $e: tt) => {
        use crate::instruction::instruction_macros::__extract_branch;
        let (imm, rs1, rs2) = __extract_branch($raw as u32);
        if (crate::read_reg!(rs1) as u32) $e (crate::read_reg!(rs2) as u32) {
            let temp_pc = crate::get_pc!() as i64;
            crate::set_pc!(temp_pc.wrapping_add(imm  as i64).wrapping_sub(4));
        }
    };
    ($raw: expr, $e: tt, int) => {
        use crate::instruction::instruction_macros::__extract_branch;
        let (imm, rs1, rs2) = __extract_branch($raw as u32);
        if (crate::read_reg!(rs1) as i32) $e (crate::read_reg!(rs2) as i32) {
            let temp_pc = crate::get_pc!() as i64;
            crate::set_pc!(temp_pc.wrapping_add(imm as i64).wrapping_sub(4));
        }
    };
}

#[macro_export]
macro_rules! rd {
    ($raw: expr) => {
        ($raw as u32 >> 7) & 0x1F
    };
}

#[macro_export]
macro_rules! rs1 {
    ($raw: expr) => {
        ($raw as u32 >> 15) & 0x1F
    };
}

#[macro_export]
macro_rules! rs2 {
    ($raw: expr) => {
        ($raw as u32 >> 20) & 0x1F
    };
}

#[macro_export]
macro_rules! imm {
    (I, $raw: expr) => {
        ($raw as u32 >> 20) & 0x7FF
    };
    (U, $raw: expr) => {
        ($raw as u32 >> 12) & 0x7FFFF
    };
    (S, $raw:expr) => {
        (($raw & 0xfe000000) >> 20) | (($raw >> 7) & 0x1f)
    };
}

#[macro_export]
macro_rules! t_i64 {
    ($val: expr) => {
        unsafe { std::mem::transmute::<u64, i64>($val) }
    };
}

#[macro_export]
macro_rules! t_u64 {
    ($val: expr) => {
        unsafe{std::mem::transmute::<i64, u64>(read_reg!$val)}
    };
}
