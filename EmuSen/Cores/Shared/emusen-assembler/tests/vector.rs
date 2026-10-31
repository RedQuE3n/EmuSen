//! The vector capability's grading (1.3, EmuSen_Recompiler.md §14): every operation against a lane model, in every register.

use emusen_assembler::x64::{self, *};
use emusen_assembler::*;

/// Executable pages holding a block's hot section and data; freed on drop.
struct Exec {
    base: *mut u8,
    len: usize,
}

impl Exec {
    fn new(mut code: Code) -> Exec {
        let len = 1 << 16;
        // SAFETY: an anonymous private mapping, written below before it is made executable.
        let base = unsafe { libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_PRIVATE | libc::MAP_ANONYMOUS, -1, 0) } as *mut u8;
        assert!(!base.is_null() && base as isize != -1, "mapped");
        let (hot, cold, data) = (base as u64, base as u64 + 0x4000, base as u64 + 0x8000);
        assert!(code.link(hot, cold, data, &|_| 0), "linked");
        // SAFETY: inside the mapping, which nothing else uses.
        unsafe {
            std::ptr::copy_nonoverlapping(code.hot.as_ptr(), hot as *mut u8, code.hot.len());
            std::ptr::copy_nonoverlapping(code.cold.as_ptr(), cold as *mut u8, code.cold.len());
            std::ptr::copy_nonoverlapping(code.data.as_ptr(), data as *mut u8, code.data.len());
            assert_eq!(libc::mprotect(base.cast(), len, libc::PROT_READ | libc::PROT_EXEC), 0);
        }
        Exec { base, len }
    }

    /// Runs the code as `fn(*mut [u64; 8])`.
    fn run(&self, io: &mut [u64; 8]) {
        // SAFETY: written as this signature by `frame`, keeping to the System V convention.
        let f: extern "C" fn(*mut [u64; 8]) = unsafe { std::mem::transmute(self.base) };
        f(io)
    }
}

impl Drop for Exec {
    fn drop(&mut self) {
        // SAFETY: the mapping made in `new`, no longer called.
        unsafe { libc::munmap(self.base.cast(), self.len) };
    }
}

/// `fn(io)`: `body` run with rdi holding io; every vector register is the caller's to clobber.
fn frame(body: impl FnOnce(&mut X64)) -> Code {
    let mut a = X64::new();
    let entry = a.label();
    a.bind(entry);
    body(&mut a);
    a.ret();
    a.finish(entry)
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    /// Lanes with edges: zero, ones, each lane width's signed limits, random.
    fn quad(&mut self) -> u128 {
        let pick = |r: &mut Rng| -> u64 {
            match r.next() % 6 {
                0 => 0,
                1 => u64::MAX,
                2 => 0x8000_8000_8000_8000,
                3 => 0x7FFF_7FFF_8000_0000,
                4 => 0x80FF_7F01_FF80_007F,
                _ => r.next(),
            }
        };
        u128::from(pick(self)) | u128::from(pick(self)) << 64
    }
    fn vreg(&mut self) -> Vreg {
        Vreg((self.next() % 16) as u8)
    }
}

fn lane(v: u128, w: u32, i: u32) -> u64 {
    (v >> (i * w)) as u64 & if w == 64 { u64::MAX } else { (1 << w) - 1 }
}
fn signed(x: u64, w: u32) -> i64 {
    ((x << (64 - w)) as i64) >> (64 - w)
}
fn build(w: u32, f: impl Fn(u32) -> u64) -> u128 {
    (0..128 / w).fold(0, |r, i| r | u128::from(f(i) & if w == 64 { u64::MAX } else { (1 << w) - 1 }) << (i * w))
}
fn zip(a: u128, b: u128, w: u32, f: impl Fn(u64, u64) -> u64) -> u128 {
    build(w, |i| f(lane(a, w, i), lane(b, w, i)))
}
fn sat_s(v: i64, w: u32) -> u64 {
    v.clamp(-(1 << (w - 1)), (1 << (w - 1)) - 1) as u64
}
fn sat_u(v: i64, w: u32) -> u64 {
    v.clamp(0, (1 << w) - 1) as u64
}
fn ones(c: bool) -> u64 {
    if c { u64::MAX } else { 0 }
}
fn f(x: u64) -> f32 {
    f32::from_bits(x as u32)
}

