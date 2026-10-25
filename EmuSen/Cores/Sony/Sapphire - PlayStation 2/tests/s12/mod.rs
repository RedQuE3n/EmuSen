//! The project's S12 programs (Sapphire_Native.md §21.4): bare EE programs over every macro instruction VU0 takes from
//! the EE and every transfer, generated as assembly for `emusen-mips` in S6's form (`crate::s6::Program`). Each enables
//! COP2, loads its operands into VU0 by LQ and QMTC2, runs the instruction, and records what it wrote by QMFC2 and CFC2;
//! a lane carries one input case, so one instruction tests four. The expected record comes from `manual`, the VU User's
//! Manual's pages transcribed apart from `src/vu` and `src/ee/cop2.rs`.

#![allow(dead_code)]

pub mod manual;

use crate::s6::{self, AREA, Program};
use crate::s9::{ACCUMULATORS, FEW, INTEGERS, VALUES};
use manual::{Flags, Lanes, MAX, R, SIGN};

/// What the destination register holds before an instruction, so that an unwritten lane shows.
pub const SENTINEL: Lanes = [0x5A5A_C3C3, 0x5A5A_C3C4, 0x5A5A_C3C5, 0x5A5A_C3C6];
const NEG_ZERO: Lanes = [SIGN; 4];
const VF00: Lanes = [0, 0, 0, 0x3F80_0000];

pub fn quad(l: Lanes) -> u128 {
    l.iter().rev().fold(0u128, |a, &f| a << 32 | f as u128)
}
fn sext(v: u32) -> u64 {
    v as i32 as i64 as u64
}
fn mask_text(dest: u32) -> String {
    if dest == 0 { String::new() } else { format!(".{}", (0..4).filter(|l| dest & 8 >> l != 0).map(|l| ['x', 'y', 'z', 'w'][l]).collect::<String>()) }
}

/// A program and the VU0 state the generator tracks as it emits: the flags, ACC, I, Q, R and the clip flags.
pub struct Gen {
    pub p: Program,
    /// Where each FMAC or divider case's records start (a word index) and what it ran: (lane, fs, second, ACC).
    pub notes: Vec<Note>,
    pub flags: Flags,
    pub acc: Lanes,
    pub i: u32,
    pub q: u32,
    pub clip: u32,
}

impl Gen {
    /// COP2 enabled; t5 the sentinel, vf31 −0 in every lane; the divider's flags cleared by VSQRT of +0, the FMAC's
    /// set by an addition into VF00, the sticky flags cleared.
    pub fn new(name: &'static str) -> Gen {
        let mut p = Program::new(name);
        p.emit("mfc0 t0, Status");
        p.emit("lui t1, 0x4000");
        p.emit("or t0, t0, t1");
        p.emit("mtc0 t0, Status");
        p.emit("sync.p");
        p.quad("t5", quad(SENTINEL));
        p.quad("t0", quad(NEG_ZERO));
        p.emit("qmtc2 t0, vf31");
        p.emit("vsqrt q, vf0x");
        p.emit("vwaitq");
        p.emit("vadd.xyzw vf0, vf0, vf0");
        p.emit("ctc2 zero, vi16");
        p.emit("ctc2 zero, vi18");
        let mut g = Gen { p, notes: Vec::new(), flags: Flags::default(), acc: [0; 4], i: 0, q: 0, clip: 0 };
        g.flags.divide(false, false);
        let r: Vec<_> = (0..4).map(|l| (l, manual::add(VF00[l], VF00[l]), None)).collect();
        g.flags.fmac(&r);
        g.flags.write_status(0);
        g
    }

