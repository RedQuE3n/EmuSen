//! The project's S6 programs (Sapphire_Native.md §16.5): bare EE programs over every integer encoding, generated as
//! assembly for `emusen-mips`, each writing its record from 00200000h and the done word 5A99D00Eh at 001FFFF0h as
//! `record.s` does, with the record the manual's definitions give (`manual`). The same programs run on the interpreter
//! (tests/interpreter.rs) and, as ELFs written by examples/s6.rs, on the referees.

#![allow(dead_code)]

pub mod manual;

use manual::{Outcome, not_word};

pub const ORIGIN: u32 = 0x0010_0000;
pub const RECORD: u32 = 0x0020_0000;
pub const DONE_AT: u32 = 0x001F_FFF0;
pub const DONE: u32 = 0x5A99_D00E;
/// The scratch quadwords the memory programs use, and the copy target of the 128-bit stores.
pub const AREA: u32 = 0x0030_0000;
/// The value a destination holds before the instruction under test, so that an unwritten one shows.
pub const SENTINEL: u64 = 0xF0E1_D2C3_B4A5_9687;

/// The operands: word values (bits 63..31 equal) first, then values a word instruction may not take.
pub const WORDS: [u64; 11] = [0, 1, 2, 31, 32, 0x7FFF_FFFF, 0xFFFF_FFFF_8000_0000, u64::MAX, 0x1234_5678, 0xFFFF_FFFF_89AB_CDEF, 63];
pub const WIDE: [u64; 6] =
    [0x0123_4567_89AB_CDEF, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0xFFFF_FFFF_0000_0000, 0x0000_0000_FFFF_FFFF, 0x0000_0000_8000_0000];
pub const IMMEDIATES: [u16; 7] = [0, 1, 0x7FFF, 0x8000, 0xFFFF, 0x1234, 0x8001];
pub const SHIFTS: [u32; 5] = [0, 1, 7, 16, 31];

/// A 16-bit immediate as a signed field is written.
pub fn signed_text(imm: u16) -> String {
    let v = imm as i16;
    if v < 0 { format!("-0x{:x}", -(v as i32)) } else { format!("0x{v:x}") }
}

pub fn values() -> Vec<u64> {
    WORDS.iter().chain(WIDE.iter()).copied().collect()
}

/// One program: its lines, its constants (read through s0), and its expected record, a word or "any" each.
pub struct Program {
    pub name: &'static str,
    lines: Vec<String>,
    count: u32,
    data: Vec<u64>,
    data_base: u32,
    labels: u32,
    /// S7's form: constants and the record through kseg1, the code run from kseg0 (`kernel`).
    kseg1: bool,
    pub expected: Vec<Option<u64>>,
}

impl Program {
    pub fn new(name: &'static str) -> Program {
        let mut p = Program { name, lines: Vec::new(), count: 0, data: Vec::new(), data_base: 0, labels: 0, kseg1: false, expected: Vec::new() };
        p.emit("lui s0, 0");
        p.emit("ori s0, s0, 0");
        p.emit("lui s1, 0x20");
        p
    }

    /// A program that owns the TLB and the caches: it reads its constants and writes its record and done word
    /// through kseg1, uncached and unmapped, and runs from kseg0 once its first jump is taken.
    pub fn kernel(name: &'static str) -> Program {
        let mut p = Program::new(name);
        p.kseg1 = true;
        p.lines[2] = "lui s1, 0xa020".into();
        let target = (p.here() + 16) | 0x8000_0000;
        p.emit(format!("lui t9, 0x{:x}", target >> 16));
        p.emit(format!("ori t9, t9, 0x{:x}", target & 0xFFFF));
        p.emit("jr t9");
        p.emit("nop");
        p
    }

    pub fn emit(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
        self.count += 1;
    }

    pub fn here(&self) -> u32 {
        ORIGIN + 4 * self.count
    }

    pub fn label(&mut self) -> String {
        self.labels += 1;
        format!("l{}", self.labels)
    }

    pub fn place(&mut self, label: &str) {
        self.lines.push(format!("{label}:"));
    }

    /// `reg` = `v`, read from the constants; s0 moves on before an offset leaves the 16-bit range.
    pub fn constant(&mut self, reg: &str, v: u64) {
        let at = self.data.len() as u32 * 8;
        if at - self.data_base > 0x7FF0 {
            self.emit("daddiu s0, s0, 0x7ff0");
            self.data_base += 0x7FF0;
        }
        self.data.push(v);
        let off = at - self.data_base;
        self.emit(format!("ld {reg}, 0x{off:x}(s0)"));
    }