fn model(op: VOp, a: u128, b: u128) -> u128 {
    use VOp::*;
    let add = |w| zip(a, b, w, |x, y| x.wrapping_add(y));
    let sub = |w| zip(a, b, w, |x, y| x.wrapping_sub(y));
    let s = |w, g: fn(i64, i64) -> i64, sat: fn(i64, u32) -> u64| zip(a, b, w, move |x, y| sat(g(signed(x, w), signed(y, w)), w));
    let u = |w, g: fn(i64, i64) -> i64| zip(a, b, w, move |x, y| sat_u(g(x as i64, y as i64), w));
    let unpack = |w: u32, from: u32| build(w, |i| lane(if i % 2 == 0 { a } else { b }, w, from + i / 2));
    let pack = |w: u32, sat: fn(i64, u32) -> u64| build(w / 2, |i| sat(signed(lane(if i < 128 / w { a } else { b }, w, i % (128 / w)), w), w / 2));
    let fl = |g: fn(f32, f32) -> f32| zip(a, b, 32, move |x, y| u64::from(g(f(x), f(y)).to_bits()));
    match op {
        AddB => add(8),
        AddH => add(16),
        AddW => add(32),
        AddQ => add(64),
        SubB => sub(8),
        SubH => sub(16),
        SubW => sub(32),
        SubQ => sub(64),
        AddSB => s(8, |x, y| x + y, sat_s),
        AddSH => s(16, |x, y| x + y, sat_s),
        SubSB => s(8, |x, y| x - y, sat_s),
        SubSH => s(16, |x, y| x - y, sat_s),
        AddUB => u(8, |x, y| x + y),
        AddUH => u(16, |x, y| x + y),
        SubUB => u(8, |x, y| x - y),
        SubUH => u(16, |x, y| x - y),
        MinSH => zip(a, b, 16, |x, y| if signed(x, 16) < signed(y, 16) { x } else { y }),
        MaxSH => zip(a, b, 16, |x, y| if signed(x, 16) > signed(y, 16) { x } else { y }),
        MinSW => zip(a, b, 32, |x, y| if signed(x, 32) < signed(y, 32) { x } else { y }),
        MaxSW => zip(a, b, 32, |x, y| if signed(x, 32) > signed(y, 32) { x } else { y }),
        MinUB => zip(a, b, 8, |x, y| x.min(y)),
        MaxUB => zip(a, b, 8, |x, y| x.max(y)),
        MinUH => zip(a, b, 16, |x, y| x.min(y)),
        MaxUH => zip(a, b, 16, |x, y| x.max(y)),
        MinUW => zip(a, b, 32, |x, y| x.min(y)),
        MaxUW => zip(a, b, 32, |x, y| x.max(y)),
        EqB => zip(a, b, 8, |x, y| ones(x == y)),
        EqH => zip(a, b, 16, |x, y| ones(x == y)),
        EqW => zip(a, b, 32, |x, y| ones(x == y)),
        GtB => zip(a, b, 8, |x, y| ones(signed(x, 8) > signed(y, 8))),
        GtH => zip(a, b, 16, |x, y| ones(signed(x, 16) > signed(y, 16))),
        GtW => zip(a, b, 32, |x, y| ones(signed(x, 32) > signed(y, 32))),
        And => a & b,
        AndNot => !a & b,
        Or => a | b,
        Xor => a ^ b,
        LowB => unpack(8, 0),
        LowH => unpack(16, 0),
        LowW => unpack(32, 0),
        LowQ => unpack(64, 0),
        HighB => unpack(8, 8),
        HighH => unpack(16, 4),
        HighW => unpack(32, 2),
        HighQ => unpack(64, 1),
        PackSH => pack(16, sat_s),
        PackSW => pack(32, sat_s),
        PackUH => pack(16, sat_u),
        PackUW => pack(32, sat_u),
        MulLowH => zip(a, b, 16, |x, y| x.wrapping_mul(y)),
        MulHighSH => zip(a, b, 16, |x, y| ((signed(x, 16) * signed(y, 16)) >> 16) as u64),
        MulLowW => zip(a, b, 32, |x, y| x.wrapping_mul(y)),
        MulEvenSW => zip(a, b, 64, |x, y| (signed(x & 0xFFFF_FFFF, 32) * signed(y & 0xFFFF_FFFF, 32)) as u64),
        MulEvenUW => zip(a, b, 64, |x, y| (x & 0xFFFF_FFFF) * (y & 0xFFFF_FFFF)),
        MulAddH => build(32, |i| (signed(lane(a, 16, 2 * i), 16) * signed(lane(b, 16, 2 * i), 16) + signed(lane(a, 16, 2 * i + 1), 16) * signed(lane(b, 16, 2 * i + 1), 16)) as u64),
        ShuffleB => build(8, |i| {
            let c = lane(b, 8, i);
            if c & 0x80 != 0 { 0 } else { lane(a, 8, (c & 15) as u32) }
        }),
        AddF => fl(|x, y| x + y),
        SubF => fl(|x, y| x - y),
        MulF => fl(|x, y| x * y),
        MinF => fl(|x, y| if x < y { x } else { y }),
        MaxF => fl(|x, y| if x > y { x } else { y }),
    }
}

