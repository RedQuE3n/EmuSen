//! For measurement only: the emitter's smaller exact forms and the shadow byte count (Sapphire_Redesign.md, "The emitter's exact forms").

use std::cell::Cell;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use cranelift_codegen::control::ControlPlane;
use cranelift_codegen::ir::SourceLoc;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::JITModule;
use cranelift_module::Module;

use super::shape::{self, Class};
use super::{Options, emit};

/// The data cache's hit inline, every other case of a load or store out of line.
pub const CACHE: u8 = 1;
/// Each compiled load and store that stays in its block counted by its page's kind (`kind`).
pub const COUNT: u8 = 8;
static KINDS: [AtomicU64; 4] = [const { AtomicU64::new(0) }; 4];

/// The address of `KINDS`, which `COUNT`'s code adds to.
pub fn kinds() -> i64 {
    KINDS.as_ptr() as i64
}

thread_local! {
    static LEVERS: Cell<u8> = const { Cell::new(0) };
    static STATIC: Cell<bool> = const { Cell::new(false) };
    static BASE: Cell<u32> = const { Cell::new(0) };
}

/// The levers `EMUSEN_SAPPHIRE_FORMS` names (`cache`, `count`, comma-separated).
pub fn chosen() -> u8 {
    static CHOSEN: OnceLock<u8> = OnceLock::new();
    *CHOSEN.get_or_init(|| {
        let v = std::env::var("EMUSEN_SAPPHIRE_FORMS").unwrap_or_default();
        v.split(',').map(|s| match s.trim() {
            "cache" => CACHE,
            "count" => COUNT,
            _ => 0,
        }).fold(0, |a, b| a | b)
    })
}

pub fn set(levers: u8) {
    LEVERS.with(|c| c.set(levers));
}

pub fn on(lever: u8) -> bool {
    LEVERS.with(|c| c.get()) & lever != 0
}

/// The static prototype of R0.2: C1's constant issue charges, no replay store, no update of Random, no undo log.
pub fn is_static() -> bool {
    STATIC.with(|s| s.get())
}

/// The purposes a tag numbers, 0 to 11.
pub const PURPOSES: [&str; 12] = ["untagged", "entry", "timing", "guest registers", "horizon and line checks", "address and alignment", "translation", "cache tags and dispatch", "data access", "the instruction's work", "exits' write-back", "exception checks"];

pub fn tag(b: &mut FunctionBuilder, t: u32) {
    BASE.with(|c| c.set(t));
    b.set_srcloc(SourceLoc::new(t));
}

pub fn push(b: &mut FunctionBuilder, t: u32) -> u32 {
    let p = BASE.with(|c| c.get());
    tag(b, t);
    p
}

/// The forms the shadow compiles: today's, C1, the static prototype, and the last two with the chosen levers.
const FORMS: [&str; 5] = ["today", "C1", "static", "static+levers", "today+levers"];
const N: usize = FORMS.len();
/// For each form: total, hot, cold bytes, hot-tagged bytes past the first cold one, then the hot bytes by purpose.
const ROW: usize = 4 + 12;
static BYTES: [AtomicU64; N * ROW] = [const { AtomicU64::new(0) }; N * ROW];
/// Pure IR operations of hot blocks by purpose, before optimisation, for each form.
static PURE: [AtomicU64; N * 12] = [const { AtomicU64::new(0) }; N * 12];
/// Guest words, blocks, and the words by class: memory, plain, branch.
static WORDS: [AtomicU64; 5] = [const { AtomicU64::new(0) }; 5];
static DUMPED: AtomicU64 = AtomicU64::new(0);

fn purpose(l: SourceLoc) -> usize {
    if l.is_default() || l.bits() > 11 { 0 } else { l.bits() as usize }
}

