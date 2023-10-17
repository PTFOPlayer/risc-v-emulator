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
    program_header: u64,
    section_header_addres: u64,
    elf_header_size: u16,
    program_header_size: u16,
    program_header_entries: u16,
    section_header_size: u16,
    section_header_entries: u16,
    section_header_names: u16
}

pub fn elf_parser(data: &Vec<u8>) -> ELF {
    let mut bin_arc = BinArc::X32;
    let mut endian = Endian::Little;

    let magic_number = &data[0..4];
    println!("{:?}", magic_number);
    let magic = unsafe {
        std::mem::transmute([
            magic_number[0],
            magic_number[1],
            magic_number[2],
            magic_number[3],
        ])
    };

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

    let abi: ABI = unsafe { std::mem::transmute([data[7], data[8]]) };

    let _padding = &data[9..16];

    let obj_type = unsafe { std::mem::transmute([data[16], data[17]]) };

    // 0xF3 means RISC-V
    let arc: u16 = unsafe { std::mem::transmute([data[18], data[19]]) };
    if arc != 0xF3 && arc != 0x00 {
        panic!("this ELF is not for RISC-V architecture: {:x}", arc)
    } else if arc == 0x00 {
        println!(
            "\x1b[93mWARNING\x1b[0m: no guarantee that this ELF is made for RISC-V architecture"
        )
    }
    let _e_version: u32 = unsafe { std::mem::transmute([data[20], data[21], data[22], data[23]]) };

    let entry_point;
    let program_header;
    let section_header_addres;
    let rp; //reference point

    if bin_arc == BinArc::X64 {
        entry_point = unsafe {
            std::mem::transmute([
                data[24], data[25], data[26], data[27], data[28], data[29], data[30], data[31],
            ])
        };
        program_header = unsafe {
            std::mem::transmute([
                data[32], data[33], data[34], data[35], data[36], data[37], data[38], data[39],
            ])
        };
        section_header_addres = unsafe {
            std::mem::transmute([
                data[40], data[41], data[42], data[43], data[44], data[45], data[46], data[47],
            ])
        };
        rp = 48;
    } else {
        entry_point = unsafe {
            let temp: u32 = std::mem::transmute([data[24], data[25], data[26], data[27]]);
            temp as u64
        };
        program_header = unsafe {
            let temp: u32 = std::mem::transmute([data[28], data[29], data[30], data[31]]);
            temp as u64
        };
        section_header_addres = unsafe {
            let temp: u32 = std::mem::transmute([data[32], data[33], data[34], data[35]]);
            temp as u64
        };

        rp = 36;
    }

    let _unknown: u32 =
        unsafe { std::mem::transmute([data[rp], data[rp + 1], data[rp + 2], data[rp + 3]]) };

    let elf_header_size: u16 = unsafe { std::mem::transmute([data[rp + 4], data[rp + 5]]) };
    let program_header_size: u16 = unsafe { std::mem::transmute([data[rp + 6], data[rp + 7]]) };
    let program_header_entries: u16 = unsafe { std::mem::transmute([data[rp + 8], data[rp + 9]]) };
    let section_header_size: u16 =
        unsafe { std::mem::transmute([data[rp + 10], data[rp + 11]]) };
    let section_header_entries: u16 = unsafe { std::mem::transmute([data[rp + 12], data[rp + 13]]) };
    let section_header_names: u16 = unsafe { std::mem::transmute([data[rp + 14], data[rp + 15]]) };
    ELF {
        magic,
        bin_arc,
        endian,
        abi,
        obj_type,
        entry_point,
        program_header,
        section_header_addres,
        elf_header_size,
        program_header_size,
        program_header_entries,
        section_header_size,
        section_header_entries,
        section_header_names,
    }
}

pub fn section_header_parser(data: &Vec<u8>, elf: &ELF) {
    let start = elf.section_header_addres;
    let bin_arc = &elf.bin_arc;
}