    pub fn set_vf(&mut self, n: u32, v: Lanes) {
        self.p.quad("t0", quad(v));
        self.p.emit(format!("qmtc2 t0, vf{n}"));
    }
    pub fn set_i(&mut self, v: u32) {
        self.p.constant("t0", v as u64);
        self.p.emit("ctc2 t0, vi21");
        self.i = v;
    }
    pub fn set_q(&mut self, v: u32) {
        self.p.constant("t0", v as u64);
        self.p.emit("ctc2 t0, vi22");
        self.q = v;
    }
    /// ACC ← `v` by ADDA with −0, which leaves every lane as it is (an addition of a zero, D-30's rule).
    pub fn set_acc(&mut self, v: Lanes) {
        self.set_vf(4, v);
        self.p.emit("vadda.xyzw acc, vf4, vf31");
        let r: Vec<_> = (0..4).map(|l| (l, manual::add(v[l], SIGN), None)).collect();
        self.flags.fmac(&r);
        self.acc = v;
    }
    pub fn clear_sticky(&mut self) {
        self.p.emit("ctc2 zero, vi16");
        self.flags.write_status(0);
    }
    pub fn record_vf(&mut self, n: u32, v: Option<Lanes>) {
        self.p.emit(format!("qmfc2 t2, vf{n}"));
        self.align();
        self.p.record_quad("t2", v.map(quad));
    }
    /// A zero doubleword where a quadword record would start at an odd doubleword.
    pub fn align(&mut self) {
        if self.p.expected.len() % 2 == 1 {
            self.p.record("zero", Some(0));
        }
    }
    pub fn record_ccr(&mut self, n: u32, v: Option<u32>) {
        self.p.emit(format!("cfc2 t2, vi{n}"));
        self.p.record("t2", v.map(sext));
    }
    pub fn record_flags(&mut self) {
        let f = self.flags;
        self.record_ccr(16, Some(f.status));
        self.record_ccr(17, Some(f.mac));
    }
    /// ACC read into vf3 by MADDx with a product of −0 (vf31 by VF00's x), which adds a zero; the flags it sets tracked.
    pub fn record_acc(&mut self) {
        self.p.emit("vmaddx.xyzw vf3, vf31, vf0");
        let product = (SIGN, false, false);
        let r: Vec<_> = (0..4).map(|l| (l, manual::fused(self.acc[l], SIGN, 0, false).0, Some(product))).collect();
        self.flags.fmac(&r);
        let acc = self.acc;
        self.record_vf(3, Some(acc));
    }
}

/// A case for the referees' analysis (examples/s12.rs).
pub struct Note {
    pub at: usize,
    pub what: String,
    pub lanes: Vec<(usize, u32, u32, u32)>,
}

impl Gen {
    fn note(&mut self, what: String, lanes: Vec<(usize, u32, u32, u32)>) {
        let at = self.p.expected.len() * 2;
        self.notes.push(Note { at, what, lanes });
    }
}

/// The second operand of an FMAC family: VF[ft] lane by lane, a broadcast field of it, I or Q.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Second {
    Vector,
    Bc(usize),
    I,
    Q,
}

impl Second {
    fn suffix(self) -> String {
        match self {
            Second::Vector => String::new(),
            Second::Bc(k) => ['x', 'y', 'z', 'w'][k].to_string(),
            Second::I => "i".into(),
            Second::Q => "q".into(),
        }
    }
    fn operand(self) -> &'static str {
        match self {
            Second::Vector | Second::Bc(_) => "vf2",
            Second::I => "i",
            Second::Q => "q",
        }
    }
    fn value(self, g: &Gen, b: Lanes, lane: usize) -> u32 {
        match self {
            Second::Vector => b[lane],
            Second::Bc(k) => b[k],
            Second::I => g.i,
            Second::Q => g.q,
        }
    }
}

/// One FMAC case: `family` (add, sub, mul, madd, msub) into vf3 or ACC, its operands in vf1, vf2, ACC, I and Q.
pub fn fmac_case(g: &mut Gen, family: &str, to_acc: bool, second: Second, dest: u32, a: Lanes, b: Lanes) {
    g.set_vf(1, a);
    if matches!(second, Second::Vector | Second::Bc(_)) {
        g.set_vf(2, b);
    }
    g.p.emit("qmtc2 t5, vf3");
    g.clear_sticky();
    let name = format!("v{family}{}{}", if to_acc { "a" } else { "" }, second.suffix());
    let target = if to_acc { "acc".to_string() } else { "vf3".to_string() };
    g.p.emit(format!("{name}{} {target}, vf1, {}", mask_text(dest), second.operand()));
    let mut out = if to_acc { g.acc } else { SENTINEL };
    let mut results = Vec::new();
    let lanes = manual::lanes_of(dest).map(|l| (l, a[l], second.value(g, b, l), g.acc[l])).collect();
    g.note(format!("{name}{}{}", mask_text(dest), if to_acc { " acc" } else { "" }), lanes);
    for l in manual::lanes_of(dest) {
        let y = second.value(g, b, l);
        let (r, p): (R, Option<R>) = match family {
            "add" => (manual::add(a[l], y), None),
            "sub" => (manual::sub(a[l], y), None),
            "mul" => (manual::mul(a[l], y), None),
            "madd" | "msub" => {
                let (r, p) = manual::fused(g.acc[l], a[l], y, family == "msub");
                (r, Some(p))
            }
            _ => panic!("{family}"),
        };
        out[l] = r.0;
        results.push((l, r, p));
    }
    g.flags.fmac(&results);
    if to_acc {
        g.acc = out;
        g.record_flags();
        g.record_acc();
    } else {
        g.record_vf(3, Some(out));
        g.record_flags();
    }
}

/// Every ordered pair of `xs` by `ys`, four to a case.
pub fn pairs(xs: &[u32], ys: &[u32]) -> Vec<(Lanes, Lanes)> {
    let all: Vec<(u32, u32)> = xs.iter().flat_map(|&a| ys.iter().map(move |&b| (a, b))).collect();
    all.chunks(4)
        .map(|c| {
            let mut a = [0x3F80_0000; 4];
            let mut b = [0x3F80_0000; 4];
            for (i, &(x, y)) in c.iter().enumerate() {
                a[i] = x;
                b[i] = y;
            }
            (a, b)
        })
        .collect()
}