    /// `reg` = the 128-bit `v`, by LQ from a constant aligned to 16.
    pub fn quad(&mut self, reg: &str, v: u128) {
        if self.data.len() % 2 == 1 {
            self.data.push(0);
        }
        let at = self.data.len() as u32 * 8;
        if at - self.data_base > 0x7FE0 {
            self.emit("daddiu s0, s0, 0x7fe0");
            self.data_base += 0x7FE0;
        }
        self.data.push(v as u64);
        self.data.push((v >> 64) as u64);
        let off = at - self.data_base;
        self.emit(format!("lq {reg}, 0x{off:x}(s0)"));
    }

    /// Stores `reg`'s 64 bits at s1 and moves s1 on, expecting `v` (none: any value).
    pub fn record(&mut self, reg: &str, v: Option<u64>) {
        self.emit(format!("sd {reg}, 0(s1)"));
        self.emit("daddiu s1, s1, 8");
        self.expected.push(v);
    }

    /// The source, with the constants placed after the code and s0 set to them; the expected record as words.
    pub fn finish(mut self) -> (String, Vec<Option<u32>>) {
        self.emit("lui t0, 0x5a99");
        self.emit("ori t0, t0, 0xd00e");
        self.emit(if self.kseg1 { "lui t1, 0xa01f" } else { "lui t1, 0x1f" });
        self.emit("ori t1, t1, 0xfff0");
        self.emit("sw t0, 0(t1)");
        self.place("end");
        self.emit("beq zero, zero, end");
        self.emit("nop");
        while !self.here().is_multiple_of(16) {
            self.emit("nop");
        }
        let base = self.here() | if self.kseg1 { 0xA000_0000 } else { 0 };
        self.lines[0] = format!("lui s0, 0x{:x}", base >> 16);
        self.lines[1] = format!("ori s0, s0, 0x{:x}", base & 0xFFFF);
        let mut text = self.lines.join("\n");
        for v in &self.data {
            text.push_str(&format!("\n.word 0x{:08x}\n.word 0x{:08x}", *v as u32, (*v >> 32) as u32));
        }
        text.push('\n');
        let words = self.expected.iter().flat_map(|v| [v.map(|v| v as u32), v.map(|v| (v >> 32) as u32)]).collect();
        (text, words)
    }
}

const WORD_OPERANDS: &[&str] = &["add", "addu", "sub", "subu"];
const SHIFT_VARIABLE: &[&str] = &["sllv", "srlv", "srav"];
const THREE: &[&str] = &[
    "add", "addu", "sub", "subu", "dadd", "daddu", "dsub", "dsubu", "and", "or", "xor", "nor", "slt", "sltu", "movz", "movn", "sllv", "srlv", "srav", "dsllv",
    "dsrlv", "dsrav",
];
const WITH_IMMEDIATE: &[&str] = &["addi", "addiu", "daddi", "daddiu", "andi", "ori", "xori", "slti", "sltiu", "lui"];
const SHIFT_IMMEDIATE: &[&str] = &["sll", "srl", "sra", "dsll", "dsrl", "dsra", "dsll32", "dsrl32", "dsra32"];
pub const MULDIV: &[&str] = &["mult", "multu", "mult1", "multu1", "madd", "maddu", "madd1", "maddu1", "div", "divu", "div1", "divu1"];

fn defined_three(name: &str, a: u64, b: u64) -> bool {
    if WORD_OPERANDS.contains(&name) {
        return !not_word(a) && !not_word(b);
    }
    if SHIFT_VARIABLE.contains(&name) {
        return !not_word(b);
    }
    true
}

/// Three-register operations over every pair of operands each may take; the overflowing cases go to `exceptions`.
pub fn three() -> Program {
    let mut p = Program::new("three");
    p.emit("sync");
    p.emit("sync.p");
    p.emit("pref 0, 0(s0)");
    for &name in THREE {
        for &a in &values() {
            for &b in &values() {
                if !defined_three(name, a, b) {
                    continue;
                }
                let o = manual::integer(name, a, b, 0, 0, SENTINEL, 0, 0);
                if o.exception.is_some() {
                    continue;
                }
                p.constant("t0", a);
                p.constant("t1", b);
                p.constant("t2", SENTINEL);
                p.emit(match name {
                    "sllv" | "srlv" | "srav" | "dsllv" | "dsrlv" | "dsrav" => {
                        format!("{name} t2, t1, t0")
                    }
                    _ => format!("{name} t2, t0, t1"),
                });
                p.record("t2", o.dest);
            }
        }
    }
    p
}

