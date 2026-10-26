//! Stage S40's oracles (Sapphire_Native.md §45.5): micro mode recompiled against the interpreter. Two copies of the units
//! run the same microprogram under the same calls, one interpreted and one compiled (VU1, and VU0 for the lowering's
//! generality), and the units, both micro modes' states and what PATH1 was handed are compared after every call, and in
//! the mechanism tests after every cycle. The random programs draw both pipelines' operations with registers, data
//! memory, ACC and I holding every float class of S9's set; the mechanisms are taken one at a time.

mod s6;
mod s9;

use std::cell::RefCell;
use std::rc::Rc;

use emusen_mips::vu::{self as table, I_BIT, LOWER_OP, Unit};
use emusen_native::{State, StateReader, StateWriter};
use sapphire::vu::jit::{Counts, Settings};
use sapphire::vu::micro::Micro;
use sapphire::vu::ops::{self, Class};
use sapphire::vu::pipe::Pipe;
use sapphire::vu::{Link, Path1, Units, VU0_MEM, VU1_MEM, Vu, fbrst};

/// S9's input classes (`tests/s9/mod.rs`'s `VALUES`): zeros, the smallest and largest normals, exponent 255 in IEEE 754's
/// infinity and NaN patterns, would-be denormals, ones and halves, powers whose products overflow and underflow.
const CLASSES: [u32; 32] = s9::VALUES;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
    fn class(&mut self) -> u32 {
        CLASSES[self.below(32) as usize]
    }
}

/// PATH1 recording each kick, busy a fixed time.
struct Recorder {
    kicks: Rc<RefCell<Vec<(u16, u64)>>>,
    busy: u64,
}

impl Path1 for Recorder {
    fn kick(&mut self, data: &[u8], addr: u16, at: u64) -> u64 {
        let first = u32::from_le_bytes(data[addr as usize * 16..addr as usize * 16 + 4].try_into().unwrap());
        self.kicks.borrow_mut().push((addr, at));
        self.kicks.borrow_mut().push((first as u16, first as u64 >> 16));
        at + self.busy
    }
}

/// One copy of the units and a unit's micro mode.
struct Side {
    u: Units,
    p: Pipe,
    kicks: Rc<RefCell<Vec<(u16, u64)>>>,
}

fn units(busy: u64, kicks: &Rc<RefCell<Vec<(u16, u64)>>>) -> Units {
    let link = Link { path1: Box::new(Recorder { kicks: kicks.clone(), busy }), ..Link::default() };
    Units { vu0: Vu::new(VU0_MEM), vu1: Vu::new(VU1_MEM), link, halts: false }
}

fn settings(verify: bool) -> Settings {
    Settings { threshold: 1, wait: true, verify, ..Settings::default() }
}

#[test]
#[should_panic(expected = "parted from the interpreter")]
fn the_verifier_finds_a_pair_the_code_ran_differently() {
    // A program whose word changes between the verifier's copy and the units it runs: the copy fetches the new word.
    let src = "nop | iaddiu vi1, vi0, 0x1\nnop | iaddiu vi2, vi0, 0x2\n[e] nop | move vf0, vf0\nnop | move vf0, vf0";
    let prog = table::assemble(src).unwrap();
    let mut c = Side::new(1, 0, Some(settings(true)));
    for (k, p) in prog.iter().enumerate() {
        put(&mut c.u, 1, k, *p);
    }
    assert!(c.p.start(&mut c.u, 0, 10));
    c.p.finish(&mut c.u, 10);
    assert!(c.p.start(&mut c.u, 0, 50));
    c.p.run_to(&mut c.u, 50);
    // Written behind the lending count's back, as a cache that missed the write would run it.
    let word = table::assemble_pair("nop | iaddiu vi2, vi0, 0x3").unwrap().to_le_bytes();
    c.u.vu1.micro.write_unseen(8, &word);
    c.p.run_to(&mut c.u, 60);
}

impl Side {
    fn new(unit: usize, busy: u64, tier: Option<Settings>) -> Side {
        let kicks = Rc::new(RefCell::new(Vec::new()));
        let mut p = Pipe::new_interpreted(unit);
        p.set_tier(tier);
        Side { u: units(busy, &kicks), p, kicks }
    }