/// The variants each family is run in beyond the vector form: the four broadcasts, I and Q, on fewer operands, with
/// field masks that leave lanes unwritten.
fn variants(g: &mut Gen, family: &str, to_acc: bool) {
    let quads = pairs(&FEW, &FEW[..4]);
    for (k, (a, b)) in quads.iter().enumerate() {
        let second = match k % 6 {
            0..4 => Second::Bc(k % 4),
            4 => Second::I,
            _ => Second::Q,
        };
        if second == Second::I {
            g.set_i(b[k % 4]);
        }
        if second == Second::Q {
            g.set_q(b[(k + 1) % 4]);
        }
        let dest = [0xF, 0x9, 0x6, 0xE, 0x1, 0x8, 0x3, 0xC, 0x5, 0xA, 0x7, 0xB][k % 12];
        if family.starts_with('m') && family != "mul" {
            g.set_acc([0, 1, 2, 3].map(|i| ACCUMULATORS[(k + i) % 8]));
        }
        fmac_case(g, family, to_acc, second, dest, *a, *b);
    }
}

/// VADD and VSUB over every ordered pair of the classes (S9's thirty-two), VADDA and VSUBA likewise; then the
/// broadcast, I and Q forms and the field masks.
pub fn add() -> Program {
    add_gen().p
}

/// The generator of `add`, with its notes.
pub fn add_gen() -> Gen {
    let mut g = Gen::new("s12-add");
    for family in ["add", "sub"] {
        for to_acc in [false, true] {
            for (a, b) in pairs(&VALUES, &VALUES) {
                fmac_case(&mut g, family, to_acc, Second::Vector, 0xF, a, b);
            }
            variants(&mut g, family, to_acc);
        }
    }
    for dest in 0..16 {
        fmac_case(&mut g, "add", false, Second::Vector, dest, [0x3F80_0000, 0x4000_0000, 0x4040_0000, 0x4080_0000], [0x3F80_0000; 4]);
    }
    g
}

/// VMUL and VMULA over every ordered pair (the product is not commutative in its last bit, D-26), then the variants.
pub fn mul() -> Program {
    mul_gen().p
}

/// The generator of `mul`, with its notes.
pub fn mul_gen() -> Gen {
    let mut g = Gen::new("s12-mul");
    for to_acc in [false, true] {
        for (a, b) in pairs(&VALUES, &VALUES) {
            fmac_case(&mut g, "mul", to_acc, Second::Vector, 0xF, a, b);
        }
        variants(&mut g, "mul", to_acc);
    }
    g
}

/// VMADD, VMSUB, VMADDA and VMSUBA over eight accumulators by every ordered pair of S9's twelve, then the variants;
/// VOPMULA and VOPMSUB on vectors of the classes.
pub fn fused() -> Program {
    fused_gen().p
}

/// The generator of `fused`, with its notes.
pub fn fused_gen() -> Gen {
    let mut g = Gen::new("s12-fused");
    for family in ["madd", "msub"] {
        for to_acc in [false, true] {
            for (k, (a, b)) in pairs(&FEW, &FEW).into_iter().enumerate() {
                let acc = [0, 1, 2, 3].map(|i| ACCUMULATORS[(k + i) % 8]);
                g.set_acc(acc);
                fmac_case(&mut g, family, to_acc, Second::Vector, 0xF, a, b);
            }
            variants(&mut g, family, to_acc);
        }
    }
    for (k, (a, b)) in pairs(&VALUES, &FEW).into_iter().enumerate() {
        let acc = [0, 1, 2, 3].map(|i| ACCUMULATORS[(k + i) % 8]);
        outer(&mut g, acc, a, b);
    }
    g
}

/// VOPMULA then VOPMSUB: ACC's x, y, z ← the first terms of vf1 × vf2, vf3 ← ACC less the second terms (§4.2).
fn outer(g: &mut Gen, acc: Lanes, a: Lanes, b: Lanes) {
    g.set_acc(acc);
    g.set_vf(1, a);
    g.set_vf(2, b);
    g.p.emit("qmtc2 t5, vf3");
    g.clear_sticky();
    g.p.emit("vopmula.xyz acc, vf1, vf2");
    let mut first = Vec::new();
    for l in 0..3 {
        let r = manual::mul(a[(l + 1) % 3], b[(l + 2) % 3]);
        g.acc[l] = r.0;
        first.push((l, r, None));
    }
    g.flags.fmac(&first);
    g.record_flags();
    g.clear_sticky();
    g.p.emit("vopmsub.xyz vf3, vf2, vf1");
    let mut out = SENTINEL;
    let mut second = Vec::new();
    for l in 0..3 {
        let (r, p) = manual::fused(g.acc[l], b[(l + 1) % 3], a[(l + 2) % 3], true);
        out[l] = r.0;
        second.push((l, r, Some(p)));
    }
    g.flags.fmac(&second);
    g.record_vf(3, Some(out));
    g.record_flags();
}

