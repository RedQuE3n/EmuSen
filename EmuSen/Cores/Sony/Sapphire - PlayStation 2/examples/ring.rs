//! R0.2's ring (Sapphire_Redesign.md, R0 results): compiled blocks chained in a cycle that visits every one in a random
//! order, each block entered by the redesign's chained jump (§4.2): its horizon compare, its instruction-cache line's
//! generation compared, its guest instructions, its cycles added, a tail call to the next. The guest instructions are
//! synthetic loads, operations and stores on a register file, as many as make each block's host code the bytes a guest
//! instruction asked for, measured on the compiled code; blocks hold 3 or 4 guest instructions, 3.6 on average, as Jak's
//! do (Sapphire_Native.md §44.8). Prints the bytes, nanoseconds an entry and an instruction, the median of the rounds.
//!
//! `cargo run --release --example ring -- <blocks> <host bytes a guest instruction> [laps] [rounds]`
//! `cargo run --release --example ring -- sweep [laps] [rounds]`: 1, 500, 5,000 and 15,549 blocks at 96 and 380 bytes.
//! `cargo run --release --example ring -- mix <hot> <cold> <bytes> <instructions a block> <hot laps a pass> [cycles]
//! [rounds]`: a field's working set as the census measures it, a ring of hot blocks turned `laps` times to each pass
//! through a chain of the cold ones, every block's place in memory random.

use std::time::Instant;

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types::{I32, I64};
use cranelift_codegen::ir::{AbiParam, InstBuilder, MemFlagsData, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module, default_libcall_names};

/// What every block reads and writes: the guest's registers, the cycle count, the horizon, the entries, the
/// instruction cache's line generations.
#[repr(C)]
struct Ctx {
    regs: [u64; 32],
    now: u64,
    horizon: u64,
    entries: u64,
    /// The hot ring's laps left before the cold chain, and the laps it is reset to.
    laps: u64,
    reset: u64,
    gens: [u32; 256],
}

const REGS: i32 = std::mem::offset_of!(Ctx, regs) as i32;
const NOW: i32 = std::mem::offset_of!(Ctx, now) as i32;
const HORIZON: i32 = std::mem::offset_of!(Ctx, horizon) as i32;
const ENTRIES: i32 = std::mem::offset_of!(Ctx, entries) as i32;
const LAPS: i32 = std::mem::offset_of!(Ctx, laps) as i32;
const RESET: i32 = std::mem::offset_of!(Ctx, reset) as i32;
const GENS: i32 = std::mem::offset_of!(Ctx, gens) as i32;

fn stream(mut seed: u64) -> impl FnMut() -> u64 {
    move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    }
}

fn module() -> JITModule {
    let b = JITBuilder::with_flags(&[("opt_level", "speed"), ("preserve_frame_pointers", "true")], default_libcall_names()).expect("the host's Cranelift");
    JITModule::new(b)
}

fn block_sig(m: &JITModule) -> Signature {
    let mut s = m.make_signature();
    s.call_conv = CallConv::Tail;
    s.params.push(AbiParam::new(I64));
    s.returns.push(AbiParam::new(I32));
    s
}

