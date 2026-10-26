//! The EE core's timing model at instruction granularity (Sapphire_Plan.md §5.2, Q14): the issue and latency table
//! of the EE Core User's Manual, pinned as data, the in-order two-way issue it describes, and from S7 the caches'
//! refills and the bus's transfers, from S8 the multimedia rows on both multipliers, from S9 the floating-point rows
//! with the FPU's registers, its accumulator, FCR31 and its divider, from S12 COP2's macro rows with VU0's registers by
//! number and VU0's divider (VU User's Manual §5.4). Sapphire_Native.md §16.4, §17.5, §18.4, §19.5 and §21.5 record each
//! rule's source and the losses named.

/// A physical pipe of Table 1-1 (EE Core User's Manual §1.3.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pipe {
    I0,
    I1,
    Ls,
    Br,
    C1,
    C2,
}

/// An instruction category of Table 1-1, which decides the logical pipes it may issue to (Table 1-2); its code is its
/// place in Table 1-1's order (`CATEGORIES`), which the state and the recompiler's code read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Category {
    LoadStore,
    Sync,
    Lzc,
    Eret,
    SaOperate,
    Cop0,
    Cop1Move,
    Cop2Move,
    Cop1Operate,
    Cop2Operate,
    Alu,
    Mac0,
    Mac1,
    Branch,
    WideOperate,
}

/// The last category's code before any instruction has issued.
pub const NO_CATEGORY: u8 = 0xFF;

/// The logical pipes a category may issue to, Table 1-2: (pipe 0, pipe 1).
pub const fn logical(c: Category) -> (bool, bool) {
    use Category::*;
    match c {
        LoadStore | Sync | Lzc | Eret | Cop0 | Cop1Move | Cop2Move | Mac1 => (false, true),
        SaOperate | Cop1Operate | Cop2Operate | Mac0 | WideOperate => (true, false),
        Alu | Branch => (true, true),
    }
}

/// Table 1-3: whether a category in pipe 1 issues beside one in pipe 0; `Y` issues both and stalls one cycle in A.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pair {
    O,
    X,
    Y,
}

/// Table 1-3's columns, the categories pipe 0 takes, in the manual's order.
pub const PIPE0: [Category; 7] =
    [Category::SaOperate, Category::Cop1Operate, Category::Cop2Operate, Category::Alu, Category::Mac0, Category::Branch, Category::WideOperate];

/// Table 1-3's rows, the categories pipe 1 takes, in the manual's order, each with its row of the table.
pub const PIPE1: [(Category, [Pair; 7]); 10] = {
    use Pair::*;
    [
        (Category::LoadStore, [O, O, O, O, O, O, O]),
        (Category::Eret, [O, O, O, O, O, X, O]),
        (Category::Sync, [O, O, O, O, O, O, O]),
        (Category::Lzc, [O, O, O, O, O, O, Y]),
        (Category::Cop1Move, [O, Y, O, O, O, O, O]),
        (Category::Cop2Move, [O, O, Y, O, O, O, O]),
        (Category::Alu, [O, O, O, O, O, O, Y]),
        (Category::Mac1, [O, O, O, O, O, O, Y]),
        (Category::Branch, [O, O, O, O, O, X, O]),
        (Category::Cop0, [O, O, O, O, O, O, O]),
    ]
};

/// Table 1-3's cell for `p1` in pipe 1 beside `p0` in pipe 0; `None` where the table has no such row or column.
pub fn pair(p1: Category, p0: Category) -> Option<Pair> {
    let col = PIPE0.iter().position(|&c| c == p0)?;
    PIPE1.iter().find(|(c, _)| *c == p1).map(|(_, row)| row[col])
}

/// One row of §2.12's latency table: the execution pipe(s), the latency and the throughput (repeat rate) in cycles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    pub name: &'static str,
    pub pipes: &'static [Pipe],
    pub latency: u8,
    pub throughput: u8,
}