/// A flagless lane operation into vf3: `text` written with vf3 as ft and vf1 as fs, its lanes by `f`.
fn unary_case(g: &mut Gen, text: &str, dest: u32, a: Lanes, f: impl Fn(u32) -> u32) {
    g.set_vf(1, a);
    g.p.emit("qmtc2 t5, vf3");
    g.p.emit(format!("{text}{} vf3, vf1", mask_text(dest)));
    let mut out = SENTINEL;
    for l in manual::lanes_of(dest) {
        out[l] = f(a[l]);
    }
    g.record_vf(3, Some(out));
}

/// The instructions that set no MAC flag: VABS, VMAX and VMINI with their broadcast and I forms, the conversions,
/// VMOVE, VMR32, VMFIR, VMTIR, VNOP, and VCLIP's judgements; the flags read after each group, unchanged.
pub fn misc() -> Program {
    misc_gen().p
}

/// The generator of `misc`, with its notes.
pub fn misc_gen() -> Gen {
    let mut g = Gen::new("s12-misc");
    for (a, _) in pairs(&VALUES, &[0]) {
        unary_case(&mut g, "vabs", 0xF, a, |x| x & MAX);
    }
    g.record_flags();
    for n in [0u32, 4, 12, 15] {
        for (a, _) in pairs(&VALUES, &[0]) {
            unary_case(&mut g, &format!("vftoi{n}"), 0xF, a, |x| manual::ftoi(x, n));
        }
        for (a, _) in pairs(&INTEGERS, &[0]) {
            unary_case(&mut g, &format!("vitof{n}"), 0xF, a, |x| manual::itof(x, n));
        }
    }
    for (k, (a, _)) in pairs(&VALUES, &[0]).into_iter().enumerate() {
        let dest = [0x8, 0x4, 0x2, 0x1, 0xA, 0x5, 0xF, 0x0][k % 8];
        unary_case(&mut g, "vmove", dest, a, |x| x);
        g.set_vf(1, a);
        g.p.emit("qmtc2 t5, vf3");
        g.p.emit(format!("vmr32{} vf3, vf1", mask_text(dest)));
        let mut out = SENTINEL;
        for l in manual::lanes_of(dest) {
            out[l] = a[(l + 1) % 4];
        }
        g.record_vf(3, Some(out));
    }
    g.record_flags();
    for (k, (a, b)) in pairs(&FEW, &FEW).into_iter().enumerate() {
        for name in ["max", "mini"] {
            let pick = |x: u32, y: u32| if name == "max" { manual::max(x, y) } else { manual::mini(x, y) };
            let second = match k % 6 {
                0 | 1 => Second::Vector,
                2..5 => Second::Bc(k % 4),
                _ => Second::I,
            };
            if second == Second::I {
                g.set_i(b[0]);
            }
            g.set_vf(1, a);
            g.set_vf(2, b);
            g.p.emit("qmtc2 t5, vf3");
            g.p.emit(format!("v{name}{}.xyzw vf3, vf1, {}", second.suffix(), second.operand()));
            let out = [0, 1, 2, 3].map(|l| pick(a[l], second.value(&g, b, l)));
            g.record_vf(3, Some(out));
        }
    }
    g.record_flags();
    g.p.emit("vnop");
    g.record_flags();
    for (k, &v) in [0u32, 1, 0x7FFF, 0x8000, 0xFFFF, 0x1234, 0xFEDC, 0x0080].iter().enumerate() {
        g.p.constant("t0", v as u64);
        g.p.emit("ctc2 t0, vi5");
        let dest = [0xF, 0x8, 0x4, 0x2, 0x1, 0x6, 0x9, 0xF][k];
        g.p.emit("qmtc2 t5, vf3");
        g.p.emit(format!("vmfir{} vf3, vi5", mask_text(dest)));
        let mut out = SENTINEL;
        for l in manual::lanes_of(dest) {
            out[l] = v as u16 as i16 as i32 as u32;
        }
        g.record_vf(3, Some(out));
    }
    for (k, (a, _)) in pairs(&VALUES, &[0]).into_iter().enumerate() {
        let f = k % 4;
        g.set_vf(1, a);
        g.p.emit("ctc2 t5, vi6");
        g.p.emit(format!("vmtir vi6, vf1{}", ['x', 'y', 'z', 'w'][f]));
        g.record_ccr(6, Some(a[f] & 0xFFFF));
    }
    g.p.emit("ctc2 zero, vi18");
    g.clip = 0;
    for (k, (a, b)) in pairs(&VALUES, &VALUES[..8]).into_iter().enumerate().filter(|(k, _)| k % 3 == 0) {
        g.set_vf(1, a);
        g.set_vf(2, b);
        g.p.emit("vclipw.xyz vf1, vf2");
        g.clip = (g.clip << 6 | manual::clip(a, b[3])) & 0xFF_FFFF;
        let clip = g.clip;
        if k % 2 == 0 {
            g.record_ccr(18, Some(clip));
        }
    }
    g.record_flags();
    g
}

