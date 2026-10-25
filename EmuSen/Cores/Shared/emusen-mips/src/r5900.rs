//! The R5900, the PlayStation 2's EE core, as its own table: the MIPS III subset it implements, the multimedia
//! instructions, the second multiplier, LQ and SQ, its COP0 with the debug and performance moves, its single-precision
//! COP1, and COP2 as VU0's macro mode, whose operations are `vu`'s. The map is the TX79 manual's opcode tables (Toshiba,
//! TX79 Core Architecture, rev. 2.0, A.5, B.5, C.2, D.4) where the EE agrees, and ps2tek's "EE Instruction Decoding"
//! where it differs, each difference a dispute (Sapphire_Disputes.md D-1 to D-3). Written in Appendix A's spelling, as
//! the R3000's are. A pure decode of one word; nothing executes (EmuSen_Mips.md §8).

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::GPR_NAMES;
use crate::asm::{AsmError, err, gpr, number, ranged, split_operands};
use crate::vr4300::{Instruction, Reference};
use crate::vu;

/// Which register file a load or store moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Gpr,
    Fpr,
    Vf,
}

/// An instruction's operands, in the order the TX79 manual's Format lines write them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    None,
    RdRsRt,
    RdRtRs,
    RdRtSa,
    RdRs,
    RdRt,
    RsRt,
    Rd,
    Rs,
    Rt,
    RtRsSimm,
    RtRsUimm,
    RtUimm,
    RsSimm,
    RsUimm,
    /// `rs, rt, target`
    Branch2,
    /// `rs, target`
    Branch1,
    /// `target`
    Branch0,
    Jump,
    /// `rs` when rd is 31, `rd, rs` otherwise.
    Jalr,
    /// SYSCALL's and BREAK's 20-bit code, written only when it is not zero.
    Code,
    /// `rs, rt`, and the 10-bit code when it is not zero.
    Trap,
    Mem(Kind, Reference),
    /// `op, offset(base)`, the operation in the rt field.
    Cache,
    /// `hint, offset(base)`
    Pref,
    /// `rt, register`, by COP0's names.
    Cop0,
    /// `rt, reg`, the performance register's number in bits 5-1.
    RtPerf,
    FdFsFt,
    FdFs,
    /// `fd, ft`: SQRT.S, whose operand is in the ft field (Sapphire_Disputes.md D-24).
    FdFt,
    FsFt,
    /// `rt, fs`
    RtFs,
    /// `rt, fcrN`
    RtFcr,
    /// `rt, vfN`, the vector register in the rd field.
    RtVf,
    /// `rt, $N`, VU0's control register in the rd field.
    RtVc,
}

/// A decoded operation: one of the CPU's, or a VU0 operation issued as COP2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Cpu(&'static str, Form),
    Vu0(&'static vu::Operation),
}

impl Op {
    pub fn mnemonic(&self) -> String {
        match self {
            Op::Cpu(n, _) => n.to_string(),
            Op::Vu0(o) => format!("v{}", o.name),
        }
    }
}