    /// Everything the comparison holds: the micro mode's state, both units, the link's registers and the kicks.
    fn snapshot(&self) -> Vec<u8> {
        let mut out = bytes(|w| {
            self.p.write_state(w);
            w.structure("Vu0", &self.u.vu0);
            w.structure("Vu1", &self.u.vu1);
            let l = &self.u.link;
            w.u32s("Link", &[l.fbrst, l.irq]);
        });
        for &(a, t) in self.kicks.borrow().iter() {
            out.extend_from_slice(&a.to_le_bytes());
            out.extend_from_slice(&t.to_le_bytes());
        }
        out
    }

    fn counts(&self) -> Counts {
        self.p.jit_counts().expect("the recompiler")
    }
}

fn bytes(f: impl Fn(&mut StateWriter)) -> Vec<u8> {
    let mut c = StateWriter::counter();
    f(&mut c);
    let mut v = vec![0; c.len()];
    f(&mut StateWriter::new(&mut v));
    v
}

/// Where two snapshots part, for the message.
fn first_difference(a: &[u8], b: &[u8]) -> String {
    let at = a.iter().zip(b).position(|(x, y)| x != y).unwrap_or(a.len().min(b.len()));
    format!("at byte {at} of {} and {}", a.len(), b.len())
}

fn same(i: &Side, c: &Side, what: &str) {
    let (a, b) = (i.snapshot(), c.snapshot());
    assert!(a == b, "{what}: the recompiled unit parted from the interpreter {}", first_difference(&a, &b));
}

fn put(u: &mut Units, unit: usize, pc: usize, pair: u64) {
    let m = if unit == 0 { &mut u.vu0.micro } else { &mut u.vu1.micro };
    let at = pc * 8 % m.len();
    m[at..at + 8].copy_from_slice(&pair.to_le_bytes());
}

const NOP: u32 = 11 << 6 | 0x3C | 3;
const MOVE0: u32 = LOWER_OP | 0x33C;
const E_BIT: u32 = 1 << 30;

fn pair(upper: u32, lower: u32) -> u64 {
    (upper as u64) << 32 | lower as u64
}

/// The units' starting state: registers, ACC, I, R and the data memory full of the float classes, the integer registers
/// small, some of them addresses.
fn seed_units(r: &mut Rng, u: &mut Units) {
    for vu in [&mut u.vu0, &mut u.vu1] {
        for n in 1..32 {
            let v = [r.class(), r.class(), r.class(), r.class()];
            vu.set_vf(n, v);
        }
        for n in 1..16 {
            let v = match r.below(4) {
                0 => r.below(4) as u16,
                1 => r.below(0x400) as u16,
                2 => (r.below(5) as i16 - 2) as u16,
                _ => r.next() as u16,
            };
            vu.set_vi(n, v);
        }
        vu.acc = [r.class(), r.class(), r.class(), r.class()];
        vu.i = r.class();
        vu.q = r.class();
        vu.r = r.next() as u32 & 0x7F_FFFF;
        let len = vu.data.len();
        for at in (0..len).step_by(4) {
            let v = if r.chance(80) { r.class() } else { r.next() as u32 };
            vu.data[at..at + 4].copy_from_slice(&v.to_le_bytes());
        }
    }
}

/// A register field from a small set, so that pairs meet each other's hazards.
fn reg(r: &mut Rng) -> u32 {
    [0, 1, 2, 3, 4, 5, 6, 31][r.below(8) as usize]
}

/// A random upper word: an upper operation with its fields drawn small, and sometimes the M, D, T and I bits.
fn upper_word(r: &mut Rng) -> u32 {
    loop {
        let w = (r.next() as u32 & 0x01E0_003F) | reg(r) << 16 | reg(r) << 11 | reg(r) << 6;
        if ops::of_word(w).is_some_and(|(o, _)| o.unit == Unit::Upper) {
            let mut w = w;
            if r.chance(4) {
                w |= 1 << 29;
            }
            if r.chance(2) {
                w |= 1 << 28;
            }
            if r.chance(2) {
                w |= 1 << 27;
            }
            if r.chance(8) {
                w |= I_BIT;
            }
            return w;
        }
    }
}