/// VDIV, VSQRT and VRSQRT, Q read by CFC2 after VWAITQ with the status flag, over every pair of the classes in fields
/// the fsf and ftf bits choose; and Q read before the quotient lands (§5.4.7).
pub fn fdiv() -> Program {
    fdiv_gen().p
}

/// The generator of `fdiv`, with its notes.
pub fn fdiv_gen() -> Gen {
    let mut g = Gen::new("s12-fdiv");
    for (name, f) in [("vdiv", manual::div as fn(u32, u32) -> (u32, bool, bool)), ("vrsqrt", manual::rsqrt)] {
        for (k, (a, b)) in pairs(&VALUES, &VALUES).into_iter().enumerate() {
            g.set_vf(1, a);
            g.set_vf(2, b);
            for lane in 0..4 {
                let (fs, ft) = (lane, (lane + k) % 4);
                g.clear_sticky();
                g.p.emit(format!("{name} q, vf1{}, vf2{}", ['x', 'y', 'z', 'w'][fs], ['x', 'y', 'z', 'w'][ft]));
                g.p.emit("vwaitq");
                let (q, invalid, divide) = f(a[fs], b[ft]);
                g.note(name.to_string(), vec![(0, a[fs], b[ft], 0)]);
                g.flags.divide(invalid, divide);
                g.q = q;
                g.record_ccr(22, Some(q));
                let s = g.flags.status;
                g.record_ccr(16, Some(s));
            }
        }
    }
    for (a, _) in pairs(&VALUES, &[0]) {
        g.set_vf(2, a);
        for lane in 0..4 {
            g.clear_sticky();
            g.p.emit(format!("vsqrt q, vf2{}", ['x', 'y', 'z', 'w'][lane]));
            g.p.emit("vwaitq");
            let (q, invalid, divide) = manual::sqrt(a[lane]);
            g.note("vsqrt".into(), vec![(0, 0, a[lane], 0)]);
            g.flags.divide(invalid, divide);
            g.q = q;
            g.record_ccr(22, Some(q));
            let s = g.flags.status;
            g.record_ccr(16, Some(s));
        }
    }
    // Q read by VADDq k VNOPs after VDIV: the old Q until the quotient lands six VNOPs on (§5.4.7, §5.4.1).
    for k in [0usize, 1, 5, 6, 7] {
        g.set_q(0x3F80_0000);
        g.set_vf(1, [0x4040_0000; 4]);
        g.set_vf(2, [0x4000_0000; 4]);
        g.p.emit("qmtc2 t5, vf3");
        g.p.emit("vdiv q, vf1x, vf2x");
        for _ in 0..k {
            g.p.emit("vnop");
        }
        g.p.emit("vaddq.x vf3, vf0, q");
        g.p.emit("vwaitq");
        g.flags.divide(false, false);
        let q = if k >= 6 { 0x3FC0_0000 } else { 0x3F80_0000 };
        let r = manual::add(0, q);
        g.flags.fmac(&[(0, r, None)]);
        g.q = 0x3FC0_0000;
        let mut out = SENTINEL;
        out[0] = r.0;
        g.record_vf(3, Some(out));
    }
    g
}

/// The integer unit: VIADD, VISUB, VIAND and VIOR over pairs of values written by CTC2, VIADDI over its immediates;
/// the loads and stores on VU0's data memory, written and read by the EE at B1004000h, and VU1's registers through VU0's
/// window from 4000h; the random unit; the flags read at the end, unchanged by all of them.
pub fn int() -> Program {
    int_gen().p
}

