//! The project's S21 programs (Sapphire_Native.md §22.8): bare IOP programs on its memory map, I-cache and scratchpad,
//! its INTC, its DMAC and its six timers, generated as R3000A assembly for `emusen-mips`. Each runs from IOP RAM at
//! 00100000h, writes its record of 32-bit words from 00180000h and the done word 5A99D00Eh at 0017FFF0h. On Sapphire
//! the IOP is entered at the program; for a referee, `wrap` puts it inside an EE program that copies it into IOP RAM
//! through the EE's window at 1C000000h, points the IOP's exception vector at it, waits for its done word and copies
//! its record into EE RAM. Every expected word is derived here from ps2tek's IOP chapters and psx-spx, written apart
//! from `src/iop/`; where the documents give no value the word is recorded ungraded, or graded by the range they give.

#![allow(dead_code)]

/// Where the IOP program sits, its record and its done word, as IOP physical addresses.
pub const ORIGIN: u32 = 0x0010_0000;
pub const RECORD: u32 = 0x0018_0000;
pub const DONE_AT: u32 = 0x0017_FFF0;
pub const DONE: u32 = 0x5A99_D00E;
/// The handler's words: the entry flag, the resume address, the log's pointer, and the log of 16-byte entries
/// (Cause, EPC, BadVaddr, SR).
pub const FLAG: u32 = 0x0017_0000;
pub const RESUME: u32 = 0x0017_0004;
pub const LOGP: u32 = 0x0017_0008;
pub const LOG: u32 = 0x0017_0010;
/// Scratch memory the programs use, past the record.
pub const AREA: u32 = 0x001A_0000;
pub const K0: u32 = 0x8000_0000;
pub const K1: u32 = 0xA000_0000;
/// The EE program `wrap` writes: its origin, and where it keeps the IOP program's words.
pub const EE_ORIGIN: u32 = 0x0010_0000;
pub const BLOB: u32 = 0x0011_0000;

/// The IOP's registers the programs reach (ps2tek's I/O map), through kseg1.
pub mod reg {
    pub const I_STAT: u32 = 0xBF80_1070;
    pub const I_MASK: u32 = 0xBF80_1074;
    pub const I_CTRL: u32 = 0xBF80_1078;
    pub const DPCR: u32 = 0xBF80_10F0;
    pub const DICR: u32 = 0xBF80_10F4;
    pub const DPCR2: u32 = 0xBF80_1570;
    pub const DICR2: u32 = 0xBF80_1574;
    pub const DMACEN: u32 = 0xBF80_1578;
    pub const DMACINTEN: u32 = 0xBF80_157C;
    pub const BCC: u32 = 0xFFFE_0130;
    pub const fn ch(n: u32) -> u32 {
        if n < 7 { 0xBF80_1080 + 0x10 * n } else { 0xBF80_1500 + 0x10 * (n - 7) }
    }
    pub const fn madr(n: u32) -> u32 {
        ch(n)
    }
    pub const fn bcr(n: u32) -> u32 {
        ch(n) + 4
    }
    pub const fn chcr(n: u32) -> u32 {
        ch(n) + 8
    }
    pub const fn tadr(n: u32) -> u32 {
        ch(n) + 12
    }
    pub const fn count(n: u32) -> u32 {
        if n < 3 { 0xBF80_1100 + 0x10 * n } else { 0xBF80_1480 + 0x10 * (n - 3) }
    }
    pub const fn mode(n: u32) -> u32 {
        count(n) + 4
    }
    pub const fn target(n: u32) -> u32 {
        count(n) + 8
    }
}

use reg::*;

/// Timer mode bits (ps2tek "IOP Timers", psx-spx "Timers").
pub mod tm {
    pub const GATE: u32 = 1;
    pub const ZRET: u32 = 1 << 3;
    pub const CMP: u32 = 1 << 4;
    pub const OVF: u32 = 1 << 5;
    pub const REPT: u32 = 1 << 6;
    pub const LEVL: u32 = 1 << 7;
    pub const EXTC: u32 = 1 << 8;
    pub const PSCL2: u32 = 1 << 9;
    pub const INTF: u32 = 1 << 10;
    pub const EQUF: u32 = 1 << 11;
    pub const OVFF: u32 = 1 << 12;
    pub const fn gatm(m: u32) -> u32 {
        m << 1
    }
    pub const fn pscl45(m: u32) -> u32 {
        m << 13
    }
}

/// The interrupt sources by I_STAT bit (ps2tek "IOP Interrupts").
pub mod irq {
    pub const VBLANK_START: u32 = 1;
    pub const DMA: u32 = 1 << 3;
    pub const TIMER2: u32 = 1 << 6;
    pub const VBLANK_END: u32 = 1 << 11;
    pub const TIMER4: u32 = 1 << 15;
    pub const TIMER5: u32 = 1 << 16;
    pub const fn timer(n: u32) -> u32 {
        [1 << 4, 1 << 5, 1 << 6, 1 << 14, 1 << 15, 1 << 16][n as usize]
    }
}

/// NTSC's field as ps2tek gives it ("EE Timers": 262 scanlines a frame, 22 of them V-blank) and its length in IOP
/// cycles at 9,370 bus cycles a line, four bus cycles an IOP cycle.
pub const NTSC_LINES: u32 = 262;
pub const NTSC_FIELD: u32 = 262 * 9370 / 4;

/// One IOP program: its lines and its expected record, a word or "any" each.
pub struct Program {
    pub name: &'static str,
    lines: Vec<String>,
    count: u32,
    labels: u32,
    /// Run from kseg1 throughout, so that the program's own code never enters the I-cache.
    uncached: bool,
    /// The exceptions the documents expect so far, the index of the next log entry.
    pub exceptions: u32,
    pub expected: Vec<Option<u32>>,
}

impl Program {
    /// A program entered at `ORIGIN` through kseg1. A first entry sets the flag, the log's pointer, SR to 0 and s1 to
    /// the record, and continues from kseg0; a later entry, through the exception vector, is the handler, which logs
    /// Cause, EPC, BadVaddr and SR and resumes at the address in `RESUME` with interrupts disabled.
    pub fn new(name: &'static str) -> Program {
        Program::with(name, false)
    }

    /// The same, run from kseg1 throughout.
    pub fn uncached(name: &'static str) -> Program {
        Program::with(name, true)
    }

    fn with(name: &'static str, uncached: bool) -> Program {
        let mut p = Program { name, lines: Vec::new(), count: 0, labels: 0, uncached, exceptions: 0, expected: Vec::new() };
        p.emit(format!("lui k0, 0x{:x}", (K1 | FLAG) >> 16));
        p.emit("lw k1, 0(k0)");
        p.emit("nop");
        p.emit("bne k1, zero, handler");
        p.emit("nop");
        p.emit("ori k1, zero, 1");
        p.emit("sw k1, 0(k0)");
        p.li("k1", K1 | LOG);
        p.emit(format!("sw k1, {}(k0)", LOGP - FLAG));
        p.emit("mtc0 zero, SR");
        p.emit(format!("lui s1, 0x{:x}", (K1 | RECORD) >> 16));
        if !uncached {
            let target = p.here() + 16;
            p.li("t9", K0 | target);
            p.emit("jr t9");
            p.emit("nop");
        }
        p.quiesce();
        p
    }

