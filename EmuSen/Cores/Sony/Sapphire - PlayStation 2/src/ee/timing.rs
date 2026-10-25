//! The EE core's timing model at instruction granularity (Sapphire_Plan.md §5.2, Q14): the issue and latency table
//! of the EE Core User's Manual, pinned as data, and the in-order two-way issue it describes. Sapphire_Native.md §16.4
//! records each rule's source and the losses the model names.

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

/// An instruction category of Table 1-1, which decides the logical pipes it may issue to (Table 1-2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
pub const PIPE0: [Category; 7] = [
    Category::SaOperate,
    Category::Cop1Operate,
    Category::Cop2Operate,
    Category::Alu,
    Category::Mac0,
    Category::Branch,
    Category::WideOperate,
];

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

/// §2.12's "Floating-Point Instructions", in its order, pinned now and used from S9.
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
}

/// The resources the scoreboard tracks: the 32 GPRs, then HI/LO, HI1/LO1 and SA.
pub const HILO0: usize = 32;
pub const HILO1: usize = 33;
pub const SA: usize = 34;
const RESOURCES: usize = 35;

/// What one instruction asks of the model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Issue {
    pub category: Category,
    pub row: usize,
    /// Resources read, as a bit per resource index.
    pub reads: u64,
    /// Resources written.
    pub writes: u64,
    /// A control transfer that is taken ends the issue group after its delay slot.
    pub taken: bool,
}

/// The in-order two-way issue of §1.3.6: each instruction issues in the cycle its operands are ready and its unit is
/// free, beside the one before it when Table 1-3 and the pipes allow, never ahead of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timing {
    ready: [u64; RESOURCES],
    mac0_free: u64,
    mac1_free: u64,
    /// The cycle the last instruction issued in, its category and pipe, its writes, and whether it may take a partner.
    last: u64,
    last_category: Option<Category>,
    last_pipe: u8,
    last_writes: u64,
    open: bool,
    /// The earliest cycle the next instruction may issue in (a Y pair's stall, a group's end).
    floor: u64,
    group_ends: u8,
    pub instructions: u64,
}

impl Default for Timing {
    fn default() -> Self {
        Timing {
            ready: [0; RESOURCES],
            mac0_free: 0,
            mac1_free: 0,
            last: 0,
            last_category: None,
            last_pipe: 0,
            last_writes: 0,
            open: false,
            floor: 0,
            group_ends: 0,
            instructions: 0,
        }
    }
}

impl Timing {
    /// The cycles elapsed: the last issue's cycle and the one it took.
    pub fn cycles(&self) -> u64 {
        if self.instructions == 0 { 0 } else { self.last + 1 }
    }

    /// Charges one instruction and returns the cycle it issued in.
    pub fn issue(&mut self, i: Issue) -> u64 {
        let row = &INTEGER[i.row];
        let mut earliest = 0;
        for r in 0..RESOURCES {
            if i.reads >> r & 1 == 1 {
                earliest = earliest.max(self.ready[r]);
            }
        }
        let uses_mac0 = matches!(i.row, rows::MULT | rows::DIV | rows::MADD);
        let uses_mac1 = matches!(i.row, rows::MULT1 | rows::DIV1 | rows::MADD1);
        if uses_mac0 {
            earliest = earliest.max(self.mac0_free);
        }
        if uses_mac1 {
            earliest = earliest.max(self.mac1_free);
        }
        let (p0, p1) = logical(i.category);
        let mut stall = 0;
        let paired = self.instructions > 0
            && self.open
            && earliest <= self.last
            && i.writes & self.last_writes == 0
            && i.reads & self.last_writes == 0
            && match self.partner(i.category, p0, p1) {
                Some((pipe, cell)) => {
                    stall = u64::from(cell == Pair::Y);
                    self.last_pipe = pipe;
                    true
                }
                None => false,
            };
        let at = if paired { self.last } else { earliest.max(self.floor).max(if self.instructions == 0 { 0 } else { self.last + 1 }) };
        if !paired {
            self.last_pipe = if p0 { 0 } else { 1 };
        }
        let done = at + stall + u64::from(row.latency.max(1));
        for r in 0..RESOURCES {
            if i.writes >> r & 1 == 1 {
                self.ready[r] = done;
            }
        }
        if uses_mac0 {
            self.mac0_free = at + u64::from(row.throughput);
        }
        if uses_mac1 {
            self.mac1_free = at + u64::from(row.throughput);
        }
        self.open = !paired;
        self.last = at;
        self.last_category = Some(i.category);
        self.last_writes = if paired { self.last_writes | i.writes } else { i.writes };
        self.floor = at + 1 + stall;
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
        let last = self.last_category?;
        let (l0, l1) = logical(last);
        let mut best: Option<(u8, Pair)> = None;
        for (mine, theirs) in [(0u8, 1u8), (1, 0)] {
            let fits_me = if mine == 0 { p0 } else { p1 };
            let fits_them = if theirs == 0 { l0 } else { l1 };
            let same_pipe_as_issued = (l0 && l1) || theirs == self.last_pipe;
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
}