/// A random lower word for the pair at `pc` of a program of `len` pairs from `start`: any lower or micro-only operation,
/// branches forward within the program, and with `loops` backward ones and register jumps too.
fn lower_word(r: &mut Rng, pc: u16, start: u16, len: u16, loops: bool) -> u32 {
    loop {
        let top = if r.chance(55) { LOWER_OP >> 25 } else { r.below(128) as u32 };
        let w = top << 25 | (r.next() as u32 & 0x01E0_07C0 & !(31 << 6)) | reg(r) << 16 | reg(r) << 11 | (r.next() as u32 & 0x7FF);
        let Some((o, e)) = ops::of_lower(w) else { continue };
        if e.class == Class::Branch {
            if !loops && matches!(o.name, "jr" | "jalr") {
                continue;
            }
            let end = start + len;
            let target = if loops && r.chance(30) { start + r.below(len as u64) as u16 } else { (pc + 2 + r.below(4) as u16).min(end) };
            let off = (target as i32 - pc as i32 - 1) & 0x7FF;
            return w & !0x7FF | off as u32;
        }
        return w;
    }
}

/// A program of `len` pairs at `start`, its last an E bit, the rest of memory E bits so that a wild jump ends at once.
fn program(r: &mut Rng, u: &mut Units, unit: usize, start: u16, len: u16, loops: bool) -> Vec<u64> {
    let pairs = if unit == 0 { VU0_MEM } else { VU1_MEM } / 8;
    for pc in 0..pairs {
        put(u, unit, pc, pair(NOP | E_BIT, MOVE0));
    }
    let mut prog = Vec::new();
    for k in 0..len {
        let pc = start + k;
        let up = upper_word(r);
        let low = if up & I_BIT != 0 { r.class() } else { lower_word(r, pc, start, len, loops) };
        let up = if k == len - 2 { up | E_BIT } else { up & !E_BIT };
        prog.push(pair(up, low));
    }
    for (k, p) in prog.iter().enumerate() {
        put(u, unit, start as usize + k, *p);
    }
    prog
}

/// Both copies under one random schedule of calls, compared after each.
fn random_run(seed: u64, unit: usize, loops: bool, verify: bool) -> (Counts, u64) {
    let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let busy = r.below(12);
    let mut i = Side::new(unit, busy, None);
    let mut c = Side::new(unit, busy, Some(settings(verify)));
    let len = 8 + r.below(40) as u16;
    let start = r.below(64) as u16;
    seed_units(&mut r, &mut i.u);
    c.u.vu0 = i.u.vu0.clone();
    c.u.vu1 = i.u.vu1.clone();
    let prog = program(&mut r, &mut i.u, unit, start, len, loops);
    c.u.vu0.micro = i.u.vu0.micro.clone();
    c.u.vu1.micro = i.u.vu1.micro.clone();
    let enables = (r.next() as u32) & ((fbrst::DE0 | fbrst::TE0) * 0x101);
    for s in [&mut i, &mut c] {
        s.u.link.fbrst = enables;
        s.u.link.top = [0x12, 0x2A5];
        s.u.link.itop = [0x7, 0x1C3];
    }
    let mut now = 10;
    for s in [&mut i, &mut c] {
        assert!(s.p.start(&mut s.u, start, now));
    }
    let mut calls = 0;
    for step in 0..600 {
        calls += 1;
        let roll = r.below(100);
        match roll {
            0..=79 => {
                now += r.below(9);
                for s in [&mut i, &mut c] {
                    s.p.run_to(&mut s.u, now);
                }
            }
            80..=83 if !loops => {
                let (a, b) = (i.p.finish(&mut i.u, now), c.p.finish(&mut c.u, now));
                assert_eq!(a, b, "seed {seed}: finish's answer");
            }
            84..=87 if !loops => {
                let (a, b) = (i.p.release(&mut i.u, now), c.p.release(&mut c.u, now));
                assert_eq!(a, b, "seed {seed}: release's answer");
            }
            88..=89 => {
                // The program's pair replaced while the unit may run, as an MPG or an EE store would.
                let k = r.below(len as u64) as usize;
                let up = upper_word(&mut r) & !E_BIT;
                let low = if up & I_BIT != 0 { r.class() } else { lower_word(&mut r, start + k as u16, start, len, loops) };
                let p = if k == len as usize - 2 { pair(up | E_BIT, low) } else { pair(up, low) };
                put(&mut i.u, unit, start as usize + k, p);
                put(&mut c.u, unit, start as usize + k, p);
                let _ = &prog;
            }
            90..=91 => {
                // A state saved while the unit runs, loaded into units with an empty cache.
                let state = bytes(|w| c.p.write_state(w));
                let mut fresh = Pipe::new_interpreted(unit);
                fresh.set_tier(Some(settings(verify)));
                fresh.read_state(&mut StateReader::new(&state)).unwrap();
                c.p = fresh;
            }
            92 => {
                now += 1;
                for s in [&mut i, &mut c] {
                    s.p.force_break(&mut s.u, now);
                }
            }
            _ => {}
        }
        same(&i, &c, &format!("seed {seed}, call {step}"));
        if !i.p.busy() && r.chance(30) {
            now += r.below(6);
            for s in [&mut i, &mut c] {
                assert!(s.p.start(&mut s.u, start, now));
            }
        }
    }
    (c.counts(), calls)
}