    pub fn emit(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
        self.count += 1;
    }

    /// The physical address of the next instruction.
    pub fn here(&self) -> u32 {
        ORIGIN + 4 * self.count
    }

    /// The address the next instruction runs at.
    pub fn pc(&self) -> u32 {
        self.here() | if self.uncached { K1 } else { K0 }
    }

    pub fn label(&mut self) -> String {
        self.labels += 1;
        format!("l{}", self.labels)
    }

    pub fn place(&mut self, label: &str) {
        self.lines.push(format!("{label}:"));
    }

    pub fn li(&mut self, reg: &str, v: u32) {
        self.emit(format!("lui {reg}, 0x{:x}", v >> 16));
        self.emit(format!("ori {reg}, {reg}, 0x{:x}", v & 0xFFFF));
    }

    /// Stores `reg` at s1 and moves s1 on, expecting `v` (none: any value).
    pub fn record(&mut self, reg: &str, v: Option<u32>) {
        self.emit(format!("sw {reg}, 0(s1)"));
        self.emit("addiu s1, s1, 4");
        self.expected.push(v);
    }

    /// Records whether `reg` is `v` (1) or not (0), expecting 1.
    pub fn record_is(&mut self, reg: &str, v: u32) {
        self.li("t7", v);
        self.emit(format!("xor t7, t7, {reg}"));
        self.emit("sltiu t7, t7, 1");
        self.record("t7", Some(1));
    }

    /// Records whether `lo <= reg <= hi` (unsigned), expecting 1.
    pub fn record_within(&mut self, reg: &str, lo: u32, hi: u32) {
        self.li("t7", lo);
        self.emit(format!("sltu t7, {reg}, t7"));
        self.li("t8", hi);
        self.emit(format!("sltu t8, t8, {reg}"));
        self.emit("or t7, t7, t8");
        self.emit("xori t7, t7, 1");
        self.record("t7", Some(1));
    }

    /// The word at `address` written with `v`.
    pub fn poke(&mut self, address: u32, v: u32) {
        self.li("t6", address);
        self.li("t7", v);
        self.emit("sw t7, 0(t6)");
    }

    /// The word at `address` read into t2.
    pub fn peek(&mut self, address: u32) {
        self.li("t6", address);
        self.emit("lw t2, 0(t6)");
        self.emit("nop");
    }

    /// Records the word at `address`.
    pub fn rec(&mut self, address: u32, want: Option<u32>) {
        self.peek(address);
        self.record("t2", want);
    }

    /// Records the word at `address` ANDed with `mask`.
    pub fn rec_masked(&mut self, address: u32, mask: u32, want: u32) {
        self.peek(address);
        self.li("t7", mask);
        self.emit("and t2, t2, t7");
        self.record("t2", Some(want));
    }

    /// Waits, at most `limit` passes, until the word at `address` ANDed with `mask` is non-zero (`set`) or zero; the
    /// passes left are in t5.
    pub fn wait(&mut self, address: u32, mask: u32, set: bool, limit: u32) {
        let l = self.label();
        let out = self.label();
        self.li("t6", address);
        self.li("t4", mask);
        self.li("t5", limit);
        self.place(&l);
        self.emit("lw t2, 0(t6)");
        self.emit("nop");
        self.emit("and t2, t2, t4");
        self.emit(format!("{} t2, zero, {out}", if set { "bne" } else { "beq" }));
        self.emit("nop");
        self.emit("addiu t5, t5, -1");
        self.emit(format!("bne t5, zero, {l}"));
        self.emit("nop");
        self.place(&out);
    }

    /// A delay of `n` passes of a two-instruction loop.
    pub fn delay(&mut self, n: u32) {
        let l = self.label();
        self.li("t5", n);
        self.place(&l);
        self.emit("addiu t5, t5, -1");
        self.emit(format!("bne t5, zero, {l}"));
        self.emit("nop");
    }

    /// The units put in a known state: interrupts off and acknowledged, every DMA channel stopped with its flags and
    /// masks clear and the DMAC enabled, every timer free-running with no interrupt, the cache control at the
    /// PlayStation kernel's value; whatever a BIOS left behind on a referee is undone.
    pub fn quiesce(&mut self) {
        self.poke(I_CTRL, 0);
        self.poke(I_MASK, 0);
        for n in 0..13 {
            self.poke(chcr(n), 0);
        }
        self.poke(DICR, 0x7F00_0000);
        self.poke(DICR2, 0x3F00_0000);
        self.poke(DMACEN, 1);
        self.poke(DMACINTEN, 1);
        for n in 0..6 {
            self.poke(mode(n), 0);
            self.poke(target(n), 0);
        }
        self.poke(BCC, 0x0001_E988);
        self.poke(I_STAT, 0);
    }

    /// `body` emitted with `RESUME` pointing past it, so that an exception inside it resumes there.
    pub fn guarded(&mut self, body: impl FnOnce(&mut Program)) {
        let out = self.label();
        let at = self.count;
        self.li("t6", K1 | RESUME);
        self.li("t7", 0);
        let fix = self.lines.len() - 1;
        self.emit("sw t7, 0(t6)");
        body(self);
        let resume = self.pc();
        self.place(&out);
        self.lines[fix] = format!("ori t7, t7, 0x{:x}", resume & 0xFFFF);
        self.lines[fix - 1] = format!("lui t7, 0x{:x}", resume >> 16);
        let _ = at;
    }

    /// Records the exceptions taken so far (expecting the count the documents give, `expected` more than before), and
    /// the last entry's ExcCode and EPC when one is expected.
    pub fn rec_log(&mut self, expected: u32, code: Option<u32>, epc: Option<u32>) {
        self.exceptions += expected;
        self.peek(K1 | LOGP);
        self.li("t7", K1 | LOG);
        self.emit("subu t3, t2, t7");
        self.emit("srl t3, t3, 4");
        self.record("t3", Some(self.exceptions));
        if expected > 0 {
            self.emit("lw t3, -16(t2)");
            self.emit("nop");
            self.emit("andi t3, t3, 0x7c");
            self.record("t3", code.map(|c| c << 2));
            self.emit("lw t3, -12(t2)");
            self.emit("nop");
            self.record("t3", epc);
        }
    }

    /// Timer `n`'s count into t2.
    pub fn read_count(&mut self, n: u32) {
        self.peek(count(n));
    }