const fn row(name: &'static str, pipes: &'static [Pipe], latency: u8, throughput: u8) -> Row {
    Row { name, pipes, latency, throughput }
}

/// §2.12's "Integer Instructions", in its order. Load and store assume a cache hit; a store has no latency.
pub const INTEGER: [Row; 15] = [
    row("Add/Sub, and logical operations", &[Pipe::I0, Pipe::I1], 1, 1),
    row("Transfer to HI / LO registers", &[Pipe::I0, Pipe::I1], 1, 1),
    row("Shift / LUI", &[Pipe::I0, Pipe::I1], 1, 1),
    row("Branch / Jump", &[Pipe::Br], 1, 1),
    row("Conditional Move", &[Pipe::I0, Pipe::I1], 1, 1),
    row("MULT / MULTU", &[Pipe::I0], 4, 2),
    row("MULT1 / MULTU1", &[Pipe::I1], 4, 2),
    row("DIV / DIVU", &[Pipe::I0], 37, 37),
    row("DIV1 / DIVU1", &[Pipe::I1], 37, 37),
    row("MADD / MADDU", &[Pipe::I0], 4, 2),
    row("MADD1 / MADDU1", &[Pipe::I1], 4, 2),
    row("Load", &[Pipe::Ls], 1, 1),
    row("Store", &[Pipe::Ls], 0, 1),
    row("Multimedia Multiply", &[Pipe::I0, Pipe::I1], 4, 2),
    row("Multimedia Divide", &[Pipe::I0, Pipe::I1], 37, 37),
];

/// §2.12's "Floating-Point Instructions", in its order, used from S9 as rows `rows::FLOAT` onward.
pub const FLOAT: [Row; 11] = [
    row("MTC1", &[Pipe::C1, Pipe::Ls], 2, 1),
    row("Add / Sub / Abs / Neg / C.cond", &[Pipe::C1], 4, 1),
    row("CVT", &[Pipe::C1], 4, 1),
    row("Mul", &[Pipe::C1], 4, 1),
    row("MFC1", &[Pipe::C1, Pipe::Ls], 2, 1),
    row("Move", &[Pipe::C1], 4, 1),
    row("DIV.S", &[Pipe::C1], 8, 7),
    row("SQRT.S", &[Pipe::C1], 8, 7),
    row("RSQRT.S", &[Pipe::C1], 14, 13),
    row("MADD", &[Pipe::C1], 4, 1),
    row("LWC1", &[Pipe::C1, Pipe::Ls], 2, 1),
];

/// COP2's macro rows, from S12 (VU User's Manual §5.4.1 to §5.4.4, §7.4), as the macro instruction reader sees them:
/// an FMAC result four cycles on, an integer one the next cycle, a transfer to VU0 two cycles on (§5.4.4's stall).
pub const COP2: [Row; 6] = [
    row("FMAC", &[Pipe::C2], 4, 1),
    row("IALU", &[Pipe::C2], 1, 1),
    row("VDIV / VSQRT", &[Pipe::C2], 7, 7),
    row("VRSQRT", &[Pipe::C2], 13, 13),
    row("QMTC2 / CTC2 / LQC2", &[Pipe::C2, Pipe::Ls], 2, 1),
    row("QMFC2 / CFC2 / SQC2", &[Pipe::C2, Pipe::Ls], 1, 1),
];

