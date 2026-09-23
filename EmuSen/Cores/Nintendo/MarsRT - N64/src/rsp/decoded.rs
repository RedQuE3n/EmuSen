//! The processor's instruction memory decoded once a word: each word's handler chosen when it is first run, checked against the word
//! in IMEM before every use, and straight runs of pure instructions taken without the program counter's per-step bookkeeping.
//! The handlers are the interpreter's own arms. See Mars_Native.md §6.12.

#[cfg(target_arch = "x86_64")]
use super::simd::{SHUFFLED, WHOLE};
use super::{Memory, PC_MASK, Rsp, VECTOR_OPERATION, rd, rs, rt};
use crate::memory::sp::Lent;

/// Instruction memory in words.
pub const WORDS: usize = 1024;
/// The longest straight run an entry records; a hint, since every word is checked as it runs.
const LONGEST: u32 = 64;
/// The entry's `info`: the straight run's length from here, and the two kinds that are not pure.
const LENGTH: u32 = 0xFF;
const EVENT: u32 = 1 << 8;
const BRANCH: u32 = 1 << 9;

/// One instruction's handler, given the instruction; it neither fetches nor moves the program counter.
pub type Handler = unsafe fn(&mut Rsp<Lent<'_>>, u32);

#[derive(Clone, Copy, Debug)]
#[repr(C, align(16))]
pub struct Entry {
    handler: Handler,
    /// The four bytes of IMEM this entry was decoded from, as the host reads them: the whole of its validity.
    raw: u32,
    info: u32,
}

/// The table, one entry a word, and whether the machine uses it; in no state and no equality, since every entry is checked against IMEM.
#[derive(Clone)]
pub struct Decoded {
    pub on: bool,
    entries: Box<[Entry; WORDS]>,
    /// Entries decoded since the table was made, for measurement.
    pub decodes: u64,
}

impl std::fmt::Debug for Decoded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Decoded {{ on: {}, decodes: {} }}", self.on, self.decodes)
    }
}

/// The default: on, unless `EMUSEN_MARSRT_RSP_BLOCKS=0`.
pub fn default_on() -> bool {
    static DEFAULT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *DEFAULT.get_or_init(|| std::env::var("EMUSEN_MARSRT_RSP_BLOCKS").map_or(true, |v| v != "0"))
}

impl Default for Decoded {
    fn default() -> Self {
        let mut table = Decoded { on: default_on(), entries: Box::new([decode(0); WORDS]), decodes: 0 };
        for index in (0..WORDS).rev() {
            table.entries[index].info = table.length_at(index);
        }
        table
    }
}

#[inline(always)]
fn raw_at(imem: &[u8; 4096], at: usize) -> u32 {
    u32::from_ne_bytes(imem[at..at + 4].try_into().unwrap())
}

#[inline(always)]
fn pure(info: u32) -> bool {
    info & (EVENT | BRANCH) == 0
}

impl Decoded {
    /// `Rsp::step` through the table: one instruction unless it is an event, which is left unrun; returns whether it ran.
    #[inline(always)]
    pub fn step(&mut self, r: &mut Rsp<Lent<'_>>) -> bool {
        let at = (r.m.pc() & PC_MASK) as usize;
        let raw = raw_at(r.m.imem, at);
        let mut e = self.entries[at >> 2];
        if e.raw != raw {
            e = self.redecode(at >> 2, raw);
        }
        if e.info & EVENT != 0 {
            return false;
        }
        let next = r.m.next_pc();
        r.m.set_pc(next);
        r.m.set_next_pc((next + 4) & PC_MASK);
        // SAFETY: a handler that needs the host's vector features is chosen only where `simd_supported` found them.
        unsafe { (e.handler)(r, u32::from_be(raw)) };
        true
    }

    /// `Rsp::run` through the table: up to `budget` instructions, stopping before an event, straight runs taken whole.
    pub fn run(&mut self, r: &mut Rsp<Lent<'_>>, budget: u64) -> u64 {
        // Only an event halts the processor, and the run stops before one, so the halt is tested once.
        if r.m.halted() != 0 {
            return 0;
        }
        let mut ran = 0;
        while ran < budget {
            let pc = r.m.pc() & PC_MASK;
            if r.m.next_pc() == (pc + 4) & PC_MASK {
                let length = (self.entries[(pc >> 2) as usize].info & LENGTH) as u64;
                if length > 1 {
                    let k = self.straight(r, pc, length.min(budget - ran));
                    if k > 0 {
                        ran += k;
                        continue;
                    }
                }
            }
            if !self.step(r) {
                break;
            }
            ran += 1;
        }
        ran
    }

    /// Up to `n` pure instructions from `pc`, outside a delay slot, each checked against IMEM as it runs; the counters moved once at the end.
    #[inline(always)]
    fn straight(&mut self, r: &mut Rsp<Lent<'_>>, pc: u32, n: u64) -> u64 {
        let base = (pc >> 2) as usize;
        let mut k = 0usize;
        while (k as u64) < n {
            let index = base + k;
            let raw = raw_at(r.m.imem, index << 2);
            let mut e = self.entries[index];
            if e.raw != raw {
                e = self.redecode(index, raw);
                if !pure(e.info) {
                    break;
                }
            }
            // SAFETY: as in `step`.
            unsafe { (e.handler)(r, u32::from_be(raw)) };
            k += 1;
        }
        if k > 0 {
            let end = (pc + 4 * k as u32) & PC_MASK;
            r.m.set_pc(end);
            r.m.set_next_pc((end + 4) & PC_MASK);
        }
        k as u64
    }