#[derive(Clone, Copy)]
enum Slot {
    No,
    Op(&'static str, Form),
    Sub(&'static Table),
    Vu0,
}

/// One of the manual's tables: the field it is indexed by and its cells.
struct Table {
    at: u32,
    width: u32,
    slots: &'static [Slot],
}

impl Table {
    fn index(&self, w: u32) -> usize {
        (w >> self.at & ((1 << self.width) - 1)) as usize
    }
}

use Form::*;
use Slot::No as N;

const fn o(name: &'static str, form: Form) -> Slot {
    Slot::Op(name, form)
}

const LOAD: Form = Mem(Kind::Gpr, Reference::Read);
const STORE: Form = Mem(Kind::Gpr, Reference::Write);

static PRIMARY: Table = Table {
    at: 26,
    width: 6,
    slots: &[
        Slot::Sub(&SPECIAL), Slot::Sub(&REGIMM), o("j", Jump), o("jal", Jump), o("beq", Branch2), o("bne", Branch2), o("blez", Branch1), o("bgtz", Branch1),
        o("addi", RtRsSimm), o("addiu", RtRsSimm), o("slti", RtRsSimm), o("sltiu", RtRsSimm), o("andi", RtRsUimm), o("ori", RtRsUimm), o("xori", RtRsUimm), o("lui", RtUimm),
        Slot::Sub(&COP0), Slot::Sub(&COP1), Slot::Sub(&COP2), N, o("beql", Branch2), o("bnel", Branch2), o("blezl", Branch1), o("bgtzl", Branch1),
        o("daddi", RtRsSimm), o("daddiu", RtRsSimm), o("ldl", LOAD), o("ldr", LOAD), Slot::Sub(&MMI), N, o("lq", LOAD), o("sq", STORE),
        o("lb", LOAD), o("lh", LOAD), o("lwl", LOAD), o("lw", LOAD), o("lbu", LOAD), o("lhu", LOAD), o("lwr", LOAD), o("lwu", LOAD),
        o("sb", STORE), o("sh", STORE), o("swl", STORE), o("sw", STORE), o("sdl", STORE), o("sdr", STORE), o("swr", STORE), o("cache", Cache),
        N, o("lwc1", Mem(Kind::Fpr, Reference::Read)), N, o("pref", Pref), N, N, o("lqc2", Mem(Kind::Vf, Reference::Read)), o("ld", LOAD),
        N, o("swc1", Mem(Kind::Fpr, Reference::Write)), N, N, N, N, o("sqc2", Mem(Kind::Vf, Reference::Write)), o("sd", STORE),
    ],
};

static SPECIAL: Table = Table {
    at: 0,
    width: 6,
    slots: &[
        o("sll", RdRtSa), N, o("srl", RdRtSa), o("sra", RdRtSa), o("sllv", RdRtRs), N, o("srlv", RdRtRs), o("srav", RdRtRs),
        o("jr", Rs), o("jalr", Jalr), o("movz", RdRsRt), o("movn", RdRsRt), o("syscall", Code), o("break", Code), N, Slot::Sub(&SYNC),
        o("mfhi", Rd), o("mthi", Rs), o("mflo", Rd), o("mtlo", Rs), o("dsllv", RdRtRs), N, o("dsrlv", RdRtRs), o("dsrav", RdRtRs),
        o("mult", RdRsRt), o("multu", RdRsRt), o("div", RsRt), o("divu", RsRt), N, N, N, N,
        o("add", RdRsRt), o("addu", RdRsRt), o("sub", RdRsRt), o("subu", RdRsRt), o("and", RdRsRt), o("or", RdRsRt), o("xor", RdRsRt), o("nor", RdRsRt),
        o("mfsa", Rd), o("mtsa", Rs), o("slt", RdRsRt), o("sltu", RdRsRt), o("dadd", RdRsRt), o("daddu", RdRsRt), o("dsub", RdRsRt), o("dsubu", RdRsRt),
        o("tge", Trap), o("tgeu", Trap), o("tlt", Trap), o("tltu", Trap), o("teq", Trap), N, o("tne", Trap), N,
        o("dsll", RdRtSa), N, o("dsrl", RdRtSa), o("dsra", RdRtSa), o("dsll32", RdRtSa), N, o("dsrl32", RdRtSa), o("dsra32", RdRtSa),
    ],
};

/// SYNC's stype, bits 10-6: 1xxxx is SYNC.P, 0xxxx SYNC (SYNC.L) (the TX79 manual, SYNC.stype).
static SYNC: Table = Table { at: 10, width: 1, slots: &[o("sync", None), o("sync.p", None)] };

static REGIMM: Table = Table {
    at: 16,
    width: 5,
    slots: &[
        o("bltz", Branch1), o("bgez", Branch1), o("bltzl", Branch1), o("bgezl", Branch1), N, N, N, N,
        o("tgei", RsSimm), o("tgeiu", RsSimm), o("tlti", RsSimm), o("tltiu", RsSimm), o("teqi", RsSimm), N, o("tnei", RsSimm), N,
        o("bltzal", Branch1), o("bgezal", Branch1), o("bltzall", Branch1), o("bgezall", Branch1), N, N, N, N,
        o("mtsab", RsUimm), o("mtsah", RsUimm), N, N, N, N, N, N,
    ],
};

static MMI: Table = Table {
    at: 0,
    width: 6,
    slots: &[
        o("madd", RdRsRt), o("maddu", RdRsRt), N, N, o("plzcw", RdRs), N, N, N,
        Slot::Sub(&MMI0), Slot::Sub(&MMI2), N, N, N, N, N, N,
        o("mfhi1", Rd), o("mthi1", Rs), o("mflo1", Rd), o("mtlo1", Rs), N, N, N, N,
        o("mult1", RdRsRt), o("multu1", RdRsRt), o("div1", RsRt), o("divu1", RsRt), N, N, N, N,
        o("madd1", RdRsRt), o("maddu1", RdRsRt), N, N, N, N, N, N,
        Slot::Sub(&MMI1), Slot::Sub(&MMI3), N, N, N, N, N, N,
        Slot::Sub(&PMFHL), Slot::Sub(&PMTHL), N, N, o("psllh", RdRtSa), N, o("psrlh", RdRtSa), o("psrah", RdRtSa),
        N, N, N, N, o("psllw", RdRtSa), N, o("psrlw", RdRtSa), o("psraw", RdRtSa),
    ],
};

static MMI0: Table = Table {
    at: 6,
    width: 5,
    slots: &[
        o("paddw", RdRsRt), o("psubw", RdRsRt), o("pcgtw", RdRsRt), o("pmaxw", RdRsRt),
        o("paddh", RdRsRt), o("psubh", RdRsRt), o("pcgth", RdRsRt), o("pmaxh", RdRsRt),
        o("paddb", RdRsRt), o("psubb", RdRsRt), o("pcgtb", RdRsRt), N,
        N, N, N, N,
        o("paddsw", RdRsRt), o("psubsw", RdRsRt), o("pextlw", RdRsRt), o("ppacw", RdRsRt),
        o("paddsh", RdRsRt), o("psubsh", RdRsRt), o("pextlh", RdRsRt), o("ppach", RdRsRt),
        o("paddsb", RdRsRt), o("psubsb", RdRsRt), o("pextlb", RdRsRt), o("ppacb", RdRsRt),
        N, N, o("pext5", RdRt), o("ppac5", RdRt),
    ],
};

static MMI1: Table = Table {
    at: 6,
    width: 5,
    slots: &[
        N, o("pabsw", RdRt), o("pceqw", RdRsRt), o("pminw", RdRsRt),
        o("padsbh", RdRsRt), o("pabsh", RdRt), o("pceqh", RdRsRt), o("pminh", RdRsRt),
        N, N, o("pceqb", RdRsRt), N,
        N, N, N, N,
        o("padduw", RdRsRt), o("psubuw", RdRsRt), o("pextuw", RdRsRt), N,
        o("padduh", RdRsRt), o("psubuh", RdRsRt), o("pextuh", RdRsRt), N,
        o("paddub", RdRsRt), o("psubub", RdRsRt), o("pextub", RdRsRt), o("qfsrv", RdRsRt),
        N, N, N, N,
    ],
};

static MMI2: Table = Table {
    at: 6,
    width: 5,
    slots: &[
        o("pmaddw", RdRsRt), N, o("psllvw", RdRtRs), o("psrlvw", RdRtRs),
        o("pmsubw", RdRsRt), N, N, N,
        o("pmfhi", Rd), o("pmflo", Rd), o("pinth", RdRsRt), N,
        o("pmultw", RdRsRt), o("pdivw", RsRt), o("pcpyld", RdRsRt), N,
        o("pmaddh", RdRsRt), o("phmadh", RdRsRt), o("pand", RdRsRt), o("pxor", RdRsRt),
        o("pmsubh", RdRsRt), o("phmsbh", RdRsRt), N, N,
        N, N, o("pexeh", RdRt), o("prevh", RdRt),
        o("pmulth", RdRsRt), o("pdivbw", RsRt), o("pexew", RdRt), o("prot3w", RdRt),
    ],
};

static MMI3: Table = Table {
    at: 6,
    width: 5,
    slots: &[
        o("pmadduw", RdRsRt), N, N, o("psravw", RdRtRs),
        N, N, N, N,
        o("pmthi", Rs), o("pmtlo", Rs), o("pinteh", RdRsRt), N,
        o("pmultuw", RdRsRt), o("pdivuw", RsRt), o("pcpyud", RdRsRt), N,
        N, N, o("por", RdRsRt), o("pnor", RdRsRt),
        N, N, N, N,
        N, N, o("pexch", RdRt), o("pcpyh", RdRt),
        N, N, o("pexcw", RdRt), N,
    ],
};

/// PMFHL's fmt in the sa field; the manual names 0-4 (PMFHL.fmt).
static PMFHL: Table = Table {
    at: 6,
    width: 5,
    slots: &[
        o("pmfhl.lw", Rd), o("pmfhl.uw", Rd), o("pmfhl.slw", Rd), o("pmfhl.lh", Rd), o("pmfhl.sh", Rd), N, N, N,
        N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N,
    ],
};

/// PMTHL's fmt; the manual names 0 alone (PMTHL.fmt).
static PMTHL: Table = Table {
    at: 6,
    width: 5,
    slots: &[o("pmthl.lw", Rs), N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N],
};

static COP0: Table = Table {
    at: 21,
    width: 5,
    slots: &[
        Slot::Sub(&MF0), N, N, N, Slot::Sub(&MT0), N, N, N,
        Slot::Sub(&BC0), N, N, N, N, N, N, N,
        Slot::Sub(&C0), N, N, N, N, N, N, N,
        N, N, N, N, N, N, N, N,
    ],
};

/// MF0 and MT0 by the rd field: Debug (24) and Perf (25) hold the C790's own moves (the manual's C.2).
const fn by_register(mv: &'static str, debug: &'static Table, perf: &'static Table) -> [Slot; 32] {
    let mut t = [o(mv, Cop0); 32];
    t[24] = Slot::Sub(debug);
    t[25] = Slot::Sub(perf);
    t
}

static MF0: Table = Table { at: 11, width: 5, slots: &by_register("mfc0", &MF0_DEBUG, &MF0_PERF) };
static MT0: Table = Table { at: 11, width: 5, slots: &by_register("mtc0", &MT0_DEBUG, &MT0_PERF) };

static MF0_DEBUG: Table = Table {
    at: 0,
    width: 3,
    slots: &[o("mfbpc", Rt), N, o("mfiab", Rt), o("mfiabm", Rt), o("mfdab", Rt), o("mfdabm", Rt), o("mfdvb", Rt), o("mfdvbm", Rt)],
};
static MT0_DEBUG: Table = Table {
    at: 0,
    width: 3,
    slots: &[o("mtbpc", Rt), N, o("mtiab", Rt), o("mtiabm", Rt), o("mtdab", Rt), o("mtdabm", Rt), o("mtdvb", Rt), o("mtdvbm", Rt)],
};
static MF0_PERF: Table = Table { at: 0, width: 1, slots: &[o("mfps", RtPerf), o("mfpc", RtPerf)] };
static MT0_PERF: Table = Table { at: 0, width: 1, slots: &[o("mtps", RtPerf), o("mtpc", RtPerf)] };

static BC0: Table = Table {
    at: 16,
    width: 5,
    slots: &[
        o("bc0f", Branch0), o("bc0t", Branch0), o("bc0fl", Branch0), o("bc0tl", Branch0), N, N, N, N,
        N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N,
    ],
};

static C0: Table = Table {
    at: 0,
    width: 6,
    slots: &[
        N, o("tlbr", None), o("tlbwi", None), N, N, N, o("tlbwr", None), N,
        o("tlbp", None), N, N, N, N, N, N, N,
        N, N, N, N, N, N, N, N,
        o("eret", None), N, N, N, N, N, N, N,
        N, N, N, N, N, N, N, N,
        N, N, N, N, N, N, N, N,
        N, N, N, N, N, N, N, N,
        o("ei", None), o("di", None), N, N, N, N, N, N,
    ],
};

/// ps2tek's COP1: no 64-bit moves, no D or L format (D-1).
static COP1: Table = Table {
    at: 21,
    width: 5,
    slots: &[
        o("mfc1", RtFs), N, o("cfc1", RtFcr), N, o("mtc1", RtFs), N, o("ctc1", RtFcr), N,
        Slot::Sub(&BC1), N, N, N, N, N, N, N,
        Slot::Sub(&FPU_S), N, N, N, Slot::Sub(&FPU_W), N, N, N,
        N, N, N, N, N, N, N, N,
    ],
};

/// The likely forms are ps2tek's; the TX79 has BC1F and BC1T alone (D-2).
static BC1: Table = Table {
    at: 16,
    width: 5,
    slots: &[
        o("bc1f", Branch0), o("bc1t", Branch0), o("bc1fl", Branch0), o("bc1tl", Branch0), N, N, N, N,
        N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N,
    ],
};

/// ps2tek's "FPU.S" (D-1). The accumulator forms write fs and ft, SQRT.S fd and ft (D-24), the others the TX79's
/// computational layout.
static FPU_S: Table = Table {
    at: 0,
    width: 6,
    slots: &[
        o("add.s", FdFsFt), o("sub.s", FdFsFt), o("mul.s", FdFsFt), o("div.s", FdFsFt), o("sqrt.s", FdFt), o("abs.s", FdFs), o("mov.s", FdFs), o("neg.s", FdFs),
        N, N, N, N, N, N, N, N,
        N, N, N, N, N, N, o("rsqrt.s", FdFsFt), N,
        o("adda.s", FsFt), o("suba.s", FsFt), o("mula.s", FsFt), N, o("madd.s", FdFsFt), o("msub.s", FdFsFt), o("madda.s", FsFt), o("msuba.s", FsFt),
        N, N, N, N, o("cvt.w.s", FdFs), N, N, N,
        o("max.s", FdFsFt), o("min.s", FdFsFt), N, N, N, N, N, N,
        o("c.f.s", FsFt), N, o("c.eq.s", FsFt), N, o("c.lt.s", FsFt), N, o("c.le.s", FsFt), N,
        N, N, N, N, N, N, N, N,
    ],
};

static FPU_W: Table = Table {
    at: 0,
    width: 6,
    slots: &{
        let mut t = [N; 64];
        t[0x20] = o("cvt.s.w", FdFs);
        t
    },
};

/// ps2tek's COP2: VU0's macro mode, every rs from 10h a VU operation with its field mask in bits 24-21 (D-3).
static COP2: Table = Table {
    at: 21,
    width: 5,
    slots: &[
        N, o("qmfc2", RtVf), o("cfc2", RtVc), N, N, o("qmtc2", RtVf), o("ctc2", RtVc), N,
        Slot::Sub(&BC2), N, N, N, N, N, N, N,
        Slot::Vu0, Slot::Vu0, Slot::Vu0, Slot::Vu0, Slot::Vu0, Slot::Vu0, Slot::Vu0, Slot::Vu0,
        Slot::Vu0, Slot::Vu0, Slot::Vu0, Slot::Vu0, Slot::Vu0, Slot::Vu0, Slot::Vu0, Slot::Vu0,
    ],
};

static BC2: Table = Table {
    at: 16,
    width: 5,
    slots: &[
        o("bc2f", Branch0), o("bc2t", Branch0), o("bc2fl", Branch0), o("bc2tl", Branch0), N, N, N, N,
        N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N,
    ],
};

/// COP0's registers by the TX79 manual's Table 4-1, which ps2tek's "EE COP0 Registers" repeats; a reserved one `$n`.
pub const COP0_NAMES: [&str; 32] = [
    "Index", "Random", "EntryLo0", "EntryLo1", "Context", "PageMask", "Wired", "", "BadVAddr", "Count", "EntryHi", "Compare", "Status", "Cause", "EPC",
    "PRId", "Config", "", "", "", "", "", "", "BadPAddr", "Debug", "Perf", "", "", "TagLo", "TagHi", "ErrorEPC", "",
];

/// The operation a word encodes, or none where the EE reserves the encoding.
pub fn operation(w: u32) -> Option<Op> {
    let mut slot = PRIMARY.slots[PRIMARY.index(w)];
    loop {
        match slot {
            Slot::No => return Option::None,
            Slot::Op(n, f) => return Some(Op::Cpu(n, f)),
            Slot::Vu0 => return vu::operation(w).map(Op::Vu0),
            Slot::Sub(t) => slot = t.slots[t.index(w)],
        }
    }
}

fn rs(w: u32) -> usize {
    (w >> 21 & 31) as usize
}
fn rt(w: u32) -> usize {
    (w >> 16 & 31) as usize
}
fn rd(w: u32) -> usize {
    (w >> 11 & 31) as usize
}
fn sa(w: u32) -> u32 {
    w >> 6 & 31
}

fn signed(v: i32) -> String {
    if v < 0 { format!("-0x{:x}", v.unsigned_abs()) } else { format!("0x{v:x}") }
}

fn cop0_name(n: usize) -> String {
    if COP0_NAMES[n].is_empty() { format!("${n}") } else { COP0_NAMES[n].to_string() }
}

/// What the operands of `form` say for `w` fetched at `pc`, and the address the word alone names.
fn operands(form: Form, w: u32, pc: u32) -> (String, Option<(Reference, u32)>) {
    let (s, t, d) = (GPR_NAMES[rs(w)], GPR_NAMES[rt(w)], GPR_NAMES[rd(w)]);
    let simm = w as u16 as i16 as i32;
    let target = pc.wrapping_add(4).wrapping_add((simm << 2) as u32);
    let mem = |reg: String, kind: Option<Reference>| {
        (format!("{reg}, {}({s})", signed(simm)), kind.filter(|_| rs(w) == 0).map(|k| (k, simm as u32)))
    };
    let plain = |text: String| (text, Option::None);
    match form {
        None => plain(String::new()),
        RdRsRt => plain(format!("{d}, {s}, {t}")),
        RdRtRs => plain(format!("{d}, {t}, {s}")),
        RdRtSa => plain(format!("{d}, {t}, {}", sa(w))),
        RdRs => plain(format!("{d}, {s}")),
        RdRt => plain(format!("{d}, {t}")),
        RsRt => plain(format!("{s}, {t}")),
        Rd => plain(d.to_string()),
        Rs => plain(s.to_string()),
        Rt => plain(t.to_string()),
        RtRsSimm => plain(format!("{t}, {s}, {}", signed(simm))),
        RtRsUimm => plain(format!("{t}, {s}, 0x{:x}", w & 0xFFFF)),
        RtUimm => plain(format!("{t}, 0x{:x}", w & 0xFFFF)),
        RsSimm => plain(format!("{s}, {}", signed(simm))),
        RsUimm => plain(format!("{s}, 0x{:x}", w & 0xFFFF)),
        Branch2 => (format!("{s}, {t}, 0x{target:08x}"), Some((Reference::Call, target))),
        Branch1 => (format!("{s}, 0x{target:08x}"), Some((Reference::Call, target))),
        Branch0 => (format!("0x{target:08x}"), Some((Reference::Call, target))),
        Jump => {
            let j = pc.wrapping_add(4) & 0xF000_0000 | (w & 0x03FF_FFFF) << 2;
            (format!("0x{j:08x}"), Some((Reference::Call, j)))
        }
        Jalr => plain(if rd(w) == 31 { s.to_string() } else { format!("{d}, {s}") }),
        Code => plain(if w >> 6 & 0xF_FFFF == 0 { String::new() } else { format!("0x{:x}", w >> 6 & 0xF_FFFF) }),
        Trap => plain(if w >> 6 & 0x3FF == 0 { format!("{s}, {t}") } else { format!("{s}, {t}, 0x{:x}", w >> 6 & 0x3FF) }),
        Mem(Kind::Gpr, r) => mem(t.to_string(), Some(r)),
        Mem(Kind::Fpr, r) => mem(format!("f{}", rt(w)), Some(r)),
        Mem(Kind::Vf, r) => mem(format!("vf{}", rt(w)), Some(r)),
        Cache | Pref => mem(format!("0x{:x}", rt(w)), Option::None),
        Cop0 => plain(format!("{t}, {}", cop0_name(rd(w)))),
        RtPerf => plain(format!("{t}, {}", w >> 1 & 31)),
        FdFsFt => plain(format!("f{}, f{}, f{}", sa(w), rd(w), rt(w))),
        FdFs => plain(format!("f{}, f{}", sa(w), rd(w))),
        FdFt => plain(format!("f{}, f{}", sa(w), rt(w))),
        FsFt => plain(format!("f{}, f{}", rd(w), rt(w))),
        RtFs => plain(format!("{t}, f{}", rd(w))),
        RtFcr => plain(format!("{t}, fcr{}", rd(w))),
        RtVf => plain(format!("{t}, vf{}", rd(w))),
        RtVc => plain(format!("{t}, ${}", rd(w))),
    }
}

/// One R5900 word fetched from `pc`: `.word` where the EE reserves the encoding, `nop` for the zero word.
pub fn decode(w: u32, pc: u32) -> Instruction {
    let text = |mnemonic: String, operands: String, reference| Instruction { mnemonic, operands, reference };
    match operation(w) {
        _ if w == 0 => text("nop".into(), String::new(), Option::None),
        Option::None => text(".word".into(), format!("0x{w:08x}"), Option::None),
        Some(Op::Cpu(name, form)) => {
            let (ops, reference) = operands(form, w, pc);
            text(name.into(), ops, reference)
        }
        Some(Op::Vu0(o)) => {
            let full = vu::text(o, w, true);
            let (m, ops) = full.split_once(' ').unwrap_or((&full, ""));
            text(m.into(), ops.into(), Option::None)
        }
    }
}

/// Every CPU mnemonic with its word with every operand zero, the first cell that names it.
fn canonicals() -> &'static HashMap<&'static str, (u32, Form)> {
    static MAP: OnceLock<HashMap<&'static str, (u32, Form)>> = OnceLock::new();
    MAP.get_or_init(|| {
        fn walk(t: &'static Table, bits: u32, map: &mut HashMap<&'static str, (u32, Form)>) {
            for (i, s) in t.slots.iter().enumerate() {
                let w = bits | (i as u32) << t.at;
                match *s {
                    Slot::Op(n, f) => {
                        map.entry(n).or_insert((w, f));
                    }
                    Slot::Sub(sub) => walk(sub, w, map),
                    Slot::No | Slot::Vu0 => {}
                }
            }
        }
        let mut map = HashMap::new();
        walk(&PRIMARY, 0, &mut map);
        map
    })
}

fn register(s: &str, prefix: &str) -> Result<u32, AsmError> {
    match s.strip_prefix(prefix).map(str::parse::<u32>) {
        Some(Ok(n)) if n < 32 => Ok(n),
        _ => err(format!("not a {prefix} register: {s}")),
    }
}

fn cop0_register(s: &str) -> Result<u32, AsmError> {
    if let Some(i) = COP0_NAMES.iter().position(|n| !n.is_empty() && *n == s) {
        return Ok(i as u32);
    }
    register(s, "$")
}

/// One instruction as `decode` writes it, at `pc`, a branch's or jump's target given as an address or a label.
pub(crate) fn encode(text: &str, pc: u32, labels: &dyn Fn(&str) -> Option<u32>) -> Result<u32, AsmError> {
    let text = text.trim();
    let (mnemonic, rest) = text.split_once(char::is_whitespace).unwrap_or((text, ""));
    let mnemonic = mnemonic.to_ascii_lowercase();
    let ops = split_operands(rest);
    match mnemonic.as_str() {
        "nop" if ops.is_empty() => return Ok(0),
        ".word" => {
            return match ops.as_slice() {
                [v] => ranged(v, i32::MIN as i64, u32::MAX as i64),
                _ => err(".word takes one value"),
            };
        }
        _ => {}
    }
    let Some(&(canonical, form)) = canonicals().get(mnemonic.as_str()) else {
        if mnemonic.starts_with('v') {
            let (bits, _) = vu::encode(text, true)?;
            return Ok(0x12 << 26 | 1 << 25 | bits);
        }
        return err(format!("unknown mnemonic: {mnemonic}"));
    };
    let target = |s: &str| -> Result<u32, AsmError> {
        match labels(s).or_else(|| number(s).filter(|v| (0..=u32::MAX as i64).contains(v)).map(|v| v as u32)) {
            Some(t) => Ok(t),
            Option::None => err(format!("not an address or a known label: {s}")),
        }
    };
    let next = pc.wrapping_add(4);
    let branch = |s: &str| -> Result<u32, AsmError> {
        let delta = target(s)?.wrapping_sub(next) as i32;
        if delta & 3 != 0 || !(-0x20000..=0x1FFFC).contains(&delta) {
            return err(format!("branch out of reach: {s}"));
        }
        Ok((delta >> 2) as u32 & 0xFFFF)
    };
    let mem = |s: &str| -> Result<u32, AsmError> {
        let Some((offset, base)) = s.strip_suffix(')').and_then(|t| t.split_once('(')) else {
            return err(format!("not offset(base): {s}"));
        };
        let offset = if offset.trim().is_empty() { 0 } else { ranged(offset.trim(), -0x8000, 0x7FFF)? & 0xFFFF };
        Ok(gpr(base.trim())? << 21 | offset)
    };
    let simm = |s: &str| ranged(s, -0x8000, 0x7FFF).map(|v| v & 0xFFFF);
    let uimm = |s: &str| ranged(s, 0, 0xFFFF);
    let count = |n: usize| -> Result<(), AsmError> {
        if ops.len() == n { Ok(()) } else { err(format!("{mnemonic} takes {n} operands")) }
    };
    let (d, s, t) = (11, 21, 16);
    let fields = match form {
        None => count(0).map(|_| 0)?,
        RdRsRt if ops.len() == 2 && matches!(mnemonic.as_str(), "mult" | "multu" | "madd" | "maddu" | "mult1" | "multu1" | "madd1" | "maddu1") => gpr(ops[0])? << s | gpr(ops[1])? << t,
        RdRsRt => count(3).and_then(|_| Ok(gpr(ops[0])? << d | gpr(ops[1])? << s | gpr(ops[2])? << t))?,
        RdRtRs => count(3).and_then(|_| Ok(gpr(ops[0])? << d | gpr(ops[1])? << t | gpr(ops[2])? << s))?,
        RdRtSa => count(3).and_then(|_| Ok(gpr(ops[0])? << d | gpr(ops[1])? << t | ranged(ops[2], 0, 31)? << 6))?,
        RdRs => count(2).and_then(|_| Ok(gpr(ops[0])? << d | gpr(ops[1])? << s))?,
        RdRt => count(2).and_then(|_| Ok(gpr(ops[0])? << d | gpr(ops[1])? << t))?,
        RsRt => count(2).and_then(|_| Ok(gpr(ops[0])? << s | gpr(ops[1])? << t))?,
        Rd => count(1).and_then(|_| Ok(gpr(ops[0])? << d))?,
        Rs => count(1).and_then(|_| Ok(gpr(ops[0])? << s))?,
        Rt => count(1).and_then(|_| Ok(gpr(ops[0])? << t))?,
        RtRsSimm => count(3).and_then(|_| Ok(gpr(ops[0])? << t | gpr(ops[1])? << s | simm(ops[2])?))?,
        RtRsUimm => count(3).and_then(|_| Ok(gpr(ops[0])? << t | gpr(ops[1])? << s | uimm(ops[2])?))?,
        RtUimm => count(2).and_then(|_| Ok(gpr(ops[0])? << t | uimm(ops[1])?))?,
        RsSimm => count(2).and_then(|_| Ok(gpr(ops[0])? << s | simm(ops[1])?))?,
        RsUimm => count(2).and_then(|_| Ok(gpr(ops[0])? << s | uimm(ops[1])?))?,
        Branch2 => count(3).and_then(|_| Ok(gpr(ops[0])? << s | gpr(ops[1])? << t | branch(ops[2])?))?,
        Branch1 => count(2).and_then(|_| Ok(gpr(ops[0])? << s | branch(ops[1])?))?,
        Branch0 => count(1).and_then(|_| branch(ops[0]))?,
        Jump => {
            count(1)?;
            let j = target(ops[0])?;
            if j & 3 != 0 || j & 0xF000_0000 != next & 0xF000_0000 {
                return err(format!("jump out of reach: {}", ops[0]));
            }
            j >> 2 & 0x03FF_FFFF
        }
        Jalr => match ops.as_slice() {
            [r] => 31 << d | gpr(r)? << s,
            [a, b] => gpr(a)? << d | gpr(b)? << s,
            _ => return err("jalr takes one or two operands"),
        },
        Code => match ops.as_slice() {
            [] => 0,
            [c] => ranged(c, 0, 0xF_FFFF)? << 6,
            _ => return err(format!("{mnemonic} takes at most one operand")),
        },
        Trap => match ops.as_slice() {
            [a, b] => gpr(a)? << s | gpr(b)? << t,
            [a, b, c] => gpr(a)? << s | gpr(b)? << t | ranged(c, 0, 0x3FF)? << 6,
            _ => return err(format!("{mnemonic} takes two or three operands")),
        },
        Mem(kind, _) => {
            count(2)?;
            let r = match kind {
                Kind::Gpr => gpr(ops[0])?,
                Kind::Fpr => register(ops[0], "f")?,
                Kind::Vf => register(ops[0], "vf")?,
            };
            r << t | mem(ops[1])?
        }
        Cache | Pref => count(2).and_then(|_| Ok(ranged(ops[0], 0, 31)? << t | mem(ops[1])?))?,
        Cop0 => count(2).and_then(|_| Ok(gpr(ops[0])? << t | cop0_register(ops[1])? << d))?,
        RtPerf => count(2).and_then(|_| Ok(gpr(ops[0])? << t | ranged(ops[1], 0, 31)? << 1))?,
        FdFsFt => count(3).and_then(|_| Ok(register(ops[0], "f")? << 6 | register(ops[1], "f")? << d | register(ops[2], "f")? << t))?,
        FdFs => count(2).and_then(|_| Ok(register(ops[0], "f")? << 6 | register(ops[1], "f")? << d))?,
        FdFt => count(2).and_then(|_| Ok(register(ops[0], "f")? << 6 | register(ops[1], "f")? << t))?,
        FsFt => count(2).and_then(|_| Ok(register(ops[0], "f")? << d | register(ops[1], "f")? << t))?,
        RtFs => count(2).and_then(|_| Ok(gpr(ops[0])? << t | register(ops[1], "f")? << d))?,
        RtFcr => count(2).and_then(|_| Ok(gpr(ops[0])? << t | register(ops[1], "fcr")? << d))?,
        RtVf => count(2).and_then(|_| Ok(gpr(ops[0])? << t | register(ops[1], "vf")? << d))?,
        RtVc => count(2).and_then(|_| Ok(gpr(ops[0])? << t | register(ops[1], "$")? << d))?,
    };
    let word = canonical | fields;
    match operation(word) {
        Some(Op::Cpu(n, _)) if n == mnemonic => Ok(word),
        other => err(format!("{text} encodes {}", other.map_or(".word".into(), |o| o.mnemonic()))),
    }
}

#[cfg(test)]
pub(crate) mod tests;