#[test]
fn random_microprograms_on_vu1_leave_the_units_as_the_interpreter_does() {
    let mut total = Counts::default();
    for seed in 0..240 {
        let (k, _) = random_run(seed, 1, seed % 2 == 1, seed % 5 == 0);
        total.compiled_pairs += k.compiled_pairs;
        total.interpreted_pairs += k.interpreted_pairs;
        total.programs += k.programs;
        total.invalidated += k.invalidated;
        total.refused += k.refused;
        total.calls += k.calls;
    }
    assert_eq!(total.refused, 0, "a microprogram Cranelift refused");
    assert!(total.compiled_pairs > 10 * total.interpreted_pairs, "most pairs compiled: {total:?}");
    assert!(total.invalidated > 0, "a replaced pair invalidated its program: {total:?}");
    println!("VU1 random programs: {total:?}");
}

#[test]
fn random_microprograms_on_vu0_leave_the_units_as_the_interpreter_does() {
    for seed in 1000..1080 {
        random_run(seed, 0, seed % 2 == 1, seed % 4 == 0);
    }
}

/// Every operation either pipeline decodes, one to a program, its fields drawn at random; the copies compared after
/// every cycle, so that each flag, quotient and EFU result is seen landing on the same cycle.
#[test]
fn every_operation_lands_on_the_same_cycle() {
    let mut words: Vec<(&str, u32, bool)> = Vec::new();
    let mut r = Rng(0x5EED_0001);
    for top in 0..128u32 {
        for row in 0..32u32 {
            for low6 in 0..64u32 {
                let w = top << 25 | (r.next() as u32 & 0x01FF_F800) | row << 6 | low6;
                if top == 0 && let Some((o, _)) = ops::of_word(w).filter(|(o, _)| o.unit == Unit::Upper) && !words.iter().any(|x| x.0 == o.name && x.2) {
                    words.push((o.name, w, true));
                }
                if let Some((o, e)) = ops::of_lower(w) && !words.iter().any(|x| x.0 == o.name && !x.2) {
                    let w = if e.class == Class::Branch { w & !0x7FF | 2 } else { w };
                    words.push((o.name, w, false));
                }
            }
        }
    }
    assert!(words.len() >= 160, "{} operations found", words.len());
    for (n, &(name, w, upper)) in words.iter().enumerate() {
        for unit in [0, 1] {
            let mut i = Side::new(unit, 3, None);
            let mut c = Side::new(unit, 3, Some(settings(true)));
            let mut r = Rng(n as u64 * 77 + unit as u64 + 1);
            seed_units(&mut r, &mut i.u);
            let (up, low) = if upper { (w, MOVE0) } else { (NOP, w) };
            let prog = [pair(up, low), pair(NOP, MOVE0), pair(up & !(1 << 29), low), pair(NOP, MOVE0), pair(NOP, MOVE0), pair(NOP | E_BIT, MOVE0), pair(NOP, MOVE0)];
            let pairs = if unit == 0 { VU0_MEM } else { VU1_MEM } / 8;
            for pc in 0..pairs {
                put(&mut i.u, unit, pc, pair(NOP | E_BIT, MOVE0));
            }
            for (k, p) in prog.iter().enumerate() {
                put(&mut i.u, unit, k, *p);
            }
            c.u.vu0 = i.u.vu0.clone();
            c.u.vu1 = i.u.vu1.clone();
            for s in [&mut i, &mut c] {
                assert!(s.p.start(&mut s.u, 0, 5));
            }
            for t in 5..120 {
                for s in [&mut i, &mut c] {
                    s.p.run_to(&mut s.u, t);
                }
                same(&i, &c, &format!("{name} on VU{unit}, cycle {t}"));
            }
            let k = c.counts();
            assert!(k.compiled_pairs >= 3 && k.interpreted_pairs == 0, "{name} on VU{unit} ran compiled: {k:?}");
        }
    }
}