/// One block: `n` guest instructions of `units` synthetic operations in all, its line `slot`, its tail call to `next`;
/// with `alt`, the hot ring's last block, a call to `alt` instead once its laps are done.
#[allow(clippy::too_many_arguments)]
fn lower(m: &mut JITModule, ctx: &mut cranelift_codegen::Context, fb: &mut FunctionBuilderContext, id: FuncId, next: FuncId, alt: Option<FuncId>, n: u32, units: u32, slot: u32, seed: u64) -> usize {
    m.clear_context(ctx);
    ctx.func.signature = block_sig(m);
    let next_ref = m.declare_func_in_func(next, &mut ctx.func);
    let alt_ref = alt.map(|a| m.declare_func_in_func(a, &mut ctx.func));
    let cfg = m.target_config();
    let mut b = FunctionBuilder::new(&mut ctx.func, fb);
    let (entry, body, out) = (b.create_block(), b.create_block(), b.create_block());
    b.append_block_params_for_function_params(entry);
    b.switch_to_block(entry);
    let c = b.block_params(entry)[0];
    let f = MemFlagsData::trusted();
    let now = b.ins().load(I64, f, c, NOW);
    let horizon = b.ins().load(I64, f, c, HORIZON);
    let past = b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, now, horizon);
    let line = b.ins().load(I32, f, c, GENS + 4 * slot as i32);
    let stale = b.ins().icmp_imm_s(IntCC::NotEqual, line, i64::from(slot & 7));
    let leave = b.ins().bor(past, stale);
    b.ins().brif(leave, out, &[], body, &[]);
    b.switch_to_block(out);
    b.set_cold_block(out);
    let r = b.ins().iconst(I32, 1);
    b.ins().return_(&[r]);
    b.switch_to_block(body);
    let mut r = stream(seed | 1);
    for _ in 0..units {
        let x = r();
        let (s, t) = ((x & 31) as i32, (x >> 5 & 31) as i32);
        let v = b.ins().load(I64, f, c, REGS + 8 * s);
        let w = b.ins().load(I64, f, c, REGS + 8 * t);
        let k = b.ins().iadd_imm_s(v, (x >> 16) as i32 as i64);
        let y = match x >> 10 & 3 {
            0 => b.ins().bxor(k, w),
            1 => b.ins().iadd(k, w),
            2 => b.ins().band(k, w),
            _ => b.ins().bor(k, w),
        };
        b.ins().store(f, y, c, REGS + 8 * ((x >> 12 & 31) as i32));
    }
    let moved = b.ins().iadd_imm_s(now, i64::from(n));
    b.ins().store(f, moved, c, NOW);
    let e = b.ins().load(I64, f, c, ENTRIES);
    let e = b.ins().iadd_imm_s(e, 1);
    b.ins().store(f, e, c, ENTRIES);
    if let Some(alt_ref) = alt_ref {
        let (stay, go) = (b.create_block(), b.create_block());
        let laps = b.ins().load(I64, f, c, LAPS);
        let left = b.ins().iadd_imm_s(laps, -1);
        b.ins().brif(left, stay, &[], go, &[]);
        b.switch_to_block(stay);
        b.ins().store(f, left, c, LAPS);
        b.ins().return_call(next_ref, &[c]);
        b.switch_to_block(go);
        let reset = b.ins().load(I64, f, c, RESET);
        b.ins().store(f, reset, c, LAPS);
        b.ins().return_call(alt_ref, &[c]);
    } else {
        b.ins().return_call(next_ref, &[c]);
    }
    b.seal_all_blocks();
    b.finalize(cfg);
    m.define_function(id, ctx).expect("the block compiles");
    ctx.compiled_code().map_or(0, |c| c.code_info().total_size as usize)
}

struct Ring {
    module: JITModule,
    enter: extern "C" fn(*mut Ctx) -> u32,
    bytes: usize,
    /// The guest instructions of every block once.
    code: u64,
    /// The guest instructions and the entries of one cycle: the hot ring's laps, then the cold chain once.
    instructions: u64,
    entries: u64,
}

/// `hot` blocks chained in a random ring and `cold` in a random chain, `laps` turns of the ring to one pass of the chain;
/// blocks of `len` guest instructions on average, the code about `per` bytes a guest instruction, every block's place in
/// memory random.
fn ring(hot: usize, cold: usize, per: f64, len: f64, laps: u64) -> Ring {
    let blocks = hot + cold;
    let mut m = module();
    let sig = block_sig(&m);
    let ids: Vec<FuncId> = (0..blocks).map(|i| m.declare_function(&format!("b{i}"), Linkage::Local, &sig).expect("declared")).collect();
    let mut next = stream(0x0000_0000_51A9_0002);
    let (floor, frac) = (len.floor() as u32, len.fract());
    let lens: Vec<u32> = (0..blocks).map(|_| floor + u32::from(((next() % 1000) as f64) < frac * 1000.0)).collect();
    let mut shuffled = |mut v: Vec<usize>| {
        for i in (1..v.len()).rev() {
            v.swap(i, next() as usize % (i + 1));
        }
        v
    };
    let ring_order = shuffled((0..hot).collect());
    let chain_order = shuffled((hot..blocks).collect());
    let placed = shuffled((0..blocks).collect());
    // Each block's successor, and the hot ring's last block's way into the chain.
    let mut to = vec![(0usize, None); blocks];
    for k in 0..hot {
        to[ring_order[k]] = (ring_order[(k + 1) % hot], None);
    }
    if cold > 0 {
        to[ring_order[hot - 1]].1 = Some(chain_order[0]);
        for k in 0..cold {
            to[chain_order[k]] = (if k + 1 < cold { chain_order[k + 1] } else { ring_order[0] }, None);
        }
    }
    let mut ctx = m.make_context();
    let mut fb = FunctionBuilderContext::new();
    // The size of a block's fixed part and of a unit, from two throwaway blocks, so the units hit the bytes asked.
    let (a, u) = {
        let mut probe = module();
        let s = block_sig(&probe);
        let p0 = probe.declare_function("probe0", Linkage::Local, &s).expect("declared");
        let p40 = probe.declare_function("probe40", Linkage::Local, &s).expect("declared");
        let mut c = probe.make_context();
        let s0 = lower(&mut probe, &mut c, &mut fb, p0, p0, None, 4, 0, 1, 7) as f64;
        let s40 = lower(&mut probe, &mut c, &mut fb, p40, p40, None, 4, 40, 1, 7) as f64;
        (s0, (s40 - s0) / 40.0)
    };
    let mut bytes = 0;
    for &i in &placed {
        let units = ((per * f64::from(lens[i]) - a) / u).round().max(0.0) as u32;
        let (n, alt) = to[i];
        bytes += lower(&mut m, &mut ctx, &mut fb, ids[i], ids[n], alt.map(|j| ids[j]), lens[i], units, (i % 256) as u32, i as u64 * 0x9E37_79B9);
    }
    // The way in: a call from the host's convention into the ring.
    let mut s = m.make_signature();
    s.params.push(AbiParam::new(I64));
    s.returns.push(AbiParam::new(I32));
    let enter = m.declare_function("enter", Linkage::Local, &s).expect("declared");
    m.clear_context(&mut ctx);
    ctx.func.signature = s;
    let first = m.declare_func_in_func(ids[ring_order[0]], &mut ctx.func);
    let cfg = m.target_config();
    {
        let mut b = FunctionBuilder::new(&mut ctx.func, &mut fb);
        let e = b.create_block();
        b.append_block_params_for_function_params(e);
        b.switch_to_block(e);
        let c = b.block_params(e)[0];
        let call = b.ins().call(first, &[c]);
        let r = b.inst_results(call)[0];
        b.ins().return_(&[r]);
        b.seal_all_blocks();
        b.finalize(cfg);
    }
    m.define_function(enter, &mut ctx).expect("the way in compiles");
    m.finalize_definitions().expect("finalized");
    // SAFETY: a finalized function of this signature.
    let enter = unsafe { std::mem::transmute::<*const u8, extern "C" fn(*mut Ctx) -> u32>(m.get_finalized_function(enter)) };
    let sum = |r: &[usize]| r.iter().map(|&i| u64::from(lens[i])).sum::<u64>();
    let laps = if cold > 0 { laps } else { 1 };
    Ring { module: m, enter, bytes, code: lens.iter().map(|&n| u64::from(n)).sum(), instructions: laps * sum(&ring_order) + sum(&chain_order), entries: laps * hot as u64 + cold as u64 }
}