    /// The source and the expected record.
    pub fn finish(mut self) -> (String, Vec<Option<u32>>) {
        self.li("t0", DONE);
        self.li("t1", K1 | DONE_AT);
        self.emit("sw t0, 0(t1)");
        self.place("end");
        self.emit("beq zero, zero, end");
        self.emit("nop");
        self.place("handler");
        self.emit(format!("lw k0, {}(k0)", LOGP - FLAG));
        self.emit("nop");
        for (i, r) in ["CAUSE", "EPC", "BadVaddr", "SR"].iter().enumerate() {
            self.emit(format!("mfc0 k1, {r}"));
            self.emit("nop");
            self.emit(format!("sw k1, {}(k0)", 4 * i));
        }
        self.emit("ori k1, k1, 4");
        self.emit("xori k1, k1, 4");
        self.emit("mtc0 k1, SR");
        self.emit("addiu k0, k0, 16");
        self.emit(format!("lui k1, 0x{:x}", (K1 | FLAG) >> 16));
        self.emit(format!("sw k0, {}(k1)", LOGP - FLAG));
        self.emit(format!("lw k0, {}(k1)", RESUME - FLAG));
        self.emit("nop");
        self.emit("jr k0");
        self.emit("rfe");
        let mut text = self.lines.join("\n");
        text.push('\n');
        (text, self.expected)
    }
}

/// The IOP's dialect: the R3000A with its coprocessor 0 names and no coprocessor 2.
pub fn dialect() -> emusen_mips::Dialect {
    blackmoon_r3000::dialect(None)
}

pub fn assemble(source: &str) -> Vec<u32> {
    emusen_mips::assemble(source, K1 | ORIGIN, &dialect()).unwrap_or_else(|e| panic!("{e:?}"))
}

/// The general exception vector's four words at IOP RAM 80h: a jump to the program's kseg1 entry.
pub fn vector() -> [u32; 4] {
    let entry = K1 | ORIGIN;
    [0x3C1A_0000 | entry >> 16, 0x375A_0000 | (entry & 0xFFFF), 0x0340_0008, 0]
}

/// The EE program a referee runs for an IOP program of `words` with a record of `record` words (§22.9): it copies the
/// program to IOP RAM at `ORIGIN` through the EE's window at BC000000h, clears the done word and the handler's flag,
/// writes `vector` at the IOP's general exception vector 80000080h, so that the IOP's next interrupt enters the
/// program, waits a bounded time for the done word, and copies the done word and the record into EE RAM at 00200000h
/// before writing the EE's own done word. Returns the EE program's assembly for the R5900.
pub fn wrap(words: &[u32], record: usize) -> String {
    let iop = 0xBC00_0000u32;
    let mut s: Vec<String> = Vec::new();
    let li = |s: &mut Vec<String>, r: &str, v: u32| {
        s.push(format!("lui {r}, 0x{:x}", v >> 16));
        s.push(format!("ori {r}, {r}, 0x{:x}", v & 0xFFFF));
    };
    li(&mut s, "t0", iop | DONE_AT);
    s.push("sw zero, 0(t0)".into());
    li(&mut s, "t0", iop | FLAG);
    s.push("sw zero, 0(t0)".into());
    li(&mut s, "t0", iop | ORIGIN);
    li(&mut s, "t1", BLOB);
    li(&mut s, "t2", words.len() as u32);
    s.push("copy: lw t3, 0(t1)".into());
    s.push("sw t3, 0(t0)".into());
    s.push("addiu t1, t1, 4".into());
    s.push("addiu t0, t0, 4".into());
    s.push("addiu t2, t2, -1".into());
    s.push("bne t2, zero, copy".into());
    s.push("nop".into());
    s.push("sync".into());
    li(&mut s, "t0", iop | 0x80);
    for (i, w) in vector().iter().enumerate() {
        li(&mut s, "t3", *w);
        s.push(format!("sw t3, {}(t0)", 4 * i));
    }
    s.push("sync".into());
    li(&mut s, "t0", iop | DONE_AT);
    li(&mut s, "t4", DONE);
    li(&mut s, "t5", 300_000_000);
    s.push("wait: lw t3, 0(t0)".into());
    s.push("beq t3, t4, got".into());
    s.push("nop".into());
    s.push("addiu t5, t5, -1".into());
    s.push("bne t5, zero, wait".into());
    s.push("nop".into());
    s.push("got: lui t6, 0xa020".into());
    s.push("sw t3, 0(t6)".into());
    li(&mut s, "t0", iop | RECORD);
    li(&mut s, "t2", record as u32);
    s.push("beq t2, zero, out".into());
    s.push("nop".into());
    s.push("rec: lw t3, 0(t0)".into());
    s.push("sw t3, 4(t6)".into());
    s.push("addiu t0, t0, 4".into());
    s.push("addiu t6, t6, 4".into());
    s.push("addiu t2, t2, -1".into());
    s.push("bne t2, zero, rec".into());
    s.push("nop".into());
    s.push("out: lui t0, 0x5a99".into());
    s.push("ori t0, t0, 0xd00e".into());
    s.push("lui t1, 0xa01f".into());
    s.push("ori t1, t1, 0xfff0".into());
    s.push("sw t0, 0(t1)".into());
    s.push("end: beq zero, zero, end".into());
    s.push("nop".into());
    let blob = ((BLOB - EE_ORIGIN) / 4) as usize;
    assert!(s.len() < blob);
    s.resize(blob, "nop".into());
    for w in words {
        s.push(format!(".word 0x{w:08x}"));
    }
    s.join("\n") + "\n"
}

