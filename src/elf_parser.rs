#![allow(dead_code)]
#![allow(unused_unsafe)]

use std::{mem::transmute, rc::Rc};

use crate::error::EmulatorError;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum ABI {
    SystemV(u8) = 0x00,
    HpUx(u8) = 0x01,
    NetBSD(u8) = 0x02,
    Linux(u8) = 0x03,
    GnuHurd(u8) = 0x04,
    Solaris(u8) = 0x06,
    AixMonterey(u8) = 0x07,
    IRIX(u8) = 0x08,
    FreeBSD(u8) = 0x09,
    Tru64(u8) = 0x0A,
    NovellModesto(u8) = 0x0B,
    OpenBSD(u8) = 0x0C,
    OpenVMS(u8) = 0x0D,
    NonStopKernel(u8) = 0x0E,
    AROS(u8) = 0x0F,
    FenixOS(u8) = 0x10,
    NuxiCloudAbi(u8) = 0x11,
    StratusTechnologiesOpenVos(u8) = 0x12,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinArc {
    X32,
    X64,
}

#[derive(Debug, Clone)]
pub enum Endian {
    Little,
    Big,
}

#[repr(u16)]
#[derive(Debug, Clone)]
pub enum ObjType {
    EtNone = 0x00,
    EtRel = 0x01,
    EtExec = 0x02,
    EtDyn = 0x03,
    EtCore = 0x04,
    EtLoos = 0xFE00,
    EtHios = 0xFEFF,
    EtLoproc = 0xFF00,
    EtHiproc = 0xFFFF,
}

#[derive(Debug, Clone)]
pub struct ELF {
    magic: u32,
    bin_arc: BinArc,
    endian: Endian,
    abi: ABI,
    obj_type: ObjType,
    entry_point: u64,
    program_header_address: u64,
    section_header_address: u64,
    elf_header_size: u16,
    program_header_size: u16,
    program_header_entries: u16,
    section_header_size: u16,
    section_header_entries: u16,
    section_header_names: u16,
}

macro_rules! fast_transmute {
    (<$start:expr, u16>, $data: expr ) => {
        unsafe { transmute::<[u8; 2], u16>([$data[$start + 0], $data[$start + 1]]) }
    };
    (<$start:expr, u32>, $data: expr ) => {
        unsafe {
            transmute::<[u8; 4], u32>([
                $data[$start + 0],
                $data[$start + 1],
                $data[$start + 2],
                $data[$start + 3],
            ])
        }
    };
    (<$start:expr, u64>, $data: expr ) => {
        unsafe {
            transmute::<[u8; 8], u64>([
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
    (<$start:expr, $t_type:tt, $t_temp:tt>, $data: expr) => {
        unsafe {
                transmute::<$t_temp, $t_type>(fast_transmute!(<$start, $t_temp>, $data))
        }
    }
}
pub fn elf_parser(data: &[u8]) -> ELF {
    let mut bin_arc = BinArc::X32;
    let mut endian = Endian::Little;

    let t_magic = &data[0..4];
    let magic = fast_transmute!(<0, u32>, t_magic);
    let ident = data[4];
    if ident == 2 {
        bin_arc = BinArc::X64;
    }

    let ident = data[5];
    if ident == 2 {
        endian = Endian::Big;
    }

    let set_to_one = data[6];
    if set_to_one != 1 {
        panic!("that is supposed to be set to 1");
    }

    let abi = fast_transmute!(<7, ABI, u16>, data);
    let _padding = &data[9..16];

    let obj_type = fast_transmute!(<16, ObjType, u16>, data);

    // 0xF3 means RISC-V
    let arc = fast_transmute!(<18, u16>, data);
    if arc != 0xF3 && arc != 0x00 {
        panic!("this ELF is not for RISC-V architecture: {:x}", arc)
    } else if arc == 0x00 {
        println!(
            "\x1b[93mWARNING\x1b[0m: no guarantee that this ELF is made for RISC-V architecture"
        )
    }
    let _e_version = fast_transmute!(<20, u32>, data);

    let entry_point;
    let program_header_address;
    let section_header_address;
    let rp; //reference point
    unsafe {
        if bin_arc == BinArc::X64 {
            entry_point = fast_transmute!(<24, u64>, data);
            program_header_address = fast_transmute!(<32, u64>, data);
            section_header_address = fast_transmute!(<40, u64>, data);

            rp = 48;
        } else {
            entry_point = fast_transmute!(<24, u32>, data) as u64;
            program_header_address = fast_transmute!(<28, u32>, data) as u64;
            section_header_address = fast_transmute!(<32, u32>, data) as u64;

            rp = 36;
        }
    }

    let _unknown = fast_transmute!(<rp, u32>, data);
    let elf_header_size = fast_transmute!(<rp+4, u16>, data);
    let program_header_size = fast_transmute!(<rp+6, u16>, data);
    let program_header_entries = fast_transmute!(<rp+8, u16>, data);
    let section_header_size = fast_transmute!(<rp+10, u16>, data);
    let section_header_entries = fast_transmute!(<rp+12, u16>, data);
    let section_header_names = fast_transmute!(<rp+14, u16>, data);
    ELF {
        magic,
        bin_arc,
        endian,
        abi,
        obj_type,
        entry_point,
        program_header_address,
        section_header_address,
        elf_header_size,
        program_header_size,
        program_header_entries,
        section_header_size,
        section_header_entries,
        section_header_names,
    }
}

#[repr(u32)]
#[derive(Debug, Clone)]
pub enum ProgramHeaderType {
    PtNull = 0x00000000,
    PtLoad = 0x00000001,
    PtDynamic = 0x00000002,
    PtInterp = 0x00000003,
    PtNote = 0x00000004,
    PtShlib = 0x00000005,
    PtPhdr = 0x00000006,
    PtTls = 0x00000007,
    PtLoos = 0x60000000,
    PtHios = 0x6FFFFFFF,
    PtLoproc = 0x70000000,
    PtHiproc = 0x7FFFFFFF,
}

#[repr(u32)]
#[derive(Debug, Clone)]
pub enum ProgramHeaderFlags {
    PfX = 0x1,
    PfW = 0x2,
    PfR = 0x4,

    Error = 0x0,
}

#[derive(Debug, Clone)]
pub struct ProgramHeader {
    pub p_type: ProgramHeaderType,
    pub p_flags: Vec<ProgramHeaderFlags>,
    pub segment_offset: u64,
    pub virtual_address: u64,
    pub physical_address: u64,
    pub segment_size: u64,
    pub mem_size: u64,
    pub alignment: u64,
}

pub fn program_header_parser(data: &[u8], elf: &ELF) -> Vec<ProgramHeader> {
    let start = elf.program_header_address as usize;
    let bin_arc = &elf.bin_arc;
    let size = elf.program_header_size as usize;
    let entries = elf.program_header_entries as usize;

    let mut v = vec![];

    for i in 0..entries {
        let of = (i * size) + start;
        let p_type = fast_transmute!(<of, ProgramHeaderType , u32>, data);

        let p_flags = {
            let mut flags = vec![];
            let temp = if *bin_arc == BinArc::X64 {
                fast_transmute!(<of+4, u32>, data)
            } else {
                fast_transmute!(<of+24, u32>, data)
            };
            if temp & 0x1 == 0x1 {
                flags.push(ProgramHeaderFlags::PfX);
            }
            if temp & 0x2 == 0x2 {
                flags.push(ProgramHeaderFlags::PfX);
            }
            if temp & 0x4 == 0x4 {
                flags.push(ProgramHeaderFlags::PfR);
            }
            flags
        };

        let segment_offset;
        let virtual_address;
        let physical_address;
        let segment_size;
        let mem_size;
        let alignment;

        unsafe {
            if *bin_arc == BinArc::X64 {
                segment_offset = fast_transmute!(<of+8, u64>, data);
                virtual_address = fast_transmute!(<of+16, u64>, data);
                physical_address = fast_transmute!(<of+24, u64>, data);
                segment_size = fast_transmute!(<of+32, u64>, data);
                mem_size = fast_transmute!(<of+40, u64>, data);
                alignment = fast_transmute!(<of+48, u64>, data);
            } else {
                segment_offset = fast_transmute!(<of+4, u32>, data) as u64;
                virtual_address = fast_transmute!(<of+8, u32>, data) as u64;
                physical_address = fast_transmute!(<of+12, u32>, data) as u64;
                segment_size = fast_transmute!(<of+16, u32>, data) as u64;
                mem_size = fast_transmute!(<of+20, u32>, data) as u64;
                alignment = fast_transmute!(<of+28, u32>, data) as u64;
            }
        };

        v.push(ProgramHeader {
            p_type,
            p_flags,
            segment_offset,
            virtual_address,
            physical_address,
            segment_size,
            mem_size,
            alignment,
        });
    }

    v
}

#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionHeaderType {
    ShtNull = 0x0,
    ShtProgbits = 0x1,
    ShtSymtab = 0x2,
    ShtStrtab = 0x3,
    ShtRela = 0x4,
    ShtHash = 0x5,
    ShtDynamic = 0x6,
    ShtNote = 0x7,
    ShtNobits = 0x8,
    ShtRel = 0x9,
    ShtShlib = 0x0A,
    ShtDynsym = 0x0B,
    ShtInitArray = 0x0E,
    ShtFiniArray = 0x0F,
    ShtPreinitArray = 0x10,
    ShtGroup = 0x11,
    ShtSymtabShndx = 0x12,
    ShtNum = 0x13,
    ShtLoos = 0x60000000,
}

#[derive(Debug, Clone)]
pub struct SectionHeader {
    pub name: u32,
    pub name_str: Option<Rc<str>>,
    pub section_type: SectionHeaderType,
    pub flags: u64,
    pub section_address: u64,
    pub section_offset: u64,
    pub section_size: u64,
    pub link: u32,
    pub info: u32,
    pub alignment: u64,
    pub entry_size: u64,
}

pub fn raw_section_header_parser(data: &[u8], elf: &ELF) -> Vec<SectionHeader> {
    let start = elf.section_header_address as usize;
    let bin_arc = &elf.bin_arc;
    let size = elf.section_header_size as usize;
    let entries = elf.section_header_entries as usize;

    let mut v = vec![];

    for i in 0..entries {
        let of = (i * size) + start;

        let name = fast_transmute!(<of, u32>, data);
        let section_type = fast_transmute!(<of + 4, SectionHeaderType, u32>, data);

        let flags;
        let section_address;
        let section_offset;
        let section_size;
        let link;
        let info;
        let alignment;
        let entry_size;

        unsafe {
            if *bin_arc == BinArc::X64 {
                flags = fast_transmute!(<of+8, u64>, data);
                section_address = fast_transmute!(<of+16, u64>, data);
                section_offset = fast_transmute!(<of+24, u64>, data);
                section_size = fast_transmute!(<of+32, u64>, data);
                link = fast_transmute!(<of+40, u32>, data);
                info = fast_transmute!(<of+44, u32>, data);
                alignment = fast_transmute!(<of+48, u64>, data);
                entry_size = fast_transmute!(<of+56, u64>, data);
            } else {
                flags = fast_transmute!(<of+8, u32>, data) as u64;
                section_address = fast_transmute!(<of+12, u32>, data) as u64;
                section_offset = fast_transmute!(<of+16, u32>, data) as u64;
                section_size = fast_transmute!(<of+20, u32>, data) as u64;
                link = fast_transmute!(<of+24, u32>, data);
                info = fast_transmute!(<of+28, u32>, data);
                alignment = fast_transmute!(<of+32, u32>, data) as u64;
                entry_size = fast_transmute!(<of+36, u32>, data) as u64;
            }
        }

        v.push(SectionHeader {
            name,
            section_type,
            flags,
            section_address,
            section_offset,
            section_size,
            link,
            info,
            alignment,
            entry_size,
            name_str: None,
        })
    }

    v
}

pub fn section_header_final(
    data: &[u8],
    v: &mut Vec<SectionHeader>,
) -> Result<Vec<Rc<str>>, EmulatorError> {
    let strtab = v
        .iter()
        .find(|x| x.section_type == SectionHeaderType::ShtStrtab)
        .ok_or(EmulatorError::StrTabError)?
        .clone();

    let mut list = vec![];

    for e in v {
        let name_start = strtab.section_offset + e.name as u64 + strtab.section_size;
        let mut name_vec = vec![];
        for i in name_start.. {
            if data[i as usize] == b'\0' {
                break;
            }
            name_vec.push(data[i as usize])
        }
        let name: Rc<str> = String::from_utf8(name_vec)?.as_str().into();
        e.name_str = Some(name.clone());
        list.push(name.clone())
    }

    Ok(list)
}
