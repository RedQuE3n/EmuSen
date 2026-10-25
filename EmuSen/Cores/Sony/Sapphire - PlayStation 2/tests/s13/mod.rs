//! The project's S13 programs (Sapphire_Native.md §23): bare EE programs in S6's form that write microprograms into
//! VU0's and VU1's micro memory, their constants into the data memory, start them (VCALLMS for VU0, CMSAR1 for VU1),
//! wait for their end, and copy what they stored back into the record. Each microprogram stores its results with SQ
//! and ISW, so the record shows what micro mode computed and when a value became visible inside the program; the
//! expected record comes from `manual` and `s12::manual`, the VU User's Manual's pages and pipeline figures transcribed
//! apart from `src/vu`.

#![allow(dead_code)]

pub mod manual;

use crate::s12::manual as vm;
use crate::s6::Program;

pub type Lanes = [u32; 4];

pub const ONE: u32 = 0x3F80_0000;
pub const TWO: u32 = 0x4000_0000;
pub const THREE: u32 = 0x4040_0000;
pub const FOUR: u32 = 0x4080_0000;
pub const NEG: u32 = 0xBF80_0000;
pub const SENT: Lanes = [0x5A5A_C3C3, 0x5A5A_C3C4, 0x5A5A_C3C5, 0x5A5A_C3C6];
const VF00: Lanes = [0, 0, 0, ONE];

pub fn quad(l: Lanes) -> u128 {
    l.iter().rev().fold(0u128, |a, &f| a << 32 | f as u128)
}

/// One microprogram, its constants and the results it stores, with what the manual says each holds.
pub struct Seg {
    pub unit: usize,
    lines: Vec<String>,
    consts: Vec<Lanes>,
    pub results: Vec<Option<Lanes>>,
    pub notes: Vec<(usize, String)>,
    /// TPC as CFC2 reads it after the microprogram (VU0 only), when the case asks.
    pub tpc: Option<u16>,
}

impl Seg {
    pub fn new(unit: usize) -> Seg {
        let mut s = Seg { unit, lines: Vec::new(), consts: Vec::new(), results: Vec::new(), notes: Vec::new(), tpc: None };
        s.konst(SENT);
        s.load(31, SENT);
        s
    }
    /// The first result quadword in the data memory, past the constants.
    pub fn res(&self) -> usize {
        if self.unit == 0 { 128 } else { 256 }
    }
    pub fn here(&self) -> usize {
        self.lines.len()
    }
    pub fn pair(&mut self, s: impl Into<String>) {
        self.lines.push(s.into());
    }
    pub fn low(&mut self, s: impl AsRef<str>) {
        self.pair(format!("nop | {}", s.as_ref()));
    }
    pub fn nops(&mut self, n: usize) {
        for _ in 0..n {
            self.low("move vf0, vf0");
        }
    }
    pub fn konst(&mut self, v: Lanes) -> usize {
        if let Some(k) = self.consts.iter().position(|c| *c == v) {
            return k;
        }
        self.consts.push(v);
        assert!(self.consts.len() < 128, "constants run into the results");
        self.consts.len() - 1
    }
    /// VF[n] ← `v` by LQ from a constant.
    pub fn load(&mut self, n: u32, v: Lanes) {
        let k = self.konst(v);
        self.low(format!("lq.xyzw vf{n}, 0x{k:x}(vi0)"));
    }
    /// VI[n] ← `v`: IADDIU below 8000h, ILW from a constant otherwise.
    pub fn vi(&mut self, n: u32, v: u16) {
        if v <= 0x7FFF {
            self.low(format!("iaddiu vi{n}, vi0, 0x{v:x}"));
        } else {
            let k = self.konst([v as u32, 0, 0, 0]);
            self.low(format!("ilw.x vi{n}, 0x{k:x}(vi0)"));
        }
    }
    fn slot(&mut self, want: Option<Lanes>, note: &str) -> usize {
        let r = self.res() + self.results.len();
        self.notes.push((self.results.len(), note.to_string()));
        self.results.push(want);
        r
    }
    /// The next result's quadword index, with the sentinel stored there first.
    pub fn prefilled(&mut self, want: Option<Lanes>, note: &str) -> usize {
        let r = self.slot(want, note);
        self.low(format!("sq.xyzw vf31, 0x{r:x}(vi0)"));
        r
    }
    pub fn store(&mut self, n: u32, want: Option<Lanes>, note: &str) {
        let r = self.slot(want, note);
        self.low(format!("sq.xyzw vf{n}, 0x{r:x}(vi0)"));
    }
    /// VI[n] stored zero-extended in every field (ISW).
    pub fn store_vi(&mut self, n: u32, want: Option<u16>, note: &str) {
        let r = self.slot(want.map(|v| [v as u32; 4]), note);
        self.low(format!("isw.xyzw vi{n}, 0x{r:x}(vi0)"));
    }
    pub fn end(&mut self) {
        self.pair("[e] nop | move vf0, vf0");
        self.low("move vf0, vf0");
    }
    pub fn source(&self) -> String {
        self.lines.join("\n")
    }
    pub fn assembled(&self) -> Vec<u64> {
        emusen_mips::vu::assemble(&self.source()).unwrap_or_else(|e| panic!("{e}\n{}", self.source()))
    }
}