/// Immediate and shift-amount forms.
pub fn immediates() -> Program {
    let mut p = Program::new("immediates");
    for &name in WITH_IMMEDIATE {
        for &a in &values() {
            if matches!(name, "addi" | "addiu") && not_word(a) {
                continue;
            }
            for &imm in &IMMEDIATES {
                let o = manual::integer(name, a, 0, imm, 0, SENTINEL, 0, 0);
                if o.exception.is_some() {
                    continue;
                }
                p.constant("t0", a);
                p.constant("t2", SENTINEL);
                p.emit(match name {
                    "lui" => format!("lui t2, 0x{imm:x}"),
                    "andi" | "ori" | "xori" => format!("{name} t2, t0, 0x{imm:x}"),
                    _ => format!("{name} t2, t0, {}", signed_text(imm)),
                });
                p.record("t2", o.dest);
            }
        }
    }
    for &name in SHIFT_IMMEDIATE {
        for &b in &values() {
            if matches!(name, "sll" | "srl" | "sra") && not_word(b) {
                continue;
            }
            for &s in &SHIFTS {
                let o = manual::integer(name, 0, b, 0, s, SENTINEL, 0, 0);
                p.constant("t1", b);
                p.emit(format!("{name} t2, t1, {s}"));
                p.record("t2", o.dest);
            }
        }
    }
    p
}

fn hilo_presets(i: usize) -> [u64; 4] {
    let v = values();
    [v[i * 3 % v.len()], v[(i * 5 + 1) % v.len()], v[(i * 7 + 2) % v.len()], v[(i * 11 + 3) % v.len()]]
}

/// The multiply, multiply-add and divide forms of both pipelines, HI and LO of both read back after each.
pub fn muldiv() -> Program {
    let mut p = Program::new("muldiv");
    let mut i = 0;
    for &name in MULDIV {
        for &a in &WORDS {
            for &b in &WORDS {
                if name.starts_with("div") && b as u32 == 0 {
                    continue;
                }
                let [h0, l0, h1, l1] = hilo_presets(i);
                i += 1;
                let (hi, lo) = ((h1 as u128) << 64 | h0 as u128, (l1 as u128) << 64 | l0 as u128);
                let o: Outcome = manual::muldiv(name, a, b, SENTINEL, hi, lo);
                for (reg, v) in [("mthi", h0), ("mtlo", l0), ("mthi1", h1), ("mtlo1", l1)] {
                    p.constant("t3", v);
                    p.emit(format!("{reg} t3"));
                }
                p.constant("t0", a);
                p.constant("t1", b);
                p.constant("t2", SENTINEL);
                p.emit(if name.starts_with("div") { format!("{name} t0, t1") } else { format!("{name} t2, t0, t1") });
                p.record("t2", o.dest);
                for (reg, v) in [("mfhi", o.hi as u64), ("mflo", o.lo as u64), ("mfhi1", (o.hi >> 64) as u64), ("mflo1", (o.lo >> 64) as u64)] {
                    p.emit(format!("{reg} t3"));
                    p.record("t3", Some(v));
                }
            }
        }
    }
    p
}

/// SA: MTSAB and MTSAH read back by MFSA, and MTSA restoring MFSA's value. The manual writes SA as a bit count and
/// calls MFSA's value opaque; both referee lineages read back the byte count, the ruling of Sapphire_Disputes.md D-9.
pub fn shift_amount() -> Program {
    let mut p = Program::new("sa");
    for &a in &values() {
        for &imm in &[0u16, 1, 5, 0xF, 0x7, 0xFFFF] {
            p.constant("t0", a);
            p.emit(format!("mtsab t0, 0x{imm:x}"));
            p.emit("mfsa t2");
            p.record("t2", Some((a ^ imm as u64) & 15));
            p.emit(format!("mtsah t0, 0x{imm:x}"));
            p.emit("mfsa t2");
            p.record("t2", Some(((a ^ imm as u64) & 7) * 2));
        }
    }
    p.emit("mtsab zero, 9");
    p.emit("mfsa t3");
    p.emit("mtsab zero, 0");
    p.emit("mtsa t3");
    p.emit("mfsa t2");
    p.record("t2", Some(9));
    p
}

const LOADS: &[&str] = &["lb", "lbu", "lh", "lhu", "lw", "lwu", "ld", "lwl", "lwr", "ldl", "ldr"];
const STORES: &[&str] = &["sb", "sh", "sw", "sd", "swl", "swr", "sdl", "sdr"];
pub const PATTERN: [u8; 16] = [0x80, 0x91, 0x22, 0xB3, 0x44, 0xD5, 0x66, 0xF7, 0x08, 0x19, 0xAA, 0x3B, 0xCC, 0x5D, 0xEE, 0x7F];
const STORED: u64 = 0x8877_6655_4433_2211;

