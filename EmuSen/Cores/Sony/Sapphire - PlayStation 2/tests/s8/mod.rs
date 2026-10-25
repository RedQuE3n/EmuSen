//! The project's S8 programs (Sapphire_Native.md §18.5): bare EE programs over every multimedia encoding, generated as
//! assembly for `emusen-mips` in S6's form (`crate::s6::Program`), each case loading its operands from constants,
//! setting the destination to a sentinel, running the instruction and recording GPR[rd] and, where the instruction
//! touches them, HI and LO as quadwords. The operands are chosen for the edges: each lane width's list of boundary
//! values set against every rotation of itself, so that every pair of edges meets in some lane; saturation, signed and
//! unsigned boundaries, every shift amount class, the divisors that overflow, and HI and LO's halves. The expected
//! record comes from `manual`, the pages' Operation sections transcribed apart from `src/ee/mmi.rs`.

#![allow(dead_code)]

pub mod manual;

use crate::s6::Program;
use manual::{Input, Outcome};

/// The quadword every destination holds before the instruction under test, kept at `SENTINEL_AT` and read by LQ.
pub const SENTINEL: u128 = 0xF0E1_D2C3_B4A5_9687_7869_5A4B_3C2D_1E0F;
const SENTINEL_AT: u32 = 0x0030_0000;

pub const BYTES: [u8; 16] = [0x00, 0x01, 0x7F, 0x80, 0xFF, 0x81, 0x7E, 0x40, 0xC0, 0x02, 0xFE, 0x3F, 0x55, 0xAA, 0x10, 0xEF];
pub const HALVES: [u16; 8] = [0x0000, 0x0001, 0x7FFF, 0x8000, 0xFFFF, 0x8001, 0x7FFE, 0x4000];
pub const HALVES2: [u16; 8] = [0x0002, 0xFFFE, 0x3FFF, 0xC000, 0x00FF, 0xFF00, 0x1234, 0xEDCC];
pub const WORDS_A: [u32; 4] = [0x7FFF_FFFF, 0x8000_0000, 0xFFFF_FFFF, 0x0000_0001];
pub const WORDS_B: [u32; 4] = [0x0000_0000, 0x8000_0001, 0x7FFF_FFFE, 0x1234_5678];
pub const PATTERN: u128 = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210;
/// Values with no lane structure: zero, all ones, a counting pattern, alternate bits.
pub const PLAIN: [u128; 5] = [0, u128::MAX, PATTERN, 0x5555_5555_5555_5555_5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA];
/// Word values (bits 63..31 equal) for the word multiplies and divides, which leave others undefined.
pub const WORD_VALUES: [u32; 7] = [0, 1, 0x7FFF_FFFF, 0x8000_0000, 0xFFFF_FFFF, 0x1234_5678, 0x89AB_CDEF];
/// HI and LO presets, by doubleword: each word an edge of the accumulators.
pub const PRESETS: [u64; 8] = [
    0x7FFF_FFFF_8000_0000,
    0xFFFF_FFFF_0000_0000,
    0x0000_0001_7FFF_FFFF,
    0x8000_0000_FFFF_FFFF,
    0x1234_5678_9ABC_DEF0,
    0,
    u64::MAX,
    0x4000_0000_C000_0000,
];

fn rot<T: Copy>(l: &[T], k: usize) -> Vec<T> {
    (0..l.len()).map(|i| l[(i + k) % l.len()]).collect()
}
pub fn bytes(l: &[u8]) -> u128 {
    l.iter().enumerate().fold(0, |q, (i, &b)| q | (b as u128) << (8 * i))
}
pub fn halves(l: &[u16]) -> u128 {
    l.iter().enumerate().fold(0, |q, (i, &h)| q | (h as u128) << (16 * i))
}
pub fn words(l: &[u32]) -> u128 {
    l.iter().enumerate().fold(0, |q, (i, &w)| q | (w as u128) << (32 * i))
}
/// Two sign-extended words as GPR doublewords 0 and 1.
pub fn word_pair(a: u32, b: u32) -> u128 {
    (b as i32 as i64 as u64 as u128) << 64 | a as i32 as i64 as u64 as u128
}
fn hilo(i: usize) -> (u128, u128) {
    let p = |k: usize| PRESETS[k % PRESETS.len()] as u128;
    (p(3 * i + 1) << 64 | p(i), p(5 * i + 3) << 64 | p(7 * i + 2))
}