/// The integer rows by index, for the instruction table.
pub mod rows {
    pub const ALU: usize = 0;
    pub const HILO: usize = 1;
    pub const SHIFT: usize = 2;
    pub const BRANCH: usize = 3;
    pub const CMOVE: usize = 4;
    pub const MULT: usize = 5;
    pub const MULT1: usize = 6;
    pub const DIV: usize = 7;
    pub const DIV1: usize = 8;
    pub const MADD: usize = 9;
    pub const MADD1: usize = 10;
    pub const LOAD: usize = 11;
    pub const STORE: usize = 12;
    pub const MMULT: usize = 13;
    pub const MMDIV: usize = 14;
    /// The floating-point rows follow the integer ones, in `FLOAT`'s order.
    pub const FLOAT: usize = 15;
    pub const MTC1: usize = FLOAT;
    pub const FADD: usize = FLOAT + 1;
    pub const CVT: usize = FLOAT + 2;
    pub const FMUL: usize = FLOAT + 3;
    pub const MFC1: usize = FLOAT + 4;
    pub const FMOVE: usize = FLOAT + 5;
    pub const FDIV: usize = FLOAT + 6;
    pub const FSQRT: usize = FLOAT + 7;
    pub const FRSQRT: usize = FLOAT + 8;
    pub const FMADD: usize = FLOAT + 9;
    pub const LWC1: usize = FLOAT + 10;
    /// COP2's rows follow the floating-point ones, in `COP2`'s order.
    pub const COP2: usize = FLOAT + 11;
    pub const VFMAC: usize = COP2;
    pub const VIALU: usize = COP2 + 1;
    pub const VDIV: usize = COP2 + 2;
    pub const VRSQRT: usize = COP2 + 3;
    pub const VTO: usize = COP2 + 4;
    pub const VFROM: usize = COP2 + 5;
}

/// A row of either table by its index (`rows`).
pub fn row_of(i: usize) -> &'static Row {
    if i < rows::FLOAT {
        &INTEGER[i]
    } else if i < rows::COP2 {
        &FLOAT[i - rows::FLOAT]
    } else {
        &COP2[i - rows::COP2]
    }
}

/// The resources the scoreboard tracks: the 32 GPRs, then HI/LO, HI1/LO1 and SA, the 32 FPRs, ACC and FCR31, and VU0's
/// registers 1 to 31 by number, a VF and the VI or control register of its number being one (§5.4.2).
pub const HILO0: usize = 32;
pub const HILO1: usize = 33;
pub const SA: usize = 34;
pub const FPR0: usize = 35;
pub const ACC: usize = FPR0 + 32;
pub const FCR31: usize = ACC + 1;
/// VU0's register `n` (1 to 31) is resource `VU0R + n - 1`; register 0 is no hazard.
pub const VU0R: usize = FCR31 + 1;
const RESOURCES: usize = VU0R + 31;
/// Every resource's bit, so that only these are visited.
const ALL: u128 = if RESOURCES >= 128 { u128::MAX } else { (1u128 << RESOURCES) - 1 };
/// The VU0 resources, for a transfer from VU0, which reads a cycle after a macro instruction would (§5.4.4).
const VU0_MASK: u128 = ((1u128 << 31) - 1) << VU0R;

/// A cache line's refill: "a burst of 4 bus cycles (8 CPU cycles)" (EE Core User's Manual §6.3.3), the minimum.
pub const REFILL: u64 = 8;
/// A dirty line written back before its refill (§6.3.4), taken as the same burst [A].
pub const WRITEBACK: u64 = 8;
/// One uncached transfer: one cycle of the bus at half the core's clock (Config.EC) [A].
pub const UNCACHED: u64 = 2;
/// The uncached accelerated buffer's refill of 128 bytes, eight bus cycles [A].
pub const UCAB_REFILL: u64 = 16;

/// What one instruction asks of the model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Issue {
    pub category: Category,
    pub row: usize,
    /// Resources read, as a bit per resource index.
    pub reads: u128,
    /// Resources written.
    pub writes: u128,
    /// A control transfer that is taken ends the issue group after its delay slot.
    pub taken: bool,
    /// Cycles its fetch waited for the instruction cache's refill or an uncached fetch.
    pub fetch: u64,
    /// Cycles its data access waited for memory: a refill, a write-back, an uncached or UCAB transfer.
    pub data: u64,
    /// Config.NBE clear: a data wait stalls the pipe rather than only the result (§6.3.2).
    pub blocking: bool,
    /// Config.DIE clear: one instruction a cycle (§3.2, "Config Register").
    pub single: bool,
    /// The earliest cycle it may issue in for a reason outside the pipe: VU0 running a microprogram, a VWAITQ.
    pub earliest: u64,
    /// Nothing issues beside it: VWAITQ, which holds the pipeline until Q is written (§5.4.7).
    pub alone: bool,
}