fn pattern_into_area(p: &mut Program) {
    p.constant("t3", u64::from_le_bytes(PATTERN[..8].try_into().unwrap()));
    p.emit("sd t3, 0(t4)");
    p.constant("t3", u64::from_le_bytes(PATTERN[8..].try_into().unwrap()));
    p.emit("sd t3, 8(t4)");
}

/// Every load and store at every byte of a quadword its alignment allows, LQ and SQ at every byte (the low four bits
/// masked).
pub fn memory() -> Program {
    let mut p = Program::new("memory");
    p.emit(format!("lui t4, 0x{:x}", AREA >> 16));
    pattern_into_area(&mut p);
    for &name in LOADS {
        for k in (0..16).step_by(manual::alignment(name)) {
            p.constant("t2", SENTINEL);
            p.emit(format!("{name} t2, {k}(t4)"));
            p.record("t2", Some(manual::load(name, &PATTERN, k, SENTINEL)));
        }
    }
    for &name in STORES {
        for k in (0..16).step_by(manual::alignment(name)) {
            pattern_into_area(&mut p);
            p.constant("t1", STORED);
            p.emit(format!("{name} t1, {k}(t4)"));
            let mut m = PATTERN;
            manual::store(name, &mut m, k, STORED);
            for half in 0..2 {
                p.emit(format!("ld t3, {}(t4)", half * 8));
                p.record("t3", Some(u64::from_le_bytes(m[half * 8..half * 8 + 8].try_into().unwrap())));
            }
        }
    }
    let quad = u128::from_le_bytes(PATTERN);
    pattern_into_area(&mut p);
    for k in 0..16 {
        p.emit(format!("lq t2, {k}(t4)"));
        p.emit("sq t2, 0x40(t4)");
        p.emit("ld t3, 0x40(t4)");
        p.record("t3", Some(quad as u64));
        p.emit("ld t3, 0x48(t4)");
        p.record("t3", Some((quad >> 64) as u64));
    }
    p.quad("t5", 0x0011_2233_4455_6677_8899_AABB_CCDD_EEFF);
    for k in 0..16 {
        p.emit("sq zero, 0x20(t4)");
        p.emit(format!("sq t5, {}(t4)", 0x20 + k));
        p.emit("ld t3, 0x20(t4)");
        p.record("t3", Some(0x8899_AABB_CCDD_EEFF));
        p.emit("ld t3, 0x28(t4)");
        p.record("t3", Some(0x0011_2233_4455_6677));
    }
    p
}

const TWO: &[&str] = &["beq", "bne", "beql", "bnel"];
const ONE: &[&str] = &["blez", "bgtz", "bltz", "bgez", "blezl", "bgtzl", "bltzl", "bgezl", "bltzal", "bgezal", "bltzall", "bgezall"];
pub const BRANCH_VALUES: [u64; 6] = [0, 1, u64::MAX, 0x8000_0000_0000_0000, 0x1_0000_0000, 0x7FFF_FFFF_FFFF_FFFF];

fn likely(name: &str) -> bool {
    matches!(name, "beql" | "bnel" | "blezl" | "bgtzl" | "bltzl" | "bgezl" | "bltzall" | "bgezall")
}

/// One branch case: the delay slot counts into t3, the path taken marks t4 (2 fell through, 4 branched), ra the link.
fn branch_case(p: &mut Program, text: String, taken: bool, likely: bool, link: Option<usize>) {
    let (target, join) = (p.label(), p.label());
    p.emit("daddu t3, zero, zero");
    p.emit("daddu t4, zero, zero");
    p.emit("daddu ra, zero, zero");
    p.emit("daddu t6, zero, zero");
    let at = p.here();
    p.emit(format!("{text}{target}"));
    p.emit("addiu t3, t3, 1");
    p.emit(format!("beq zero, zero, {join}"));
    p.emit("addiu t4, t4, 2");
    p.place(&target);
    p.emit("addiu t4, t4, 4");
    p.place(&join);
    p.record("t3", Some(if taken || !likely { 1 } else { 0 }));
    p.record("t4", Some(if taken { 4 } else { 2 }));
    p.record("ra", Some(if link == Some(31) { at as u64 + 8 } else { 0 }));
    p.record("t6", Some(if link == Some(14) { at as u64 + 8 } else { 0 }));
}