/// A program running each segment in turn: COP2 enabled; for each, the microprogram and the constants written through
/// kseg1, the unit started and waited for (VNOP after VCALLMS; BC2T while VU1 runs), its results copied to the record.
pub fn program(name: &'static str, segs: &[Seg]) -> Program {
    let mut p = Program::new(name);
    p.emit("mfc0 t0, Status");
    p.emit("lui t1, 0x4000");
    p.emit("or t0, t0, t1");
    p.emit("mtc0 t0, Status");
    p.emit("sync.p");
    for (n, seg) in segs.iter().enumerate() {
        let (micro, data) = if seg.unit == 0 { (0xB100_0000u32, 0xB100_4000u32) } else { (0xB100_8000, 0xB100_C000) };
        p.emit(format!("lui t1, 0x{:x}", micro >> 16));
        p.emit(format!("ori t1, t1, 0x{:x}", micro & 0xFFFF));
        p.emit(format!("lui t2, 0x{:x}", data >> 16));
        p.emit(format!("ori t2, t2, 0x{:x}", data & 0xFFFF));
        for (i, pair) in seg.assembled().iter().enumerate() {
            p.constant("t0", *pair);
            p.emit(format!("sd t0, 0x{:x}(t1)", 8 * i));
        }
        for (k, c) in seg.consts.iter().enumerate() {
            p.quad("t0", quad(*c));
            p.emit(format!("sq t0, 0x{:x}(t2)", 16 * k));
        }
        if seg.unit == 0 {
            p.emit("vcallms 0x0");
            p.emit("vnop");
        } else {
            p.emit("ctc2 zero, vi31");
            let wait = format!("w{n}");
            p.place(&wait);
            p.emit(format!("bc2t {wait}"));
            p.emit("nop");
        }
        for (r, want) in seg.results.iter().enumerate() {
            p.emit(format!("lq t0, 0x{:x}(t2)", 16 * (seg.res() + r)));
            p.record_quad("t0", want.map(quad));
        }
        if let Some(tpc) = seg.tpc {
            p.emit("cfc2 t0, vi26");
            p.record("t0", Some(tpc as u64));
            p.record("zero", Some(0));
        }
    }
    p
}

fn lanes_of(dest: &str) -> u32 {
    dest.chars().fold(0, |m, c| m | 8 >> "xyzw".find(c).unwrap())
}

/// `base` with the fields `dest` names taken from `v`.
fn merge(base: Lanes, v: Lanes, dest: &str) -> Lanes {
    let m = lanes_of(dest);
    std::array::from_fn(|l| if m & 8 >> l != 0 { v[l] } else { base[l] })
}