/// The generator of `int`, with its notes.
pub fn int_gen() -> Gen {
    let mut g = Gen::new("s12-int");
    let vals: [u32; 10] = [0, 1, 2, 0xFFFF, 57005, 3855, 0x7FFF, 0x8000, 0xFFFF_FFFF, 0x1234_5678];
    for (name, f) in [
        ("viadd", (|a: u16, b: u16| a.wrapping_add(b)) as fn(u16, u16) -> u16),
        ("visub", |a, b| a.wrapping_sub(b)),
        ("viand", |a, b| a & b),
        ("vior", |a, b| a | b),
    ] {
        for &a in &vals {
            for &b in &vals {
                g.p.constant("t0", a as u64);
                g.p.emit("ctc2 t0, vi1");
                g.p.constant("t0", b as u64);
                g.p.emit("ctc2 t0, vi2");
                g.p.emit(format!("{name} vi3, vi1, vi2"));
                g.record_ccr(3, Some(f(a as u16, b as u16) as u32));
            }
        }
    }
    for &a in &vals {
        for imm in [-16i32, -1, 0, 1, 7, 15] {
            g.p.constant("t0", a as u64);
            g.p.emit("ctc2 t0, vi1");
            g.p.emit(format!("viaddi vi4, vi1, {imm}"));
            g.record_ccr(4, Some((a as u16).wrapping_add(imm as u16) as u32));
        }
    }
    g.p.emit("viadd vi0, vi1, vi1");
    g.record_ccr(0, Some(0));
    // VU0's data memory: qwords 0 to 15 written by the EE, each lane its own number.
    let mem = |q: u32, l: u32| 0x1000_0000 | q << 8 | l;
    g.p.emit("lui t9, 0xb100");
    g.p.emit("ori t9, t9, 0x4000");
    for q in 0..16u32 {
        g.p.quad("t0", quad([0, 1, 2, 3].map(|l| mem(q, l))));
        g.p.emit(format!("sq t0, 0x{:x}(t9)", q * 16));
    }
    let row = |q: u32| [0, 1, 2, 3].map(|l| mem(q, l));
    let set_vi = |g: &mut Gen, n: u32, v: u32| {
        g.p.constant("t0", v as u64);
        g.p.emit(format!("ctc2 t0, vi{n}"));
    };
    for (k, dest) in [0xF, 0x8, 0x3, 0x6].into_iter().enumerate() {
        let at = 2 + k as u32;
        set_vi(&mut g, 1, at);
        g.p.emit("qmtc2 t5, vf3");
        g.p.emit(format!("vlqi{} vf3, (vi1++)", mask_text(dest)));
        let mut out = SENTINEL;
        for l in manual::lanes_of(dest) {
            out[l] = row(at)[l];
        }
        g.record_vf(3, Some(out));
        g.record_ccr(1, Some(at + 1));
        g.p.emit("qmtc2 t5, vf3");
        g.p.emit(format!("vlqd{} vf3, (--vi1)", mask_text(dest)));
        g.record_vf(3, Some(out));
        g.record_ccr(1, Some(at));
    }
    // A data address past 4 KiB and below 4000h reads the memory again (D-50).
    set_vi(&mut g, 1, 0x105);
    g.p.emit("vlqi.xyzw vf3, (vi1++)");
    g.record_vf(3, Some(row(5)));
    let src = [0x1111_1111, 0x2222_2222, 0x3333_3333, 0x4444_4444];
    g.set_vf(1, src);
    let mut image: Vec<Lanes> = (0..16).map(row).collect();
    for (k, dest) in [0xF, 0x9, 0x4, 0x2].into_iter().enumerate() {
        let at = 8 + k as u32;
        set_vi(&mut g, 2, at);
        g.p.emit(format!("vsqi{} vf1, (vi2++)", mask_text(dest)));
        g.record_ccr(2, Some(at + 1));
        g.p.emit(format!("vsqd{} vf1, (--vi2)", mask_text(dest)));
        for l in manual::lanes_of(dest) {
            image[at as usize][l] = src[l];
        }
        g.p.emit(format!("lq t2, 0x{:x}(t9)", at * 16));
        g.align();
        g.p.record_quad("t2", Some(quad(image[at as usize])));
    }
    set_vi(&mut g, 3, 0xBEEF);
    set_vi(&mut g, 1, 13);
    g.p.emit("viswr.xz vi3, (vi1)");
    image[13][0] = 0xBEEF;
    image[13][2] = 0xBEEF;
    g.p.emit(format!("lq t2, 0x{:x}(t9)", 13 * 16));
    g.align();
    g.p.record_quad("t2", Some(quad(image[13])));
    for (k, dest) in [0x8, 0x4, 0x2, 0x1].into_iter().enumerate() {
        set_vi(&mut g, 1, 14);
        g.p.emit(format!("vilwr{} vi4, (vi1)", mask_text(dest)));
        g.record_ccr(4, Some(image[14][k] & 0xFFFF));
    }
    // VU1's registers through the window: VF05 at 4050h, VI03 at 4230h, the status flag at 4300h (§5.5.3 to §5.5.5).
    g.set_vf(1, [0xA1A1_A1A1, 0xB2B2_B2B2, 0xC3C3_C3C3, 0xD4D4_D4D4]);
    set_vi(&mut g, 2, 0x405);
    g.p.emit("vsqi.xyzw vf1, (vi2++)");
    set_vi(&mut g, 2, 0x405);
    g.p.emit("qmtc2 t5, vf3");
    g.p.emit("vlqi.xyzw vf3, (vi2++)");
    g.record_vf(3, Some([0xA1A1_A1A1, 0xB2B2_B2B2, 0xC3C3_C3C3, 0xD4D4_D4D4]));
    set_vi(&mut g, 2, 0x423);
    g.p.emit("vsqi.xyzw vf1, (vi2++)");
    set_vi(&mut g, 2, 0x423);
    g.p.emit("vlqi.xyzw vf3, (vi2++)");
    g.record_vf(3, Some([0xA1A1, 0, 0, 0]));
    // The random unit (§4.3): RINIT, RGET, RXOR, RNEXT.
    let seeds = [0x0123_4567u32, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210];
    g.set_vf(1, seeds);
    let mut r = 0u32;
    for f in 0..4 {
        g.p.emit(format!("vrinit r, vf1{}", ['x', 'y', 'z', 'w'][f]));
        r = seeds[f] & 0x7F_FFFF;
        g.p.emit("qmtc2 t5, vf3");
        g.p.emit("vrget.xyzw vf3, r");
        g.record_vf(3, Some([0x3F80_0000 | r; 4]));
    }
    for f in 0..4 {
        g.p.emit(format!("vrxor r, vf1{}", ['x', 'y', 'z', 'w'][f]));
        r ^= seeds[f] & 0x7F_FFFF;
        g.p.emit("qmtc2 t5, vf3");
        g.p.emit("vrget.xw vf3, r");
        let mut out = SENTINEL;
        out[0] = 0x3F80_0000 | r;
        out[3] = 0x3F80_0000 | r;
        g.record_vf(3, Some(out));
    }
    for f in 0..4 {
        g.p.emit(format!("vrinit r, vf1{}", ['x', 'y', 'z', 'w'][f]));
        r = seeds[f] & 0x7F_FFFF;
        for _ in 0..4 {
            g.p.emit("vrnext.xyzw vf3, r");
            r = manual::next_r(r);
            g.record_vf(3, Some([0x3F80_0000 | r; 4]));
        }
    }
    g.record_flags();
    g
}