/// Every branch over operands whose 64-bit and 32-bit readings differ, and the jumps; then a jump into kseg0 and the
/// links made there, which the manual zero-extends.
pub fn branches() -> Program {
    let mut p = Program::new("branches");
    for &name in TWO {
        for &a in &BRANCH_VALUES {
            for &b in &BRANCH_VALUES {
                p.constant("t0", a);
                p.constant("t1", b);
                branch_case(&mut p, format!("{name} t0, t1, "), manual::branch(name, a, b), likely(name), None);
            }
        }
    }
    for &name in ONE {
        for &a in &BRANCH_VALUES {
            p.constant("t0", a);
            let link = name.contains("al").then_some(31);
            branch_case(&mut p, format!("{name} t0, "), manual::branch(name, a, 0), likely(name), link);
        }
    }
    branch_case(&mut p, "j ".into(), true, false, None);
    branch_case(&mut p, "jal ".into(), true, false, Some(31));
    for (text, link) in [("jr t5", None), ("jalr t5", Some(31)), ("jalr t6, t5", Some(14))] {
        let (target, join) = (p.label(), p.label());
        p.emit("daddu t3, zero, zero");
        p.emit("daddu t4, zero, zero");
        p.emit("daddu ra, zero, zero");
        p.emit("daddu t6, zero, zero");
        let address = p.here() + 4 * 4;
        let dest = address + 4 * 4;
        p.emit(format!("lui t5, 0x{:x}", dest >> 16));
        p.emit(format!("ori t5, t5, 0x{:x}", dest & 0xFFFF));
        p.emit("nop");
        p.emit("nop");
        assert_eq!(p.here(), address);
        p.emit(text);
        p.emit("addiu t3, t3, 1");
        p.emit(format!("beq zero, zero, {join}"));
        p.emit("addiu t4, t4, 2");
        assert_eq!(p.here(), dest);
        p.place(&target);
        p.emit("addiu t4, t4, 4");
        p.place(&join);
        p.record("t3", Some(1));
        p.record("t4", Some(4));
        p.record("ra", Some(if link == Some(31) { address as u64 + 8 } else { 0 }));
        p.record("t6", Some(if link == Some(14) { address as u64 + 8 } else { 0 }));
    }
    // Into kseg0 (the same RAM), a link made there, and back.
    let k0 = |a: u32| a | 0x8000_0000;
    let entry = p.here() + 4 * 4;
    p.emit(format!("lui t5, 0x{:x}", k0(entry) >> 16));
    p.emit(format!("ori t5, t5, 0x{:x}", k0(entry) & 0xFFFF));
    p.emit("jr t5");
    p.emit("nop");
    assert_eq!(p.here(), entry);
    let (sub, back) = (p.label(), p.label());
    let jal_at = k0(p.here());
    p.emit(format!("bgezal zero, {sub}"));
    p.emit("nop");
    p.emit(format!("beq zero, zero, {back}"));
    p.emit("nop");
    p.place(&sub);
    p.emit("daddu t7, ra, zero");
    p.emit("jalr t6, t7");
    p.emit("nop");
    p.place(&back);
    let jalr_at = k0(p.here() - 4 * 2);
    let home = p.here() + 4 * 4;
    p.emit(format!("lui t5, 0x{:x}", home >> 16));
    p.emit(format!("ori t5, t5, 0x{:x}", home & 0xFFFF));
    p.emit("jr t5");
    p.emit("nop");
    assert_eq!(p.here(), home);
    p.record("t7", Some(jal_at as u64 + 8));
    p.record("t6", Some(jalr_at as u64 + 8));
    p
}

/// The exception handler at 80000180h: it records Cause's BD and ExcCode, its CE, EPC, BadVAddr and Status, and
/// returns past the instruction, or past the branch and its slot when Cause.BD is set.
pub fn handler() -> String {
    [
        "mfc0 k0, Cause",
        "lui k1, 0x8000",
        "ori k1, k1, 0x7c",
        "and k1, k0, k1",
        "sd k1, 0(s1)",
        "srl k0, k0, 28",
        "andi k0, k0, 3",
        "sd k0, 8(s1)",
        "mfc0 k0, EPC",
        "sd k0, 16(s1)",
        "mfc0 k0, BadVAddr",
        "sd k0, 24(s1)",
        "mfc0 k0, Status",
        "sd k0, 32(s1)",
        "daddiu s1, s1, 40",
        "mfc0 k0, Cause",
        "mfc0 k1, EPC",
        "bltz k0, bd",
        "nop",
        "addiu k1, k1, 4",
        "beq zero, zero, out",
        "nop",
        "bd: addiu k1, k1, 8",
        "out: mtc0 k1, EPC",
        "sync.p",
        "eret",
    ]
    .join("\n")
}

pub const VECTOR: u32 = 0x8000_0180;

/// What the handler records for an exception of `code` from `epc`; CE is defined for the unusable exception alone.
fn taken(p: &mut Program, code: u32, epc: u32, bd: bool, ce: u32, badvaddr: Option<u64>, status: u32) {
    let cause = code << 2 | (bd as u32) << 31;
    p.expected.push(Some(cause as i32 as i64 as u64));
    p.expected.push(if code == 11 { Some(ce as u64) } else { None });
    p.expected.push(Some(epc as i32 as i64 as u64));
    p.expected.push(badvaddr);
    p.expected.push(Some(status as i32 as i64 as u64));
}