/// `s21-map`: PRId, RAM and its mirrors, byte and halfword lanes, the scratchpad and where it is not, the cache
/// control's bits, and the bus errors, syscall and break the handler logs (ps2tek "Memory Map"; psx-spx "Memory Map",
/// "Memory Mirrors", "Memory Exceptions", "FFFE0130h").
pub fn map() -> Program {
    let mut p = Program::new("s21-map");
    p.emit("mfc0 t2, PRID");
    p.emit("nop");
    p.record("t2", None);
    // ps2tek's BIOS chapter: the IOP's PRId is below 59h and at least 0Fh.
    p.record_within("t2", 0x0F, 0x58);
    let a = AREA;
    p.poke(K1 | a, 0x1122_3344);
    p.rec(K0 | a, Some(0x1122_3344));
    p.rec(a, Some(0x1122_3344));
    p.li("t6", K1 | a);
    p.li("t7", 0xAABB_CCDD);
    p.emit("sb t7, 1(t6)");
    p.emit("sh t7, 2(t6)");
    p.rec(K1 | a, Some(0xCCDD_DD44));
    p.emit("lb t2, 3(t6)");
    p.emit("nop");
    p.record("t2", Some(0xFFFF_FFCC));
    p.emit("lhu t2, 0(t6)");
    p.emit("nop");
    p.record("t2", Some(0xDD44));
    for mirror in [0x20_0000, 0x40_0000, 0x60_0000] {
        p.rec(K1 | (a + mirror), Some(0xCCDD_DD44));
    }
    p.poke(K1 | (a + 0x60_0004), 0x5566_7788);
    p.rec(K1 | (a + 4), Some(0x5566_7788));
    // Past the 8 MiB window: a bus error on the load.
    p.guarded(|p| p.peek(K1 | 0x0080_0000));
    p.rec_log(1, Some(7), None);
    // The scratchpad: KUSEG and KSEG0, 1 KiB.
    p.poke(0x1F80_0000, 0x0BAD_F00D);
    p.poke(0x9F80_03FC, 0xFEED_FACE);
    p.rec(0x9F80_0000, Some(0x0BAD_F00D));
    p.rec(0x1F80_03FC, Some(0xFEED_FACE));
    p.guarded(|p| p.peek(0xBF80_0000));
    p.rec_log(1, Some(7), None);
    p.guarded(|p| p.peek(0x9F80_0400));
    p.rec_log(1, Some(7), None);
    p.guarded(|p| p.peek(0x9E00_0000));
    p.rec_log(1, Some(7), None);
    // The cache control register's bits: 6 and 10 read zero (IBLKSZ kept at 1, since psx-spx calls 2 and 3 a crash).
    p.poke(BCC, 0xFFFF_FDFF);
    p.rec(BCC, Some(0xFFFF_F9BF));
    p.poke(BCC, 0x0001_E988);
    p.rec(BCC, Some(0x0001_E988));
    // Syscall and break: their codes and EPC.
    let mut at = 0;
    p.guarded(|p| {
        at = p.pc();
        p.emit("syscall");
    });
    p.rec_log(1, Some(8), Some(at));
    p.guarded(|p| {
        at = p.pc();
        p.emit("break");
    });
    p.rec_log(1, Some(9), Some(at));
    p
}

/// A routine at physical `at` returning `v` in v0: `jr ra` with the `ori` in its delay slot.
fn routine(p: &mut Program, at: u32, v: u32) {
    p.poke(K1 | at, 0x03E0_0008);
    p.poke(K1 | (at + 4), 0x3402_0000 | v);
}

/// Calls the routine at `address` and records v0.
fn call(p: &mut Program, address: u32, want: Option<u32>) {
    p.li("t0", address);
    p.emit("jalr t0");
    p.emit("nop");
    p.record("v0", want);
}

/// Invalidates the I-cache line of physical `at`, as psx-spx's flush does: TAG and IS1 in the cache control, the cache
/// isolated, a zero stored at the line's offset.
fn flush_line(p: &mut Program, at: u32) {
    p.poke(BCC, 0x0000_0804);
    p.li("t0", 0x0001_0000);
    p.emit("mtc0 t0, SR");
    p.emit("nop");
    p.li("t6", at & 0xFF0);
    p.emit("sw zero, 0(t6)");
    p.emit("mtc0 zero, SR");
    p.emit("nop");
    p.poke(BCC, 0x0001_E988);
}

/// A loop of `n` passes at physical `at`, called through `jalr`: `addiu a0, a0, -1; bne a0, zero, -4; nop; jr ra; nop`.
fn loop_routine(p: &mut Program, at: u32) {
    for (i, w) in [0x2484_FFFF, 0x1480_FFFE, 0, 0x03E0_0008, 0].iter().enumerate() {
        p.poke(K1 | (at + 4 * i as u32), *w);
    }
}

/// Times a call of the loop at `address` with `n` passes on timer 5 (the IOP's clock): the cycles into t3.
fn time_loop(p: &mut Program, address: u32, n: u32) {
    p.poke(mode(5), 0);
    p.li("a0", n);
    p.li("t0", address);
    p.peek(count(5));
    p.emit("or t3, t2, zero");
    p.emit("jalr t0");
    p.emit("nop");
    p.peek(count(5));
    p.emit("subu t3, t2, t3");
}

/// `s21-cache`: the I-cache as psx-spx describes the PlayStation's, 4 KiB direct-mapped, run from kseg1 so that only
/// the routines it calls are cached: stale code after a store until the line is flushed, the 4 KiB conflict, a refill
/// from the word to the line's end, the isolated cache, and the cached loop's speed against the uncached one's and
/// against the cache disabled.
pub fn cache() -> Program {
    let mut p = Program::uncached("s21-cache");
    let r = AREA + 0x1_0000;
    routine(&mut p, r, 0xAAAA);
    call(&mut p, K0 | r, Some(0xAAAA));
    routine(&mut p, r, 0xBBBB);
    call(&mut p, K0 | r, Some(0xAAAA));
    call(&mut p, K1 | r, Some(0xBBBB));
    flush_line(&mut p, r);
    call(&mut p, K0 | r, Some(0xBBBB));
    // The same line 4 KiB on evicts it; 2 KiB on does not.
    let (r1, r2) = (r + 0x100, r + 0x200);
    routine(&mut p, r1, 0x1111);
    routine(&mut p, r1 + 0x1000, 0x2222);
    call(&mut p, K0 | r1, Some(0x1111));
    call(&mut p, K0 | (r1 + 0x1000), Some(0x2222));
    routine(&mut p, r1, 0x3333);
    call(&mut p, K0 | r1, Some(0x3333));
    routine(&mut p, r2, 0x4444);
    routine(&mut p, r2 + 0x800, 0x5555);
    call(&mut p, K0 | r2, Some(0x4444));
    call(&mut p, K0 | (r2 + 0x800), Some(0x5555));
    routine(&mut p, r2, 0x6666);
    call(&mut p, K0 | r2, Some(0x4444));
    // Entered at word 2, a line fills words 2 and 3 only; word 0 then misses with its tag matching, and the whole line
    // is refilled.
    let r3 = r + 0x300;
    routine(&mut p, r3, 0x7777);
    routine(&mut p, r3 + 8, 0x8888);
    call(&mut p, K0 | (r3 + 8), Some(0x8888));
    routine(&mut p, r3, 0x9999);
    routine(&mut p, r3 + 8, 0xCCCC);
    call(&mut p, K0 | (r3 + 8), Some(0x8888));
    call(&mut p, K0 | r3, Some(0x9999));
    call(&mut p, K0 | (r3 + 8), Some(0xCCCC));
    // Isolated: a store reaches the cache and not memory, and a load reads the cache.
    let d = AREA + 0xF00;
    p.poke(K1 | d, 0x1357_9BDF);
    p.poke(BCC, 0x0000_0800);
    p.li("t0", 0x0001_0000);
    p.li("t6", K0 | d);
    p.li("t7", 0x2468_ACE0);
    p.emit("mtc0 t0, SR");
    p.emit("nop");
    p.emit("sw t7, 0(t6)");
    p.emit("lw t3, 0(t6)");
    p.emit("nop");
    p.emit("mtc0 zero, SR");
    p.emit("nop");
    p.record("t3", Some(0x2468_ACE0));
    p.poke(BCC, 0x0001_E988);
    p.rec(K1 | d, Some(0x1357_9BDF));
    // Speed: the same loop cached, uncached, and through kseg0 with the cache disabled.
    let lp = r + 0x400;
    loop_routine(&mut p, lp);
    time_loop(&mut p, K0 | lp, 4);
    time_loop(&mut p, K0 | lp, 1000);
    p.emit("or s2, t3, zero");
    time_loop(&mut p, K1 | lp, 1000);
    p.emit("or s3, t3, zero");
    p.poke(BCC, 0x0001_E188);
    time_loop(&mut p, K0 | lp, 1000);
    p.emit("or s4, t3, zero");
    p.poke(BCC, 0x0001_E988);
    p.record("s2", None);
    p.record("s3", None);
    p.record("s4", None);
    // Cached at least twice as fast as uncached (psx-spx: a RAM access is the cycle and six wait states); with IS1 clear
    // kseg0 runs uncached (within an eighth).
    p.emit("sll t3, s2, 1");
    p.emit("sltu t3, t3, s3");
    p.record("t3", Some(1));
    p.emit("srl t4, s3, 3");
    p.emit("subu t3, s4, s3");
    p.emit("bgez t3, abs_ok");
    p.emit("nop");
    p.emit("subu t3, s3, s4");
    p.place("abs_ok");
    p.emit("sltu t3, t3, t4");
    p.record("t3", Some(1));
    p
}

