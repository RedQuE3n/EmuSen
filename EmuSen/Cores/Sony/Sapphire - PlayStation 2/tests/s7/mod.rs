//! The project's S7 programs (Sapphire_Native.md §17.8): bare EE programs on the TLB's refills and pages, the
//! scratchpad, the data cache's write-back, the instruction cache, the counters, the breakpoints and the bus errors.
//! Each owns the machine: it installs its own handlers at every vector, writes back and clears both caches, clears
//! the TLB, then runs from kseg0 with its constants and record in kseg1, so that nothing depends on a kernel's
//! mappings. Each expected word is derived here from the manuals' rules (EE Core User's Manual chapters 3 to 7 and 9,
//! the EE Core Instruction Set Manual's Operation sections), written apart from `src/ee/`.

#![allow(dead_code)]

use crate::s6::Program;

/// Physical scratch pages the programs mark and read through kseg1.
pub const REFILL_PAGE: u32 = 0x0030_0000;
const SENTINEL: u64 = 0x0BAD_0BAD_0BAD_0BAD;

pub fn sx(v: u32) -> Option<u64> {
    Some(v as i32 as i64 as u64)
}

fn k0(a: u32) -> u32 {
    a | 0x8000_0000
}

fn k1(a: u32) -> u32 {
    a | 0xA000_0000
}

pub fn li(p: &mut Program, reg: &str, v: u32) {
    p.emit(format!("lui {reg}, 0x{:x}", v >> 16));
    p.emit(format!("ori {reg}, {reg}, 0x{:x}", v & 0xFFFF));
}