/// The operand pairs for lanes of `w` bits: the width's edge list against each of its rotations, then the plain values
/// against one another.
pub fn pairs(w: u32) -> Vec<(u128, u128)> {
    let mut v = Vec::new();
    match w {
        8 => v.extend((0..16).map(|k| (bytes(&BYTES), bytes(&rot(&BYTES, k))))),
        16 => {
            v.extend((0..8).map(|k| (halves(&HALVES), halves(&rot(&HALVES, k)))));
            v.extend((0..8).map(|k| (halves(&HALVES2), halves(&rot(&HALVES, k)))));
        }
        _ => {
            for k in 0..4 {
                v.push((words(&WORDS_A), words(&rot(&WORDS_A, k))));
                v.push((words(&WORDS_B), words(&rot(&WORDS_A, k))));
                v.push((words(&WORDS_A), words(&rot(&WORDS_B, k))));
            }
        }
    }
    for &a in &PLAIN {
        for &b in &PLAIN {
            v.push((a, b));
        }
    }
    v
}

/// Values with structure at every width, for the instructions that move lanes rather than compute in them.
pub fn shapes() -> Vec<u128> {
    let mut v = PLAIN.to_vec();
    v.extend([bytes(&BYTES), halves(&HALVES), words(&WORDS_A), words(&WORDS_B)]);
    v
}

/// The operand order each form's assembly text takes: rs is t0, rt t1, rd t2.
pub fn text(name: &str, sa: u32) -> String {
    match name {
        "psllvw" | "psrlvw" | "psravw" => format!("{name} t2, t1, t0"),
        "psllh" | "psrlh" | "psrah" | "psllw" | "psrlw" | "psraw" => format!("{name} t2, t1, {sa}"),
        "pabsh" | "pabsw" | "pcpyh" | "pexch" | "pexcw" | "pexeh" | "pexew" | "prevh" | "prot3w" | "pext5" | "ppac5" => format!("{name} t2, t1"),
        "pdivw" | "pdivuw" | "pdivbw" => format!("{name} t0, t1"),
        "pmfhi" | "pmflo" | "pmfhl.lw" | "pmfhl.uw" | "pmfhl.slw" | "pmfhl.lh" | "pmfhl.sh" => format!("{name} t2"),
        "pmthi" | "pmtlo" | "pmthl.lw" => format!("{name} t0"),
        "plzcw" => "plzcw t2, t0".into(),
        _ => format!("{name} t2, t0, t1"),
    }
}

fn begin(name: &'static str) -> Program {
    let mut p = Program::new(name);
    p.quad("t2", SENTINEL);
    p.emit(format!("lui s3, 0x{:x}", SENTINEL_AT >> 16));
    p.emit("sq t2, 0(s3)");
    p
}

/// What a case records besides GPR[rd].
#[derive(Clone, Copy, PartialEq, Eq)]
enum Hilo {
    None,
    /// HI and LO in full, read back by the S6 moves (MFHI, MFHI1, MFLO, MFLO1).
    Full,
    /// HI's and LO's words 0 and 2 alone, the figures calling words 1 and 3 undefined (PHMADH, PHMSBH).
    EvenWords,
}

/// One case: the operands, HI and LO preset by the S6 moves, the instruction, then its record.
fn case(p: &mut Program, name: &str, i: Input, record_rd: bool, hilo: Hilo) -> Outcome {
    let o = manual::mmi(name, &i);
    p.quad("t0", i.rs);
    p.quad("t1", i.rt);
    p.emit("lq t2, 0(s3)");
    for (mv, v) in [("mthi", i.hi as u64), ("mthi1", (i.hi >> 64) as u64), ("mtlo", i.lo as u64), ("mtlo1", (i.lo >> 64) as u64)] {
        p.constant("t3", v);
        p.emit(format!("{mv} t3"));
    }
    p.emit(text(name, i.sa));
    if record_rd {
        p.record_quad("t2", Some(o.rd));
    }
    if hilo != Hilo::None {
        for (mv, v) in [("mfhi", o.hi as u64), ("mfhi1", (o.hi >> 64) as u64), ("mflo", o.lo as u64), ("mflo1", (o.lo >> 64) as u64)] {
            p.emit(format!("{mv} t3"));
            if hilo == Hilo::EvenWords {
                p.emit("dsll32 t3, t3, 0");
                p.emit("dsrl32 t3, t3, 0");
                p.record("t3", Some(v as u32 as u64));
            } else {
                p.record("t3", Some(v));
            }
        }
    }
    o
}