/// The transfers: QMFC2 and QMTC2 with VF00 and the GPR's 128 bits, with and without the interlock; CFC2 and CTC2 on
/// every control register, all ones written and read, then zero; LQC2 and SQC2 at offsets, VF00 loaded; BC2F, BC2T and
/// the likely forms with VU1 idle; COP2 unusable, and LQC2's and SQC2's address errors, through the handler.
pub fn transfer() -> Program {
    transfer_gen().p
}

/// The generator of `transfer`, with its notes.
pub fn transfer_gen() -> Gen {
    let mut g = Gen::new("s12-transfer");
    let v = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210u128;
    g.p.quad("t1", v);
    for (mv, mf) in [("qmtc2", "qmfc2"), ("qmtc2.i", "qmfc2.i"), ("qmtc2.ni", "qmfc2.ni")] {
        g.p.emit(format!("{mv} t1, vf7"));
        g.p.emit(format!("{mf} t2, vf7"));
        g.p.record_quad("t2", Some(v));
    }
    g.p.emit("qmtc2 t1, vf0");
    g.record_vf(0, Some(VF00));
    g.p.quad("t2", v);
    g.p.emit("qmfc2 zero, vf7");
    g.p.record_quad("zero", Some(0));
    g.p.quad("t2", u128::MAX);
    g.p.constant("t0", 0x1234_5678);
    g.p.emit("ctc2 t0, vi21");
    g.p.emit("cfc2 t2, vi21");
    g.p.record_quad("t2", Some(0xFFFF_FFFF_FFFF_FFFF_0000_0000_1234_5678));
    g.p.constant("t0", 0x8765_4321);
    g.p.emit("ctc2.i t0, vi21");
    g.p.quad("t2", u128::MAX);
    g.p.emit("cfc2.i t2, vi21");
    g.p.record_quad("t2", Some(0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_8765_4321));
    // Every control register but CMSAR1, whose write starts VU1 (S13): all ones, then zero.
    g.p.emit("addiu t0, zero, -1");
    for n in (0..31u32).filter(|&n| n != 26) {
        g.p.emit(format!("ctc2 t0, vi{n}"));
        let want = match n {
            0 => Some(0),
            1..16 => Some(0xFFFF),
            16 => {
                g.flags.write_status(0xFFFF_FFFF);
                Some(g.flags.status)
            }
            17 => Some(g.flags.mac),
            18 => Some(0xFF_FFFF),
            19 => Some(0x2E30),
            20 => Some(0x7F_FFFF),
            21 | 22 => Some(0xFFFF_FFFF),
            24 => Some(0x0C0C),
            27 => Some(0xFFFF),
            28 => {
                g.flags = Flags::default();
                Some(0)
            }
            _ => Some(0),
        };
        g.record_ccr(n, want);
    }
    for n in (0..31u32).filter(|&n| !matches!(n, 26 | 28)) {
        g.p.emit(format!("ctc2 zero, vi{n}"));
        let want = match n {
            17 => g.flags.mac,
            19 => 0x2E30,
            _ => 0,
        };
        if n == 16 {
            g.flags.write_status(0);
        }
        g.record_ccr(n, Some(want));
    }
    g.record_ccr(29, Some(0));
    // LQC2 and SQC2 at the area, offsets on both sides of the base.
    g.p.emit(format!("lui t4, 0x{:x}", (AREA + 0x100) >> 16));
    g.p.emit(format!("ori t4, t4, 0x{:x}", (AREA + 0x100) & 0xFFFF));
    let rows: [u128; 3] = [v, !v, v.rotate_left(32)];
    for (k, off) in [0i32, 16, -16].into_iter().enumerate() {
        g.p.quad("t0", rows[k]);
        g.p.emit(format!("sq t0, {off}(t4)"));
        g.p.emit(format!("lqc2 vf8, {off}(t4)"));
        g.record_vf(8, Some([rows[k] as u32, (rows[k] >> 32) as u32, (rows[k] >> 64) as u32, (rows[k] >> 96) as u32]));
        g.p.emit(format!("sqc2 vf7, {}(t4)", off + 0x40));
        g.p.emit(format!("lq t2, {}(t4)", off + 0x40));
        g.align();
        g.p.record_quad("t2", Some(v));
    }
    g.p.emit("lqc2 vf0, 0(t4)");
    g.record_vf(0, Some(VF00));
    // The branches with VU1 idle: the slot sets t2's bit, the fall-through t3's.
    for name in ["bc2f", "bc2t", "bc2fl", "bc2tl"] {
        let after = g.p.label();
        g.p.emit("ori t2, zero, 0");
        g.p.emit("ori t3, zero, 0");
        g.p.emit(format!("{name} {after}"));
        g.p.emit("ori t2, t2, 1");
        g.p.emit("ori t3, t3, 1");
        g.p.place(&after);
        let (slot, fall) = match name {
            "bc2f" | "bc2fl" => (1, 0),
            "bc2t" => (1, 1),
            _ => (0, 1),
        };
        g.p.record("t2", Some(slot));
        g.p.record("t3", Some(fall));
    }
    exceptions(&mut g);
    g
}