fn assemble(source: &str, origin: u32) -> Vec<u32> {
    emusen_mips::assemble(source, origin, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"))
}

/// Writes `value` to physical `pa` through kseg1.
fn poke(p: &mut Program, pa: u32, value: u32) {
    li(p, "t6", k1(pa));
    li(p, "t7", value);
    p.emit("sw t7, 0(t6)");
}

/// Loads the word at virtual `va` into t2 (preset to the sentinel) and records it; the load's address is returned.
fn load(p: &mut Program, va: u32, expected: Option<u64>) -> u32 {
    li(p, "t6", va);
    p.constant("t2", SENTINEL);
    let at = k0(p.here());
    p.emit("lw t2, 0(t6)");
    p.record("t2", expected);
    at
}

/// Records the physical word at `pa` as an uncached load sees it.
fn memory(p: &mut Program, pa: u32, value: u32) {
    li(p, "t6", k1(pa));
    p.emit("lw t2, 0(t6)");
    p.record("t2", sx(value));
}

/// Stores `value` at virtual `va`; the store's address is returned.
fn store(p: &mut Program, va: u32, value: u32) -> u32 {
    li(p, "t6", va);
    li(p, "t7", value);
    let at = k0(p.here());
    p.emit("sw t7, 0(t6)");
    at
}

fn cop0(p: &mut Program, reg: &str, expected: Option<u64>) {
    p.emit(format!("mfc0 t2, {reg}"));
    p.record("t2", expected);
}

/// TLBWI of one entry (EE Core User's Manual §5.3.2).
fn map(p: &mut Program, index: u32, pagemask: u32, entryhi: u32, lo0: u32, lo1: u32) {
    li(p, "t0", index);
    p.emit("mtc0 t0, Index");
    li(p, "t0", pagemask);
    p.emit("mtc0 t0, PageMask");
    li(p, "t0", entryhi);
    p.emit("mtc0 t0, EntryHi");
    li(p, "t0", lo0);
    p.emit("mtc0 t0, EntryLo0");
    li(p, "t0", lo1);
    p.emit("mtc0 t0, EntryLo1");
    p.emit("sync.p");
    p.emit("tlbwi");
    p.emit("sync.p");
    p.emit("mtc0 zero, EntryHi");
    p.emit("sync.p");
}

/// An EntryLo word: PFN, C, D, V, G.
fn lo(pfn: u32, c: u32, d: bool, v: bool, g: bool) -> u32 {
    pfn << 6 | c << 3 | u32::from(d) << 2 | u32::from(v) << 1 | u32::from(g)
}

/// The record a common or refill handler leaves: Cause's BD and ExcCode, EPC, BadVAddr, Context, EntryHi.
fn taken(p: &mut Program, code: u32, epc: u32, bd: bool, bad: Option<u32>, context: Option<u32>, entryhi: Option<u32>) {
    p.expected.push(sx(code << 2 | u32::from(bd) << 31));
    p.expected.push(sx(epc));
    p.expected.push(bad.and_then(sx));
    p.expected.push(context.and_then(sx));
    p.expected.push(entryhi.and_then(sx));
}

/// Context after a TLB exception at `va`, with PTEBase set to 1FFh by the prologue (§3.2, "Context Register").
fn context(va: u32) -> u32 {
    0xFF80_0000 | (va >> 13) << 4
}

const RECORD5: [&str; 14] = [
    "mfc0 k0, Cause",
    "lui k1, 0x8000",
    "ori k1, k1, 0x7c",
    "and k1, k0, k1",
    "sd k1, 0(s1)",
    "mfc0 k0, EPC",
    "sd k0, 8(s1)",
    "mfc0 k0, BadVAddr",
    "sd k0, 16(s1)",
    "mfc0 k0, Context",
    "sd k0, 24(s1)",
    "mfc0 k0, EntryHi",
    "sd k0, 32(s1)",
    "daddiu s1, s1, 40",
];

/// 80000000h: records, then maps the faulting pair as two uncached 4 KiB pages at REFILL_PAGE in entry 7 and retries.
fn refill_handler() -> String {
    let mut h: Vec<String> = RECORD5.iter().map(|s| s.to_string()).collect();
    let even = lo(REFILL_PAGE >> 12, 2, true, true, true);
    let odd = lo((REFILL_PAGE >> 12) + 1, 2, true, true, true);
    h.extend(
        [
            "mtc0 zero, PageMask".to_string(),
            "ori k0, zero, 7".into(),
            "mtc0 k0, Index".into(),
            format!("ori k0, zero, 0x{even:x}"),
            "mtc0 k0, EntryLo0".into(),
            format!("ori k0, zero, 0x{odd:x}"),
            "mtc0 k0, EntryLo1".into(),
            "sync.p".into(),
            "tlbwi".into(),
            "sync.p".into(),
            "eret".into(),
        ]
        .into_iter(),
    );
    h.join("\n")
}

/// 80000180h: records, and returns past the instruction, or past the branch and its slot when Cause.BD is set.
fn common_handler() -> String {
    let mut h: Vec<&str> = RECORD5.to_vec();
    h.extend([
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
    ]);
    h.join("\n")
}

/// 80000080h: records Cause's EXC2 and BD2, Status.ERL and PCR0's OVFL, sets t8, stops the counters, returns.
fn counter_handler() -> String {
    [
        "mfc0 k0, Cause",
        "lui k1, 0x4007",
        "and k1, k0, k1",
        "sd k1, 0(s1)",
        "mfc0 k0, Status",
        "andi k0, k0, 4",
        "sd k0, 8(s1)",
        "mfpc k0, 0",
        "srl k0, k0, 31",
        "sd k0, 16(s1)",
        "daddiu s1, s1, 24",
        "addiu t8, zero, 1",
        "mtps zero, 0",
        "sync.p",
        "eret",
    ]
    .join("\n")
}

/// 80000100h: records Cause's EXC2 and BD2, ErrorEPC and BPC, clears IAE, DRE, DWE and the three flags, returns.
fn debug_handler() -> String {
    [
        "mfc0 k0, Cause",
        "lui k1, 0x4007",
        "and k1, k0, k1",
        "sd k1, 0(s1)",
        "mfc0 k0, ErrorEPC",
        "sd k0, 8(s1)",
        "mfbpc k0",
        "sd k0, 16(s1)",
        "daddiu s1, s1, 24",
        "lui k1, 0x1fff",
        "ori k1, k1, 0xfff8",
        "and k0, k0, k1",
        "mtbpc k0",
        "sync.p",
        "eret",
    ]
    .join("\n")
}

/// 80000200h: records Cause's IP7 and ExcCode, writes Compare, records IP7 again, sets t8, returns to EPC.
fn interrupt_handler() -> String {
    [
        "mfc0 k0, Cause",
        "ori k1, zero, 0x807c",
        "and k1, k0, k1",
        "sd k1, 0(s1)",
        "mtc0 zero, Compare",
        "sync.p",
        "mfc0 k0, Cause",
        "andi k0, k0, 0x8000",
        "sd k0, 8(s1)",
        "daddiu s1, s1, 16",
        "addiu t8, zero, 1",
        "eret",
    ]
    .join("\n")
}

/// The words of every handler at its vector, for the prologue and the coverage count.
pub fn handlers() -> Vec<(u32, Vec<u32>)> {
    let h = [
        (0x8000_0000, refill_handler()),
        (0x8000_0080, counter_handler()),
        (0x8000_0100, debug_handler()),
        (0x8000_0180, common_handler()),
        (0x8000_0200, interrupt_handler()),
    ];
    h.into_iter()
        .map(|(at, src)| {
            let w = assemble(&src, at);
            assert!(w.len() <= 32, "a handler fits before the next vector");
            (at, w)
        })
        .collect()
}

/// The machine made the program's own: interrupts off, the handlers written through kseg1, both caches on,
/// every data cache line written back and its tag cleared, every instruction cache line invalidated, TLB entries 0 to
/// 7, the programs' own, cleared to invalid pairs in kseg3 (ASID 0, Context's PTEBase 1FFh). The entries above 7 are
/// left to whatever kernel entered the program: PCSX2 stops once they are rewritten (Sapphire_Native.md §17.9).
fn prologue(p: &mut Program) {
    prologue_to(p, 6);
}

/// The prologue's first `steps` parts, for finding the part a referee does not take.
fn prologue_to(p: &mut Program, steps: u32) {
    p.emit("di");
    p.emit("mtc0 zero, Status");
    p.emit("sync.p");
    if steps < 2 {
        return;
    }
    li(p, "t9", 0xA000_0000);
    for (at, words) in handlers() {
        for (i, w) in words.iter().enumerate() {
            li(p, "t3", *w);
            p.emit(format!("sw t3, 0x{:x}(t9)", (at & 0xFFFF) as usize + 4 * i));
        }
    }
    if steps < 3 {
        return;
    }
    p.emit("mfc0 t0, Config");
    p.emit("lui t1, 0x3");
    p.emit("or t0, t0, t1");
    p.emit("sync");
    p.emit("mtc0 t0, Config");
    p.emit("sync.p");
    if steps < 4 {
        return;
    }
    let d = p.label();
    p.emit("lui t0, 0x8000");
    p.emit("ori t1, zero, 64");
    p.emit("mtc0 zero, TagLo");
    p.emit("sync.p");
    p.place(&d);
    for line in ["sync", "cache 0x14, 0(t0)", "sync", "cache 0x14, 1(t0)", "sync", "cache 0x12, 0(t0)", "sync", "cache 0x12, 1(t0)", "sync"] {
        p.emit(line);
    }
    p.emit("addiu t1, t1, -1");
    p.emit(format!("bne t1, zero, {d}"));
    p.emit("addiu t0, t0, 64");
    if steps < 5 {
        return;
    }
    let i = p.label();
    p.emit("ori t0, zero, 0");
    p.emit("ori t1, zero, 128");
    p.place(&i);
    for line in ["sync.p", "cache 0x07, 0(t0)", "sync.p", "cache 0x07, 1(t0)", "sync.p"] {
        p.emit(line);
    }
    p.emit("addiu t1, t1, -1");
    p.emit(format!("bne t1, zero, {i}"));
    p.emit("addiu t0, t0, 64");
    if steps < 6 {
        return;
    }
    let t = p.label();
    p.emit("mtc0 zero, PageMask");
    p.emit("mtc0 zero, EntryLo0");
    p.emit("mtc0 zero, EntryLo1");
    let (first, end) = match steps {
        8 => (8, 48),
        9 => (47, 48),
        _ => (0, 8),
    };
    p.emit("lui t0, 0xe000");
    p.emit(format!("ori t1, zero, {first}"));
    p.place(&t);
    p.emit("mtc0 t1, Index");
    p.emit("mtc0 t0, EntryHi");
    p.emit("sync.p");
    p.emit("tlbwi");
    p.emit("sync.p");
    p.emit("addiu t1, t1, 1");
    p.emit(format!("slti t2, t1, {end}"));
    p.emit(format!("bne t2, zero, {t}"));
    p.emit("addiu t0, t0, 0x2000");
    p.emit("mtc0 zero, EntryHi");
    p.emit("lui t1, 0xff80");
    p.emit("mtc0 t1, Context");
    p.emit("sync.p");
}

/// TLB refills on loads and stores, even and odd pages of every page size, TLBR, TLBP, the ASID and G, the
/// invalid and modified exceptions, a refill taken at EXL, kuseg unmapped at ERL, and ksseg mapped.
pub fn tlb() -> Program {
    let mut p = Program::kernel("s7-tlb");
    prologue(&mut p);
    let mark = |pa: u32| 0x5A00_0000 | (pa >> 4 & 0xFF_FFFF);
    for off in [0x040, 0x080, 0x010, 0x020, 0x1010, 0x1080] {
        let pa = REFILL_PAGE + off;
        poke(&mut p, pa, mark(pa));
    }
    // Refills on loads (§4.2.7): the handler maps the pair to REFILL_PAGE, the load retried reads it.
    let refill_value = |va: u32| sx(mark(REFILL_PAGE | va & 0x1FFF));
    for (va, refills) in [(0x1000_0040u32, true), (0x1000_1080, false), (0x1002_3010, true)] {
        let at = load(&mut p, va, None);
        p.expected.pop();
        if refills {
            taken(&mut p, 2, at, false, Some(va), Some(context(va)), Some(va & 0xFFFF_E000));
        }
        p.expected.push(refill_value(va));
    }
    // A refill on a store (code 3): the retried store reaches REFILL_PAGE.
    let va = 0x1004_2020;
    let at = store(&mut p, va, 0x5151);
    taken(&mut p, 3, at, false, Some(va), Some(context(va)), Some(va & 0xFFFF_E000));
    memory(&mut p, REFILL_PAGE + 0x20, 0x5151);
    // Every page size (PageMask's table, §3.2): even and odd pages and the even page's last word, uncached.
    let sizes: [(u32, u32); 7] = [
        (0x0000_0000, 4 << 10),
        (0x0000_6000, 16 << 10),
        (0x0001_E000, 64 << 10),
        (0x0007_E000, 256 << 10),
        (0x001F_E000, 1 << 20),
        (0x007F_E000, 4 << 20),
        (0x01FF_E000, 16 << 20),
    ];
    let base = 0x4000_0000;
    for (k, &(mask, size)) in sizes.iter().enumerate() {
        let even = if size == 16 << 20 { 0 } else { 0x0080_0000 };
        let odd = even + size;
        map(&mut p, 1, mask, base, lo(even >> 12, 2, true, true, true), lo(odd >> 12, 2, true, true, true));
        let cases = [(base + size / 2 + 0x40, even + size / 2 + 0x40), (base + size + size / 2 + 0x80, odd + size / 2 + 0x80), (base + size - 4, even + size - 4)];
        for (i, &(_, pa)) in cases.iter().enumerate() {
            poke(&mut p, pa, 0x6000_0000 | (k as u32) << 8 | i as u32);
        }
        for (i, &(va, _)) in cases.iter().enumerate() {
            load(&mut p, va, sx(0x6000_0000 | (k as u32) << 8 | i as u32));
        }
    }
    // Past the 16 MiB pair: a refill.
    let va = base + (32 << 20) + 0x40;
    let at = load(&mut p, va, None);
    p.expected.pop();
    taken(&mut p, 2, at, false, Some(va), Some(context(va)), Some(va & 0xFFFF_E000));
    p.expected.push(refill_value(va));
    // TLBR (EE Core Instruction Set Manual, TLBWI and TLBR): VPN2 masked by PageMask, G the AND of both G bits.
    let (mask, hi) = (0x6000u32, 0x5001_E033u32);
    let (lo0, lo1) = (lo(0x500, 2, true, true, true), lo(0x502, 3, false, true, false));
    map(&mut p, 2, mask, hi, lo0, lo1);
    for r in ["PageMask", "EntryHi", "EntryLo0", "EntryLo1"] {
        p.emit(format!("mtc0 zero, {r}"));
    }
    p.emit("ori t0, zero, 2");
    p.emit("mtc0 t0, Index");
    p.emit("sync.p");
    p.emit("tlbr");
    p.emit("sync.p");
    let stored_hi = (hi & 0xFFFF_E000 | hi & 0xFFF) & !mask;
    cop0(&mut p, "PageMask", sx(mask));
    cop0(&mut p, "EntryHi", sx(stored_hi & 0xFFFF_E0FF & !mask));
    cop0(&mut p, "EntryLo0", sx(lo0 & !1));
    cop0(&mut p, "EntryLo1", sx(lo1 & !1));
    // TLBP: a match under the entry's mask and ASID gives its index; a mismatch sets P.
    li(&mut p, "t0", 0x5001_A033);
    p.emit("mtc0 t0, EntryHi");
    p.emit("sync.p");
    p.emit("tlbp");
    p.emit("sync.p");
    cop0(&mut p, "Index", sx(2));
    li(&mut p, "t0", 0x5001_A034);
    p.emit("mtc0 t0, EntryHi");
    p.emit("sync.p");
    p.emit("tlbp");
    p.emit("sync.p");
    p.emit("mfc0 t2, Index");
    p.emit("srl t2, t2, 31");
    p.record("t2", Some(1));
    // The ASID: the entry (G clear) translates under ASID 33h, and under 34h refills.
    poke(&mut p, 0x0050_0010, 0x7333);
    li(&mut p, "t0", 0x33);
    p.emit("mtc0 t0, EntryHi");
    p.emit("sync.p");
    load(&mut p, 0x5001_8010, sx(0x7333));
    li(&mut p, "t0", 0x34);
    p.emit("mtc0 t0, EntryHi");
    p.emit("sync.p");
    let va = 0x5001_8010;
    let at = load(&mut p, va, None);
    p.expected.pop();
    taken(&mut p, 2, at, false, Some(va), Some(context(va)), Some(va & 0xFFFF_E000 | 0x34));
    p.expected.push(refill_value(va));
    p.emit("mtc0 zero, EntryHi");
    p.emit("sync.p");
    // TLB Invalid (§4.2.8) on a load and a store, TLB Modified (§4.2.6) on a store to a clean page.
    map(&mut p, 3, 0, 0x6000_0000, lo(0x600, 2, true, false, true), lo(0x601, 2, false, true, true));
    poke(&mut p, 0x0060_1030, 0x7601);
    let va = 0x6000_0010;
    let at = load(&mut p, va, None);
    p.expected.pop();
    taken(&mut p, 2, at, false, Some(va), Some(context(va)), Some(0x6000_0000));
    p.expected.push(Some(SENTINEL));
    let va = 0x6000_0020;
    let at = store(&mut p, va, 1);
    taken(&mut p, 3, at, false, Some(va), Some(context(va)), Some(0x6000_0000));
    let va = 0x6000_1020;
    let at = store(&mut p, va, 2);
    taken(&mut p, 1, at, false, Some(va), Some(context(va)), Some(0x6000_0000));
    load(&mut p, 0x6000_1030, sx(0x7601));
    // A refill while EXL is set goes to the common vector and leaves EPC (§4.1.2).
    let va = 0x0FF0_0040;
    li(&mut p, "t6", va);
    let at = k0(p.here() + 4 * 6);
    li(&mut p, "t0", at);
    p.emit("mtc0 t0, EPC");
    p.emit("ori t0, zero, 2");
    p.emit("mtc0 t0, Status");
    p.emit("sync.p");
    assert_eq!(k0(p.here()), at);
    p.emit("lw t2, 0(t6)");
    taken(&mut p, 2, at, false, Some(va), Some(context(va)), Some(va & 0xFFFF_E000));
    // kuseg unmapped and uncached while ERL is set (§5.1.6).
    poke(&mut p, REFILL_PAGE + 0x80, 0x7080);
    p.emit("ori t0, zero, 4");
    p.emit("mtc0 t0, Status");
    p.emit("sync.p");
    li(&mut p, "t6", REFILL_PAGE + 0x80);
    p.emit("lw t2, 0(t6)");
    p.emit("mtc0 zero, Status");
    p.emit("sync.p");
    p.record("t2", sx(0x7080));
    // ksseg is mapped in kernel mode: a refill.
    let va = 0xC000_0040;
    let at = load(&mut p, va, None);
    p.expected.pop();
    taken(&mut p, 2, at, false, Some(va), Some(context(va)), Some(va & 0xFFFF_E000));
    p.expected.push(refill_value(va));
    p
}

/// The scratchpad through two S entries at once, LQ and SQ there, RAM under the ignored PFN untouched, TLBR's C of
/// 2, and the modified and invalid exceptions of S entries (§5.2.5, §6.4.3).
pub fn scratchpad() -> Program {
    let mut p = Program::kernel("s7-spr");
    prologue(&mut p);
    let s = 0x8000_0000u32;
    let lo0 = s | lo(REFILL_PAGE >> 12, 3, true, true, true);
    let lo1 = lo(0, 3, true, true, true);
    map(&mut p, 3, 0, 0x7100_0000, lo0, lo1);
    map(&mut p, 4, 0, 0x5000_4000, lo0, lo1);
    poke(&mut p, REFILL_PAGE, 0x3000);
    poke(&mut p, REFILL_PAGE + 0x2000, 0x3200);
    store(&mut p, 0x7100_0000, 0xA0A0);
    store(&mut p, 0x7100_2000, 0xB0B0);
    let q: u128 = 0x0F1E_2D3C_4B5A_6978_8796_A5B4_C3D2_E1F0;
    p.quad("t3", q);
    li(&mut p, "t6", 0x7100_3FF0);
    p.emit("sq t3, 0(t6)");
    load(&mut p, 0x5000_4000, sx(0xA0A0));
    load(&mut p, 0x5000_6000, sx(0xB0B0));
    for (off, half) in [(0x7FF0u32, q as u64), (0x7FF8, (q >> 64) as u64)] {
        li(&mut p, "t6", 0x5000_0000 | off);
        p.emit("ld t2, 0(t6)");
        p.record("t2", Some(half));
    }
    p.emit("lq t4, 0(t6)");
    p.emit("sd t4, 0(s1)");
    p.emit("daddiu s1, s1, 8");
    p.expected.push(Some(q as u64));
    memory(&mut p, REFILL_PAGE, 0x3000);
    memory(&mut p, REFILL_PAGE + 0x2000, 0x3200);
    p.emit("ori t0, zero, 3");
    p.emit("mtc0 t0, Index");
    p.emit("sync.p");
    p.emit("tlbr");
    p.emit("sync.p");
    let c2 = |l: u32| l & !(7 << 3) | 2 << 3;
    cop0(&mut p, "EntryLo0", sx(c2(lo0)));
    cop0(&mut p, "EntryLo1", sx(c2(lo1)));
    cop0(&mut p, "EntryHi", sx(0x7100_0000));
    // A clean S entry reads the scratchpad and refuses a store; an invalid one refuses a load.
    map(&mut p, 5, 0, 0x7100_4000, s | lo(0, 3, false, true, true), lo(0, 3, false, true, true));
    map(&mut p, 6, 0, 0x7100_8000, s | lo(0, 3, false, false, true), lo(0, 3, false, false, true));
    load(&mut p, 0x7100_4000, sx(0xA0A0));
    let va = 0x7100_4010;
    let at = store(&mut p, va, 3);
    taken(&mut p, 1, at, false, Some(va), Some(context(va)), Some(0x7100_4000));
    let va = 0x7100_8020;
    let at = load(&mut p, va, None);
    p.expected.pop();
    taken(&mut p, 2, at, false, Some(va), Some(context(va)), Some(0x7100_8000));
    p.expected.push(Some(SENTINEL));
    p
}

/// The data cache (chapter 6): write-back, the hit operations and Status.CH, the LRF replacement of Table 6-4 with
/// a dirty victim written back, the index operations and the tags they read, a locked line, write-allocate, and
/// the uncached accelerated pages.
pub fn data_cache() -> Program {
    let mut p = Program::kernel("s7-dcache");
    prologue(&mut p);
    let cached = |pfn: u32| lo(pfn, 3, true, true, true);
    map(&mut p, 0, 0, 0x4000_0000, cached(0x400), cached(0x401));
    map(&mut p, 1, 0, 0x4000_2000, cached(0x402), cached(0x403));
    map(&mut p, 2, 0, 0x4001_0000, cached(0x410), cached(0x411));
    map(&mut p, 3, 0, 0x4001_2000, cached(0x412), cached(0x413));
    map(&mut p, 4, 0, 0x4002_0000, lo(0x420, 7, true, true, true), lo(0x421, 7, true, true, true));
    map(&mut p, 5, 0, 0x4003_0000, cached(0x430), cached(0x431));
    let (a, b, c) = (0x4000_0000u32, 0x4000_1000u32, 0x4000_2000u32);
    let (pa, pb, pc) = (0x0040_0000u32, 0x0040_1000u32, 0x0040_2000u32);
    let tag = |p: &mut Program, reg_va: u32, expected: u32| {
        li(p, "t6", reg_va);
        p.emit("sync");
        p.emit("cache 0x10, 0(t6)");
        p.emit("sync");
        cop0(p, "TagLo", sx(expected));
    };
    let op = |p: &mut Program, code: u32, va: u32| {
        li(p, "t6", va);
        p.emit("sync");
        p.emit(format!("cache 0x{code:x}, 0(t6)"));
        p.emit("sync");
    };
    let ch = |p: &mut Program, hit: bool| {
        p.emit("mfc0 t2, Status");
        li(p, "t3", 0x4_0000);
        p.emit("and t2, t2, t3");
        p.record("t2", Some(if hit { 0x4_0000 } else { 0 }));
    };
    // Set 0, both ways invalid with LRF 0 and 0 after the prologue.
    poke(&mut p, pa, 0x1111);
    load(&mut p, a, sx(0x1111)); // a miss fills way 0 (LRF 0^0), way 0's LRF flips to 1
    store(&mut p, a, 0x2222); // a hit: the line dirty, memory unchanged
    memory(&mut p, pa, 0x1111);
    load(&mut p, a, sx(0x2222));
    tag(&mut p, 0x8000_0000, 0x0040_0070); // way 0: PFN 400h, D, V, R
    tag(&mut p, 0x8000_0001, 0);
    op(&mut p, 0x1C, a); // DHWOIN: written back, kept clean
    memory(&mut p, pa, 0x2222);
    tag(&mut p, 0x8000_0000, 0x0040_0030);
    poke(&mut p, pa, 0x3333); // behind the cache's back
    load(&mut p, a, sx(0x2222));
    op(&mut p, 0x1A, a); // DHIN hits: way 0 invalid, its LRF kept at 1
    ch(&mut p, true);
    load(&mut p, a, sx(0x3333)); // a miss: LRF 1^0 picks way 1, which flips to 1
    tag(&mut p, 0x8000_0001, 0x0040_0030);
    op(&mut p, 0x1A, b); // DHIN misses
    ch(&mut p, false);
    store(&mut p, a, 0x4444); // way 1 dirty
    op(&mut p, 0x1A, a); // DHIN discards the dirty data
    memory(&mut p, pa, 0x3333);
    load(&mut p, a, sx(0x3333)); // LRF 1^1 picks way 0, which flips to 0
    // Three lines of set 0: B into way 1 (0^1), C into way 0 (0^0) over clean A, then A into way 1 (1^0)
    // over dirty B, which is written back first.
    poke(&mut p, pb, 0xB0);
    poke(&mut p, pc, 0xC0);
    store(&mut p, b, 0xBBBB);
    store(&mut p, c, 0xCCCC);
    load(&mut p, a, sx(0x3333));
    memory(&mut p, pb, 0xBBBB);
    memory(&mut p, pc, 0xC0);
    load(&mut p, c, sx(0xCCCC));
    op(&mut p, 0x14, 0x8000_0000); // DXWBIN of set 0, way 0: C written back, the line invalid, LRF kept
    memory(&mut p, pc, 0xCCCC);
    tag(&mut p, 0x8000_0000, 0x0040_2010);
    // A locked line in set 1 (§6.3.7): a store stays in the cache and leaves D, refills take the other way.
    let (e, f, g) = (0x4001_0040u32, 0x4001_1040u32, 0x4001_2040u32);
    poke(&mut p, 0x0041_0040, 0xE0);
    poke(&mut p, 0x0041_1040, 0xF0);
    poke(&mut p, 0x0041_2040, 0x60);
    li(&mut p, "t0", 0x0041_0068);
    p.emit("mtc0 t0, TagLo");
    op(&mut p, 0x12, 0x8000_0040); // DXSTG way 0: PFN 410h, D, V, L
    store(&mut p, e, 0xEEEE);
    memory(&mut p, 0x0041_0040, 0xE0);
    load(&mut p, f, sx(0xF0)); // way 1, LRF 0 to 1
    load(&mut p, g, sx(0x60)); // way 1 again over clean F, LRF back to 0
    load(&mut p, e, sx(0xEEEE));
    tag(&mut p, 0x8000_0040, 0x0041_0068);
    tag(&mut p, 0x8000_0041, 0x0041_2020);
    li(&mut p, "t0", 0x0041_0060);
    p.emit("mtc0 t0, TagLo");
    op(&mut p, 0x12, 0x8000_0040); // unlocked, still dirty
    op(&mut p, 0x18, e); // DHWBIN: written back
    ch(&mut p, true);
    memory(&mut p, 0x0041_0040, 0xEEEE);
    // Write-allocate (§5.2.6): a store miss reads the line, so its other words come from memory.
    let h = 0x4003_0080u32;
    poke(&mut p, 0x0043_0080, 0x8000);
    poke(&mut p, 0x0043_0084, 0x4444);
    store(&mut p, h, 0x8888);
    load(&mut p, h + 4, sx(0x4444));
    memory(&mut p, 0x0043_0080, 0x8000);
    load(&mut p, h, sx(0x8888));
    // Uncached accelerated pages: loads read memory, a store reaches it and the next load sees it.
    poke(&mut p, 0x0042_0000, 0x77);
    poke(&mut p, 0x0042_0004, 0x78);
    load(&mut p, 0x4002_0000, sx(0x77));
    store(&mut p, 0x4002_0004, 0x7A);
    memory(&mut p, 0x0042_0004, 0x7A);
    load(&mut p, 0x4002_0004, sx(0x7A));
    p
}

/// The instruction cache keeps code a store has replaced until CACHE IHIN or IXIN invalidates its line (§6.3.6).
pub fn instruction_cache() -> Program {
    let mut p = Program::kernel("s7-icache");
    prologue(&mut p);
    let (sub, past) = (p.label(), p.label());
    p.emit(format!("beq zero, zero, {past}"));
    p.emit("nop");
    while !p.here().is_multiple_of(64) {
        p.emit("nop");
    }
    let at = p.here();
    p.place(&sub);
    p.emit("addiu t0, zero, 1");
    p.emit("jr ra");
    p.emit("nop");
    p.place(&past);
    let call = |p: &mut Program, expected: u64| {
        p.emit("or t0, zero, zero");
        p.emit(format!("jal {sub}"));
        p.emit("nop");
        p.record("t0", Some(expected));
    };
    let replace = |p: &mut Program, v: u32| {
        let w = assemble(&format!("addiu t0, zero, {v}"), 0)[0];
        poke(p, at, w);
    };
    call(&mut p, 1);
    replace(&mut p, 2);
    call(&mut p, 1);
    li(&mut p, "t1", k0(at));
    p.emit("sync.p");
    p.emit("cache 0x0b, 0(t1)");
    p.emit("sync.p");
    call(&mut p, 2);
    replace(&mut p, 3);
    call(&mut p, 2);
    li(&mut p, "t1", at & 0x1FC0);
    p.emit("sync.p");
    p.emit("cache 0x07, 0(t1)");
    p.emit("sync.p");
    p.emit("cache 0x07, 1(t1)");
    p.emit("sync.p");
    call(&mut p, 3);
    p
}

/// Count, Compare's interrupt and its acknowledgement, and the performance counters: their registers' moves, the
/// events counted in each mode, and the overflow's level 2 exception (chapter 7).
pub fn counters() -> Program {
    let mut p = Program::kernel("s7-counters");
    prologue(&mut p);
    // Count advances once a cycle: 100 instructions take between 50 and 1,000 cycles.
    p.emit("mfc0 t0, Count");
    for _ in 0..100 {
        p.emit("nop");
    }
    p.emit("mfc0 t1, Count");
    p.emit("subu t2, t1, t0");
    p.emit("sltiu t3, t2, 1000");
    p.emit("sltiu t4, t2, 50");
    p.record("t3", Some(1));
    p.record("t4", Some(0));
    // Compare's interrupt through IM7 (§3.2): IP7 set and ExcCode 0, cleared by writing Compare.
    let (spin, out) = (p.label(), p.label());
    p.emit("or t8, zero, zero");
    p.emit("mfc0 t0, Count");
    p.emit("addiu t0, t0, 400");
    p.emit("mtc0 t0, Compare");
    li(&mut p, "t1", 0x0001_8001);
    p.emit("mtc0 t1, Status");
    p.emit("sync.p");
    p.emit("lui t2, 0x10");
    p.place(&spin);
    p.emit(format!("bne t8, zero, {out}"));
    p.emit("addiu t2, t2, -1");
    p.emit(format!("bne t2, zero, {spin}"));
    p.emit("nop");
    p.place(&out);
    p.emit("mtc0 zero, Status");
    p.emit("sync.p");
    p.expected.push(Some(0x8000));
    p.expected.push(Some(0));
    p.record("t8", Some(1));
    // The registers' moves, as ps2autotests' cpu/ee_cop0/performance records them on a console.
    li(&mut p, "t0", 0x7FFF_FFFF);
    p.emit("mtps t0, 0");
    p.emit("sync.p");
    p.emit("mfps t1, 0");
    p.record("t1", sx(0x000F_FBFE));
    p.emit("mtps zero, 1");
    p.emit("sync.p");
    p.emit("mfps t1, 3");
    p.record("t1", sx(0x000F_FBFE));
    p.emit("mtps zero, 0");
    li(&mut p, "t0", 0x89AB_CDEF);
    p.emit("mtpc t0, 0");
    li(&mut p, "t0", 0x0123_4567);
    p.emit("mtpc t0, 3");
    p.emit("sync.p");
    p.emit("mfpc t1, 2");
    p.record("t1", sx(0x89AB_CDEF));
    p.emit("mfpc t1, 1");
    p.record("t1", sx(0x0123_4567));
    // Events counted over two bodies of different length; the difference is the events of the extra part.
    let measure = |p: &mut Program, pccr: u32, exl: bool, body: &dyn Fn(&mut Program), out0: &str, out1: &str| {
        p.emit("mtpc zero, 0");
        p.emit("mtpc zero, 1");
        p.emit("sync.p");
        li(p, "t0", pccr);
        p.emit("mtps t0, 0");
        p.emit("sync.p");
        if exl {
            p.emit("ori t1, zero, 2");
            p.emit("mtc0 t1, Status");
            p.emit("sync.p");
        }
        body(p);
        if exl {
            p.emit("mtc0 zero, Status");
            p.emit("sync.p");
        }
        p.emit("mtps zero, 0");
        p.emit("sync.p");
        p.emit(format!("mfpc {out0}, 0"));
        p.emit(format!("mfpc {out1}, 1"));
    };
    let nops = |n: usize| move |p: &mut Program| (0..n).for_each(|_| p.emit("nop"));
    let difference = |p: &mut Program, a: &str, b: &str, expected: u64| {
        p.emit(format!("subu t2, {b}, {a}"));
        p.record("t2", Some(expected));
    };
    // Instructions completed (event 12) in kernel mode: 20 more.
    measure(&mut p, 0x8000_0184, false, &nops(10), "t4", "t5");
    measure(&mut p, 0x8000_0184, false, &nops(30), "t6", "t7");
    difference(&mut p, "t4", "t6", 20);
    // Loads completed on PCR0 and stores completed on PCR1 (event 15 on each): 8 and 3 more.
    let memory_ops = |loads: usize, stores: usize| {
        move |p: &mut Program| {
            p.emit("lui t3, 0xa030");
            (0..loads).for_each(|_| p.emit("lw t1, 0(t3)"));
            (0..stores).for_each(|_| p.emit("sw zero, 4(t3)"));
        }
    };
    measure(&mut p, 0x8007_91E4, false, &memory_ops(2, 1), "t4", "t5");
    measure(&mut p, 0x8007_91E4, false, &memory_ops(10, 4), "t6", "t7");
    difference(&mut p, "t4", "t6", 8);
    difference(&mut p, "t5", "t7", 3);
    // The level 1 handler's mode (EXL0) counts at EXL; kernel mode's bit (K0) does not.
    measure(&mut p, 0x8000_0182, true, &nops(10), "t4", "t5");
    measure(&mut p, 0x8000_0182, true, &nops(30), "t6", "t7");
    difference(&mut p, "t4", "t6", 20);
    measure(&mut p, 0x8000_0184, true, &nops(10), "t4", "t5");
    measure(&mut p, 0x8000_0184, true, &nops(30), "t6", "t7");
    difference(&mut p, "t4", "t6", 0);
    // An overflow of PCR0 takes the level 2 counter exception (§7.2.4): EXC2 2, ERL set, OVFL set.
    p.emit("or t8, zero, zero");
    li(&mut p, "t0", 0x7FFF_FFF0);
    p.emit("mtpc t0, 0");
    p.emit("sync.p");
    li(&mut p, "t0", 0x8000_0184);
    p.emit("mtps t0, 0");
    p.emit("sync.p");
    for _ in 0..40 {
        p.emit("nop");
    }
    p.expected.extend([Some(0x2_0000), Some(4), Some(1)]);
    p.record("t8", Some(1));
    p
}

/// The breakpoints (chapter 9): an instruction address breakpoint, BED, the mode bits, a data read breakpoint, and
/// a data value breakpoint on stores.
pub fn breakpoints() -> Program {
    let mut p = Program::kernel("s7-debug");
    prologue(&mut p);
    let bp = |p: &mut Program, iab: u32, bpc: u32| {
        p.emit("sync.p");
        li(p, "t0", iab);
        p.emit("mtiab t0");
        li(p, "t0", 0xFFFF_FFFC);
        p.emit("mtiabm t0");
        li(p, "t0", bpc);
        p.emit("mtbpc t0");
        p.emit("sync.p");
    };
    let mfbpc = |p: &mut Program, expected: u32| {
        p.emit("mfbpc t2");
        p.record("t2", sx(expected));
        p.emit("mtbpc zero");
        p.emit("sync.p");
    };
    // IAE and IKE: the debug exception at the instruction, before it runs (§4.2.4; EXC2 per D-12).
    for (bpc, fires) in [(0x8100_0000u32, true), (0x8100_8000, false), (0x8400_0000, false)] {
        let target = k0(p.here() + 4 * 11);
        bp(&mut p, target, bpc);
        assert_eq!(k0(p.here()), target);
        p.emit("addiu t2, zero, 0x55");
        if fires {
            p.expected.extend([sx(0x3_0000), sx(target), sx(bpc | 1)]);
        }
        p.record("t2", Some(0x55));
        mfbpc(&mut p, if fires { bpc & 0x1FFF_FFF8 } else if bpc & 0x8000 != 0 { bpc | 1 } else { bpc });
    }
    // DRE and DKE: a load at DAB stops before it completes, and runs once the handler clears DRE.
    let data = 0x0030_0040u32;
    poke(&mut p, data, 0x4040);
    li(&mut p, "t0", k1(data));
    p.emit("mtdab t0");
    p.emit("addiu t0, zero, -1");
    p.emit("mtdabm t0");
    li(&mut p, "t0", 0x4008_0000);
    p.emit("mtbpc t0");
    p.emit("sync.p");
    let at = load(&mut p, k1(data), Some(0x4040));
    let rec = p.expected.pop().unwrap();
    p.expected.extend([sx(0x3_0000), sx(at), sx(0x4008_0002), rec]);
    mfbpc(&mut p, 0x0008_0000);
    // DWE, DVE and DKE: a store of a value DVB does not match passes; one that matches stops.
    li(&mut p, "t0", 0x1234);
    p.emit("mtdvb t0");
    li(&mut p, "t0", 0xFFFF);
    p.emit("mtdvbm t0");
    li(&mut p, "t0", 0x3008_0000);
    p.emit("mtbpc t0");
    p.emit("sync.p");
    store(&mut p, k1(data), 0x5555);
    memory(&mut p, data, 0x5555);
    let at = store(&mut p, k1(data), 0x00AB_1234);
    p.expected.extend([sx(0x3_0000), sx(at), sx(0x3008_0004)]);
    memory(&mut p, data, 0x00AB_1234);
    mfbpc(&mut p, 0x1008_0000);
    p
}

/// Bus errors (EE User's Manual §2.1.2) on a load and a store mapped past 20000000h: ExcCode 7, BadPAddr, BEM set,
/// and BEM masking the next.
pub fn bus_errors() -> Program {
    let mut p = Program::kernel("s7-buserror");
    prologue(&mut p);
    map(&mut p, 0, 0, 0x4000_0000, lo(0x2_0000, 2, true, true, true), lo(0x2_0001, 2, true, true, true));
    let at = load(&mut p, 0x4000_0010, None);
    p.expected.pop();
    taken(&mut p, 7, at, false, None, Some(0xFF80_0000), Some(0));
    p.expected.push(Some(SENTINEL));
    cop0(&mut p, "BadPAddr", sx(0x2000_0010));
    p.emit("mfc0 t2, Status");
    p.emit("andi t2, t2, 0x1000");
    p.record("t2", Some(0x1000));
    li(&mut p, "t6", 0x4000_0020);
    p.emit("lw t2, 0(t6)");
    p.emit("addiu t3, zero, 0x77");
    p.record("t3", Some(0x77));
    p.emit("mtc0 zero, Status");
    p.emit("sync.p");
    let at = store(&mut p, 0x4000_1030, 5);
    taken(&mut p, 7, at, false, None, Some(0xFF80_0000), Some(0));
    cop0(&mut p, "BadPAddr", sx(0x2000_1030));
    p
}

/// Every program, in the order the referees run them.
pub fn all() -> Vec<Program> {
    vec![tlb(), scratchpad(), data_cache(), instruction_cache(), counters(), breakpoints(), bus_errors()]
}

/// A program of the prologue's first `steps` parts and one record word, to find where a referee stops.
pub fn probe(steps: u32) -> Program {
    let mut p = Program::kernel(["s7-probe0", "s7-probe1", "s7-probe2", "s7-probe3", "s7-probe4", "s7-probe5", "s7-probe6", "s7-probe7", "s7-probe8", "s7-probe9"][steps as usize]);
    prologue_to(&mut p, steps);
    p.emit("ori t2, zero, 0x1234");
    p.record("t2", Some(0x1234));
    p
}
