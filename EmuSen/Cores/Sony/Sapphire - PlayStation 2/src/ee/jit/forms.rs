//! For measurement only: the emitter's smaller exact forms and the shadow byte count (Sapphire_Redesign.md, "The emitter's exact forms" and "Option 1").

use std::cell::Cell;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use cranelift_codegen::control::ControlPlane;
use cranelift_codegen::ir::SourceLoc;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::JITModule;
use cranelift_module::Module;

use super::shape::{self, Class};
use super::{Context, Options, emit};
use crate::ee::Ee;
use crate::ee::timing::Timing;

/// The data cache's hit inline, every other case of a load or store out of line.
pub const CACHE: u16 = 1;
/// An exit stores a model field, Random or the group only when the block changed it from the entry's value.
pub const CHANGED: u16 = 2;
/// What the block keeps in the core or the context read where it is used, after the calls, not held from the entry.
pub const RELOAD: u16 = 4;
/// Each compiled load and store that stays in its block counted by its page's kind (`kind`).
pub const COUNT: u16 = 8;
/// Each helper call checked to leave alone what the levers hold across it (`Watched`).
pub const WATCH: u16 = 16;
/// The replay, `leave` and the fill called through trampolines that preserve every register (`trampolines`).
pub const PRESERVE: u16 = 32;
/// For the shadow count only: the context pointer in Cranelift's pinned register.
pub const PINNED: u16 = 64;
/// One translation a page a block: a later access on the same base register reuses an earlier one's page when one compare shows the same page (Sapphire_Redesign.md, "The exact design's work a block").
pub const PAGE: u16 = 128;
/// One tag check a line a block: a later access on the same base register reuses an earlier one's line when one compare shows the same line.
pub const LINE: u16 = 256;
/// One shared exit: the hot exit's replay and commit made by one helper, handed the exit's few values.
pub const STUB: u16 = 512;
/// One horizon check a block: none before each access, the dispatcher's one at the entry instead (Sapphire_Redesign.md, "One horizon check a block").
pub const HORIZON: u16 = 1024;
/// The kinds `COUNT` counts (1 to 3), then the reuses: a page reused, its guard failed, a line reused, its guard failed.
static KINDS: [AtomicU64; 8] = [const { AtomicU64::new(0) }; 8];

/// The address of `KINDS`, which `COUNT`'s code adds to.
pub fn kinds() -> i64 {
    KINDS.as_ptr() as i64
}

thread_local! {
    static LEVERS: Cell<u16> = const { Cell::new(0) };
    static STATIC: Cell<bool> = const { Cell::new(false) };
    static BASE: Cell<u32> = const { Cell::new(0) };
}

/// The levers `EMUSEN_SAPPHIRE_FORMS` names (`cache`, `count`, comma-separated).
pub fn chosen() -> u16 {
    static CHOSEN: OnceLock<u16> = OnceLock::new();
    *CHOSEN.get_or_init(|| {
        let v = std::env::var("EMUSEN_SAPPHIRE_FORMS").unwrap_or_default();
        v.split(',').map(|s| match s.trim() {
            "cache" => CACHE,
            "changed" => CHANGED,
            "reload" => RELOAD,
            "count" => COUNT,
            "watch" => WATCH,
            "preserve" => PRESERVE,
            "pinned" => PINNED,
            "page" => PAGE,
            "line" => LINE,
            "stub" => STUB,
            "horizon" => HORIZON,
            _ => 0,
        }).fold(0, |a, b| a | b)
    })
}

pub fn set(levers: u16) {
    LEVERS.with(|c| c.set(levers));
}