/// The exception programs, one a class, so that a referee that takes one class and not another reads clearly.
pub const EXCEPTION_CLASSES: [&str; 9] =
    ["exc-address", "exc-overflow", "exc-trap", "exc-call", "exc-reserved", "exc-unusable", "exc-delay", "exc-eidi", "exc-user"];

/// The exceptions an integer instruction raises, each from its own instruction, through the handler.
pub fn exceptions(class: &'static str) -> Program {
    let mut p = Program::new(class);
    let on = |c: &str| c == class;
    p.emit("di");
    p.emit("mtc0 zero, Status");
    p.emit("sync.p");
    let words = emusen_mips::assemble(&handler(), VECTOR, &emusen_mips::Dialect::R5900).expect("the handler");
    p.emit("lui t9, 0xa000");
    for (i, w) in words.iter().enumerate() {
        p.emit(format!("lui t3, 0x{:x}", w >> 16));
        p.emit(format!("ori t3, t3, 0x{:x}", w & 0xFFFF));
        p.emit(format!("sw t3, 0x{:x}(t9)", 0x180 + 4 * i));
    }
    p.emit("sync.p");
    let exl = 2;
    let mut bad: Option<u64> = None;
    p.emit(format!("lui t4, 0x{:x}", AREA >> 16));
    for (name, k, store) in
        [("lh", 1, false), ("lhu", 3, false), ("lw", 2, false), ("lwu", 1, false), ("ld", 4, false), ("sh", 1, true), ("sw", 2, true), ("sd", 12, true)]
    {
        if !on("exc-address") {
            break;
        }
        p.constant("t2", SENTINEL);
        let at = p.here();
        p.emit(format!("{name} t2, {k}(t4)"));
        bad = Some((AREA + k) as u64);
        taken(&mut p, if store { 5 } else { 4 }, at, false, 0, bad, exl);
        p.record("t2", Some(SENTINEL));
    }
    // Overflow: the destination keeps its value.
    let overflowing: &[(&str, u64, u64)] = &[
        ("add", 0x7FFF_FFFF, 1),
        ("add", 0xFFFF_FFFF_8000_0000, u64::MAX),
        ("sub", 0xFFFF_FFFF_8000_0000, 1),
        ("sub", 0x7FFF_FFFF, u64::MAX),
        ("dadd", 0x7FFF_FFFF_FFFF_FFFF, 1),
        ("dadd", 0x8000_0000_0000_0000, u64::MAX),
        ("dsub", 0x8000_0000_0000_0000, 1),
        ("dsub", 0x7FFF_FFFF_FFFF_FFFF, u64::MAX),
    ];
    for &(name, a, b) in overflowing.iter().filter(|_| on("exc-overflow")) {
        assert_eq!(manual::integer(name, a, b, 0, 0, 0, 0, 0).exception, Some(manual::OVERFLOW));
        p.constant("t0", a);
        p.constant("t1", b);
        p.constant("t2", SENTINEL);
        let at = p.here();
        p.emit(format!("{name} t2, t0, t1"));
        taken(&mut p, 12, at, false, 0, bad, exl);
        p.record("t2", Some(SENTINEL));
    }
    let immediate: &[(&str, u64, u16)] =
        &[("addi", 0x7FFF_FFFF, 1), ("addi", 0xFFFF_FFFF_8000_0000, 0xFFFF), ("daddi", 0x7FFF_FFFF_FFFF_FFFF, 1), ("daddi", 0x8000_0000_0000_0000, 0x8000)];
    for &(name, a, imm) in immediate.iter().filter(|_| on("exc-overflow")) {
        assert_eq!(manual::integer(name, a, 0, imm, 0, 0, 0, 0).exception, Some(manual::OVERFLOW));
        p.constant("t0", a);
        p.constant("t2", SENTINEL);
        let at = p.here();
        p.emit(format!("{name} t2, t0, {}", signed_text(imm)));
        taken(&mut p, 12, at, false, 0, bad, exl);
        p.record("t2", Some(SENTINEL));
    }
    // Traps, true and false, over operands whose signed and unsigned readings differ.
    let operands = [0u64, 1, u64::MAX, 0x8000_0000_0000_0000];
    for name in ["tge", "tgeu", "tlt", "tltu", "teq", "tne"].into_iter().filter(|_| on("exc-trap")) {
        for &a in &operands {
            for &b in &operands {
                p.constant("t0", a);
                p.constant("t1", b);
                let at = p.here();
                p.emit(format!("{name} t0, t1"));
                if manual::trap(name, a, b) {
                    taken(&mut p, 13, at, false, 0, bad, exl);
                }
                p.emit("addiu t2, zero, 0x77");
                p.record("t2", Some(0x77));
            }
        }
    }
    for name in ["tgei", "tgeiu", "tlti", "tltiu", "teqi", "tnei"].into_iter().filter(|_| on("exc-trap")) {
        for &a in &operands {
            for &imm in &[0u16, 1, 0xFFFF, 0x8000] {
                p.constant("t0", a);
                let at = p.here();
                p.emit(format!("{name} t0, {}", signed_text(imm)));
                if manual::trap(name, a, manual::sign_extend(imm as u128, 16)) {
                    taken(&mut p, 13, at, false, 0, bad, exl);
                }
                p.emit("addiu t2, zero, 0x66");
                p.record("t2", Some(0x66));
            }
        }
    }
    // SYSCALL and BREAK, with and without a code; reserved encodings; coprocessors 1 and 2 unusable.
    for (text, code) in [("syscall", 8), ("syscall 0x12345", 8), ("break", 9), ("break 0x3ff", 9)].into_iter().filter(|_| on("exc-call")) {
        let at = p.here();
        p.emit(text);
        taken(&mut p, code, at, false, 0, bad, exl);
    }
    for word in [0x0000_0001u32, 0x7000_0002, 0xC000_0000, 0xD000_0000, 0xE000_0000, 0x0000_001C, 0x0000_001E].into_iter().filter(|_| on("exc-reserved")) {
        let at = p.here();
        p.emit(format!(".word 0x{word:08x}"));
        taken(&mut p, 10, at, false, 0, bad, exl);
    }
    for (text, ce) in [("mfc1 t2, f0", 1), ("qmfc2 t2, vf0", 2)].into_iter().filter(|_| on("exc-unusable")) {
        let at = p.here();
        p.emit(text);
        taken(&mut p, 11, at, false, ce, bad, exl);
    }
    // An exception in a delay slot: EPC is the branch's, BD set; the branch's target is past its slot.
    for branch in ["beq zero, zero, ", "bne zero, zero, ", "beql zero, zero, ", "bgezal zero, "].into_iter().filter(|_| on("exc-delay")) {
        let past = p.label();
        p.constant("t0", 0x7FFF_FFFF);
        p.constant("t1", 1);
        let at = p.here();
        p.emit(format!("{branch}{past}"));
        p.emit("add t2, t0, t1");
        p.place(&past);
        taken(&mut p, 12, at, true, 0, bad, exl);
    }
    if on("exc-eidi") {
        p.emit("ei");
        p.emit("mfc0 t2, Status");
        p.record("t2", Some(0x1_0000));
        p.emit("di");
        p.emit("mfc0 t2, Status");
        p.record("t2", Some(0));
    }
    if !on("exc-user") {
        return p;
    }
    // User mode: kernel addresses refused, COP0 unusable; the done word is in kuseg.
    p.emit("addiu t3, zero, 0x10");
    p.emit("mtc0 t3, Status");
    p.emit("sync.p");
    let user = 0x10 | exl;
    for (text, code, addr) in [("lw t2, 0(t5)", 4, 0x8000_0000u32), ("sw t2, 0(t6)", 5, 0xA000_0000)] {
        p.emit(format!("lui t5, 0x{:x}", 0x8000));
        p.emit(format!("lui t6, 0x{:x}", 0xA000));
        let at = p.here();
        p.emit(text);
        bad = Some(addr as i32 as i64 as u64);
        taken(&mut p, code, at, false, 0, bad, user);
    }
    let at = p.here();
    p.emit("mfc0 t2, Status");
    taken(&mut p, 11, at, false, 0, bad, user);
    p
}