/// Runs `src` on `unit` from cycle `from`, both copies stepped a cycle at a time and compared after each, until both
/// stop; the two copies for further checks.
fn cycle_by_cycle(unit: usize, src: &str, busy: u64, prepare: impl Fn(&mut Units)) -> (Side, Side) {
    let prog = table::assemble(src).unwrap_or_else(|e| panic!("{e}"));
    let mut i = Side::new(unit, busy, None);
    let mut c = Side::new(unit, busy, Some(settings(true)));
    for s in [&mut i, &mut c] {
        let pairs = if unit == 0 { VU0_MEM } else { VU1_MEM } / 8;
        for pc in 0..pairs {
            put(&mut s.u, unit, pc, pair(NOP | E_BIT, MOVE0));
        }
        for (k, p) in prog.iter().enumerate() {
            put(&mut s.u, unit, k, *p);
        }
        prepare(&mut s.u);
        assert!(s.p.start(&mut s.u, 0, 10));
    }
    let mut t = 10;
    while i.p.busy() || c.p.busy() {
        t += 1;
        for s in [&mut i, &mut c] {
            s.p.run_to(&mut s.u, t);
        }
        same(&i, &c, &format!("cycle {t}"));
        assert!(t < 10_000, "the program ended");
    }
    for _ in 0..20 {
        t += 1;
        for s in [&mut i, &mut c] {
            s.p.run_to(&mut s.u, t);
        }
        same(&i, &c, &format!("cycle {t}, after the end"));
    }
    (i, c)
}

const END: &str = "[e] nop | move vf0, vf0\nnop | move vf0, vf0";

#[test]
fn the_flags_land_four_cycles_on_and_their_readers_see_them_alike() {
    let src = format!(
        "add.xyzw vf1, vf2, vf3 | move vf0, vf0\nnop | fmand vi1, vi2\nnop | fsand vi3, 0xfff\nclipw.xyz vf4, vf5 | fcget vi4\nnop | fmeq vi5, vi2\n\
         sub.xy vf6, vf2, vf3 | fcget vi4\nnop | fsand vi3, 0x3f\nnop | fcset 0x123456\nnop | fsset 0xfc0\nnop | fcget vi4\nnop | fsand vi3, 0xfff\n{END}"
    );
    cycle_by_cycle(1, &src, 0, |u| {
        u.vu1.set_vf(2, [0x3F80_0000, 0xBF80_0000, 0x7FFF_FFFF, 0x0000_0001]);
        u.vu1.set_vf(3, [0xBF80_0000, 0x3F80_0000, 0x7FFF_FFFF, 0x0080_0000]);
        u.vu1.set_vi(2, 0xFFFF);
    });
}

#[test]
fn q_and_p_land_at_their_latency_alike() {
    let src = format!(
        "nop | div q, vf1x, vf2y\naddq.x vf3, vf0, q | move vf0, vf0\naddq.y vf3, vf0, q | move vf0, vf0\nnop | rsqrt q, vf1z, vf2w\nnop | sqrt q, vf2x\n\
         addq.z vf3, vf0, q | waitq\naddq.w vf3, vf0, q | move vf0, vf0\nnop | esadd p, vf1\nnop | ercpr p, vf2x\nnop | mfp.x vf4, p\nnop | waitp\nnop | mfp.y vf4, p\n\
         nop | eatan p, vf1y\nnop | mfp.z vf4, p\n{END}"
    );
    cycle_by_cycle(1, &src, 0, |u| {
        u.vu1.set_vf(1, [0x4040_0000, 0x3F80_0000, 0x4080_0000, 0x0000_0000]);
        u.vu1.set_vf(2, [0x4000_0000, 0x0000_0000, 0x7FFF_FFFF, 0x3F00_0000]);
    });
}

#[test]
fn branches_and_their_delay_slots_alike() {
    let src = format!(
        "nop | iaddiu vi1, vi0, 0x3\nloop: nop | isubiu vi1, vi1, 0x1\nnop | ibne vi1, vi0, loop\nnop | iaddiu vi2, vi2, 0x1\nnop | bal vi15, sub\nnop | iaddiu vi3, vi0, 0x7\n\
         nop | ibeq vi1, vi0, skip\nnop | iaddiu vi4, vi0, 0x9\nnop | iaddiu vi4, vi0, 0xa\nskip: nop | iaddiu vi5, vi0, 0x1\nnop | ibltz vi5, skip\nnop | b out\nnop | move vf0, vf0\n\
         sub: nop | iadd vi6, vi3, vi1\nnop | jr vi15\nnop | iaddiu vi7, vi0, 0x2\nout: nop | jalr vi14, vi13\nnop | move vf0, vf0\n{END}"
    );
    let (_, c) = cycle_by_cycle(1, &src, 0, |u| u.vu1.set_vi(13, 19));
    assert!(c.counts().compiled_pairs > 10);
}