/// The loads and stores, the integer unit, the moves and the random unit in micro mode (§4.3's pages).
pub fn integer() -> Seg {
    let mut s = Seg::new(0);
    let c1 = [ONE, TWO, THREE, FOUR];
    let c2 = [0x1234_5678, 0x9ABC_DEF0, 0x0000_FFFF, 0x8000_8000];
    let k1 = s.konst(c1) as u16;
    let k2 = s.konst(c2) as u16;
    s.load(1, c1);
    s.store(1, Some(c1), "lq, sq");
    s.load(2, SENT);
    s.vi(2, k1 + 2);
    s.low("lq.xz vf2, -0x2(vi2)");
    s.store(2, Some(merge(SENT, c1, "xz")), "lq.xz with a negative offset");
    let r = s.prefilled(Some(merge(SENT, c1, "yw")), "sq.yw with a negative offset");
    s.vi(3, r as u16 + 1);
    s.low("sq.yw vf1, -0x1(vi3)");
    s.low(format!("ilw.y vi4, 0x{k2:x}(vi0)"));
    s.store_vi(4, Some(0xDEF0), "ilw.y");
    s.low(format!("ilw.w vi5, 0x{k2:x}(vi0)"));
    s.store_vi(5, Some(0x8000), "ilw.w");
    s.vi(6, k2 + 3);
    s.low("ilw.x vi7, -0x3(vi6)");
    s.store_vi(7, Some(0x5678), "ilw.x with a negative offset");
    s.vi(8, 0xBEEF);
    let r = s.prefilled(Some(merge(SENT, [0xBEEF; 4], "xz")), "isw.xz");
    s.low(format!("isw.xz vi8, 0x{r:x}(vi0)"));
    for (a, imm) in [(0u16, 0u16), (1, 1), (0xFFFF, 1), (0x7FFF, 0x7FFF), (0x1234, 0x4321), (0x8000, 0x7FFF)] {
        s.vi(9, a);
        s.low(format!("iaddiu vi10, vi9, 0x{imm:x}"));
        s.store_vi(10, Some(a.wrapping_add(imm)), &format!("iaddiu {a:x}, {imm:x}"));
        s.low(format!("isubiu vi11, vi9, 0x{imm:x}"));
        s.store_vi(11, Some(a.wrapping_sub(imm)), &format!("isubiu {a:x}, {imm:x}"));
    }
    for (a, b) in [(5u16, 3u16), (0xFFFF, 1), (0xF0F0, 0x0FF0)] {
        s.vi(1, a);
        s.vi(2, b);
        s.low("iadd vi3, vi1, vi2");
        s.store_vi(3, Some(a.wrapping_add(b)), "iadd");
        s.low("isub vi3, vi1, vi2");
        s.store_vi(3, Some(a.wrapping_sub(b)), "isub");
        s.low("iand vi3, vi1, vi2");
        s.store_vi(3, Some(a & b), "iand");
        s.low("ior vi3, vi1, vi2");
        s.store_vi(3, Some(a | b), "ior");
        s.low("iaddi vi3, vi1, -7");
        s.store_vi(3, Some(a.wrapping_sub(7)), "iaddi");
    }
    s.load(12, c2);
    s.low("mtir vi12, vf12y");
    s.store_vi(12, Some(0xDEF0), "mtir");
    s.vi(13, 0x8001);
    s.load(13, SENT);
    s.low("mfir.xw vf13, vi13");
    s.store(13, Some(merge(SENT, [0xFFFF_8001; 4], "xw")), "mfir sign-extends");
    s.vi(1, k1);
    s.low("lqi.xyzw vf14, (vi1++)");
    s.store(14, Some(c1), "lqi");
    s.store_vi(1, Some(k1 + 1), "lqi's increment");
    s.load(15, SENT);
    s.low("lqd.yz vf15, (--vi1)");
    s.store(15, Some(merge(SENT, c1, "yz")), "lqd");
    s.store_vi(1, Some(k1), "lqd's decrement");
    let r = s.prefilled(Some(merge(SENT, c2, "xw")), "sqi.xw");
    s.vi(2, r as u16);
    s.low("sqi.xw vf12, (vi2++)");
    s.store_vi(2, Some(r as u16 + 1), "sqi's increment");
    let r = s.prefilled(Some(c1), "sqd");
    s.vi(3, r as u16 + 1);
    s.low("sqd.xyzw vf1, (--vi3)");
    s.store_vi(3, Some(r as u16), "sqd's decrement");
    s.vi(4, k2);
    s.low("ilwr.z vi5, (vi4)");
    s.store_vi(5, Some(0xFFFF), "ilwr");
    let r = s.prefilled(Some(merge(SENT, [0xFFFF; 4], "y")), "iswr.y");
    s.vi(6, r as u16);
    s.low("iswr.y vi5, (vi6)");
    s.load(16, SENT);
    s.low("move.xz vf16, vf1");
    s.store(16, Some(merge(SENT, c1, "xz")), "move.xz");
    s.load(17, SENT);
    s.low("mr32.yw vf17, vf1");
    s.store(17, Some([SENT[0], c1[2], SENT[2], c1[0]]), "mr32.yw");
    let seeds = [0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210];
    s.load(18, seeds);
    s.low("rinit r, vf18x");
    s.low("rget.xyzw vf19, r");
    let r0 = seeds[0] & 0x7F_FFFF;
    s.store(19, Some([ONE | r0; 4]), "rinit, rget");
    s.low("rnext.xz vf20, r");
    let r1 = vm::next_r(r0);
    s.store(20, None, "rnext.xz's other fields are vf20's");
    s.low("rget.xyzw vf21, r");
    s.store(21, Some([ONE | r1; 4]), "rnext");
    s.low("rxor r, vf18w");
    s.low("rget.xyzw vf22, r");
    s.store(22, Some([ONE | (r1 ^ seeds[3] & 0x7F_FFFF); 4]), "rxor");
    s.end();
    s
}