pub fn on(lever: u16) -> bool {
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

/// What the levers rely on no helper changing (Sapphire_Redesign.md, "Option 1": what the helpers can write).
pub struct Watched {
    timing: Timing,
    cop0: [u32; 32],
    seen: u64,
    group: u8,
    regs: [u128; 32],
    hilo: [u128; 2],
    place: (u32, u32, bool, u64),
    memory: [u64; 4],
}

/// Helper calls watched and those that changed what `Watched` holds: replay, leave, fill, hook.
static WATCHES: [AtomicU64; 8] = [const { AtomicU64::new(0) }; 8];

fn watched_now(ee: &Ee, ctx: &Context) -> Watched {
    let mut cop0 = ee.cop0;
    cop0[crate::ee::cop0::CAUSE] &= !(crate::ee::cop0::cause::IP2 | crate::ee::cop0::cause::IP3);
    Watched {
        timing: ee.timing.clone(),
        cop0,
        seen: ee.cycles_seen,
        group: ee.group,
        regs: std::array::from_fn(|r| ee.gpr[r]),
        hilo: [ee.hi, ee.lo],
        place: (ee.pc, ee.next_pc, ee.delay, ee.sa),
        memory: [ctx.pages as u64, ctx.gen1, ctx.horizon, ee.dcache.data.as_ptr() as u64],
    }
}

/// Before helper `which`'s body, when `watch` is chosen.
pub fn watch(ee: &Ee, ctx: &Context) -> Option<Watched> {
    (chosen() & WATCH != 0).then(|| watched_now(ee, ctx))
}

/// After it: counted, and counted as changed when anything `Watched` holds moved.
pub fn watched(before: Option<Watched>, ee: &Ee, ctx: &Context, which: usize) {
    let Some(b) = before else { return };
    let a = watched_now(ee, ctx);
    WATCHES[2 * which].fetch_add(1, Relaxed);
    let same = a.timing == b.timing && a.cop0 == b.cop0 && a.seen == b.seen && a.group == b.group && a.regs == b.regs && a.hilo == b.hilo && a.place == b.place && a.memory == b.memory;
    if !same {
        WATCHES[2 * which + 1].fetch_add(1, Relaxed);
    }
}

/// Under `PRESERVE`, each helper behind a `preserve_all` trampoline that leaves its answer in `Context::answer`; Rust never calls one.
pub fn trampolines(helpers: [super::Helper; 3]) -> [super::Helper; 3] {
    if chosen() & PRESERVE == 0 {
        return helpers;
    }
    static MADE: OnceLock<[usize; 3]> = OnceLock::new();
    let made = MADE.get_or_init(|| {
        use cranelift_codegen::ir::{AbiParam, InstBuilder, MemFlagsData, types};
        use cranelift_codegen::isa::CallConv;
        let mut module = JITModule::new(cranelift_jit::JITBuilder::with_flags(&[("opt_level", "speed")], cranelift_module::default_libcall_names()).expect("the host"));
        let ptr = module.target_config().pointer_type();
        let mut ids = Vec::new();
        for (k, &h) in helpers.iter().enumerate() {
            let mut ctx = module.make_context();
            ctx.func.signature.call_conv = CallConv::PreserveAll;
            ctx.func.signature.params.extend([AbiParam::new(ptr), AbiParam::new(types::I32)]);
            let mut sysv = module.make_signature();
            sysv.params.extend([AbiParam::new(ptr), AbiParam::new(types::I32)]);
            sysv.returns.push(AbiParam::new(types::I32));
            let mut fc = FunctionBuilderContext::new();
            let mut b = FunctionBuilder::new(&mut ctx.func, &mut fc);
            let block = b.create_block();
            b.append_block_params_for_function_params(block);
            b.switch_to_block(block);
            b.seal_block(block);
            let (c, arg) = (b.block_params(block)[0], b.block_params(block)[1]);
            let sig = b.import_signature(sysv);
            let f = b.ins().iconst(ptr, h as usize as i64);
            let call = b.ins().call_indirect(sig, f, &[c, arg]);
            let r = b.inst_results(call)[0];
            b.ins().store(MemFlagsData::trusted(), r, c, std::mem::offset_of!(Context, answer) as i32);
            b.ins().return_(&[]);
            b.finalize(module.target_config());
            let id = module.declare_function(&format!("trampoline{k}"), cranelift_module::Linkage::Local, &ctx.func.signature).expect("declared");
            module.define_function(id, &mut ctx).expect("defined");
            ids.push(id);
        }
        module.finalize_definitions().expect("finalized");
        let out = std::array::from_fn(|k| module.get_finalized_function(ids[k]) as usize);
        std::mem::forget(module);
        out
    });
    // SAFETY: the code calls these addresses with `preserve_all`'s convention, as the trampolines were compiled; Rust never calls them.
    made.map(|a| unsafe { std::mem::transmute::<usize, super::Helper>(a) })
}

/// Under `PRESERVE`, the shared exit behind a `preserve_all` trampoline as `trampolines` makes them.
pub fn commit_trampoline(f: super::Commit) -> super::Commit {
    if chosen() & PRESERVE == 0 {
        return f;
    }
    static MADE: OnceLock<usize> = OnceLock::new();
    let made = *MADE.get_or_init(|| {
        use cranelift_codegen::ir::{AbiParam, InstBuilder, MemFlagsData, types};
        use cranelift_codegen::isa::CallConv;
        let mut module = JITModule::new(cranelift_jit::JITBuilder::with_flags(&[("opt_level", "speed")], cranelift_module::default_libcall_names()).expect("the host"));
        let ptr = module.target_config().pointer_type();
        let mut ctx = module.make_context();
        ctx.func.signature.call_conv = CallConv::PreserveAll;
        ctx.func.signature.params.extend([AbiParam::new(ptr), AbiParam::new(types::I64), AbiParam::new(types::I64), AbiParam::new(types::I64)]);
        let mut sysv = module.make_signature();
        sysv.params.extend([AbiParam::new(ptr), AbiParam::new(types::I64), AbiParam::new(types::I64), AbiParam::new(types::I64)]);
        sysv.returns.push(AbiParam::new(types::I32));
        let mut fc = FunctionBuilderContext::new();
        let mut b = FunctionBuilder::new(&mut ctx.func, &mut fc);
        let block = b.create_block();
        b.append_block_params_for_function_params(block);
        b.switch_to_block(block);
        b.seal_block(block);
        let args: Vec<_> = b.block_params(block).to_vec();
        let sig = b.import_signature(sysv);
        let h = b.ins().iconst(ptr, f as usize as i64);
        let call = b.ins().call_indirect(sig, h, &args);
        let r = b.inst_results(call)[0];
        b.ins().store(MemFlagsData::trusted(), r, args[0], std::mem::offset_of!(Context, answer) as i32);
        b.ins().return_(&[]);
        b.finalize(module.target_config());
        let id = module.declare_function("commit", cranelift_module::Linkage::Local, &ctx.func.signature).expect("declared");
        module.define_function(id, &mut ctx).expect("defined");
        module.finalize_definitions().expect("finalized");
        let a = module.get_finalized_function(id) as usize;
        std::mem::forget(module);
        a
    });
    // SAFETY: the code calls this address with `preserve_all`'s convention, as the trampoline was compiled; Rust never calls it.
    unsafe { std::mem::transmute::<usize, super::Commit>(made) }
}

thread_local! {
    static PINNED_ISA: std::cell::OnceCell<cranelift_codegen::isa::OwnedTargetIsa> = const { std::cell::OnceCell::new() };
}

/// The module's ISA, its flags and the host's, with Cranelift's pinned register reserved (`PINNED`).
fn pinned_isa(module: &JITModule) -> cranelift_codegen::isa::OwnedTargetIsa {
    use cranelift_codegen::settings::{self, Configurable};
    let isa = module.isa();
    let mut builder = cranelift_codegen::isa::lookup(isa.triple().clone()).expect("the module's triple");
    for v in isa.isa_flags() {
        builder.set(v.name, &v.value_string()).expect("the host's flag");
    }
    let mut flags = settings::builder();
    for v in isa.flags().iter() {
        flags.set(v.name, &v.value_string()).expect("the module's flag");
    }
    flags.enable("enable_pinned_reg").expect("the pinned register");
    builder.finish(settings::Flags::new(flags)).expect("an ISA")
}

/// The forms the shadow compiles: today's, C1, static, both with the chosen levers, and static with each step's levers (`STEPS`).
const FORMS: [&str; 10] = ["today", "C1", "static", "static+levers", "today+levers", "static+option1", "static+page", "static+line", "static+stub", "static+horizon"];
/// Option 1's levers, then each of this step's added in turn.
const OPTION1: u16 = CACHE | CHANGED | RELOAD | PRESERVE;
const STEPS: [u16; 5] = [OPTION1, OPTION1 | PAGE, OPTION1 | PAGE | LINE, OPTION1 | PAGE | LINE | STUB, OPTION1 | PAGE | LINE | STUB | HORIZON];
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
    let forms: [(Options, bool, u16); N] = std::array::from_fn(|k| match k {
        0 => (options, false, 0),
        1 => (c1, false, 0),
        2 => (c1, true, 0),
        3 => (c1, true, levers),
        4 => (options, false, levers & !PINNED),
        _ => (c1, true, STEPS[k - 5]),
    });
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
        let compiled = if lv & PINNED != 0 {
            PINNED_ISA.with(|i| ctx.compile(&**i.get_or_init(|| pinned_isa(module)), &mut ControlPlane::default()).is_ok())
        } else {
            ctx.compile(module.isa(), &mut ControlPlane::default()).is_ok()
        };
        let Some(c) = ctx.compiled_code().filter(|_| compiled) else { return };
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
    if w[1] == 0 && KINDS.iter().chain(WATCHES.iter()).all(|k| k.load(Relaxed) == 0) {
        return String::new();
    }
    let per = |n: u64| n as f64 / w[0].max(1) as f64;
    let k: [u64; 8] = std::array::from_fn(|i| KINDS[i].load(Relaxed));
    let wt: [u64; 8] = std::array::from_fn(|i| WATCHES[i].load(Relaxed));
    let mut s = format!("forms watch\treplay {} changed {}\tleave {} changed {}\tfill {} changed {}\thook {} changed {}\n", wt[0], wt[1], wt[2], wt[3], wt[4], wt[5], wt[6], wt[7]);
    s += &format!("forms kinds\tscratchpad {}\tcached {}\tuncached {}\n", k[1], k[2], k[3]);
    s += &format!("forms reuse\tpage {} its guard failed {}\tline {} its guard failed {}\n", k[4], k[5], k[6], k[7]);
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