    /// The entry for a word IMEM now holds, and the straight runs before it measured again.
    #[cold]
    #[inline(never)]
    fn redecode(&mut self, index: usize, raw: u32) -> Entry {
        self.decodes += 1;
        self.entries[index] = decode(raw);
        self.entries[index].info |= self.length_at(index) & LENGTH;
        for before in (index.saturating_sub(LONGEST as usize)..index).rev() {
            let info = self.length_at(before);
            if info == self.entries[before].info {
                break;
            }
            self.entries[before].info = info;
        }
        self.entries[index]
    }

    /// An entry's info from its own kind and the next entry's run, as the table holds them.
    fn length_at(&self, index: usize) -> u32 {
        let kind = self.entries[index].info & (EVENT | BRANCH);
        if kind != 0 {
            return kind;
        }
        let after = if index + 1 < WORDS { self.entries[index + 1].info } else { EVENT };
        let run = if pure(after) { (after & LENGTH) + 1 } else { 1 };
        run.min(LONGEST).min((WORDS - index) as u32)
    }
}

/// The entry for one word of IMEM, read as the host reads it.
fn decode(raw: u32) -> Entry {
    let i = u32::from_be(raw);
    let info = if Rsp::<Lent<'_>>::is_event(i) {
        EVENT
    } else if branches(i) {
        BRANCH
    } else {
        0
    };
    Entry { handler: handler(i), raw, info }
}

/// A branch or a jump: the kinds that read or move the program counter.
fn branches(i: u32) -> bool {
    let op = i >> 26;
    (1..=7).contains(&op) || (op == 0 && matches!(i & 0x3F, 0x08 | 0x09))
}

/// The handler: the interpreter's arm for the opcode and function, with the instructions whose only effect is on `r0` doing nothing.
fn handler(i: u32) -> Handler {
    let (op, function) = (i >> 26, (i & 0x3F) as usize);
    let (target, destination) = (rt(i), rd(i));
    match op {
        0x00 => match function {
            0x08 | 0x09 => SPECIAL[function],
            0x0D => nothing,
            _ if destination == 0 => nothing,
            _ => SPECIAL[function],
        },
        0x01 => regimm,
        0x08..=0x0F if target == 0 => nothing,
        0x10 => nothing,
        0x12 if i & VECTOR_OPERATION != 0 => vector(i),
        0x12 if matches!(rs(i), 0x00 | 0x02) && target == 0 => nothing,
        0x12 => cop2,
        0x20 | 0x21 | 0x23 | 0x24 | 0x25 | 0x27 if target == 0 => nothing,
        #[cfg(target_arch = "x86_64")]
        0x32 if super::simd_supported() => load_simd,
        #[cfg(target_arch = "x86_64")]
        0x3A if super::simd_supported() => store_simd,
        _ => MAIN[op as usize],
    }
}

#[cfg(target_arch = "x86_64")]
fn vector(i: u32) -> Handler {
    if !super::simd_supported() {
        return MAIN[0x12];
    }
    let function = (i & 0x3F) as usize;
    if (i >> 21) & 0xF <= 1 { VECTOR_WHOLE[function] } else { VECTOR_SHUFFLED[function] }
}

#[cfg(not(target_arch = "x86_64"))]
fn vector(_: u32) -> Handler {
    MAIN[0x12]
}

fn nothing(_: &mut Rsp<Lent<'_>>, _: u32) {}

fn main<const OP: u32>(r: &mut Rsp<Lent<'_>>, i: u32) {
    r.main::<OP>(i)
}

fn special<const F: u32>(r: &mut Rsp<Lent<'_>>, i: u32) {
    r.special_function::<F>(i)
}

fn regimm(r: &mut Rsp<Lent<'_>>, i: u32) {
    r.regimm(i)
}

fn cop2(r: &mut Rsp<Lent<'_>>, i: u32) {
    r.cop2(i)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2,ssse3,sse4.1")]
fn vector_whole<const F: u32>(r: &mut Rsp<Lent<'_>>, i: u32) {
    if r.m.simd() { r.vector_lanes::<F, WHOLE>(i) } else { r.cop2(i) }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2,ssse3,sse4.1")]
fn vector_shuffled<const F: u32>(r: &mut Rsp<Lent<'_>>, i: u32) {
    if r.m.simd() { r.vector_lanes::<F, SHUFFLED>(i) } else { r.cop2(i) }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2,ssse3,sse4.1")]
fn load_simd(r: &mut Rsp<Lent<'_>>, i: u32) {
    if r.m.simd() { r.vector_load_simd(i) } else { r.vector_load(i) }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2,ssse3,sse4.1")]
fn store_simd(r: &mut Rsp<Lent<'_>>, i: u32) {
    if r.m.simd() { r.vector_store_simd(i) } else { r.vector_store(i) }
}

macro_rules! table {
    ($f:ident; $($n:literal)*) => {
        [$($f::<$n> as Handler,)*]
    };
}

static MAIN: [Handler; 64] = sixty_four!(table!(main));
static SPECIAL: [Handler; 64] = sixty_four!(table!(special));
#[cfg(target_arch = "x86_64")]
static VECTOR_WHOLE: [Handler; 64] = sixty_four!(table!(vector_whole));
#[cfg(target_arch = "x86_64")]
static VECTOR_SHUFFLED: [Handler; 64] = sixty_four!(table!(vector_shuffled));
