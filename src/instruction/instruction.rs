#![allow(unused_unsafe)]
// opcode mask for type R: 0b11111110000000000111000001111111
// opcode mask for type I:                  0b111000001111111
// opcode mask for type S:                  0b111000001111111
// opcode mask for type B:                  0b111000001111111
// opcode mask for type U:                          0b1111111
// opcode mask for type J:                          0b1111111
use crate::*;

#[inline(always)]
pub fn get_instructions(prog_bits: &[u8], pc: u32) -> u32 {
    let pc = pc as usize;
    fast_transmute!(<0, u32>, [prog_bits[pc+0], prog_bits[pc+1], prog_bits[pc+2], prog_bits[pc+3]])
}

pub fn execute_32(op: u32, dram: &mut Dram) {
    let raw = op;
    let instruction_type = op & 0x7F;
    match instruction_type {
        // u_type
        // Lui
        0b0110111 => {
            set_reg!(rd!(raw), imm!(U, raw) << 12);
        }
        // Auipc
        0b0010111 => {
            set_reg!(
                rd!(raw),
                (get_pc!() as i64).wrapping_add((imm!(U, raw) << 12) as i64)
            );
        }
        // j_type
        // Jal
        0b1101111 => {
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
        // Jalr
        0b1100111 => {
            let imm = imm!(I, raw);
            let rs1 = read_reg!(rs1!(raw)) as i64;
            let mut imm = imm as i32;
            imm = (imm << 20) >> 20;

            let rd = rd!(raw);
            set_reg!(rd, get_pc!() + 4);
            set_pc!(rs1.wrapping_add(imm as i64).wrapping_sub(4));
        }
        // i_type RV32I+RV64I
        0b0010011 => {
            let funct = op >> 12 & 0x7;
            match funct {
                0b101 | 0b001 => {
                    let s_funct = op >> 24 & 0x3f;
                    match (s_funct, funct) {
                        // Slli
                        (0b000000, 0b001) => {
                            let shamt = imm!(I, raw) & 0x1f;
                            let rs = rs1!(raw);
                            let rd = rd!(raw);
                            set_reg!(rd, read_reg!(rs) << shamt);
                        }
                        // Srli
                        (0b000000, 0b101) => {
                            let shamt = imm!(I, raw) & 0x1f;
                            let rs = rs1!(raw);
                            let rd = rd!(raw);
                            set_reg!(rd, read_reg!(rs) >> shamt);
                        }
                        // Srai
                        (0b010000, 0b101) => {
                            let shamt = imm!(I, raw) & 0x1f;
                            let rs = rs1!(raw);
                            let rd = rd!(raw);
                            set_reg!(rd, t_i64!(read_reg!(rs)) << shamt);
                        }

                        // error?
                        _ => panic!("unknown instruction occured in i_type branch"),
                    }
                }
                // Addi
                0b000 => {
                    let rd = rd!(raw);
                    let rs = t_i64!(read_reg!(rs1!(raw)));
                    let mut imm = imm!(I, raw) as i32;
                    imm = (imm << 21) >> 21;
                    set_reg!(rd, rs.wrapping_add(imm as i64));
                }
                // Slti
                0b010 => {
                    let rd = rd!(raw);
                    let rs = read_reg!(rs1!(raw));
                    let mut imm = imm!(I, raw) as i32;
                    imm = (imm << 21) >> 21;
                    if (rs as i64) < imm as i64 {
                        set_reg!(rd, 1);
                    }
                }
                // Sltiu
                0b011 => {
                    let rd = rd!(raw);
                    let rs = read_reg!(rs1!(raw));
                    let imm = imm!(I, raw);
                    if (rs as u64) < imm as u64 {
                        set_reg!(rd, 1);
                    }
                }
                // Xori
                0b100 => {
                    let rd = rd!(raw);
                    let rs = rs1!(raw);
                    set_reg!(rd, read_reg!(rs) ^ imm!(I, raw) as u64);
                }
                // Ori
                0b110 => {
                    let rd = rd!(raw);
                    let rs = rs1!(raw);
                    set_reg!(rd, read_reg!(rs) | imm!(I, raw) as u64);
                }
                // Andi
                0b111 => {
                    let rd = rd!(raw);
                    let rs = rs1!(raw);
                    set_reg!(rd, read_reg!(rs) & imm!(I, raw) as u64);
                }

                // error?
                _ => panic!("unknown instruction occured in i_type branch"),
            }
        }
        // i_type RV64I
        0b0011011 => {
            let funct = op >> 12 & 0x7;

            match funct {
                0b101 | 0b001 => {
                    let s_funct = op >> 24 & 0x3f;
                    match (s_funct, funct) {
                        // Slliw
                        (0b000000, 0b001) => {
                            let shamt = imm!(I, raw) & 0x1f;
                            let rs = t_i32!((read_reg!(rs1!(raw)) & 0xFFFFFFFF) as u32);
                            let rd = rd!(raw);
                            set_reg!(rd, rs.wrapping_shl(shamt));
                        }
                        // Srliw
                        (0b000000, 0b101) => {
                            let shamt = imm!(I, raw) & 0x1f;
                            let rs = t_i32!((read_reg!(rs1!(raw)) & 0xFFFFFFFF) as u32);
                            let rd = rd!(raw);
                            set_reg!(rd, rs.wrapping_shr(shamt));
                        }
                        // Sraiw
                        (0b010000, 0b101) => {
                            let shamt = imm!(I, raw) & 0x1f;
                            let rs = t_i32!((read_reg!(rs1!(raw)) & 0xFFFFFFFF) as u32);
                            let rd = rd!(raw);
                            set_reg!(rd, rs.wrapping_shr(shamt));
                        }

                        // error?
                        _ => panic!("unknown instruction occured in i_type branch"),
                    }
                }
                // Addiw
                0b000 => {
                    let rd = rd!(raw);
                    let rs = t_i32!((read_reg!(rs1!(raw)) & 0xFFFFFFFF) as u32);
                    let mut imm = imm!(I, raw) as i32;
                    imm = (imm << 21) >> 21;
                    set_reg!(rd, rs.wrapping_add(imm));
                }
                // error?
                _ => panic!("unknown instruction occured in i_type RV64I branch"),
            }
        }
        // r_type RV32I
        0b0110011 => {
            let funct3 = op >> 12 & 0x7;
            let funct7 = op >> 25 & 0x7F;
            match (funct7, funct3) {
                // Add
                (0b0000000, 0b000) => {
                    let rd = rd!(raw);
                    let rs1 = t_i64!(read_reg!(rs1!(raw)));
                    let rs2 = t_i64!(read_reg!(rs2!(raw)));
                    set_reg!(rd, rs1.wrapping_add(rs2));
                }
                // Sub
                (0b0100000, 0b000) => {
                    let rd = rd!(raw);
                    let rs1 = rs1!(raw);
                    let rs2 = rs2!(raw);
                    set_reg!(rd, read_reg!(rs1).wrapping_sub(read_reg!(rs2)));
                }
                // Sll
                (0b0000000, 0b001) => {
                    let rd = rd!(raw);
                    let rs1 = rs1!(raw);
                    let rs2 = rs2!(raw);
                    set_reg!(rd, read_reg!(rs1) << read_reg!(rs2));
                }
                // Slt
                (0b0000000, 0b010) => {
                    let rd = rd!(raw);
                    let rs1 = rs1!(raw);
                    let rs2 = rs2!(raw);
                    set_reg!(rd, (read_reg!(rs1) as i64) < (read_reg!(rs2) as i64));
                }
                // Sltu
                (0b0000000, 0b011) => {
                    let rd = rd!(raw);
                    let rs1 = rs1!(raw);
                    let rs2 = rs2!(raw);
                    set_reg!(rd, read_reg!(rs1) < read_reg!(rs2));
                }
                // Xor
                (0b0000000, 0b100) => {
                    let rd = rd!(raw);
                    let rs1 = rs1!(raw);
                    let rs2 = rs2!(raw);
                    set_reg!(rd, read_reg!(rs1) ^ read_reg!(rs2));
                }
                // Srl
                (0b0000000, 0b101) => {
                    let rd = rd!(raw);
                    let rs1 = rs1!(raw);
                    let rs2 = rs2!(raw);
                    set_reg!(rd, read_reg!(rs1) >> read_reg!(rs2));
                }
                // Sra
                (0b0100000, 0b101) => {
                    let rd = rd!(raw);
                    let rs1 = rs1!(raw);
                    let rs2 = rs2!(raw);
                    set_reg!(rd, (read_reg!(rs1) as i64) >> (read_reg!(rs2) as i64));
                }
                // Or
                (0b0000000, 0b110) => {
                    let rd = rd!(raw);
                    let rs1 = rs1!(raw);
                    let rs2 = rs2!(raw);
                    set_reg!(rd, read_reg!(rs1) | read_reg!(rs2));
                }
                // And
                (0b0000000, 0b111) => {
                    let rd = rd!(raw);
                    let rs1 = rs1!(raw);
                    let rs2 = rs2!(raw);
                    set_reg!(rd, read_reg!(rs1) & read_reg!(rs2));
                }
                // Mul RV32M+RV64M
                (0000001, 0b000) => {
                    let rd = rd!(raw);
                    let rs1 = t_i64!(read_reg!(rs1!(raw)));
                    let rs2 = t_i64!(read_reg!(rs2!(raw)));
                    set_reg!(rd, rs1.wrapping_mul(rs2));
                }
                // Div  RV32M+RV64M
                (0000001, 0b100) => {
                    let rd = rd!(raw);
                    let rs1: i64 = t_i64!(read_reg!(rs1!(raw)));
                    let rs2: i64 = t_i64!(read_reg!(rs2!(raw)));
                    set_reg!(rd, rs1.wrapping_div(rs2));
                }
                // Rem  RV32M+RV64M
                (0000001, 0b110) => {
                    let rd = rd!(raw);
                    let rs1: i64 = t_i64!(read_reg!(rs1!(raw)));
                    let rs2: i64 = t_i64!(read_reg!(rs2!(raw)));
                    set_reg!(rd, rs1.wrapping_rem(rs2));
                }
                // error?
                _ => panic!("unknown instruction"),
            }
        }
        // r_type RV64I
        0b0111011 => {
            let funct3 = op >> 12 & 0x7;
            let funct7 = op >> 25 & 0x7F;
            match (funct7, funct3) {
                // Addw
                (0b0000000, 0b000) => {
                    let rd = rd!(raw);
                    let rs1 = t_i32!((read_reg!(rs1!(raw)) & 0xFFFFFFFF) as u32);
                    let rs2 = t_i32!((read_reg!(rs2!(raw)) & 0xFFFFFFFF) as u32);
                    set_reg!(rd, rs1.wrapping_add(rs2));
                }
                // Subw
                (0b0100000, 0b000) => {
                    let rd = rd!(raw);
                    let rs1 = t_i32!((read_reg!(rs1!(raw)) & 0xFFFFFFFF) as u32);
                    let rs2 = t_i32!((read_reg!(rs2!(raw)) & 0xFFFFFFFF) as u32);
                    set_reg!(rd, read_reg!(rs1).wrapping_sub(read_reg!(rs2)));
                }
                // Sllw
                (0b0000000, 0b001) => {
                    let rd = rd!(raw);
                    let rs1 = t_i32!((read_reg!(rs1!(raw)) & 0xFFFFFFFF) as u32);
                    let rs2 = t_i32!((read_reg!(rs2!(raw)) & 0xFFFFFFFF) as u32);
                    set_reg!(rd, read_reg!(rs1) << read_reg!(rs2));
                }

                // Srlw
                (0b0000000, 0b101) => {
                    let rd = rd!(raw);
                    let rs1 = t_i32!((read_reg!(rs1!(raw)) & 0xFFFFFFFF) as u32);
                    let rs2 = t_i32!((read_reg!(rs2!(raw)) & 0xFFFFFFFF) as u32);
                    set_reg!(rd, read_reg!(rs1) >> read_reg!(rs2));
                }
                // Sraw
                (0b0100000, 0b101) => {
                    let rd = rd!(raw);
                    let rs1 = t_i32!((read_reg!(rs1!(raw)) & 0xFFFFFFFF) as u32);
                    let rs2 = t_i32!((read_reg!(rs2!(raw)) & 0xFFFFFFFF) as u32);
                    set_reg!(rd, (read_reg!(rs1) as i64) >> (read_reg!(rs2) as i64));
                }
                _ => panic!("unknown instruction"),
            }
        }
        // b_type
        0b1100011 => {
            let funct3 = op >> 12 & 0x7;
            match funct3 {
                // Beq
                0b000 => {
                    branch!(raw, ==);
                }
                // Bne
                0b001 => {
                    branch!(raw, !=);
                }
                // Blt
                0b100 => {
                    branch!(raw, <, int);
                }
                // Bge
                0b101 => {
                    branch!(raw, >=, int);
                }
                // Bltu
                0b110 => {
                    branch!(raw, <);
                }
                // Bgeu
                0b111 => {
                    branch!(raw, >=);
                }

                // error?
                _ => {
                    panic!("unknown instruction")
                }
            }
        }
        // fence
        // Fence
        0b00000000000000000000000000001111 => {
            println!("fence");
        }
        // FenceI
        0b00000000000000000001000000001111 => {
            println!("fence.i");
        }
        //calls
        // Ecall
        0b00000000000000000000000001110011 => match (read_reg!(A0), read_reg!(A7)) {
            (1, 64) => {
                let addr = read_reg!(A1) as usize;
                let len = read_reg!(A2) as usize;
                let s = String::from_utf8_lossy(&dram[addr..addr + len]);
                if DEBUG {
                    println!("{:x}", addr);
                    println!("ecall: print\n{:?}", s);
                } else {
                    print!("{}", s);
                }
            }
            _ => {
                panic!("unknown syscall");
            }
        },
        // Ebreak
        0b00000000000100000000000001110011 => {}
        // loads
        0b0000011 => {
            let funct3 = op >> 12 & 0x7;
            match funct3 {
                // Lb
                0b000 => {
                    let rd = rd!(raw);
                    let rs = read_reg!(rs1!(raw));
                    let imm = imm!(I, raw);
                    let data = unsafe {
                        transmute::<u8, i8>(dram.get_u8(rs.wrapping_add(imm as u64) as usize))
                            as i64
                    };
                    set_reg!(rd, data);
                }
                // Lh
                0b001 => {
                    let rd = rd!(raw);
                    let rs = read_reg!(rs1!(raw));
                    let imm = imm!(I, raw);
                    let data = unsafe {
                        transmute::<u16, i16>(dram.get_u16(rs.wrapping_add(imm as u64) as usize))
                    };
                    set_reg!(rd, data);
                }
                // Lw
                0b010 => {
                    let rd = rd!(raw);
                    let rs = read_reg!(rs1!(raw));
                    let imm = imm!(I, raw);
                    let data = unsafe {
                        transmute::<u32, i32>(dram.get_u32(rs.wrapping_add(imm as u64) as usize))
                    };
                    set_reg!(rd, data);
                }
                // Lwu
                0b110 => {
                    let rd = rd!(raw);
                    let rs = read_reg!(rs1!(raw));
                    let imm = imm!(I, raw);
                    let data = dram.get_u32(rs.wrapping_add(imm as u64) as usize);
                    set_reg!(rd, data);
                }
                // Ld
                0b011 => {
                    let rd = rd!(raw);
                    let rs = read_reg!(rs1!(raw));
                    let imm = imm!(I, raw);
                    let data = unsafe {
                        transmute::<u64, i64>(dram.get_u64(rs.wrapping_add(imm as u64) as usize))
                    };
                    set_reg!(rd, data);
                }
                // Lbu
                0b100 => {
                    let rd = rd!(raw);
                    let rs = read_reg!(rs1!(raw));
                    let imm = imm!(I, raw);
                    let addr = (rs.wrapping_add(imm as u64)) as usize;
                    set_reg!(rd, dram.get_u8(addr));
                }
                // Lhu
                0b101 => {
                    let rd = rd!(raw);
                    let rs = read_reg!(rs1!(raw));
                    let imm = imm!(I, raw);
                    let addr = (rs.wrapping_add(imm as u64)) as usize;
                    set_reg!(rd, dram.get_u16(addr));
                }
                // error?
                _ => {
                    panic!("unknown instruction: {:x}", op);
                }
            }
        }
        // s_type
        0b0100011 => {
            let funct3 = op >> 12 & 0x7;
            match funct3 {
                // Sb
                0b000 => {
                    let rs1 = read_reg!(rs1!(raw));
                    let rs2 = read_reg!(rs2!(raw));
                    let imm = imm!(S, raw);
                    let addr = (rs1 + imm as u64) as usize;
                    let val = (rs2 & 0xFF) as u8;
                    dram.set_u8(addr, val);
                }
                // Sh
                0b001 => {
                    let rs1 = read_reg!(rs1!(raw));
                    let rs2 = read_reg!(rs2!(raw));
                    let imm = imm!(S, raw);
                    let addr = (rs1 + imm as u64) as usize;
                    let val = (rs2 & 0xFFFF) as u16;
                    dram.set_u16(addr, val);
                }
                // Sw
                0b010 => {
                    let rs1 = read_reg!(rs1!(raw));
                    let rs2 = read_reg!(rs2!(raw));
                    let imm = imm!(S, raw);
                    let addr = (rs1 + imm as u64) as usize;
                    let val = (rs2 & 0xFFFFFFFF) as u32;
                    dram.set_u32(addr, val);
                }
                // Sd
                0b011 => {
                    let rs1 = read_reg!(rs1!(raw));
                    let rs2 = read_reg!(rs2!(raw));
                    let imm = imm!(S, raw) as u64;
                    let addr = (rs1 + imm) as usize;
                    dram.set_u64(addr, rs2);
                }

                // error?
                _ => {
                    panic!("unknown instruction: {:x}", op);
                }
            }
        }
        // error?
        _ => {
            panic!("unknown instruction: {:x}", op);
        }
    }

    inc_pc!(4);
}