/// The flag instructions: FSSET, FSAND, FSEQ, FSOR, FMAND, FMEQ, FMOR, FCSET, FCGET, FCAND, FCEQ, FCOR, each read at
/// least four cycles after the flag it reads was written.
pub fn flags() -> Seg {
    let mut s = Seg::new(0);
    s.low("fsset 0x0");
    s.nops(4);
    s.load(1, [NEG, 0, ONE, TWO]);
    s.load(2, [0, 0, 0, ONE]);
    s.pair("add.xyzw vf3, vf1, vf2 | move vf0, vf0");
    s.nops(4);
    // x negative, y zero: MAC Sx and Zy; status S and Z with SS and ZS.
    let (mac, status) = (0x80 | 0x4, 0x3 | 0xC0);
    s.low("fsand vi1, 0xfff");
    s.store_vi(1, Some(status), "fsand");
    s.low("fseq vi2, 0xc3");
    s.store_vi(2, Some(1), "fseq, equal");
    s.low("fseq vi2, 0xc2");
    s.store_vi(2, Some(0), "fseq, unequal");
    s.low("fsor vi3, 0x100");
    s.store_vi(3, Some(status | 0x100), "fsor");
    for (v, op, want) in [(0xFFFFu16, "fmand", mac), (0x0084, "fmeq", 1), (0x0085, "fmeq", 0), (0x0100, "fmor", mac | 0x100)] {
        s.vi(10, v);
        s.low(format!("{op} vi4, vi10"));
        s.store_vi(4, Some(want as u16), &format!("{op} {v:x}"));
    }
    s.low("fsset 0xa80");
    s.nops(4);
    s.low("fsand vi5, 0xfff");
    s.store_vi(5, Some(0xA80 | 0x3), "fsset keeps the low six bits");
    s.low("fcset 0x0");
    s.nops(4);
    s.load(4, [TWO, 0xC000_0000, 0x3F00_0000, ONE]);
    s.pair("clipw.xyz vf4, vf4 | move vf0, vf0");
    s.nops(4);
    s.low("fcget vi6");
    s.store_vi(6, Some(0x9), "fcget: +x and -y");
    s.low("fcand vi1, 0x000008");
    s.store_vi(1, Some(1), "fcand");
    s.low("fcand vi1, 0x000006");
    s.store_vi(1, Some(0), "fcand, no bit");
    s.low("fceq vi1, 0x000009");
    s.store_vi(1, Some(1), "fceq");
    s.low("fcor vi1, 0xfffff6");
    s.store_vi(1, Some(1), "fcor, all ones");
    s.low("fcor vi1, 0xfffff0");
    s.store_vi(1, Some(0), "fcor, not all ones");
    s.load(5, [0xC000_0000, 0, TWO, ONE]);
    s.pair("clipw.xyz vf5, vf5 | move vf0, vf0");
    s.nops(4);
    s.low("fcget vi7");
    s.store_vi(7, Some((0x9 << 6 | 0x12) & 0xFFF), "fcget after a second clip: -x, +z shifted in");
    s.low("fcset 0x123456");
    s.nops(4);
    s.low("fcget vi8");
    s.store_vi(8, Some(0x456), "fcset, fcget");
    s.low("fceq vi1, 0x123456");
    s.store_vi(1, Some(1), "fceq after fcset");
    s.end();
    s
}