/// Compiles the block again outside the module in every form and adds up their bytes.
pub fn shadow(module: &JITModule, functions: &mut FunctionBuilderContext, words: &[u32], start: u32, options: Options) {
    let c1 = Options { fixed_cost: true, ..options };
    let levers = chosen();
    let forms = [(options, false, 0), (c1, false, 0), (c1, true, 0), (c1, true, levers), (options, false, levers)];
    let mut row = [[0u64; ROW]; N];
    let mut pure = [[0u64; 12]; N];
    let kept = LEVERS.with(|c| c.get());
    for (k, (o, st, lv)) in forms.into_iter().enumerate() {
        let mut ctx = cranelift_codegen::Context::new();
        ctx.func.signature = module.make_signature();
        STATIC.with(|s| s.set(st));
        set(lv);
        let r = emit::lower(module, &mut ctx.func, functions, words, start, o);
        STATIC.with(|s| s.set(false));
        set(kept);
        if r.is_err() {
            return;
        }
        let blocks: Vec<_> = ctx.func.layout.blocks().collect();
        for b in blocks {
            let cold = ctx.func.layout.is_cold(b);
            let insts: Vec<_> = ctx.func.layout.block_insts(b).collect();
            for i in insts {
                let t = purpose(ctx.func.srcloc(i));
                let op = ctx.func.dfg.insts[i].opcode();
                if !cold && !(op.can_load() || op.can_store() || op.is_branch() || op.is_call() || op.is_terminator() || op.is_return()) {
                    pure[k][t] += 1;
                }
                ctx.func.set_srcloc(i, SourceLoc::new(if cold { 100 + t as u32 } else { 200 + t as u32 }));
            }
        }
        let dump = std::env::var("EMUSEN_SAPPHIRE_FORMS_DUMP").ok().filter(|_| (k == 2 || k == 3) && DUMPED.load(Relaxed) < 8 && words.iter().filter(|&&w| shape::class(w) == Class::Memory).count() >= 3);
        ctx.set_disasm(dump.is_some());
        let Ok(c) = ctx.compile(module.isa(), &mut ControlPlane::default()) else { return };
        if let (Some(path), Some(text)) = (dump, c.vcode.as_ref()) {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path).expect("the dump file");
            writeln!(file, "== {} block at {start:08x}, {} words\n{text}", FORMS[k], words.len()).expect("written");
            if k == 3 {
                DUMPED.fetch_add(1, Relaxed);
            }
        }
        let total = u64::from(c.code_info().total_size);
        // Cranelift lays every cold block after the hot ones: the hot region ends where the first cold range starts.
        let ranges = c.buffer.get_srclocs_sorted();
        let first_cold = ranges.iter().filter(|r| (100..200).contains(&r.loc.bits())).map(|r| u64::from(r.start)).min().unwrap_or(total);
        row[k][0] = total;
        row[k][1] = first_cold;
        row[k][2] = total - first_cold;
        row[k][3] = ranges.iter().filter(|r| (200..212).contains(&r.loc.bits()) && u64::from(r.start) >= first_cold).map(|r| u64::from(r.end - r.start)).sum();
        let mut tagged = 0;
        for r in ranges.iter().filter(|r| u64::from(r.start) < first_cold) {
            let n = u64::from(r.end.min(first_cold as u32) - r.start);
            let t = if (200..212).contains(&r.loc.bits()) { r.loc.bits() as usize - 200 } else { 0 };
            row[k][4 + t] += n;
            tagged += n;
        }
        row[k][4] += first_cold - tagged;
    }
    for k in 0..N {
        for j in 0..ROW {
            BYTES[k * ROW + j].fetch_add(row[k][j], Relaxed);
        }
        for j in 0..12 {
            PURE[k * 12 + j].fetch_add(pure[k][j], Relaxed);
        }
    }
    WORDS[0].fetch_add(words.len() as u64, Relaxed);
    WORDS[1].fetch_add(1, Relaxed);
    for &w in words {
        let c = match shape::class(w) {
            Class::Memory => 2,
            Class::Branch => 4,
            _ => 3,
        };
        WORDS[c].fetch_add(1, Relaxed);
    }
}

/// The shadow's counts as tab-separated lines, bytes a guest instruction; empty if nothing was compiled.
pub fn report() -> String {
    let w: [u64; 5] = std::array::from_fn(|k| WORDS[k].load(Relaxed));
    if w[1] == 0 && KINDS.iter().all(|k| k.load(Relaxed) == 0) {
        return String::new();
    }
    let per = |n: u64| n as f64 / w[0].max(1) as f64;
    let k: [u64; 4] = std::array::from_fn(|i| KINDS[i].load(Relaxed));
    let mut s = format!("forms kinds\tscratchpad {}\tcached {}\tuncached {}\n", k[1], k[2], k[3]);
    s += &format!("forms\tblocks {}\twords {}\t{:.2} a block\tmemory {} plain {} branch {}\tlevers {}\n", w[1], w[0], w[0] as f64 / w[1] as f64, w[2], w[3], w[4], chosen());
    for (k, name) in FORMS.iter().enumerate() {
        let b: [u64; ROW] = std::array::from_fn(|j| BYTES[k * ROW + j].load(Relaxed));
        if w[1] == 0 {
            break;
        }
        s += &format!("forms bytes\t{name}\ttotal {:.1}\thot {:.1}\tcold {:.1}\thot-tagged past the first cold {:.1}\n", per(b[0]), per(b[1]), per(b[2]), per(b[3]));
        let parts: Vec<String> = (0..12).map(|t| format!("{} {:.1}", PURPOSES[t], per(b[4 + t]))).collect();
        s += &format!("forms hot\t{name}\t{}\n", parts.join("\t"));
        let p: Vec<String> = (0..12).map(|t| format!("{} {}", PURPOSES[t], PURE[k * 12 + t].load(Relaxed))).collect();
        s += &format!("forms pure\t{name}\t{}\n", p.join("\t"));
    }
    s
}