#[test]
fn the_e_d_t_and_m_bits_alike() {
    for enables in [0, fbrst::DE0 << 8, fbrst::TE0 << 8, (fbrst::DE0 | fbrst::TE0) << 8] {
        let src = "nop | iaddiu vi1, vi0, 0x1\n[m] nop | iaddiu vi2, vi0, 0x2\n[d] nop | iaddiu vi3, vi0, 0x3\nnop | b t\n[t] nop | iaddiu vi4, vi0, 0x4\nnop | move vf0, vf0\nt: [t] nop | iaddiu vi5, vi0, 0x5\n[e] nop | b t\nnop | iaddiu vi6, vi0, 0x6\nnop | iaddiu vi7, vi0, 0x7";
        let (i, c) = cycle_by_cycle(1, src, 0, |u| u.link.fbrst = enables);
        assert_eq!(i.u.vu1.tpc, c.u.vu1.tpc);
        assert_eq!(i.p.stat(), c.p.stat(), "VPU-STAT's bits, enables {enables:x}");
    }
}

#[test]
fn xgkick_s_timing_against_a_busy_path1_alike() {
    let src = format!("nop | iaddiu vi1, vi0, 0x10\nnop | xgkick vi1\nnop | iaddiu vi1, vi1, 0x8\nnop | xgkick vi1\nnop | xgkick vi1\nnop | xtop vi2\nnop | xitop vi3\n{END}");
    for busy in [0, 1, 5, 40] {
        let (i, c) = cycle_by_cycle(1, &src, busy, |u| {
            u.link.top = [0x11, 0x222];
            u.link.itop = [0x33, 0x144];
            u.vu1.data[0x100..0x104].copy_from_slice(&0x8000_1234u32.to_le_bytes());
        });
        assert_eq!(*i.kicks.borrow(), *c.kicks.borrow());
        assert_eq!(c.kicks.borrow().len(), 6, "three kicks, each recorded with its packet's first word");
    }
}

/// A microprogram compiled, then a new one uploaded over it as VIF's MPG writes the memory: the next start runs the new
/// words, its program dropped and compiled again.
#[test]
fn an_upload_over_a_compiled_program_is_run_as_uploaded() {
    let a = table::assemble(&format!("nop | iaddiu vi1, vi0, 0x11\nnop | iaddiu vi2, vi1, 0x1\n{END}")).unwrap();
    let b = table::assemble(&format!("nop | iaddiu vi1, vi0, 0x22\nnop | iaddiu vi2, vi1, 0x2\n{END}")).unwrap();
    let mut i = Side::new(1, 0, None);
    let mut c = Side::new(1, 0, Some(settings(false)));
    let mut t = 10;
    for (round, prog) in [&a, &a, &b, &b, &a].into_iter().enumerate() {
        for s in [&mut i, &mut c] {
            for (k, p) in prog.iter().enumerate() {
                put(&mut s.u, 1, k, *p);
            }
            assert!(s.p.start(&mut s.u, 0, t));
            s.p.finish(&mut s.u, t);
        }
        t += 40;
        same(&i, &c, &format!("round {round}"));
        assert_eq!(c.u.vu1.vi(2), if std::ptr::eq(prog, &a) { 0x12 } else { 0x24 });
    }
    let k = c.counts();
    assert_eq!(k.invalidated, 1, "the words found changed when the second program came: {k:?}");
    assert_eq!(k.programs, 2, "the first program's code found again when its words came back: {k:?}");
    assert_eq!(k.interpreted_pairs, 0, "{k:?}");
}