/// The branches, each taken and not: a marker counts the pairs on the path not taken past; BAL and JALR link the pair
/// after the delay slot; JR and JALR jump to an integer register's address (§4.3, in doublewords).
pub fn branches() -> Seg {
    let mut s = Seg::new(0);
    s.vi(9, 0);
    let mut n = 0;
    let mut case = |s: &mut Seg, set: &[(u32, u16)], branch: String, note: String| {
        s.vi(8, 0);
        for &(r, v) in set {
            s.vi(r, v);
        }
        s.nops(1);
        n += 1;
        s.low(branch.replace("TARGET", &format!("t{n}")));
        s.low("iaddiu vi8, vi0, 0x77");
        s.low("iaddiu vi9, vi9, 1");
        s.pair(format!("t{n}: nop | move vf0, vf0"));
        s.store_vi(9, None, "the marker");
        s.store_vi(8, Some(0x77), &format!("{note}: the delay slot ran"));
    };
    let mut expect = 0u16;
    let mut wants = Vec::new();
    for (v, cases) in [(0xFFFFu16, [("ibltz", true), ("ibgtz", false), ("iblez", true), ("ibgez", false)]), (0, [("ibltz", false), ("ibgtz", false), ("iblez", true), ("ibgez", true)]), (1, [("ibltz", false), ("ibgtz", true), ("iblez", false), ("ibgez", true)])] {
        for (op, taken) in cases {
            case(&mut s, &[(1, v)], format!("{op} vi1, TARGET"), format!("{op} {v:x}"));
            expect += u16::from(!taken);
            wants.push(expect);
        }
    }
    for (a, b, op, taken) in [(5u16, 5u16, "ibeq", true), (5, 6, "ibeq", false), (5, 5, "ibne", false), (5, 6, "ibne", true)] {
        case(&mut s, &[(1, a), (2, b)], format!("{op} vi1, vi2, TARGET"), format!("{op} {a}, {b}"));
        expect += u16::from(!taken);
        wants.push(expect);
    }
    case(&mut s, &[], "b TARGET".into(), "b".into());
    wants.push(expect);
    let mut w = wants.into_iter();
    let marker_slots: Vec<usize> = s.notes.iter().filter(|(_, n)| n == "the marker").map(|(i, _)| *i).collect();
    for i in marker_slots {
        let v = w.next().unwrap();
        s.results[i] = Some([v as u32; 4]);
    }
    // BAL: the link is the pair after the delay slot.
    let at = s.here() as u16;
    s.low("bal vi15, bal_to");
    s.low("move vf0, vf0");
    s.low("iaddiu vi9, vi9, 0x100");
    s.pair("bal_to: nop | move vf0, vf0");
    s.store_vi(15, Some(at + 2), "bal's link");
    // JR to a pair whose address is placed in VI by IADDIU (a backward target, so its address is known).
    s.low("b over");
    s.low("move vf0, vf0");
    let target = s.here() as u16;
    s.low("iaddiu vi10, vi0, 0x55");
    s.low("b after");
    s.low("move vf0, vf0");
    s.pair("over: nop | move vf0, vf0");
    s.vi(4, target);
    s.nops(1);
    let at = s.here() as u16;
    s.low("jalr vi4, vi4");
    s.low("move vf0, vf0");
    s.pair("after: nop | move vf0, vf0");
    s.store_vi(10, Some(0x55), "jalr reached its target");
    s.store_vi(4, Some(at + 2), "jalr with one register: the link written after the target read");
    // JR, the same way.
    s.low("b over_jr");
    s.low("move vf0, vf0");
    let target = s.here() as u16;
    s.low("iaddiu vi11, vi0, 0x66");
    s.low("b after_jr");
    s.low("move vf0, vf0");
    s.pair("over_jr: nop | move vf0, vf0");
    s.vi(5, target);
    s.nops(1);
    s.low("jr vi5");
    s.low("move vf0, vf0");
    s.pair("after_jr: nop | move vf0, vf0");
    s.store_vi(11, Some(0x66), "jr reached its target");
    s.end();
    s
}

/// DIV, SQRT and RSQRT in micro mode with WAITQ, Q read through ADDq.x; their I and D flags read after the quotient.
pub fn divider() -> Seg {
    let mut s = Seg::new(0);
    let cases = [("div q, vf1x, vf2x", ONE, FOUR), ("div q, vf1x, vf2x", 0, 0), ("div q, vf1x, vf2x", ONE, 0), ("sqrt q, vf2x", 0, FOUR), ("sqrt q, vf2x", 0, 0xC080_0000), ("rsqrt q, vf1x, vf2x", ONE, FOUR), ("rsqrt q, vf1x, vf2x", 0, 0), ("rsqrt q, vf1x, vf2x", ONE, 0xC080_0000)];
    for (op, a, b) in cases {
        s.load(1, [a; 4]);
        s.load(2, [b; 4]);
        s.load(3, SENT);
        s.low(op);
        s.low("waitq");
        s.pair("addq.x vf3, vf0, q | move vf0, vf0");
        s.low("fsand vi1, 0x30");
        let (q, invalid, divide) = match &op[..3] {
            "div" => vm::div(a, b),
            "sqr" => vm::sqrt(b),
            _ => {
                let (q, i, d) = vm::rsqrt(a, b);
                (q, i, d && !vm::zero(a))
            }
        };
        let flags = (invalid as u16) << 4 | (divide as u16) << 5;
        s.store(3, Some([vm::add(0, q).0, SENT[1], SENT[2], SENT[3]]), &format!("{op} {a:x} {b:x}: Q"));
        s.store_vi(1, Some(flags), &format!("{op} {a:x} {b:x}: I and D"));
    }
    s.end();
    s
}