fn run(hot: usize, cold: usize, per: f64, len: f64, laps: u64, cycles: u64, rounds: usize) {
    let r = ring(hot, cold, per, len, laps);
    let mut ctx = Box::new(Ctx { regs: [1; 32], now: 0, horizon: 0, entries: 0, laps, reset: laps, gens: [0; 256] });
    for (i, g) in ctx.gens.iter_mut().enumerate() {
        *g = (i & 7) as u32;
    }
    let mut lap = |n: u64| -> (f64, u64, u64) {
        (ctx.now, ctx.entries, ctx.laps) = (0, 0, laps);
        ctx.horizon = n * r.instructions;
        let t = Instant::now();
        (r.enter)(&mut *ctx);
        (t.elapsed().as_secs_f64() * 1e9, ctx.entries, ctx.now)
    };
    lap(2);
    let mut v: Vec<(f64, u64, u64)> = (0..rounds).map(|_| lap(cycles)).collect();
    v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let (ns, entries, instructions) = v[v.len() / 2];
    println!(
        "ring\t{hot} hot and {cold} cold blocks, {laps} laps to a pass\t{:.2} instructions an entry\t{:.1} host bytes a guest instruction\t{:.2} MB of code\t{:.2} ns an entry\t{:.3} ns a guest instruction",
        r.instructions as f64 / r.entries as f64,
        r.bytes as f64 / r.code as f64,
        r.bytes as f64 / 1e6,
        ns / entries as f64,
        ns / instructions as f64
    );
    // SAFETY: nothing of the ring runs past here.
    unsafe { r.module.free_memory() };
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = "ring <blocks> <host bytes a guest instruction> [laps] [rounds] | ring sweep [laps] [rounds] | ring mix <hot> <cold> <bytes> <instructions a block> <hot laps a pass> [cycles] [rounds]";
    let num = |i: usize, d: f64| args.get(i).map_or(d, |a| a.parse().expect(usage));
    match args.first().map(String::as_str) {
        Some("sweep") => {
            for per in [96.0, 380.0] {
                for blocks in [1, 500, 5_000, 15_549] {
                    run(blocks, 0, per, 3.6, 1, num(1, 200.0) as u64 * 15_549 / blocks as u64, num(2, 5.0) as usize);
                }
            }
        }
        Some("mix") => run(num(1, 0.0) as usize, num(2, 0.0) as usize, num(3, 96.0), num(4, 3.6), num(5, 1.0) as u64, num(6, 50.0) as u64, num(7, 5.0) as usize),
        Some(_) => run(num(0, 1.0) as usize, 0, num(1, 96.0), 3.6, 1, num(2, 200.0) as u64, num(3, 5.0) as usize),
        None => panic!("{usage}"),
    }
}