/// The EE writes a pair of a running microprogram between two of its pairs: the unit runs the new word, as the
/// interpreter fetches it.
#[test]
fn a_store_into_a_running_program_is_seen_at_its_next_fetch() {
    let src = format!("nop | iaddiu vi1, vi0, 0x1\nnop | move vf0, vf0\nnop | move vf0, vf0\nnop | iaddiu vi2, vi0, 0x2\nnop | move vf0, vf0\n{END}");
    let prog = table::assemble(&src).unwrap();
    let replaced = table::assemble_pair("nop | iaddiu vi2, vi0, 0x3").unwrap();
    let mut i = Side::new(1, 0, None);
    let mut c = Side::new(1, 0, Some(settings(true)));
    for s in [&mut i, &mut c] {
        for (k, p) in prog.iter().enumerate() {
            put(&mut s.u, 1, k, *p);
        }
        // Compiled once beforehand, so that the run below starts in its code.
        assert!(s.p.start(&mut s.u, 0, 10));
        s.p.finish(&mut s.u, 10);
        assert!(s.p.start(&mut s.u, 0, 50));
        s.p.run_to(&mut s.u, 52);
        put(&mut s.u, 1, 3, replaced);
        s.p.run_to(&mut s.u, 100);
    }
    same(&i, &c, "after the store");
    assert_eq!(c.u.vu1.vi(2), 3);
    assert!(c.counts().invalidated >= 1);
}

/// A state saved at every cycle of a running microprogram and loaded into a unit with an empty cache continues as the
/// unit it was saved from.
#[test]
fn a_state_saved_while_vu1_runs_continues_from_an_empty_cache() {
    let src = format!(
        "add.xyzw vf1, vf2, vf3 | div q, vf2x, vf3y\nnop | esadd p, vf2\nmul.xyzw vf4, vf1, vf1 | iaddiu vi1, vi0, 0x2\nloop: nop | isubiu vi1, vi1, 0x1\n\
         nop | ibne vi1, vi0, loop\naddq.x vf5, vf0, q | fsand vi2, 0xfff\nnop | waitp\nnop | mfp.x vf6, p\n{END}"
    );
    let prog = table::assemble(&src).unwrap();
    let prepare = |s: &mut Side| {
        for (k, p) in prog.iter().enumerate() {
            put(&mut s.u, 1, k, *p);
        }
        s.u.vu1.set_vf(2, [0x3F80_0000, 0x4000_0000, 0x4040_0000, 0x4080_0000]);
        s.u.vu1.set_vf(3, [0x3F00_0000, 0x3E80_0000, 0x7FFF_FFFF, 0x0000_0001]);
    };
    for cut in 11..60 {
        let mut i = Side::new(1, 0, None);
        let mut c = Side::new(1, 0, Some(settings(true)));
        prepare(&mut i);
        prepare(&mut c);
        for s in [&mut i, &mut c] {
            assert!(s.p.start(&mut s.u, 0, 10));
            s.p.run_to(&mut s.u, cut);
        }
        let state = bytes(|w| {
            c.p.write_state(w);
            w.structure("Vu1", &c.u.vu1);
        });
        let mut loaded = Side::new(1, 0, Some(settings(true)));
        let mut rd = StateReader::new(&state);
        loaded.p.read_state(&mut rd).unwrap();
        loaded.u.vu1.read_state(&mut rd).unwrap();
        for t in cut + 1..cut + 60 {
            for s in [&mut i, &mut loaded] {
                s.p.run_to(&mut s.u, t);
            }
            same(&i, &loaded, &format!("cut at {cut}, cycle {t}"));
        }
        let k = loaded.counts();
        assert!(k.programs <= 3 && k.interpreted_pairs == 0, "compiled from the address it was loaded at: {k:?}");
    }
}

/// The debugger's probe asked before every compiled pair: a breakpoint holds the unit in front of it, and coverage lists
/// the pairs run, as on the interpreter.
#[test]
fn the_debugger_s_probe_holds_compiled_code_at_a_breakpoint() {
    let src = format!("nop | iaddiu vi1, vi0, 0x1\nnop | iaddiu vi2, vi0, 0x2\nnop | iaddiu vi3, vi0, 0x3\nnop | iaddiu vi4, vi0, 0x4\n{END}");
    let prog = table::assemble(&src).unwrap();
    let mut i = Side::new(1, 0, None);
    let mut c = Side::new(1, 0, Some(settings(true)));
    for s in [&mut i, &mut c] {
        for (k, p) in prog.iter().enumerate() {
            put(&mut s.u, 1, k, *p);
        }
        s.p.probe = Some(Box::new(sapphire::debugger::Probe { breakpoints: vec![(16, 16)], coverage: true, ..Default::default() }));
        s.u.halts = true;
        assert!(s.p.start(&mut s.u, 0, 10));
        s.p.run_to(&mut s.u, 40);
    }
    same(&i, &c, "held at the breakpoint");
    let (pi, pc) = (i.p.probe.as_ref().unwrap(), c.p.probe.as_ref().unwrap());
    assert_eq!((pi.halted, &pi.steps), (pc.halted, &pc.steps));
    assert_eq!(pc.halted, Some(16));
    assert!(c.counts().calls > 0);
    for s in [&mut i, &mut c] {
        s.p.probe.as_mut().unwrap().resume = true;
        s.p.run_to(&mut s.u, 80);
    }
    same(&i, &c, "resumed");
    assert_eq!(i.p.probe.as_ref().unwrap().steps, c.p.probe.as_ref().unwrap().steps);
}