fn input(rs: u128, rt: u128, k: usize) -> Input {
    let (hi, lo) = hilo(k);
    Input { rs, rt, rd: SENTINEL, hi, lo, sa: 0, sa_register: 0 }
}

pub const ARITHMETIC: &[(&str, u32)] = &[
    ("paddb", 8),
    ("paddh", 16),
    ("paddw", 32),
    ("psubb", 8),
    ("psubh", 16),
    ("psubw", 32),
    ("paddsb", 8),
    ("paddsh", 16),
    ("paddsw", 32),
    ("psubsb", 8),
    ("psubsh", 16),
    ("psubsw", 32),
    ("paddub", 8),
    ("padduh", 16),
    ("padduw", 32),
    ("psubub", 8),
    ("psubuh", 16),
    ("psubuw", 32),
    ("padsbh", 16),
];
pub const COMPARE: &[(&str, u32)] = &[
    ("pceqb", 8),
    ("pceqh", 16),
    ("pceqw", 32),
    ("pcgtb", 8),
    ("pcgth", 16),
    ("pcgtw", 32),
    ("pmaxh", 16),
    ("pmaxw", 32),
    ("pminh", 16),
    ("pminw", 32),
    ("pabsh", 16),
    ("pabsw", 32),
];
pub const LANE_MOVES: &[&str] =
    &["pand", "por", "pxor", "pnor", "pextlb", "pextub", "pextlh", "pextuh", "pextlw", "pextuw", "ppacb", "ppach", "ppacw", "pinth", "pinteh", "pcpyld", "pcpyud"];
pub const REARRANGE: &[&str] = &["pcpyh", "pexch", "pexcw", "pexeh", "pexew", "prevh", "prot3w", "pext5", "ppac5"];

/// Parallel add and subtract, wrapping and saturating, over every pair of edges in every lane width.
pub fn arithmetic() -> Program {
    let mut p = begin("s8-arith");
    for &(name, w) in ARITHMETIC {
        for (k, (a, b)) in pairs(w).into_iter().enumerate() {
            case(&mut p, name, input(a, b, k), true, Hilo::None);
        }
    }
    p
}

/// Compare, maximum, minimum and absolute value.
pub fn compare() -> Program {
    let mut p = begin("s8-compare");
    for &(name, w) in COMPARE {
        for (k, (a, b)) in pairs(w).into_iter().enumerate() {
            case(&mut p, name, input(a, b, k), true, Hilo::None);
        }
    }
    p
}

/// Logic, interleave, pack, copy and the rearrangements, over values with structure at every width.
pub fn lanes() -> Program {
    let mut p = begin("s8-lanes");
    let s = shapes();
    for &name in LANE_MOVES {
        for (k, &a) in s.iter().enumerate() {
            for &b in &s {
                case(&mut p, name, input(a, b, k), true, Hilo::None);
            }
        }
    }
    for &name in REARRANGE {
        for (k, &b) in s.iter().enumerate() {
            case(&mut p, name, input(PATTERN, b, k), true, Hilo::None);
        }
    }
    p
}