/// `s21-intc`: I_MASK's width, I_CTRL's read that clears it, I_STAT's acknowledge by zeros, Cause's IP2 as the
/// controller's line, an interrupt taken only with SR's IEc and IM2, and the V-blank sources with the field's length
/// (ps2tek "IOP Interrupts"; psx-spx "Interrupts").
pub fn intc() -> Program {
    let mut p = Program::new("s21-intc");
    p.poke(I_MASK, 0xFFFF_FFFF);
    p.rec(I_MASK, None);
    p.rec_masked(I_MASK, 0x03FF_FFFF, 0x03FF_FFFF);
    p.poke(I_MASK, 0);
    p.rec(I_MASK, Some(0));
    p.poke(I_CTRL, 1);
    p.rec_masked(I_CTRL, 1, 1);
    p.rec_masked(I_CTRL, 1, 0);
    p.poke(I_CTRL, 1);
    // Timer 4's compare on the IOP's clock over 256 raises bit 15.
    p.poke(target(4), 4);
    p.poke(mode(4), tm::CMP | tm::ZRET | tm::pscl45(3));
    p.wait(I_STAT, irq::TIMER4, true, 100_000);
    p.rec_masked(I_STAT, irq::TIMER4, irq::TIMER4);
    p.poke(mode(4), 0);
    // Masked: no IP2.
    p.emit("mfc0 t2, CAUSE");
    p.emit("nop");
    p.emit("andi t2, t2, 0x400");
    p.record("t2", Some(0));
    p.poke(I_MASK, irq::TIMER4);
    p.emit("nop");
    p.emit("mfc0 t2, CAUSE");
    p.emit("nop");
    p.emit("andi t2, t2, 0x400");
    p.record("t2", Some(0x400));
    p.poke(I_CTRL, 0);
    p.emit("nop");
    p.emit("mfc0 t2, CAUSE");
    p.emit("nop");
    p.emit("andi t2, t2, 0x400");
    p.record("t2", Some(0));
    p.poke(I_CTRL, 1);
    // SR's IEc without IM2: not taken; with both: taken, Cause showing IP2 and code 0.
    // A window of a hundred passes, since a referee may test for interrupts only at intervals.
    p.guarded(|p| {
        p.li("t0", 0x0000_0001);
        p.emit("mtc0 t0, SR");
        p.delay(100);
        p.emit("mtc0 zero, SR");
        p.emit("nop");
    });
    p.rec_log(0, None, None);
    p.guarded(|p| {
        p.li("t0", 0x0000_0401);
        p.emit("mtc0 t0, SR");
        p.delay(100);
        p.emit("mtc0 zero, SR");
        p.emit("nop");
    });
    p.rec_log(1, Some(0), None);
    p.peek(K1 | LOGP);
    p.emit("lw t3, -16(t2)");
    p.emit("nop");
    p.emit("andi t3, t3, 0x400");
    p.record("t3", Some(0x400));
    // Acknowledged by a zero; a one leaves the bit.
    p.poke(I_STAT, 0xFFFF_FFFF);
    p.rec_masked(I_STAT, irq::TIMER4, irq::TIMER4);
    p.poke(I_STAT, !irq::TIMER4);
    p.rec_masked(I_STAT, irq::TIMER4, 0);
    p.poke(I_MASK, 0);
    // V-blank: its start and end latch, and the field and the blank's lengths on the IOP's clock.
    p.poke(I_STAT, !irq::VBLANK_START);
    p.wait(I_STAT, irq::VBLANK_START, true, 2_000_000);
    p.poke(mode(5), 0);
    p.poke(I_STAT, !(irq::VBLANK_START | irq::VBLANK_END));
    p.wait(I_STAT, irq::VBLANK_END, true, 2_000_000);
    p.read_count(5);
    p.emit("or s2, t2, zero");
    p.wait(I_STAT, irq::VBLANK_START, true, 2_000_000);
    p.read_count(5);
    p.record("s2", None);
    p.record("t2", None);
    // The field within 1% of ps2tek's NTSC field, the blank about 22 of its 262 lines.
    p.record_within("t2", NTSC_FIELD * 99 / 100, NTSC_FIELD * 101 / 100);
    p.record_within("s2", NTSC_FIELD / 262 * 20, NTSC_FIELD / 262 * 24);
    p
}