/// Every upper operation once in micro mode on fixed vectors, the result the page's (the arithmetic `s12::manual`'s).
pub fn upper() -> Seg {
    let mut s = Seg::new(0);
    let a = [0x3FC0_0000, 0xC020_0000, 0x4110_0000, 0x3E80_0000];
    let b = [0x4000_0000, 0x3F40_0000, 0xC0A0_0000, 0x4120_0000];
    let acc = [ONE, NEG, 0x4040_0000, 0];
    let (i, q) = (0x3FA0_0000u32, 0xC000_0000u32);
    s.load(1, a);
    s.load(2, b);
    s.load(4, acc);
    s.load(30, [0x8000_0000; 4]);
    s.pair(format!("[i] nop | 0x{i:08x}"));
    s.load(5, [ONE, 0, 0, 0]);
    s.load(6, [q, 0, 0, 0]);
    s.low("div q, vf6x, vf5x");
    s.low("waitq");
    for o in emusen_mips::vu::SPECIAL1.iter().chain(emusen_mips::vu::SPECIAL2.iter()).flatten() {
        if o.unit != emusen_mips::vu::Unit::Upper {
            continue;
        }
        let name = o.name;
        let base = ["madd", "msub", "mini", "max", "add", "sub", "mul"].iter().find(|b| name.starts_with(**b)).copied();
        let rest = base.map_or("", |b| &name[b.len()..]);
        let into_acc = rest.starts_with('a');
        let bc = rest.trim_start_matches('a');
        let second = |l: usize| -> u32 {
            match o.form {
                emusen_mips::vu::Form::FmacQ | emusen_mips::vu::Form::AccQ => q,
                emusen_mips::vu::Form::FmacI | emusen_mips::vu::Form::AccI => i,
                _ => match bc {
                    "x" => b[0],
                    "y" => b[1],
                    "z" => b[2],
                    "w" => b[3],
                    _ => b[l],
                },
            }
        };
        let fam = base.unwrap_or(&name[..name.len().min(4)]);
        let result: Option<Lanes> = match fam {
            "add" | "sub" | "mul" | "madd" | "msub" => Some(std::array::from_fn(|l| {
                let y = second(l);
                match fam {
                    "add" => vm::add(a[l], y).0,
                    "sub" => vm::sub(a[l], y).0,
                    "mul" => vm::mul(a[l], y).0,
                    _ => vm::fused(acc[l], a[l], y, fam == "msub").0.0,
                }
            })),
            "max" => Some(std::array::from_fn(|l| vm::max(a[l], second(l)))),
            "mini" => Some(std::array::from_fn(|l| vm::mini(a[l], second(l)))),
            _ => None,
        };
        let operands = match o.form {
            emusen_mips::vu::Form::Fmac => "vf3, vf1, vf2",
            emusen_mips::vu::Form::FmacQ => "vf3, vf1, q",
            emusen_mips::vu::Form::FmacI => "vf3, vf1, i",
            emusen_mips::vu::Form::Acc => "acc, vf1, vf2",
            emusen_mips::vu::Form::AccQ => "acc, vf1, q",
            emusen_mips::vu::Form::AccI => "acc, vf1, i",
            emusen_mips::vu::Form::FtFs => "vf3, vf1",
            emusen_mips::vu::Form::Clip => "vf1, vf2",
            _ => "",
        };
        // ACC ← acc by ADDA with −0 (D-30's sign rule keeps every lane); a result in ACC read back by MADD with VF00
        // times VF00, which adds +0 to x, y and z and 1 to w.
        s.pair("adda.xyzw acc, vf4, vf30 | move vf0, vf0");
        s.load(3, SENT);
        let text = if operands.is_empty() { name.to_string() } else { format!("{name}.xyzw {operands}") };
        match (name, into_acc) {
            ("nop", _) => {
                s.pair("nop | move vf0, vf0");
                s.store(3, Some(SENT), "nop");
            }
            ("clipw", _) => {
                s.low("fcset 0x0");
                s.pair(format!("clipw.xyz {operands} | move vf0, vf0"));
                s.nops(4);
                s.low("fcget vi1");
                s.store_vi(1, Some(vm::clip(a, b[3]) as u16 & 0xFFF), "clipw");
            }
            ("opmula", _) => {
                s.pair(format!("opmula.xyz {operands} | move vf0, vf0"));
                s.pair("opmsub.xyz vf3, vf2, vf1 | move vf0, vf0");
                let cross: Lanes = std::array::from_fn(|l| {
                    if l == 3 {
                        return SENT[3];
                    }
                    let (j, k) = ((l + 1) % 3, (l + 2) % 3);
                    let first = vm::mul(a[j], b[k]).0;
                    vm::fused(first, b[j], a[k], true).0.0
                });
                s.store(3, Some(cross), "opmula, opmsub: the outer product");
            }
            ("opmsub", _) => {}
            (_, true) if base.is_some() => {
                s.pair(format!("{text} | move vf0, vf0"));
                s.pair("madd.xyzw vf3, vf0, vf0 | move vf0, vf0");
                let want = result.map(|r| std::array::from_fn(|l| vm::add(r[l], if l == 3 { ONE } else { 0 }).0));
                s.store(3, want, &format!("{name} into ACC, read back"));
            }
            _ => {
                s.pair(format!("{text} | move vf0, vf0"));
                let want = match fam {
                    "abs" => Some(a.map(|x| x & 0x7FFF_FFFF)),
                    "itof" | "ftoi" => {
                        let bits: u32 = name[4..].parse().unwrap();
                        Some(a.map(|x| if fam == "ftoi" { vm::ftoi(x, bits) } else { vm::itof(x, bits) }))
                    }
                    _ => result,
                };
                s.store(3, want, name);
            }
        }
    }
    s.end();
    s
}

