//! COP0's registers (EE Core User's Manual §3.2, Table 3-1): their numbers, the bits MTC0 may change, the fields the
//! core reads; the debug registers behind register 24 (chapter 9) and the performance counters behind 25 (chapter 7).
//! Sapphire_Native.md §17.1 has the readings the manual leaves open.

pub const INDEX: usize = 0;
pub const RANDOM: usize = 1;
pub const ENTRYLO0: usize = 2;
pub const ENTRYLO1: usize = 3;
pub const CONTEXT: usize = 4;
pub const PAGEMASK: usize = 5;
pub const WIRED: usize = 6;
pub const BADVADDR: usize = 8;
pub const COUNT: usize = 9;
pub const ENTRYHI: usize = 10;
pub const COMPARE: usize = 11;
pub const STATUS: usize = 12;
pub const CAUSE: usize = 13;
pub const EPC: usize = 14;
pub const PRID: usize = 15;
pub const CONFIG: usize = 16;
pub const BADPADDR: usize = 23;
pub const TAGLO: usize = 28;
pub const TAGHI: usize = 29;
pub const ERROREPC: usize = 30;

/// The bits MTC0 may change, by §3.2's r/w columns; Random, BadVAddr, Cause, PRId and the reserved numbers take none.
pub const fn writable(r: usize) -> u32 {
    match r {
        INDEX | WIRED => 0x0000_003F,
        ENTRYLO0 => 0x83FF_FFFF,
        ENTRYLO1 => 0x03FF_FFFF,
        CONTEXT => 0xFF80_0000,
        PAGEMASK => 0x01FF_E000,
        ENTRYHI => 0xFFFF_E0FF,
        STATUS => 0xF0C7_9C1F,
        CONFIG => 0x0007_3007,
        BADPADDR => 0xFFFF_FFF0,
        COUNT | COMPARE | EPC | TAGLO | TAGHI | ERROREPC => u32::MAX,
        _ => 0,
    }
}

/// Cause's fields (§3.2, "Cause Register").
pub mod cause {
    pub const BD: u32 = 1 << 31;
    pub const BD2: u32 = 1 << 30;
    pub const IP7: u32 = 1 << 15;
    /// Int1 (the DMAC's line, ps2tek) and Int0 (INTC's): Sapphire_Disputes.md D-15.
    pub const IP3: u32 = 1 << 11;
    pub const IP2: u32 = 1 << 10;
    pub const IP: u32 = IP7 | IP3 | IP2;
    pub const EXC2_SHIFT: u32 = 16;
}

/// Config's fields (§3.2, "Config Register"); IC and DC read 16 KiB and 8 KiB.
pub mod config {
    pub const DIE: u32 = 1 << 18;
    pub const ICE: u32 = 1 << 17;
    pub const DCE: u32 = 1 << 16;
    pub const NBE: u32 = 1 << 13;
    pub const BPE: u32 = 1 << 12;
    pub const SIZES: u32 = 2 << 9 | 1 << 6;
    pub const K0: u32 = 7;
}

/// The debug registers by the move's number (§9.2): BPC, then IAB, IABM, DAB, DABM, DVB and DVBM at 2 to 7.
pub mod debug {
    pub const BPC: usize = 0;
    pub const IAB: usize = 2;
    pub const IABM: usize = 3;
    pub const DAB: usize = 4;
    pub const DABM: usize = 5;
    pub const DVB: usize = 6;
    pub const DVBM: usize = 7;

    pub const IAE: u32 = 1 << 31;
    pub const DRE: u32 = 1 << 30;
    pub const DWE: u32 = 1 << 29;
    pub const DVE: u32 = 1 << 28;
    /// IUE, ISE, IKE, IXE at 26 to 23; DUE, DSE, DKE, DXE at 21 to 18.
    pub const IUE: u32 = 1 << 26;
    pub const DUE: u32 = 1 << 21;
    /// BED: a debug exception is not generated when set (§9.2.1; Sapphire_Disputes.md D-14).
    pub const BED: u32 = 1 << 15;
    pub const DWB: u32 = 1 << 2;
    pub const DRB: u32 = 1 << 1;
    pub const IAB_FLAG: u32 = 1;

    /// The bits each move may change: BPC's defined fields, IAB's and IABM's low two bits fixed at zero.
    pub const fn writable(n: usize) -> u32 {
        match n {
            BPC => 0xF7BF_8007,
            IAB | IABM => 0xFFFF_FFFC,
            _ => u32::MAX,
        }
    }
}

/// The performance counter control register's fields (§7.1.1) and its writable bits.
pub mod perf {
    pub const CTE: u32 = 1 << 31;
    pub const WRITABLE: u32 = 0x800F_FBFE;
    pub const OVFL: u32 = 1 << 31;
}

/// One event of Table 7-1: the EVENT value that selects it on PCR0 and on PCR1, where the counter has it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event(pub Option<u32>, pub Option<u32>);

pub mod event {
    use super::Event;
    pub const CYCLE: Event = Event(Some(1), Some(1));
    pub const SINGLE_ISSUE: Event = Event(Some(2), None);
    pub const DUAL_ISSUE: Event = Event(None, Some(2));
    pub const BRANCH_ISSUED: Event = Event(Some(3), None);
    pub const LOW_BRANCH: Event = Event(None, Some(0));
    pub const TLB_MISS: Event = Event(None, Some(4));
    pub const ICACHE_MISS: Event = Event(Some(6), None);
    pub const DCACHE_MISS: Event = Event(None, Some(6));
    pub const DTLB_ACCESS: Event = Event(Some(7), None);
    pub const NONBLOCKING_LOAD: Event = Event(Some(8), None);
    pub const COMPLETED: Event = Event(Some(12), Some(12));
    pub const NON_BDS: Event = Event(Some(13), Some(13));
    pub const COP2_COMPLETED: Event = Event(Some(14), None);
    pub const COP1_COMPLETED: Event = Event(None, Some(14));
    pub const LOAD_COMPLETED: Event = Event(Some(15), None);
    pub const STORE_COMPLETED: Event = Event(None, Some(15));
}

/// Adds `n` to a counter's 31-bit VALUE, setting OVFL on a carry out of bit 30 (§7.1.2).
pub fn add(pcr: u32, n: u64) -> u32 {
    let sum = u64::from(pcr & !perf::OVFL) + n;
    let over = if sum > 0x7FFF_FFFF { perf::OVFL } else { 0 };
    pcr & perf::OVFL | over | (sum & 0x7FFF_FFFF) as u32
}