/// `s21-dma`: the registers' widths, OTC's reversed list in burst mode with MADR and BCR kept, DICR's flag only with
/// its mask and the master enable, the master flag's IRQ 3, DMACEN and the channel enable holding a transfer, and
/// chopping, which lets the processor run and updates MADR and BCR (ps2tek "IOP DMA"; psx-spx "DMA Channels").
pub fn dma() -> Program {
    let mut p = Program::new("s21-dma");
    let otc = 6;
    p.poke(DPCR, 0);
    p.poke(chcr(otc), 0xFFFF_FFFF);
    p.rec(chcr(otc), Some(0x5100_0002));
    p.poke(chcr(otc), 0);
    p.rec(chcr(otc), Some(0x0000_0002));
    p.poke(chcr(4), 0xFFFF_FFFF);
    p.rec(chcr(4), Some(0xF177_0F03));
    p.poke(chcr(4), 0);
    p.poke(madr(4), 0xFFFF_FFFF);
    p.rec(madr(4), Some(0x00FF_FFFF));
    p.poke(tadr(4), 0xFFFF_FFFF);
    p.rec(tadr(4), Some(0x00FF_FFFF));
    p.poke(bcr(4), 0xFFFF_FFFF);
    p.rec(bcr(4), Some(0xFFFF_FFFF));
    p.poke(DICR, 0xFFFF_FFFF);
    p.rec(DICR, Some(0x80FF_807F));
    p.poke(DICR, 0x7F00_0000);
    p.rec(DICR, Some(0));
    p.poke(I_STAT, 0);
    // OTC: sixteen entries ending at AREA, each the address of the one before, the last FFFFFFh.
    let list = |p: &mut Program, base: u32, n: u32| {
        for i in 0..n {
            let a = base - 4 * i;
            p.rec(K1 | a, Some(if i == n - 1 { 0x00FF_FFFF } else { a - 4 }));
        }
    };
    let top = AREA + 0x3C;
    p.poke(DPCR, 0x0F00_0000);
    p.poke(DICR, 0x0080_0000 | 1 << (16 + otc));
    p.poke(madr(otc), top);
    p.poke(bcr(otc), 16);
    p.poke(chcr(otc), 0x1100_0002);
    p.wait(chcr(otc), 0x0100_0000, false, 1000);
    // The processor waited for the burst: the first poll saw it done.
    p.record_is("t5", 1000);
    list(&mut p, top, 16);
    p.rec(madr(otc), Some(top));
    p.rec(bcr(otc), Some(16));
    p.rec(chcr(otc), Some(2));
    p.rec_masked(DICR, 0xFF00_0000, 0xC000_0000);
    p.rec_masked(I_STAT, irq::DMA, irq::DMA);
    p.poke(DICR, 0x7F00_0000);
    p.rec(DICR, Some(0));
    p.poke(I_STAT, 0);
    // Without the channel's mask, no flag; the bus error flag forces the master flag and IRQ 3.
    p.poke(DICR, 0x0080_0000);
    p.poke(chcr(otc), 0x1100_0002);
    p.wait(chcr(otc), 0x0100_0000, false, 1000);
    p.rec_masked(DICR, 0xFF00_0000, 0);
    p.rec_masked(I_STAT, irq::DMA, 0);
    p.poke(DICR, 0x0000_8000);
    p.rec(DICR, Some(0x8000_8000));
    p.rec_masked(I_STAT, irq::DMA, irq::DMA);
    p.poke(DICR, 0);
    p.rec(DICR, Some(0));
    p.poke(I_STAT, 0);
    // DMACINTEN: bit 0 clear keeps the flags from the master flag; bit 1 set keeps the master flag from the INTC.
    p.poke(DMACINTEN, 0);
    p.poke(DICR, 0x0080_0000 | 1 << (16 + otc));
    p.poke(chcr(otc), 0x1100_0002);
    p.wait(chcr(otc), 0x0100_0000, false, 1000);
    p.rec_masked(DICR, 0x8000_0000, 0);
    p.rec(DICR, None);
    p.poke(DICR, 0x7F00_0000);
    p.poke(DMACINTEN, 3);
    p.poke(DICR, 0x0080_0000 | 1 << (16 + otc));
    p.poke(chcr(otc), 0x1100_0002);
    p.wait(chcr(otc), 0x0100_0000, false, 1000);
    p.rec_masked(DICR, 0xC000_0000, 0xC000_0000);
    p.rec_masked(I_STAT, irq::DMA, 0);
    p.poke(DICR, 0x7F00_0000);
    p.poke(DMACINTEN, 1);
    p.poke(I_STAT, 0);
    // DMACEN clear, then the channel's enable clear, hold the transfer with START and the trigger kept.
    let base2 = AREA + 0x17C;
    p.poke(K1 | base2, 0xDEAD_BEEF);
    p.poke(DMACEN, 0);
    p.poke(madr(otc), base2);
    p.poke(bcr(otc), 32);
    p.poke(chcr(otc), 0x1100_0002);
    p.delay(200);
    p.rec(chcr(otc), Some(0x1100_0002));
    p.rec(K1 | base2, Some(0xDEAD_BEEF));
    p.poke(DPCR, 0x0700_0000);
    p.poke(DMACEN, 1);
    p.delay(200);
    p.rec(chcr(otc), Some(0x1100_0002));
    p.poke(DPCR, 0x0F00_0000);
    p.wait(chcr(otc), 0x0100_0000, false, 1000);
    p.rec(chcr(otc), Some(2));
    p.rec(K1 | base2, Some(base2 - 4));
    p
}