/// The halfword shifts by sa 0 to 15 (and PSLLH's by 16 to 31, which its page defines by sa3..0), the word shifts by
/// every class of amount, and the variable forms by amounts with bits above 4 set.
pub fn shifts() -> Program {
    let mut p = begin("s8-shift");
    let values = [halves(&HALVES), halves(&HALVES2), bytes(&BYTES), PATTERN, u128::MAX];
    for name in ["psllh", "psrlh", "psrah"] {
        let amounts: &[u32] = if name == "psllh" { &[0, 1, 7, 8, 15, 16, 17, 31] } else { &[0, 1, 7, 8, 15] };
        for (k, &v) in values.iter().enumerate() {
            for &sa in amounts {
                case(&mut p, name, Input { sa, ..input(0, v, k) }, true, Hilo::None);
            }
        }
    }
    let values = [words(&WORDS_A), words(&WORDS_B), bytes(&BYTES), PATTERN, u128::MAX];
    for name in ["psllw", "psrlw", "psraw"] {
        for (k, &v) in values.iter().enumerate() {
            for sa in [0, 1, 15, 16, 31] {
                case(&mut p, name, Input { sa, ..input(0, v, k) }, true, Hilo::None);
            }
        }
    }
    let amounts = [words(&[0, 0xDEAD_BEEF, 31, 0x1234_5678]), words(&[1, 0, 16, 0]), words(&[31, 0, 0, 0]), words(&[33, 7, 0xFFFF_FFE0, 9]), words(&[15, 0, 47, 0]), words(&[8, 0xFFFF_FFFF, 24, 0xFFFF_FFFF])];
    for name in ["psllvw", "psrlvw", "psravw"] {
        for (k, &v) in values.iter().enumerate() {
            for &s in &amounts {
                case(&mut p, name, input(s, v, k), true, Hilo::None);
            }
        }
    }
    p
}

/// The moves between the GPRs and HI and LO: HI and LO preset by the S6 moves, PMFHL's five forms over accumulators
/// at every clamp's edge, and the writes read back in full.
pub fn hilo_moves() -> Program {
    let mut p = begin("s8-hilo");
    let edges: [u128; 6] = [
        words(&[0x7FFF_FFFF, 0, 0x8000_0000, 0xFFFF_FFFF]),
        words(&[0x8000_0000, 0xFFFF_FFFF, 0x7FFF_FFFF, 0]),
        words(&[0x0000_7FFF, 0x0000_8000, 0xFFFF_8000, 0xFFFF_7FFF]),
        words(&[0xFFFF_FFFF, 0x0000_0001, 0x7FFF_FFFE, 0x8000_0001]),
        PATTERN,
        words(&[0x0000_0000, 0x8000_0000, 0x1234_5678, 0x0000_0001]),
    ];
    for name in ["pmfhi", "pmflo", "pmfhl.lw", "pmfhl.uw", "pmfhl.slw", "pmfhl.lh", "pmfhl.sh"] {
        for (k, &h) in edges.iter().enumerate() {
            for &l in &edges {
                case(&mut p, name, Input { hi: h, lo: l, ..input(0, 0, k) }, true, Hilo::None);
            }
        }
    }
    for name in ["pmthi", "pmtlo", "pmthl.lw"] {
        for (k, &v) in shapes().iter().enumerate() {
            case(&mut p, name, input(v, 0, k), false, Hilo::Full);
        }
    }
    p
}

pub const HALF_MULTIPLIES: &[&str] = &["pmulth", "pmaddh", "pmsubh", "phmadh", "phmsbh"];
pub const WORD_MULTIPLIES: &[&str] = &["pmultw", "pmultuw", "pmaddw", "pmadduw", "pmsubw"];

/// Rs and rt as word pairs: every word value meets every other in some case, each in both doublewords.
pub fn word_operands() -> Vec<(u128, u128)> {
    let n = WORD_VALUES.len();
    let mut v = Vec::new();
    for i in 0..n {
        for j in 0..n {
            v.push((word_pair(WORD_VALUES[i], WORD_VALUES[(i + 3) % n]), word_pair(WORD_VALUES[j], WORD_VALUES[(j + 5) % n])));
        }
    }
    v
}

/// The halfword and word multiplies, plain and accumulating, with GPR[rd], HI and LO recorded.
pub fn multiplies() -> Program {
    let mut p = begin("s8-multiply");
    for &name in HALF_MULTIPLIES {
        let hilo = if name.starts_with("ph") { Hilo::EvenWords } else { Hilo::Full };
        for (k, (a, b)) in pairs(16).into_iter().enumerate() {
            case(&mut p, name, input(a, b, k), true, hilo);
        }
    }
    for &name in WORD_MULTIPLIES {
        for (k, (a, b)) in word_operands().into_iter().enumerate() {
            case(&mut p, name, input(a, b, k), true, Hilo::Full);
        }
    }
    p
}

