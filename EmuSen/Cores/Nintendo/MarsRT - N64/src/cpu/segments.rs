//! The virtual address map, three maps chosen by the privilege mode: the C# `Segments`. See Mars_Privilege.md §2.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Mode {
    #[default]
    Kernel,
    Supervisor,
    User,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Segment {
    Illegal,
    Mapped,
    Direct(u32),
}

pub const COMPATIBILITY_BASE: u64 = 0xFFFF_FFFF_8000_0000;
const REGION_GAP: u64 = 0x3FFF_FF00_0000_0000;
const PHYSICAL_GAP: u64 = 0x07FF_FFFF_0000_0000;
const KERNEL_SEGMENT_END: u64 = 0xC000_00FF_8000_0000;

#[inline]
pub fn decode(address: u64, mode: Mode, wide: bool) -> Segment {
    if wide && address < COMPATIBILITY_BASE { decode_wide(address, mode) } else { decode_narrow(address, mode) }
}

fn decode_narrow(address: u64, mode: Mode) -> Segment {
    if address != address as u32 as i32 as i64 as u64 {
        return Segment::Illegal;
    }
    let top = ((address >> 29) & 7) as u32;
    if top < 4 {
        return Segment::Mapped;
    }
    match mode {
        Mode::User => Segment::Illegal,
        Mode::Supervisor => {
            if top == 6 {
                Segment::Mapped
            } else {
                Segment::Illegal
            }
        }
        Mode::Kernel => match top {
            4 | 5 => Segment::Direct((address & 0x1FFF_FFFF) as u32),
            _ => Segment::Mapped,
        },
    }
}

fn decode_wide(address: u64, mode: Mode) -> Segment {
    let addressable = address & REGION_GAP == 0;
    match address >> 62 {
        0 => return if addressable { Segment::Mapped } else { Segment::Illegal },
        1 => return if mode != Mode::User && addressable { Segment::Mapped } else { Segment::Illegal },
        3 => {
            return if mode == Mode::Kernel && addressable && address < KERNEL_SEGMENT_END { Segment::Mapped } else { Segment::Illegal };
        }
        _ => {}
    }
    if mode != Mode::Kernel || address & PHYSICAL_GAP != 0 {
        return Segment::Illegal;
    }
    Segment::Direct(address as u32)
}