/// `s21-dma-modes`, on Sapphire only, its channels ending at the stand-ins and the SIF's FIFOs: slice mode to SPU2
/// (7), a linked list to SIF2 (2), a chain from memory to SIF0 (9) with the tag's two words first and an IRQ tag, a
/// chain to memory from SIF1 (10) with the tag put in its FIFO, and a burst to SPU (4) started by its request (psx-spx;
/// ps2tek).
pub fn dma_modes() -> Program {
    let mut p = Program::new("s21-dma-modes");
    p.poke(DPCR, 0x0000_0F00 | 0x000F_0000);
    p.poke(DPCR2, 0x0000_FF0F);
    p.poke(DICR, 0x0080_0000);
    p.poke(DICR2, 1 << 9);
    // Slice: three blocks of four words.
    let s = AREA + 0x1000;
    for i in 0..12 {
        p.poke(K1 | (s + 4 * i), 0x7000_0000 | i);
    }
    p.poke(madr(7), s);
    p.poke(bcr(7), 3 << 16 | 4);
    p.poke(chcr(7), 0x0100_0201);
    p.wait(chcr(7), 0x0100_0000, false, 10_000);
    p.rec(madr(7), Some(s + 48));
    p.rec(bcr(7), Some(4));
    p.rec(chcr(7), Some(0x0000_0201));
    // A linked list of three nodes.
    let l = AREA + 0x2000;
    p.poke(K1 | l, 2 << 24 | (l + 0x100));
    p.poke(K1 | (l + 4), 0x2000_0000);
    p.poke(K1 | (l + 8), 0x2000_0001);
    p.poke(K1 | (l + 0x100), 1 << 24 | (l + 0x200));
    p.poke(K1 | (l + 0x104), 0x2000_0002);
    p.poke(K1 | (l + 0x200), 0x00FF_FFFF);
    p.poke(madr(2), l);
    p.poke(chcr(2), 0x0100_0401);
    p.wait(chcr(2), 0x0100_0000, false, 10_000);
    p.rec(madr(2), Some(0x00FF_FFFF));
    p.rec(chcr(2), Some(0x0000_0401));
    // A chain from memory: two tags with their two extra words, the first with IRQ, the second ending it.
    let c = AREA + 0x3000;
    let tags = [(c + 0x100) | 1 << 30, 2, 0x9000_0000, 0x9000_0001, (c + 0x200) | 1 << 31, 3, 0x9000_0002, 0x9000_0003];
    for (i, w) in tags.iter().enumerate() {
        p.poke(K1 | (c + 4 * i as u32), *w);
    }
    for i in 0..2 {
        p.poke(K1 | (c + 0x100 + 4 * i), 0x9100_0000 | i);
    }
    for i in 0..3 {
        p.poke(K1 | (c + 0x200 + 4 * i), 0x9200_0000 | i);
    }
    p.poke(tadr(9), c);
    p.poke(chcr(9), 0x0100_0701);
    p.wait(chcr(9), 0x0100_0000, false, 10_000);
    p.rec(tadr(9), Some(c + 32));
    p.rec(madr(9), Some(c + 0x20C));
    p.rec(chcr(9), Some(0x0000_0701));
    p.rec_masked(DICR2, 0x3F00_0000, 1 << 26);
    p.poke(DICR2, 0x3F00_0000);
    // A chain to memory: the stand-in gives a tag ending the chain with two words for AREA + 4000h.
    p.poke(chcr(10), 0x0100_0600);
    p.wait(chcr(10), 0x0100_0000, false, 10_000);
    p.rec(chcr(10), Some(0x0000_0600));
    p.rec(K1 | (AREA + 0x4000), Some(0xA000_0000));
    p.rec(K1 | (AREA + 0x4004), Some(0xA000_0001));
    // A burst to SPU started by its request, no trigger.
    p.poke(madr(4), s);
    p.poke(bcr(4), 5);
    p.poke(chcr(4), 0x0100_0001);
    p.wait(chcr(4), 0x0100_0000, false, 10_000);
    p.rec(chcr(4), Some(1));
    p.rec(madr(4), Some(s));
    // Chopping, which OTC cannot take (psx-spx: its CHCR keeps three bits): a burst to SPU in windows of one word
    // with 128 cycles for the processor between them; the processor polls meanwhile, and MADR and BC move to the end.
    p.poke(bcr(4), 32);
    p.poke(chcr(4), 0x1170_0101);
    p.wait(chcr(4), 0x0100_0000, false, 100_000);
    p.record("t5", None);
    p.li("t7", 100_000);
    p.emit("sltu t7, t5, t7");
    p.record("t7", Some(1));
    p.rec(madr(4), Some(s + 32 * 4));
    p.rec_masked(bcr(4), 0xFFFF, 0);
    p.rec(chcr(4), Some(0x0070_0101));
    p
}

/// What `dma_modes` finds in SIF1's FIFO: a tag ending the chain, its address and size in a quadword, and two words.
pub fn sif1_queue() -> [u32; 6] {
    [(AREA + 0x4000) | 1 << 31, 2, 0, 0, 0xA000_0000, 0xA000_0001]
}

/// Starts timer `n` with mode `m` and target `t`.
fn timer_start(p: &mut Program, n: u32, m: u32, t: u32) {
    p.poke(target(n), t);
    p.poke(mode(n), m);
}