/// The divisors PDIVBW is given in GPR[rt]15..0, with other bits above them that it must ignore.
pub const HALF_DIVISORS: [u16; 7] = [1, 0xFFFF, 0x7FFF, 0x8000, 3, 0xFFFD, 0x0100];

/// The divides on defined inputs: no zero divisor, word values only, MIN by −1 included.
pub fn divides() -> Program {
    let mut p = begin("s8-divide");
    for name in ["pdivw", "pdivuw"] {
        for (k, (a, b)) in word_operands().into_iter().enumerate() {
            if b as u32 == 0 || (b >> 64) as u32 == 0 {
                continue;
            }
            case(&mut p, name, input(a, b, k), false, Hilo::Full);
        }
    }
    let dividends = [words(&WORDS_A), words(&WORDS_B), bytes(&BYTES), PATTERN, words(&[0x8000_0000, 0x7FFF_FFFF, 0xFFFF_8000, 0x0000_8000])];
    for (k, &a) in dividends.iter().enumerate() {
        for &d in &HALF_DIVISORS {
            case(&mut p, "pdivbw", input(a, PATTERN << 16 | d as u128, k), false, Hilo::Full);
        }
    }
    p
}

/// PLZCW over runs of zeros and ones of every length class, and QFSRV at every byte and halfword count MTSAB and
/// MTSAH can set, the rotation the page's note gives (rs = rt) among them.
pub fn funnel_and_count() -> Program {
    let mut p = begin("s8-misc");
    let counts: [u64; 9] = [0, u64::MAX, 0x000F_FF0F_FF0F_F00F, 1, 0x8000_0000_7FFF_FFFF, 0x4000_0000_C000_0000, 0x0000_0001_FFFF_FFFE, 0x1234_5678_89AB_CDEF, 0xFFFF_FFFF_0000_0000];
    for (k, &v) in counts.iter().enumerate() {
        case(&mut p, "plzcw", input(PATTERN << 64 | v as u128, 0, k), true, Hilo::None);
    }
    for (rs, rt) in [(PATTERN, !PATTERN), (words(&WORDS_A), words(&WORDS_B))] {
        for n in 0..16u32 {
            p.emit(format!("mtsab zero, {n}"));
            case(&mut p, "qfsrv", Input { sa_register: n, ..input(rs, rt, 0) }, true, Hilo::None);
        }
    }
    for n in 0..8u32 {
        p.emit(format!("mtsah zero, {n}"));
        case(&mut p, "qfsrv", Input { sa_register: 2 * n, ..input(PATTERN, PATTERN, 0) }, true, Hilo::None);
    }
    // The page's own example: GPR[1] = 0x000F:FF0F:FF0F:F00F gives 0x0000:000B:0000:0007.
    p.constant("at", 0x000F_FF0F_FF0F_F00F);
    p.emit("plzcw at, at");
    p.record("at", Some(0x0000_000B_0000_0007));
    p.record("zero", Some(0));
    p
}

/// D-6's reading of a word divide by zero: LO = −1 (+1 for a negative signed dividend), HI = the dividend.
pub fn divide_by_zero(n: u32, signed: bool) -> (u32, u32) {
    (if signed && (n as i32) < 0 { 1 } else { u32::MAX }, n)
}