/// COP2 unusable with Status.CU2 clear, CE 2; LQC2's and SQC2's address errors; each through S6's handler.
fn exceptions(g: &mut Gen) {
    let p = &mut g.p;
    p.emit("di");
    let words = emusen_mips::assemble(&s6::handler(), s6::VECTOR, &emusen_mips::Dialect::R5900).expect("the handler");
    p.emit("lui t9, 0xa000");
    for (i, w) in words.iter().enumerate() {
        p.emit(format!("lui t3, 0x{:x}", w >> 16));
        p.emit(format!("ori t3, t3, 0x{:x}", w & 0xFFFF));
        p.emit(format!("sw t3, 0x{:x}(t9)", 0x180 + 4 * i));
    }
    p.emit("sync.p");
    p.emit("mtc0 zero, Status");
    p.emit("sync.p");
    const EXL: u32 = 2;
    const CU2: u32 = 1 << 30;
    p.emit(format!("lui t4, 0x{:x}", AREA >> 16));
    for text in ["qmfc2 t2, vf1", "cfc2 t2, vi1", "ctc2 zero, vi1", "lqc2 vf1, 0(t4)", "sqc2 vf1, 0(t4)", "vadd.xyzw vf1, vf1, vf1", "vnop"] {
        let at = p.here();
        p.emit(text);
        s6::taken(p, 11, at, false, 2, None, EXL);
    }
    let (at, after) = (p.here(), p.label());
    p.emit(format!("bc2t {after}"));
    s6::taken(p, 11, at, false, 2, None, EXL);
    p.emit("nop");
    p.place(&after);
    p.emit("lui t0, 0x4000");
    p.emit("mtc0 t0, Status");
    p.emit("sync.p");
    for (text, store, k) in [("lqc2 vf1, 8(t4)", false, 8u32), ("lqc2 vf1, 4(t4)", false, 4), ("sqc2 vf1, 1(t4)", true, 1), ("sqc2 vf1, 12(t4)", true, 12)] {
        let at = p.here();
        p.emit(text);
        s6::taken(p, if store { 5 } else { 4 }, at, false, 0, Some((AREA + k) as u64), EXL | CU2);
    }
}

/// Every program, in the order the referees run them.
pub fn all() -> Vec<Program> {
    all_gens().into_iter().map(|g| g.p).collect()
}

pub fn all_gens() -> Vec<Gen> {
    vec![add_gen(), mul_gen(), fused_gen(), misc_gen(), fdiv_gen(), int_gen(), transfer_gen()]
}