/// A microprogram that never ends, run by `finish` to the bound of 2^24 pairs, stops on the same pair.
#[test]
fn finish_s_bound_stops_compiled_code_on_the_same_pair() {
    let src = "loop: nop | iaddiu vi1, vi1, 0x1\nnop | b loop\nnop | iaddiu vi2, vi2, 0x1";
    let prog = table::assemble(src).unwrap();
    let mut i = Side::new(1, 0, None);
    let mut c = Side::new(1, 0, Some(settings(false)));
    for s in [&mut i, &mut c] {
        for (k, p) in prog.iter().enumerate() {
            put(&mut s.u, 1, k, *p);
        }
        assert!(s.p.start(&mut s.u, 0, 10));
    }
    let (a, b) = (i.p.finish(&mut i.u, 10), c.p.finish(&mut c.u, 10));
    assert_eq!(a, b);
    same(&i, &c, "at the bound");
    assert!(c.p.busy());
    assert_eq!(c.counts().compiled_pairs, 1 << 24);
}

/// The trace a test asks for is kept by compiled code as by the interpreter.
#[test]
fn the_trace_is_kept_alike() {
    let src = format!("mul.xyzw vf5, vf10, vf20 | move vf0, vf0\nmul.xyzw vf7, vf5, vf24 | move vf0, vf0\nnop | div q, vf1x, vf2x\nnop | waitq\n{END}");
    let prog = table::assemble(&src).unwrap();
    let mut i = Side::new(1, 0, None);
    let mut c = Side::new(1, 0, Some(settings(true)));
    for s in [&mut i, &mut c] {
        for (k, p) in prog.iter().enumerate() {
            put(&mut s.u, 1, k, *p);
        }
        s.p.trace = Some(Vec::new());
        assert!(s.p.start(&mut s.u, 0, 10));
        s.p.finish(&mut s.u, 10);
    }
    assert_eq!(i.p.trace, c.p.trace);
    same(&i, &c, "traced");
}

/// C4 (Sapphire_Plan.md §5.13), off by default: with it on, the flags are written at the pair's own cycle, so a reader
/// within four pairs sees them early and the units part from the interpreter; the default is the exact tier above.
#[test]
fn c4_writes_the_flags_early_and_parts_from_the_interpreter() {
    let src = format!("add.xyzw vf1, vf2, vf3 | move vf0, vf0\nnop | fsand vi3, 0xfff\nnop | fmand vi1, vi2\n{END}");
    let prog = table::assemble(&src).unwrap();
    let mut i = Side::new(1, 0, None);
    let mut c = Side::new(1, 0, Some(Settings { c4: true, ..settings(false) }));
    for s in [&mut i, &mut c] {
        for (k, p) in prog.iter().enumerate() {
            put(&mut s.u, 1, k, *p);
        }
        s.u.vu1.set_vf(2, [0x3F80_0000, 0xBF80_0000, 0, 0x7FFF_FFFF]);
        s.u.vu1.set_vf(3, [0xBF80_0000, 0x3F80_0000, 0, 0x7FFF_FFFF]);
        s.u.vu1.set_vi(2, 0xFFFF);
        assert!(s.p.start(&mut s.u, 0, 10));
        s.p.finish(&mut s.u, 10);
    }
    assert_ne!((i.u.vu1.vi(1), i.u.vu1.vi(3)), (c.u.vu1.vi(1), c.u.vu1.vi(3)), "the flag checks read the flags early");
    assert_eq!((i.u.vu1.mac, i.u.vu1.status), (c.u.vu1.mac, c.u.vu1.status), "the flags themselves end the same");
}
