use std::{
    mem::transmute,
    ops::{Deref, DerefMut},
};

use crate::DRAM_SIZE;

pub struct Dram {
    vec: Vec<u8>,
}

impl Dram {
    pub fn new_dram() -> Self {
        Self {
            vec: vec![0; DRAM_SIZE],
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