impl Issue {
    pub fn new(category: Category, row: usize) -> Issue {
        Issue { category, row, reads: 0, writes: 0, taken: false, fetch: 0, data: 0, blocking: false, single: false, earliest: 0, alone: false }
    }
}

/// Where an instruction would issue: its cycle, whether beside the one before, in which pipe, and Table 1-3's stall.
#[derive(Clone, Copy, Debug)]
struct Plan {
    at: u64,
    paired: bool,
    pipe: u8,
    stall: u64,
}

/// The in-order two-way issue of §1.3.6: each instruction issues in the cycle its operands are ready and its unit is
/// free, beside the one before it when Table 1-3 and the pipes allow, never ahead of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timing {
    ready: [u64; RESOURCES],
    mac0_free: u64,
    mac1_free: u64,
    /// The cycle the FPU's divider, which DIV.S, SQRT.S and RSQRT.S share, takes its next operation [A].
    fdiv_free: u64,
    /// The cycle VU0's divider takes its next macro VDIV, VSQRT or VRSQRT (§5.4.2, "DIV resource hazard").
    vdiv_free: u64,
    /// The earliest cycle of the next COP2 operation after a transfer to VU0 (§5.4.4: "always stalls for one cycle").
    cop2_floor: u64,
    /// The cycle the last instruction issued in, its category and pipe, its writes, and whether it may take a partner.
    last: u64,
    /// The last category's code (`Category as u8`), `NO_CATEGORY` before the first.
    last_category: u8,
    last_pipe: u8,
    last_writes: u128,
    open: bool,
    /// The earliest cycle the next instruction may issue in (a Y pair's stall, a group's end).
    floor: u64,
    group_ends: u8,
    /// Whether the last instruction issued beside the one before it.
    paired: bool,
    /// The cycle the bus is free of an earlier miss (§6.3.2: a second miss waits for the first).
    mem_free: u64,
    pub instructions: u64,
}

impl Default for Timing {
    fn default() -> Self {
        Timing {
            ready: [0; RESOURCES],
            mac0_free: 0,
            mac1_free: 0,
            fdiv_free: 0,
            vdiv_free: 0,
            cop2_floor: 0,
            last: 0,
            last_category: NO_CATEGORY,
            last_pipe: 0,
            last_writes: 0,
            open: false,
            floor: 0,
            group_ends: 0,
            paired: false,
            mem_free: 0,
            instructions: 0,
        }
    }
}

impl Timing {
    /// The cycles elapsed: the last issue's cycle and the one it took.
    pub fn cycles(&self) -> u64 {
        if self.instructions == 0 { 0 } else { self.last + 1 }
    }

    /// Whether the last instruction charged issued beside the one before it.
    pub fn paired(&self) -> bool {
        self.paired
    }

    /// The cycle an instruction would issue in, the model unchanged.
    pub fn would_issue(&self, i: Issue) -> u64 {
        self.plan(&i).at
    }

    /// Charges one instruction and returns the cycle it issued in.
    pub fn issue(&mut self, i: Issue) -> u64 {
        let p = self.plan(&i);
        self.commit(&i, p)
    }

    fn uses_mac0(i: &Issue) -> bool {
        matches!(i.row, rows::MULT | rows::DIV | rows::MADD | rows::MMULT | rows::MMDIV)
    }
    fn uses_mac1(i: &Issue) -> bool {
        matches!(i.row, rows::MULT1 | rows::DIV1 | rows::MADD1 | rows::MMULT | rows::MMDIV)
    }