/// A 64-bit or narrower instruction leaves bits 127..64 of its destination as they were; LQ writes all 128.
pub fn upper() -> Program {
    let mut p = Program::new("upper");
    let pattern: u128 = 0xA5A5_5A5A_0F0F_F0F0_1122_3344_5566_7788;
    p.emit(format!("lui t4, 0x{:x}", AREA >> 16));
    pattern_into_area(&mut p);
    p.constant("t0", 0x1234_5678);
    p.constant("t1", 3);
    let cases: &[(&str, u64)] = &[
        ("addu t2, t0, t1", 0x1234_567B),
        ("daddu t2, t0, t1", 0x1234_567B),
        ("lui t2, 0x8000", 0xFFFF_FFFF_8000_0000),
        ("sll t2, t1, 4", 0x30),
        ("dsll32 t2, t1, 0", 0x3_0000_0000),
        ("lw t2, 0(t4)", u32::from_le_bytes(PATTERN[..4].try_into().unwrap()) as i32 as i64 as u64),
        ("ld t2, 8(t4)", u64::from_le_bytes(PATTERN[8..].try_into().unwrap())),
        ("lb t2, 0(t4)", 0xFFFF_FFFF_FFFF_FF80),
        ("mult t2, t0, t1", 0x369D_0368),
        ("movn t2, t0, t1", 0x1234_5678),
        ("lwr t2, 1(t4)", 0x0000_0000_0000_0000),
    ];
    for &(text, low) in cases {
        p.quad("t2", pattern);
        p.emit(text);
        let low = if text.starts_with("lwr") { manual::load("lwr", &PATTERN, 1, pattern as u64) } else { low };
        p.emit("sq t2, 0x40(t4)");
        p.emit("ld t3, 0x40(t4)");
        p.record("t3", Some(low));
        p.emit("ld t3, 0x48(t4)");
        p.record("t3", Some((pattern >> 64) as u64));
    }
    p
}