const OPS: [VOp; 60] = {
    use VOp::*;
    [
        AddB, AddH, AddW, AddQ, SubB, SubH, SubW, SubQ, AddSB, AddSH, AddUB, AddUH, SubSB, SubSH, SubUB, SubUH, MinSH, MaxSH, MinSW, MaxSW, MinUB,
        MaxUB, MinUH, MaxUH, MinUW, MaxUW, EqB, EqH, EqW, GtB, GtH, GtW, And, AndNot, Or, Xor, LowB, LowH, LowW, LowQ, HighB, HighH, HighW, HighQ,
        PackSH, PackSW, PackUH, PackUW, MulLowH, MulHighSH, MulLowW, MulEvenSW, MulEvenUW, MulAddH, ShuffleB, AddF, SubF, MulF, MinF, MaxF,
    ]
};

fn io_of(a: u128, b: u128) -> [u64; 8] {
    [a as u64, (a >> 64) as u64, b as u64, (b >> 64) as u64, 0, 0, 0, 0]
}
fn out_of(io: &[u64; 8]) -> u128 {
    u128::from(io[4]) | u128::from(io[5]) << 64
}

/// Two quads from io in `ra` and `rb`, `body`'s result in `rd` stored back to io[4..6].
fn run2(ra: Vreg, rb: Vreg, rd: Vreg, a: u128, b: u128, body: impl FnOnce(&mut X64)) -> u128 {
    let code = frame(|m| {
        m.vload(ra, at(RDI, 0));
        if rb != ra {
            m.vload(rb, at(RDI, 16));
        }
        body(m);
        m.vstore(rd, at(RDI, 32));
    });
    let mut io = io_of(a, b);
    Exec::new(code).run(&mut io);
    out_of(&io)
}

#[test]
fn every_vector_operation_matches_its_lane_model_in_every_register() {
    if !x64::vector_supported() {
        return;
    }
    let mut rng = Rng(0x5EED_0F_7EC7_0125);
    for op in OPS {
        for round in 0..200 {
            let (ra, rb, rd) = (rng.vreg(), rng.vreg(), rng.vreg());
            let (a, mut b) = (rng.quad(), rng.quad());
            if rb == ra {
                b = a;
            }
            if matches!(op, VOp::AddF | VOp::SubF | VOp::MulF | VOp::MinF | VOp::MaxF) && round % 2 == 0 {
                // Ordinary floats, so the model's arithmetic is the host's under the default control word.
                let q = |r: &mut Rng| { let v: Vec<u64> = (0..4).map(|_| u64::from((r.next() as f32 / 3.0e9).to_bits())).collect(); build(32, |i| v[i as usize]) };
                let (x, y) = (q(&mut rng), q(&mut rng));
                let y = if rb == ra { x } else { y };
                let got = run2(ra, rb, rd, x, y, |m| m.vop(op, rd, ra, rb));
                assert_eq!(got, model(op, x, y), "{op:?} d={rd:?} a={ra:?} b={rb:?}");
                continue;
            }
            if matches!(op, VOp::AddF | VOp::SubF | VOp::MulF | VOp::MinF | VOp::MaxF) {
                continue;
            }
            let got = run2(ra, rb, rd, a, b, |m| m.vop(op, rd, ra, rb));
            assert_eq!(got, model(op, a, b), "{op:?} d={rd:?} a={ra:?} b={rb:?} {a:032x} {b:032x}");
        }
    }
}

