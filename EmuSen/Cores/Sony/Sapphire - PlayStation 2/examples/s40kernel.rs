//! Stage S40's kernel bench (Sapphire_Native.md §45.7): VU1 alone running a vertex transform written for the purpose (a
//! matrix by four FMAC pairs, a divide for the perspective, the conversion to fixed point and a store, in a counted loop
//! with an XGKICK at its end), brought forward `step` cycles at a time as the board brings it after each EE instruction,
//! on the interpreter and on the recompiler; nanoseconds a pair for each, and the two units compared at the end.
//!
//! `cargo run --release --example s40kernel -- [rounds] [step] [transform|empty|fmac]`

use std::time::Instant;

use emusen_mips::vu as table;
use emusen_native::StateWriter;
use sapphire::vu::jit::Settings;
use sapphire::vu::micro::Micro;
use sapphire::vu::pipe::Pipe;
use sapphire::vu::{Link, Units, VU0_MEM, VU1_MEM, Vu};

const KERNEL: &str = "\
nop | iaddiu vi1, vi0, 0x40\n\
nop | iaddiu vi2, vi0, 0x100\n\
nop | iaddiu vi3, vi0, 0x200\n\
nop | lq vf1, 0x0(vi0)\n\
nop | lq vf2, 0x1(vi0)\n\
nop | lq vf3, 0x2(vi0)\n\
nop | lq vf4, 0x3(vi0)\n\
loop: nop | lqi vf5, (vi2++)\n\
mulax.xyzw acc, vf1, vf5 | move vf0, vf0\n\
madday.xyzw acc, vf2, vf5 | move vf0, vf0\n\
maddaz.xyzw acc, vf3, vf5 | move vf0, vf0\n\
maddw.xyzw vf6, vf4, vf0 | move vf0, vf0\n\
nop | div q, vf0w, vf6w\n\
nop | waitq\n\
mulq.xyz vf7, vf6, q | move vf0, vf0\n\
nop | move vf0, vf0\n\
nop | move vf0, vf0\n\
nop | move vf0, vf0\n\
ftoi4.xyzw vf8, vf7 | isubiu vi1, vi1, 0x1\n\
nop | move vf0, vf0\n\
nop | move vf0, vf0\n\
nop | move vf0, vf0\n\
nop | sqi vf8, (vi3++)\n\
nop | ibne vi1, vi0, loop\n\
nop | move vf0, vf0\n\
nop | iaddiu vi4, vi0, 0x200\n\
nop | xgkick vi4\n\
[e] nop | move vf0, vf0\n\
nop | move vf0, vf0";

/// The same loop's shape with nothing in it but the integer unit's count: what a pair costs beyond its operations.
const EMPTY: &str = "\
nop | iaddiu vi1, vi0, 0x40\n\
loop: nop | move vf0, vf0\n\
nop | move vf0, vf0\n\
nop | move vf0, vf0\n\
nop | isubiu vi1, vi1, 0x1\n\
nop | move vf0, vf0\n\
nop | ibne vi1, vi0, loop\n\
nop | move vf0, vf0\n\
[e] nop | move vf0, vf0\n\
nop | move vf0, vf0";

/// Four FMAC pairs a pass with the loads and stores around them, no divide: the FMAC's share.
const FMAC: &str = "\
nop | iaddiu vi1, vi0, 0x40\n\
nop | iaddiu vi2, vi0, 0x100\n\
loop: nop | lqi vf5, (vi2++)\n\
mul.xyzw vf6, vf5, vf5 | move vf0, vf0\n\
add.xyzw vf7, vf5, vf5 | move vf0, vf0\n\
madd.xyzw vf8, vf5, vf5 | move vf0, vf0\n\
mula.xyzw acc, vf5, vf5 | isubiu vi1, vi1, 0x1\n\
nop | sq vf6, 0x0(vi2)\n\
nop | ibne vi1, vi0, loop\n\
nop | move vf0, vf0\n\
[e] nop | move vf0, vf0\n\
nop | move vf0, vf0";

fn units() -> Units {
    let mut u = Units { vu0: Vu::new(VU0_MEM), vu1: Vu::new(VU1_MEM), link: Link::default(), halts: false };
    let kind = std::env::args().nth(3).unwrap_or_default();
    let prog = table::assemble(match kind.as_str() {
        "empty" => EMPTY,
        "fmac" => FMAC,
        _ => KERNEL,
    })
    .unwrap();
    for (i, p) in prog.iter().enumerate() {
        u.vu1.micro[i * 8..i * 8 + 8].copy_from_slice(&p.to_le_bytes());
    }
    let words = |a: usize, v: [f32; 4], u: &mut Units| {
        for (k, x) in v.iter().enumerate() {
            u.vu1.data[a * 16 + 4 * k..a * 16 + 4 * k + 4].copy_from_slice(&x.to_bits().to_le_bytes());
        }
    };
    words(0, [1.0, 0.0, 0.0, 0.0], &mut u);
    words(1, [0.0, 1.0, 0.0, 0.0], &mut u);
    words(2, [0.0, 0.0, 1.0, 0.5], &mut u);
    words(3, [0.0, 0.0, 4.0, 1.0], &mut u);
    for k in 0..0x40 {
        words(0x100 + k, [k as f32 * 0.5, 2.0 - k as f32, 3.0 + k as f32 * 0.25, 1.0], &mut u);
    }
    u
}

fn state(p: &Pipe, u: &Units) -> Vec<u8> {
    let f = |w: &mut StateWriter| {
        p.write_state(w);
        w.structure("Vu1", &u.vu1);
    };
    let mut c = StateWriter::counter();
    f(&mut c);
    let mut v = vec![0; c.len()];
    f(&mut StateWriter::new(&mut v));
    v
}

fn run(tier: Option<Settings>, rounds: u32, step: u64) -> (f64, u64, Vec<u8>) {
    let mut u = units();
    let mut p = Pipe::new_interpreted(1);
    p.set_tier(tier);
    p.set_clock(true);
    let mut t = 10;
    let started = Instant::now();
    for _ in 0..rounds {
        assert!(p.start(&mut u, 0, t));
        while p.busy() {
            t += step;
            p.run_to(&mut u, t);
        }
    }
    let ns = started.elapsed().as_nanos() as f64;
    let pairs = p.clock().unwrap()[2];
    (ns / pairs as f64, pairs, state(&p, &u))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rounds: u32 = args.first().map_or(2000, |a| a.parse().unwrap());
    let step: u64 = args.get(1).map_or(1, |a| a.parse().unwrap());
    let exact = Settings { threshold: 1, wait: true, ..Settings::default() };
    let compiled = Some(exact);
    // The first round compiles; it is run and dropped before the timed rounds.
    run(compiled, 1, step);
    let c4 = Some(Settings { c4: true, ..exact });
    run(c4, 1, step);
    for round in 0..3 {
        let (a, n, sa) = run(None, rounds, step);
        let (b, m, sb) = run(compiled, rounds, step);
        let (c, _, sc) = run(c4, rounds, step);
        assert_eq!((n, &sa), (m, &sb), "the two tiers' units");
        let differ = sa.iter().zip(&sc).filter(|(x, y)| x != y).count();
        println!("round {round}\tstep {step}\tpairs {n}\tinterpreter {a:.1} ns a pair\trecompiler {b:.1} ns a pair\t{:.2} times\twith C4 {c:.1} ns a pair, {differ} state bytes differ", a / b);
    }
}