/// Inputs the pages leave undefined, each with the interpreter's reading as the expected record (Sapphire_Native.md
/// §18.6): zero divisors, operands that are not word values, PSRLH and PSRAH by 16 or more, and the words of HI and
/// LO PHMADH and PHMSBH's figures call undefined, the last two as the console's record shows them.
pub fn undefined() -> Program {
    let mut p = begin("s8-undefined");
    let dividends = [0u32, 1, 0x8000_0000, 0x7FFF_FFFF, 0xFFFF_FFFF, 0x1234_5678];
    for name in ["pdivw", "pdivuw"] {
        for (k, &n) in dividends.iter().enumerate() {
            let a = word_pair(n, n.rotate_left(8));
            let b = word_pair(0, 7);
            let signed = name == "pdivw";
            let (q0, r0) = divide_by_zero(n, signed);
            let i = input(a, b, k);
            let nonzero = manual::mmi(name, &Input { rt: word_pair(1, 7), ..i });
            let mut o = nonzero;
            o.lo = o.lo & !(u64::MAX as u128) | q0 as i32 as i64 as u64 as u128;
            o.hi = o.hi & !(u64::MAX as u128) | r0 as i32 as i64 as u64 as u128;
            undefined_case(&mut p, name, i, o, false);
        }
    }
    for (k, &n) in dividends.iter().enumerate() {
        let a = words(&[n, n.rotate_left(4), !n, n ^ 0x8000]);
        let i = input(a, PATTERN << 16, k);
        let mut o = Outcome { rd: SENTINEL, hi: i.hi, lo: i.lo };
        for (w, x) in [n, n.rotate_left(4), !n, n ^ 0x8000].into_iter().enumerate() {
            let (q, r) = divide_by_zero(x, true);
            o.lo = o.lo & !((u32::MAX as u128) << (32 * w)) | (q as u128) << (32 * w);
            o.hi = o.hi & !((u32::MAX as u128) << (32 * w)) | (r as u128) << (32 * w);
        }
        undefined_case(&mut p, "pdivbw", i, o, false);
    }
    let junk = |w: u128| w | 0x0BAD_0000_0000_0000_5EED_0000_0000_0000;
    for &name in WORD_MULTIPLIES.iter().chain(&["pdivw", "pdivuw"]) {
        for (k, (a, b)) in word_operands().into_iter().enumerate().step_by(5) {
            if name.starts_with("pdiv") && (b as u32 == 0 || (b >> 64) as u32 == 0) {
                continue;
            }
            let i = input(junk(a), junk(b) ^ 0x0100_0000_0000_0000_0000_0000_0000_0000, k);
            undefined_case(&mut p, name, i, manual::mmi(name, &i), name.starts_with("pm"));
        }
    }
    for name in ["psrlh", "psrah"] {
        for sa in [16, 17, 23, 31] {
            let i = Input { sa, ..input(0, halves(&HALVES), 0) };
            undefined_case(&mut p, name, i, manual::mmi(name, &i), true);
        }
    }
    for &name in &["phmadh", "phmsbh"] {
        for (k, (a, b)) in pairs(16).into_iter().enumerate().step_by(3) {
            let i = input(a, b, k);
            undefined_case(&mut p, name, i, beside_horizontal(name, &i), true);
        }
    }
    p
}

/// PHMADH's and PHMSBH's words the figures call undefined: product 2j+1 beside sum j, complemented for PHMSBH, the
/// reading ps2autotests' console record of `cpu/ee_simd/muldiv` gives (Sapphire_Disputes.md D-21).
pub fn beside_horizontal(name: &str, i: &Input) -> Outcome {
    let mut o = manual::mmi(name, i);
    for j in 0..4u32 {
        let h = 32 * j + 16;
        let product = ((i.rs >> h) as u16 as i16 as i32).wrapping_mul((i.rt >> h) as u16 as i16 as i32) as u32;
        let v = if name == "phmsbh" { !product } else { product };
        let at = 32 * ((j & 2) + 1);
        let reg = if j % 2 == 0 { &mut o.lo } else { &mut o.hi };
        *reg = *reg & !((u32::MAX as u128) << at) | (v as u128) << at;
    }
    o
}

fn undefined_case(p: &mut Program, name: &str, i: Input, o: Outcome, record_rd: bool) {
    p.quad("t0", i.rs);
    p.quad("t1", i.rt);
    p.emit("lq t2, 0(s3)");
    for (mv, v) in [("mthi", i.hi as u64), ("mthi1", (i.hi >> 64) as u64), ("mtlo", i.lo as u64), ("mtlo1", (i.lo >> 64) as u64)] {
        p.constant("t3", v);
        p.emit(format!("{mv} t3"));
    }
    p.emit(text(name, i.sa));
    if record_rd {
        p.record_quad("t2", Some(o.rd));
    }
    for (mv, v) in [("mfhi", o.hi as u64), ("mfhi1", (o.hi >> 64) as u64), ("mflo", o.lo as u64), ("mflo1", (o.lo >> 64) as u64)] {
        p.emit(format!("{mv} t3"));
        p.record("t3", Some(v));
    }
}

/// Every program, in the order the referees run them.
pub fn all() -> Vec<Program> {
    vec![arithmetic(), compare(), lanes(), shifts(), hilo_moves(), multiplies(), divides(), funnel_and_count(), undefined()]
}
