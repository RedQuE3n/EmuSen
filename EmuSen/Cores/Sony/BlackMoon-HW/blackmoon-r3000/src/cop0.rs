//! Coprocessor 0 as the PlayStation's R3000A has it: the registers psx-spx's "COP0 Register Summary" lists, the
//! status register's mode stack, Cause, and the debug registers' break conditions (BlackMoon_R3000.md §6).

use emusen_mips::{Command, Coprocessor};

use crate::Registers;

/// PRId of the PlayStation's CXD8606CQ (psx-spx "cop0r15").
pub const PRID_PLAYSTATION: u32 = 0x0000_0002;

pub(crate) mod sr {
    pub const IEC: u32 = 1 << 0;
    pub const KUC: u32 = 1 << 1;
    pub const ISC: u32 = 1 << 16;
    pub const BEV: u32 = 1 << 22;
    pub const CU0: u32 = 1 << 28;
    pub const CU2: u32 = 1 << 30;
    /// What MTC0 changes: the mode stack, IM, IsC, SwC, PZ, BEV, RE and the four CU bits; the rest read zero or report
    /// status (psx-spx "cop0r12").
    pub const WRITABLE: u32 = 0xF247_FF3F;
}

pub(crate) mod cause {
    pub const IP2: u32 = 1 << 10;
    /// The software interrupt bits, the only ones MTC0 changes.
    pub const SW: u32 = 3 << 8;
    /// ExcCode, CE, BT and BD: what an exception writes.
    pub const EXCEPTION: u32 = 0xF000_007C;
}

/// ExcCode values (IDT R30xx manual, Table 3.2).
pub(crate) mod code {
    pub const INT: u32 = 0x00;
    pub const ADEL: u32 = 0x04;
    pub const ADES: u32 = 0x05;
    pub const IBE: u32 = 0x06;
    pub const DBE: u32 = 0x07;
    pub const SYS: u32 = 0x08;
    pub const BP: u32 = 0x09;
    pub const RI: u32 = 0x0A;
    pub const CPU: u32 = 0x0B;
    pub const OV: u32 = 0x0C;
}

pub(crate) mod dcic {
    pub const DB: u32 = 1 << 0;
    pub const PC: u32 = 1 << 1;
    pub const DA: u32 = 1 << 2;
    pub const R: u32 = 1 << 3;
    pub const W: u32 = 1 << 4;
    pub const DE: u32 = 1 << 23;
    pub const PCE: u32 = 1 << 24;
    pub const DAE: u32 = 1 << 25;
    pub const DR: u32 = 1 << 26;
    pub const DW: u32 = 1 << 27;
    pub const KD: u32 = 1 << 29;
    pub const UD: u32 = 1 << 30;
    pub const TR: u32 = 1 << 31;
    /// The bits psx-spx marks R/W; 6-11 and 16-22 read zero.
    pub const WRITABLE: u32 = 0xFF80_F03F;
}

const fn names() -> [&'static str; 32] {
    let mut n = [""; 32];
    n[3] = "BPC";
    n[5] = "BDA";
    n[6] = "TAR";
    n[7] = "DCIC";
    n[8] = "BadVaddr";
    n[9] = "BDAM";
    n[11] = "BPCM";
    n[12] = "SR";
    n[13] = "CAUSE";
    n[14] = "EPC";
    n[15] = "PRID";
    n
}

/// Coprocessor 0's names for the disassembler: its registers and its commands, RFE and the four TLB commands the
/// PlayStation reserves (psx-spx "cop0cmd").
pub static COP0: Coprocessor = Coprocessor {
    data: names(),
    control: [""; 32],
    commands: &[
        Command { name: "rfe", mask: 0x3F, select: 0x10, canonical: 0x10 },
        Command { name: "tlbr", mask: 0x3F, select: 0x01, canonical: 0x01 },
        Command { name: "tlbwi", mask: 0x3F, select: 0x02, canonical: 0x02 },
        Command { name: "tlbwr", mask: 0x3F, select: 0x06, canonical: 0x06 },
        Command { name: "tlbp", mask: 0x3F, select: 0x08, canonical: 0x08 },
    ],
};

pub(crate) fn kernel(r: &Registers) -> bool {
    r.sr & sr::KUC == 0
}

/// MFC0 of register `n`: its value, or `None` for the registers whose read is a reserved instruction (0-2, 4, 10).
pub(crate) fn read(r: &Registers, prid: u32, n: usize) -> Option<u32> {
    Some(match n {
        3 => r.bpc,
        5 => r.bda,
        6 => r.tar,
        7 => r.dcic,
        8 => r.bad_vaddr,
        9 => r.bdam,
        11 => r.bpcm,
        12 => r.sr,
        13 => r.cause,
        14 => r.epc,
        15 => prid,
        16..=31 => 0,
        _ => return None,
    })
}

/// MTC0 of register `n`; the read-only registers and the absent ones ignore it.
pub(crate) fn write(r: &mut Registers, n: usize, v: u32) {
    match n {
        3 => r.bpc = v,
        5 => r.bda = v,
        7 => r.dcic = v & dcic::WRITABLE,
        9 => r.bdam = v,
        11 => r.bpcm = v,
        12 => r.sr = r.sr & !sr::WRITABLE | v & sr::WRITABLE,
        13 => r.cause = r.cause & !cause::SW | v & cause::SW,
        _ => {}
    }
}

/// RFE: KUp/IEp to KUc/IEc and KUo/IEo to KUp/IEp, KUo/IEo left as they were.
pub(crate) fn rfe(r: &mut Registers) {
    r.sr = r.sr & !0xF | r.sr >> 2 & 0xF;
}

/// Whether the debug unit breaks in the current mode.
fn armed(r: &Registers, enable: u32) -> bool {
    let mode = if kernel(r) { dcic::KD } else { dcic::UD };
    r.dcic & (dcic::DE | enable | mode) == dcic::DE | enable | mode
}

/// The PC breakpoint before the instruction at `pc`: its status bits set, and whether it traps.
pub(crate) fn pc_break(r: &mut Registers, pc: u32) -> bool {
    if !armed(r, dcic::PCE) || (pc ^ r.bpc) & r.bpcm != 0 {
        return false;
    }
    r.dcic |= dcic::DB | dcic::PC;
    r.dcic & dcic::TR != 0
}

/// The data breakpoint on a load (`write` false) or store at `address`: its status bits set, and whether it traps.
pub(crate) fn data_break(r: &mut Registers, address: u32, write: bool) -> bool {
    let kind = if write { dcic::DW } else { dcic::DR };
    if !armed(r, dcic::DAE | kind) || (address ^ r.bda) & r.bdam != 0 {
        return false;
    }
    r.dcic |= dcic::DB | dcic::DA | if write { dcic::W } else { dcic::R };
    r.dcic & dcic::TR != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DEBUG_REGISTERS;

    #[test]
    fn the_debugger_lists_the_names_the_disassembler_writes() {
        assert_eq!(DEBUG_REGISTERS[..32], emusen_mips::GPR_NAMES);
        for (name, n) in DEBUG_REGISTERS[35..].iter().zip([12, 13, 14, 8, 6, 7, 3, 5, 11, 9]) {
            assert_eq!(COP0.data[n], *name);
        }
    }
}