    fn plan(&self, i: &Issue) -> Plan {
        let next = if self.instructions == 0 { 0 } else { self.last + 1 };
        let mut earliest = if i.fetch > 0 { next.max(self.floor) + i.fetch } else { 0 };
        earliest = earliest.max(i.earliest);
        if i.data > 0 {
            earliest = earliest.max(self.mem_free);
        }
        let from_vu = i.row == rows::VFROM;
        let mut reads = i.reads & ALL;
        while reads != 0 {
            let r = reads.trailing_zeros() as usize;
            reads &= reads - 1;
            let late = u64::from(from_vu && VU0_MASK >> r & 1 == 1);
            earliest = earliest.max(self.ready[r] + late);
        }
        if Self::uses_mac0(i) {
            earliest = earliest.max(self.mac0_free);
        }
        if Self::uses_mac1(i) {
            earliest = earliest.max(self.mac1_free);
        }
        if matches!(i.row, rows::FDIV | rows::FSQRT | rows::FRSQRT) {
            earliest = earliest.max(self.fdiv_free);
        }
        if matches!(i.row, rows::VDIV | rows::VRSQRT) {
            earliest = earliest.max(self.vdiv_free);
        }
        if i.category == Category::Cop2Operate {
            earliest = earliest.max(self.cop2_floor);
        }
        let (p0, p1) = logical(i.category);
        let partner = if self.instructions > 0
            && self.open
            && i.fetch == 0
            && !i.single
            && earliest <= self.last
            && i.writes & self.last_writes == 0
            && i.reads & self.last_writes == 0
        {
            self.partner(i.category, p0, p1)
        } else {
            None
        };
        match partner {
            Some((pipe, cell)) => Plan { at: self.last, paired: true, pipe, stall: u64::from(cell == Pair::Y) },
            None => Plan { at: earliest.max(self.floor).max(next), paired: false, pipe: if p0 { 0 } else { 1 }, stall: 0 },
        }
    }

    fn commit(&mut self, i: &Issue, p: Plan) -> u64 {
        let row = row_of(i.row);
        let (at, stall) = (p.at, p.stall);
        self.last_pipe = p.pipe;
        let done = at + stall + u64::from(row.latency.max(1)) + i.data;
        if i.data > 0 {
            self.mem_free = at + 1 + i.data;
        }
        let mut writes = i.writes & ALL;
        while writes != 0 {
            self.ready[writes.trailing_zeros() as usize] = done;
            writes &= writes - 1;
        }
        if Self::uses_mac0(i) {
            self.mac0_free = at + u64::from(row.throughput);
        }
        if Self::uses_mac1(i) {
            self.mac1_free = at + u64::from(row.throughput);
        }
        if matches!(i.row, rows::FDIV | rows::FSQRT | rows::FRSQRT) {
            self.fdiv_free = at + u64::from(row.throughput);
        }
        if matches!(i.row, rows::VDIV | rows::VRSQRT) {
            self.vdiv_free = at + u64::from(row.throughput);
        }
        if i.row == rows::VTO {
            self.cop2_floor = at + 2;
        }
        let paired = p.paired;
        self.open = !paired && !i.alone;
        self.last = at;
        self.last_category = i.category as u8;
        self.last_writes = if paired { self.last_writes | i.writes } else { i.writes };
        self.floor = at + 1 + stall + if i.blocking { i.data } else { 0 };
        self.paired = paired;
        if self.group_ends > 0 {
            self.group_ends -= 1;
            if self.group_ends == 0 {
                self.open = false;
            }
        }
        if i.taken {
            self.group_ends = 1;
        }
        self.instructions += 1;
        at
    }

    /// The pipe this instruction would take beside the last, and Table 1-3's cell, if the two may issue together.
    fn partner(&self, c: Category, p0: bool, p1: bool) -> Option<(u8, Pair)> {
        let last = *CATEGORIES.get(usize::from(self.last_category))?;
        partner_of(c, p0, p1, last, self.last_pipe)
    }
}