/// `s21-timers`: the widths of count and target on the 16- and 32-bit timers, the mode's write and read, the
/// prescalers and the pixel clock against the IOP's clock, H-blank as a clock over a field, ZRET, the target and
/// overflow flags and requests in one-shot, repeat, pulse and toggle modes, the target's write re-arming the request,
/// and the gates (ps2tek "IOP Timers"; psx-spx "Timers").
pub fn timers() -> Program {
    let mut p = Program::new("s21-timers");
    // Widths: 16 bits on 0 to 2, 32 on 3 to 5.
    for n in 0..6 {
        p.poke(target(n), 0xFFFF_FFFF);
        p.rec(target(n), Some(if n < 3 { 0xFFFF } else { 0xFFFF_FFFF }));
        p.poke(count(n), 0xFFFF_0000);
        p.read_count(n);
        p.li("t7", 0xFFFF_0000);
        p.emit("and t2, t2, t7");
        p.record("t2", Some(if n < 3 { 0 } else { 0xFFFF_0000 }));
    }
    // Wrapping: from FFF0h, 16-bit timers wrap with OVFF; 32-bit ones pass 10000h without it.
    for n in [0, 3] {
        p.poke(mode(n), 0);
        p.poke(count(n), 0xFFF0);
        p.delay(100);
        p.read_count(n);
        if n == 0 {
            p.record_within("t2", 1, 0xFFEF);
        } else {
            p.record_within("t2", 0x1_0000, 0x10_0000);
        }
        p.rec_masked(mode(n), tm::OVFF, if n == 0 { tm::OVFF } else { 0 });
    }
    // The mode's write resets the count and sets the request bit; timer 2 stopped by its gate keeps 0.
    p.poke(count(2), 0x1234);
    p.poke(mode(2), tm::GATE);
    p.rec(count(2), Some(0));
    p.rec_masked(mode(2), 0x7FF, tm::GATE | tm::INTF);
    p.delay(50);
    p.rec(count(2), Some(0));
    p.poke(mode(2), tm::GATE | tm::gatm(3));
    p.delay(50);
    p.rec(count(2), Some(0));
    p.poke(mode(2), tm::GATE | tm::gatm(1));
    p.delay(50);
    p.read_count(2);
    p.record_within("t2", 1, 0xFFFF);
    // The prescalers against timer 4 on the IOP's clock: timer 2's 8 and timer 5's 8, 16 and 256.
    for (n, m, k) in [(2, tm::PSCL2, 8u32), (5, tm::pscl45(1), 8), (5, tm::pscl45(2), 16), (5, tm::pscl45(3), 256)] {
        p.poke(mode(4), 0);
        p.poke(mode(n), m);
        p.delay(3000);
        p.read_count(4);
        p.emit("or t3, t2, zero");
        p.read_count(n);
        p.emit(format!("sll t2, t2, {}", k.trailing_zeros()));
        // timer 4 - k × timer n within ±3k: the reads' and starts' skew and the first tick's phase.
        p.emit("subu t3, t3, t2");
        p.li("t7", 3 * k);
        p.emit("addu t3, t3, t7");
        p.record_within("t3", 0, 6 * k);
    }
    // The pixel clock, 13.5 MHz against 36.864: timer 0 × 4096 = timer 4 × 1500 within eight ticks of the slower.
    p.poke(mode(4), 0);
    p.poke(mode(0), tm::EXTC);
    p.delay(3000);
    p.read_count(4);
    p.emit("or t3, t2, zero");
    p.read_count(0);
    p.emit("sll t2, t2, 12");
    p.li("t7", 1500);
    p.emit("multu t3, t7");
    p.emit("mflo t3");
    p.emit("subu t3, t3, t2");
    p.li("t7", 8 * 4096);
    p.emit("addu t3, t3, t7");
    p.record_within("t3", 0, 16 * 4096);
    // H-blank as the clock of timers 1 and 3 over one field, between two V-blank starts.
    p.poke(I_STAT, !irq::VBLANK_START);
    p.wait(I_STAT, irq::VBLANK_START, true, 2_000_000);
    p.poke(mode(1), tm::EXTC);
    p.poke(mode(3), tm::EXTC);
    p.poke(I_STAT, !irq::VBLANK_START);
    p.wait(I_STAT, irq::VBLANK_START, true, 2_000_000);
    p.read_count(1);
    p.emit("or t3, t2, zero");
    p.read_count(3);
    p.record("t3", None);
    p.record("t2", None);
    p.record_within("t3", NTSC_LINES - 1, NTSC_LINES + 1);
    p.record_within("t2", NTSC_LINES - 1, NTSC_LINES + 1);
    // ZRET at target 9 on the clock over 256: the count never passes 9.
    timer_start(&mut p, 5, tm::ZRET | tm::pscl45(3), 9);
    p.li("s2", 0);
    p.li("s3", 400);
    let l = p.label();
    let skip = p.label();
    p.place(&l);
    p.read_count(5);
    p.emit("sltu t3, s2, t2");
    p.emit(format!("beq t3, zero, {skip}"));
    p.emit("nop");
    p.emit("or s2, t2, zero");
    p.place(&skip);
    p.emit("addiu s3, s3, -1");
    p.emit(format!("bne s3, zero, {l}"));
    p.emit("nop");
    p.record("s2", Some(9));
    // The target's flag and a repeated pulse request: EQUF and the request bit set after it.
    p.poke(I_STAT, 0);
    timer_start(&mut p, 5, tm::ZRET | tm::CMP | tm::REPT | tm::pscl45(3), 20);
    p.wait(I_STAT, irq::TIMER5, true, 100_000);
    p.rec_masked(I_STAT, irq::TIMER5, irq::TIMER5);
    p.rec_masked(mode(5), tm::EQUF | tm::INTF, tm::EQUF | tm::INTF);
    p.rec_masked(mode(5), tm::EQUF, 0);
    p.poke(I_STAT, 0);
    p.wait(I_STAT, irq::TIMER5, true, 100_000);
    p.rec_masked(I_STAT, irq::TIMER5, irq::TIMER5);
    // One-shot: the request bit cleared after the first, no second; the target's write re-arms it.
    p.poke(I_STAT, 0);
    timer_start(&mut p, 4, tm::ZRET | tm::CMP | tm::pscl45(3), 20);
    p.wait(I_STAT, irq::TIMER4, true, 100_000);
    p.rec_masked(mode(4), tm::INTF, 0);
    p.poke(I_STAT, 0);
    p.delay(8000);
    p.rec_masked(I_STAT, irq::TIMER4, 0);
    p.poke(target(4), 20);
    p.rec_masked(mode(4), tm::INTF, tm::INTF);
    p.wait(I_STAT, irq::TIMER4, true, 100_000);
    p.rec_masked(I_STAT, irq::TIMER4, irq::TIMER4);
    // Toggle with repeat: the bit inverts at each target, a request only as it falls.
    p.poke(I_STAT, 0);
    timer_start(&mut p, 5, tm::ZRET | tm::CMP | tm::REPT | tm::LEVL | tm::pscl45(3), 20);
    p.wait(I_STAT, irq::TIMER5, true, 100_000);
    p.rec_masked(mode(5), tm::INTF, 0);
    p.poke(I_STAT, 0);
    p.wait(mode(5), tm::INTF, true, 100_000);
    p.rec_masked(I_STAT, irq::TIMER5, 0);
    p.wait(I_STAT, irq::TIMER5, true, 100_000);
    p.rec_masked(I_STAT, irq::TIMER5, irq::TIMER5);
    // Overflow on timer 2 over 8 from FFF0h: OVFF and bit 6.
    p.poke(I_STAT, 0);
    p.poke(mode(2), tm::OVF | tm::PSCL2);
    p.poke(count(2), 0xFFF0);
    p.wait(I_STAT, irq::TIMER2, true, 100_000);
    p.rec_masked(I_STAT, irq::TIMER2, irq::TIMER2);
    p.rec_masked(mode(2), tm::OVFF, tm::OVFF);
    // The V-blank gate of timer 1, mode 0: no count during the blank, a count after it.
    p.poke(I_STAT, !irq::VBLANK_START);
    p.wait(I_STAT, irq::VBLANK_START, true, 2_000_000);
    p.poke(mode(1), tm::GATE);
    p.delay(1000);
    p.rec(count(1), Some(0));
    p.poke(I_STAT, !irq::VBLANK_END);
    p.wait(I_STAT, irq::VBLANK_END, true, 2_000_000);
    p.delay(1000);
    p.read_count(1);
    p.record_within("t2", 1000, 0xFFFF);
    // Mode 3 outside the blank waits for its start, then runs.
    p.poke(mode(1), tm::GATE | tm::gatm(3));
    p.delay(1000);
    p.rec(count(1), Some(0));
    p.poke(I_STAT, !irq::VBLANK_START);
    p.wait(I_STAT, irq::VBLANK_START, true, 2_000_000);
    p.delay(1000);
    p.read_count(1);
    p.record_within("t2", 1000, 0xFFFF);
    // Timer 0's H-blank gate, mode 0: over many lines it counts less than fifteen sixteenths of the clock.
    p.poke(mode(4), 0);
    p.poke(mode(0), tm::GATE);
    p.delay(20_000);
    p.read_count(0);
    p.emit("or t3, t2, zero");
    p.read_count(4);
    p.emit("srl t4, t2, 4");
    p.emit("subu t2, t2, t4");
    p.emit("sltu t3, t3, t2");
    p.record("t3", Some(1));
    // Timers 3 to 5's gates, which ps2tek does not describe: recorded, not graded.
    for (n, m) in [(3, tm::GATE), (4, tm::GATE), (5, tm::GATE | tm::gatm(1))] {
        p.poke(mode(n), m);
        p.delay(1000);
        p.read_count(n);
        p.record("t2", None);
    }
    p
}

/// `s21-fetch`, on Sapphire only (PCSX2 stops when the IOP fetches there, §22.9): a fetch from the scratchpad is an
/// instruction bus error at its address (psx-spx "Scratchpad": "NOT executable").
pub fn fetch() -> Program {
    let mut p = Program::new("s21-fetch");
    p.guarded(|p| {
        p.li("t0", 0x9F80_0000);
        p.emit("jr t0");
        p.emit("nop");
    });
    p.rec_log(1, Some(6), Some(0x9F80_0000));
    p
}

/// The programs a referee can run.
pub fn all() -> Vec<Program> {
    vec![map(), cache(), intc(), dma(), timers()]
}
