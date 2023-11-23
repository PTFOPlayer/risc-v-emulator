use std::{
    mem::transmute,
    ops::{Deref, DerefMut},
};

pub const DRAM_SIZE: usize = 64 * 1024 * 1024;
pub struct Dram {
    vec: Vec<u8>,
}

impl Dram {
    pub fn new_dram() -> Self {
        Self {
            vec: vec![0u8; DRAM_SIZE],
        }
    }

    pub fn set_u8(&mut self, addr: usize, val: u8) {
        self.vec[addr] = val;
    }

    pub fn set_u16(&mut self, addr: usize, val: u16) {
        let (v1, v2) = unsafe { transmute(val) };
        self.vec[addr] = v1;
        self.vec[addr + 1] = v2;
    }

    pub fn set_u32(&mut self, addr: usize, val: u32) {
        let (v1, v2, v3, v4) = unsafe { transmute(val) };
        self.vec[addr] = v1;
        self.vec[addr + 1] = v2;
        self.vec[addr + 2] = v3;
        self.vec[addr + 3] = v4;
    }

    pub fn set_u64(&mut self, addr: usize, val: u64) {
        let a: [u8; 8] = unsafe { transmute(val) };
        for i in 0..8 {
            self.vec[addr + i] = a[i];
        }
    }

    pub fn get_u8(&mut self, addr: usize) -> u8 {
        self.vec[addr]
    }

    pub fn get_u16(&mut self, addr: usize) -> u16 {
        unsafe { transmute([self.vec[addr], self.vec[addr + 1]]) }
    }

    pub fn get_u32(&mut self, addr: usize) -> u32 {
        unsafe {
            transmute([
                self.vec[addr],
                self.vec[addr + 1],
                self.vec[addr + 2],
                self.vec[addr + 3],
            ])
        }
    }

    pub fn get_u64(&mut self, addr: usize) -> u64 {
        unsafe {
            transmute([
                self.vec[addr],
                self.vec[addr + 1],
                self.vec[addr + 2],
                self.vec[addr + 3],
                self.vec[addr + 4],
                self.vec[addr + 5],
                self.vec[addr + 6],
                self.vec[addr + 7],
            ])
        }
    }
}

impl Deref for Dram {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl DerefMut for Dram {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}