/// The pipe a category `c` (with logical pipes `p0`, `p1`) would take beside `last`, issued in pipe `last_pipe`, and
/// Table 1-3's cell, if the two may issue together.
pub fn partner_of(c: Category, p0: bool, p1: bool, last: Category, last_pipe: u8) -> Option<(u8, Pair)> {
    let (l0, l1) = logical(last);
    let mut best: Option<(u8, Pair)> = None;
    for (mine, theirs) in [(0u8, 1u8), (1, 0)] {
        let fits_me = if mine == 0 { p0 } else { p1 };
        let fits_them = if theirs == 0 { l0 } else { l1 };
        let same_pipe_as_issued = (l0 && l1) || theirs == last_pipe;
        if !(fits_me && fits_them && same_pipe_as_issued) {
            continue;
        }
        let cell = if mine == 1 { pair(c, last) } else { pair(last, c) };
        match cell {
            Some(Pair::O) => return Some((mine, Pair::O)),
            Some(Pair::Y) => best = best.or(Some((mine, Pair::Y))),
            _ => {}
        }
    }
    best
}

/// Where the issue model's fields lie, for the recompiler's code, which reads and writes them as the model does
/// (Sapphire_Native.md §44).
pub(crate) mod layout {
    use super::Timing;
    use std::mem::offset_of;

    pub const READY: i32 = offset_of!(Timing, ready) as i32;
    pub const MAC0_FREE: i32 = offset_of!(Timing, mac0_free) as i32;
    pub const MAC1_FREE: i32 = offset_of!(Timing, mac1_free) as i32;
    pub const LAST: i32 = offset_of!(Timing, last) as i32;
    pub const LAST_CATEGORY: i32 = offset_of!(Timing, last_category) as i32;
    pub const LAST_PIPE: i32 = offset_of!(Timing, last_pipe) as i32;
    pub const LAST_WRITES: i32 = offset_of!(Timing, last_writes) as i32;
    pub const OPEN: i32 = offset_of!(Timing, open) as i32;
    pub const FLOOR: i32 = offset_of!(Timing, floor) as i32;
    pub const GROUP_ENDS: i32 = offset_of!(Timing, group_ends) as i32;
    pub const PAIRED: i32 = offset_of!(Timing, paired) as i32;
    pub const INSTRUCTIONS: i32 = offset_of!(Timing, instructions) as i32;
}

/// The categories in Table 1-1's order, a category's code in `SPH2` its place here.
pub const CATEGORIES: [Category; 15] = {
    use Category::*;
    [LoadStore, Sync, Lzc, Eret, SaOperate, Cop0, Cop1Move, Cop2Move, Cop1Operate, Cop2Operate, Alu, Mac0, Mac1, Branch, WideOperate]
};

/// The issue model's block of `SPH2`: every resource's ready cycle, the units' free cycles, and the last issue.
impl emusen_native::State for Timing {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        w.u64s("Ready", &self.ready);
        w.u64s("Free", &[self.mac0_free, self.mac1_free, self.fdiv_free, self.vdiv_free, self.cop2_floor, self.last, self.floor, self.mem_free, self.instructions]);
        w.u8("LastCategory", self.last_category);
        w.u8("LastPipe", self.last_pipe);
        w.u128("LastWrites", self.last_writes);
        w.bools("Open", &[self.open, self.paired]);
        w.u8("GroupEnds", self.group_ends);
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        r.u64s(&mut self.ready)?;
        let mut f = [0u64; 9];
        r.u64s(&mut f)?;
        [self.mac0_free, self.mac1_free, self.fdiv_free, self.vdiv_free, self.cop2_floor, self.last, self.floor, self.mem_free, self.instructions] = f;
        let c = r.u8()?;
        self.last_category = if usize::from(c) < CATEGORIES.len() { c } else { NO_CATEGORY };
        self.last_pipe = r.u8()?;
        self.last_writes = r.u128()?;
        let mut b = [false; 2];
        r.bools(&mut b)?;
        [self.open, self.paired] = b;
        self.group_ends = r.u8()?;
        Ok(())
    }
}