/// VU1's EFU over the inputs `vu/lower/efu`'s console text names, each read after WAITP by MFP (§3.4.6, §7.2).
pub fn efu() -> Seg {
    let mut s = Seg::new(1);
    for name in manual::EFU {
        for (label, v) in manual::EFU_INPUTS {
            s.load(1, v);
            if manual::efu_scalar(name) {
                s.low(format!("{name} p, vf1z"));
            } else {
                s.low(format!("{name} p, vf1"));
            }
            s.low("waitp");
            s.low("mfp.xyzw vf2, p");
            s.store(2, Some([manual::efu(name, v, 2); 4]), &format!("{name} {label}"));
        }
    }
    s.load(3, SENT);
    s.low("esqrt p, vf1z");
    s.low("waitp");
    s.low("mfp.yw vf3, p");
    s.store(3, Some(merge(SENT, [manual::efu("esqrt", manual::EFU_INPUTS[15].1, 2); 4], "yw")), "mfp.yw");
    s.end();
    s
}

/// When each result becomes visible inside a microprogram (§3.4): Q 1 to 8 pairs after DIV and 1 to 14 after RSQRT;
/// the flags 1 to 5 pairs after the instruction that writes them; a branch on an integer written just before, one pair
/// before, and after a flag check; the I bit; the write priority; the E bit's delay slot and TPC; JALR's register.
pub fn visibility() -> Vec<Seg> {
    let mut segs = Vec::new();
    let mut s = Seg::new(0);
    for (op, latency) in [("div q, vf2x, vf1x", manual::DIV), ("rsqrt q, vf2x, vf1x", manual::RSQRT)] {
        for gap in 1..=latency + 1 {
            s.load(1, [FOUR; 4]);
            s.load(2, [TWO; 4]);
            s.load(5, [THREE; 4]);
            s.low("div q, vf5x, vf2x");
            s.low("waitq");
            let old = vm::div(THREE, TWO).0;
            s.low(op);
            let quotient = if op.starts_with("div") { vm::div(TWO, FOUR).0 } else { vm::rsqrt(TWO, FOUR).0 };
            s.nops(gap as usize - 1);
            s.pair("addq.xyzw vf3, vf0, q | move vf0, vf0");
            let q = if gap >= latency { quotient } else { old };
            s.store(3, Some([q, q, q, vm::add(ONE, q).0]), &format!("{op}, Q read {gap} pairs on"));
            s.low("waitq");
        }
    }
    for gap in 1..=5usize {
        let seen = gap >= manual::FLAGS as usize;
        s.load(1, [NEG; 4]);
        s.load(2, [TWO, 0, 0, ONE]);
        s.vi(10, 0xFFFF);
        s.pair("add.xyzw vf4, vf0, vf0 | move vf0, vf0");
        s.low("fsset 0x0");
        s.low("fcset 0x0");
        s.nops(4);
        s.pair("add.xyzw vf3, vf1, vf0 | move vf0, vf0");
        s.nops(gap - 1);
        s.low("fsand vi1, 0xfc0");
        s.store_vi(1, Some(if seen { 0xC0 } else { 0 }), &format!("fsand {gap} pairs after add"));
        s.nops(4);
        s.pair("add.xyzw vf4, vf0, vf0 | move vf0, vf0");
        s.nops(4);
        s.pair("add.xyzw vf3, vf1, vf0 | move vf0, vf0");
        s.nops(gap - 1);
        s.low("fmand vi2, vi10");
        s.store_vi(2, Some(if seen { 0xE1 } else { 0x0E }), &format!("fmand {gap} pairs after add"));
        s.pair("clipw.xyz vf2, vf2 | move vf0, vf0");
        s.nops(gap - 1);
        s.low("fcget vi3");
        s.store_vi(3, Some(u16::from(seen)), &format!("fcget {gap} pairs after clip"));
        s.nops(4);
    }
    s.end();
    segs.push(s);

    let mut s = Seg::new(0);
    for (setter, gap, taken) in [("iaddiu vi1, vi0, 0x0", 0usize, true), ("iaddiu vi1, vi0, 0x0", 1, false), ("fsand vi1, 0x0", 0, false), ("fsand vi1, 0x800", 0, false)] {
        s.vi(1, 1);
        s.vi(9, 0);
        s.low("fsset 0x0");
        s.nops(4);
        s.low(setter);
        s.nops(gap);
        let l = format!("b{}", s.here());
        s.low(format!("ibne vi1, vi0, {l}"));
        s.low("move vf0, vf0");
        s.low("iaddiu vi9, vi0, 0x1");
        s.pair(format!("{l}: nop | move vf0, vf0"));
        s.store_vi(9, Some(u16::from(!taken)), &format!("ibne after {setter} with {gap} pairs between"));
    }
    s.load(2, SENT);
    s.load(3, SENT);
    s.pair(format!("[i] nop | 0x{ONE:08x}"));
    s.pair(format!("[i] addi.x vf2, vf0, i | 0x{FOUR:08x}"));
    s.pair("addi.x vf3, vf0, i | move vf0, vf0");
    s.store(2, Some([ONE, SENT[1], SENT[2], SENT[3]]), "addi in the I bit's pair: the old I (D-81)");
    s.store(3, Some([FOUR, SENT[1], SENT[2], SENT[3]]), "addi in the next pair: the new I");
    s.load(5, [TWO; 4]);
    s.load(6, [ONE; 4]);
    s.load(7, [FOUR; 4]);
    s.pair("add.xy vf5, vf5, vf6 | move.w vf5, vf7");
    s.store(5, Some([THREE, THREE, TWO, TWO]), "upper and lower write vf5: the lower's move.w discarded");
    s.load(8, [TWO; 4]);
    s.pair("add.xyzw vf8, vf8, vf6 | move.xyzw vf9, vf8");
    s.store(9, Some([TWO; 4]), "the lower reads vf8 as the pair found it");
    s.end();
    segs.push(s);

    let mut s = Seg::new(0);
    s.vi(2, 0);
    s.vi(3, 0);
    s.pair("[e] nop | iaddiu vi1, vi0, 0x11");
    s.low("iaddiu vi2, vi0, 0x22");
    s.low("iaddiu vi3, vi0, 0x33");
    s.tpc = Some(s.here() as u16 - 1);
    segs.push(s);
    let mut s = Seg::new(0);
    s.store_vi(2, Some(0x22), "the E bit's delay slot ran (read in a second program)");
    s.store_vi(3, Some(0), "the pair after the delay slot did not");
    s.end();
    segs.push(s);
    segs
}