#[test]
fn shifts_shuffles_blends_and_absolutes_match_their_models() {
    if !x64::vector_supported() {
        return;
    }
    let mut rng = Rng(0x0BAD_C0DE_1234_5678);
    for _ in 0..2000 {
        let (ra, rd) = (rng.vreg(), rng.vreg());
        let a = rng.quad();
        let by = (rng.next() % 70) as u8;
        let k = rng.next() % 8;
        let (op, want) = match k {
            0 => (VShift::LeftH, build(16, |i| if by < 16 { lane(a, 16, i) << by } else { 0 })),
            1 => (VShift::LeftW, build(32, |i| if by < 32 { lane(a, 32, i) << by } else { 0 })),
            2 => (VShift::LeftQ, build(64, |i| if by < 64 { lane(a, 64, i) << by } else { 0 })),
            3 => (VShift::RightH, build(16, |i| if by < 16 { lane(a, 16, i) >> by } else { 0 })),
            4 => (VShift::RightW, build(32, |i| if by < 32 { lane(a, 32, i) >> by } else { 0 })),
            5 => (VShift::RightQ, build(64, |i| if by < 64 { lane(a, 64, i) >> by } else { 0 })),
            6 => (VShift::ArithH, build(16, |i| (signed(lane(a, 16, i), 16) >> by.min(15)) as u64)),
            _ => (VShift::ArithW, build(32, |i| (signed(lane(a, 32, i), 32) >> by.min(31)) as u64)),
        };
        assert_eq!(run2(ra, ra, rd, a, a, |m| m.vshift(op, rd, ra, by)), want, "{op:?} by {by}");
        let order = rng.next() as u8;
        let pick = |i: u32| u32::from(order >> (2 * i) & 3);
        for (s, want) in [
            (VShuffle::W, build(32, |i| lane(a, 32, pick(i)))),
            (VShuffle::LowH, build(16, |i| if i < 4 { lane(a, 16, pick(i)) } else { lane(a, 16, i) })),
            (VShuffle::HighH, build(16, |i| if i < 4 { lane(a, 16, i) } else { lane(a, 16, 4 + pick(i - 4)) })),
        ] {
            assert_eq!(run2(ra, ra, rd, a, a, |m| m.vshuffle(s, rd, ra, order)), want, "{s:?} {order:02x}");
        }
        for (u, w) in [(VUnary::AbsB, 8), (VUnary::AbsH, 16), (VUnary::AbsW, 32)] {
            let want = build(w, |i| signed(lane(a, w, i), w).unsigned_abs());
            assert_eq!(run2(ra, ra, rd, a, a, |m| m.vunary(u, rd, ra)), want, "{u:?}");
        }
        let rb = rng.vreg();
        let b = if rb == ra { a } else { rng.quad() };
        let mask = rng.next() as u8;
        let want = build(16, |i| lane(if mask >> i & 1 == 1 { b } else { a }, 16, i));
        assert_eq!(run2(ra, rb, rd, a, b, |m| m.vblend_h(rd, ra, rb, mask)), want, "blend {mask:02x}");
        assert_eq!(run2(ra, ra, rd, a, a, |m| m.vmov(rd, ra)), a, "mov");
    }
}

#[test]
fn joins_splits_data_and_the_control_register_reach_their_places() {
    if !x64::vector_supported() {
        return;
    }
    let mut rng = Rng(0x7777_1111_2222_3333);
    let gprs: Vec<Reg> = x64::ALL.iter().copied().filter(|&r| r != RDI).collect();
    for _ in 0..500 {
        let (lo, hi) = (gprs[(rng.next() % 14) as usize], gprs[(rng.next() % 14) as usize]);
        let v = rng.vreg();
        let q = rng.quad();
        let code = frame(|m| {
            m.push(RBX);
            m.push(RBP);
            m.push(R12);
            m.push(R13);
            m.push(R14);
            m.push(R15);
            m.load(8, false, lo, at(RDI, 0));
            m.load(8, false, hi, at(RDI, 8));
            m.vjoin(v, lo, if hi == lo { lo } else { hi });
            m.vstore(v, at(RDI, 32));
            if hi != lo {
                m.vsplit(hi, lo, v);
                m.store(8, hi, at(RDI, 48));
                m.store(8, lo, at(RDI, 56));
            }
            m.pop(R15);
            m.pop(R14);
            m.pop(R13);
            m.pop(R12);
            m.pop(RBP);
            m.pop(RBX);
        });
        let mut io = io_of(q, 0);
        Exec::new(code).run(&mut io);
        if hi == lo {
            assert_eq!(out_of(&io), u128::from((q >> 64) as u64) * ((1 << 64) + 1), "join of one register");
        } else {
            assert_eq!(out_of(&io), q, "join {lo:?} {hi:?} {v:?}");
            assert_eq!((io[6], io[7]), (q as u64, (q >> 64) as u64), "split {lo:?} {hi:?} {v:?}");
        }
    }
    let q = rng.quad();
    let code = frame(|m| {
        let k = m.data(&q.to_le_bytes());
        m.vload_data(Vreg(9), k);
        m.vstore(Vreg(9), at(RDI, 32));
        // The control word with rounding towards zero, read back, then the caller's restored.
        m.fpcr_store(at(RDI, 48));
        m.load(4, false, RAX, at(RDI, 48));
        m.alu(Op::Or, Width::W32, RAX, RAX, Src::Imm(0x6000));
        m.store(4, RAX, at(RDI, 52));
        m.fpcr_load(at(RDI, 52));
        m.fpcr_store(at(RDI, 56));
        m.fpcr_load(at(RDI, 48));
    });
    let mut io = [0; 8];
    Exec::new(code).run(&mut io);
    assert_eq!(out_of(&io), q, "data");
    assert_eq!(io[7] as u32 & 0xFFFF, (io[6] >> 32) as u32 & 0xFFFF, "the control word as loaded");
    assert_eq!(io[7] as u32 & 0x6000, 0x6000, "rounding towards zero");
}