/// Inputs the manual leaves undefined: a zero divisor, word instructions on values that are not words, MTSA of a
/// value MFSA did not give. Expected is the interpreter's reading (Sapphire_Disputes.md D-6); the referees decide.
pub fn undefined() -> Program {
    let mut p = Program::new("undefined");
    for name in ["div", "divu", "div1", "divu1"] {
        for &a in &[0u64, 1, u64::MAX, 0x7FFF_FFFF, 0xFFFF_FFFF_8000_0000, 5, 0xFFFF_FFFF_FFFF_FFFB] {
            let one = name.ends_with('1');
            let (q, r): (u32, u32) = if name.starts_with("divu") { (u32::MAX, a as u32) } else { (if (a as i32) < 0 { 1 } else { u32::MAX }, a as u32) };
            p.constant("t0", a);
            p.emit(format!("{name} t0, zero"));
            p.emit(if one { "mflo1 t2" } else { "mflo t2" });
            p.record("t2", Some(q as i32 as i64 as u64));
            p.emit(if one { "mfhi1 t2" } else { "mfhi t2" });
            p.record("t2", Some(r as i32 as i64 as u64));
        }
    }
    let low = |x: u64| x as u32;
    for &(a, b) in &[(0x0000_0000_8000_0000u64, 1u64), (0x1_0000_0001, 1), (0x0000_0000_FFFF_FFFF, 1), (0x8000_0000_0000_0000, 0x8000_0000_0000_0000)] {
        p.constant("t0", a);
        p.constant("t1", b);
        p.emit("addu t2, t0, t1");
        p.record("t2", Some(low(a).wrapping_add(low(b)) as i32 as i64 as u64));
        p.emit("subu t2, t0, t1");
        p.record("t2", Some(low(a).wrapping_sub(low(b)) as i32 as i64 as u64));
        p.emit("sll t2, t0, 0");
        p.record("t2", Some(low(a) as i32 as i64 as u64));
        p.emit("sra t2, t0, 4");
        p.record("t2", Some((low(a) as i32 >> 4) as i64 as u64));
        p.emit("srlv t2, t0, t1");
        p.record("t2", Some((low(a) >> (b & 31)) as i32 as i64 as u64));
        p.emit("mult t2, t0, t1");
        let prod = (low(a) as i32 as i64).wrapping_mul(low(b) as i32 as i64) as u64;
        p.record("t2", Some(prod as u32 as i32 as i64 as u64));
        p.emit("mfhi t3");
        p.record("t3", Some((prod >> 32) as u32 as i32 as i64 as u64));
    }
    for v in [0x1234_5678_9ABC_DEF0u64, 5, 0x1F, 0x13, 0xFF, 0x10, 0x7F, 0x8F] {
        p.constant("t0", v);
        p.emit("mtsa t0");
        p.emit("mfsa t2");
        p.record("t2", Some(v & 15));
    }
    p
}

/// Every program, in the order the referees run them.
pub fn all() -> Vec<Program> {
    let mut all = vec![three(), immediates(), muldiv(), shift_amount(), memory(), branches()];
    all.extend(EXCEPTION_CLASSES.iter().map(|c| exceptions(c)));
    all.extend([upper(), undefined()]);
    all
}

/// The mnemonics the programs execute, for the gate's count against the interpreter's table.
pub fn covered(source: &str) -> std::collections::BTreeSet<String> {
    let words = emusen_mips::assemble(source, ORIGIN, &emusen_mips::Dialect::R5900).expect("assembles");
    words.iter().filter_map(|&w| emusen_mips::r5900::operation(w)).map(|o| o.mnemonic()).collect()
}