/// P 1 to 12 pairs after ESADD on VU1 (§3.4.6), with WAITP after.
pub fn efu_visibility() -> Seg {
    let mut s = Seg::new(1);
    let latency = manual::efu_latency("esadd");
    for gap in 1..=latency + 1 {
        s.load(1, [ONE, ONE, ONE, 0]);
        s.load(2, [ONE, TWO, ONE, 0]);
        s.low("esadd p, vf1");
        s.low("waitp");
        s.low("esadd p, vf2");
        s.nops(gap as usize - 1);
        s.low("mfp.xyzw vf3, p");
        let p = if gap >= latency { 0x40C0_0000 } else { THREE };
        s.store(3, Some([p; 4]), &format!("esadd, P read {gap} pairs on"));
        s.low("waitp");
    }
    s.end();
    s
}

pub fn all_segs() -> Vec<(&'static str, Vec<Seg>)> {
    vec![
        ("s13-integer", vec![integer()]),
        ("s13-flags", vec![flags()]),
        ("s13-branch", vec![branches()]),
        ("s13-divider", vec![divider()]),
        ("s13-upper", vec![upper()]),
        ("s13-efu", vec![efu()]),
        ("s13-visible", visibility().into_iter().chain([efu_visibility()]).collect()),
    ]
}

pub fn all() -> Vec<Program> {
    all_segs().into_iter().map(|(name, segs)| program(name, &segs)).collect()
}

/// Every micro instruction name the programs' microprograms carry, upper and lower.
pub fn covered() -> std::collections::BTreeSet<String> {
    let mut names = std::collections::BTreeSet::new();
    for (_, segs) in all_segs() {
        for seg in segs {
            for pair in seg.assembled() {
                let (up, low) = ((pair >> 32) as u32, pair as u32);
                if let Some(o) = emusen_mips::vu::operation(up).filter(|o| o.unit == emusen_mips::vu::Unit::Upper) {
                    names.insert(o.name.to_string());
                }
                if up & emusen_mips::vu::I_BIT == 0
                    && let Some(o) = emusen_mips::vu::lower(low)
                {
                    names.insert(o.name.to_string());
                }
            }
        }
    }
    names
}
