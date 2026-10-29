//! The backend's own grading (EmuSen_Recompiler.md §7.1, §14). See EmuSen_Recompiler.md §14.

use emusen_assembler::x64::{self, *};
use emusen_assembler::*;

/// Executable pages holding a block's sections; freed on drop.
struct Exec {
    base: *mut u8,
    len: usize,
    entry: *const u8,
}

impl Exec {
    fn new(code: Code, stubs: &[Code]) -> Exec {
        Exec::with(code, stubs, None)
    }

    /// As `new`, the 8-byte data word at `slot` holding the cold section's address.
    fn with(mut code: Code, stubs: &[Code], slot: Option<u32>) -> Exec {
        let len = 1 << 20;
        // SAFETY: an anonymous private mapping, written below before it is made executable.
        let base = unsafe { libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_PRIVATE | libc::MAP_ANONYMOUS, -1, 0) } as *mut u8;
        assert!(!base.is_null() && base as isize != -1, "mapped");
        let (hot, cold, data, stub0) = (base as u64, base as u64 + 0x40000, base as u64 + 0x80000, base as u64 + 0xC0000);
        let mut stub_at = Vec::new();
        let mut at = stub0;
        for s in stubs {
            let mut s = s.clone();
            assert!(s.link(at, at + s.hot.len() as u64, at, &|_| 0));
            // SAFETY: inside the mapping.
            unsafe { std::ptr::copy_nonoverlapping(s.hot.as_ptr(), at as *mut u8, s.hot.len()) };
            stub_at.push(at + u64::from(s.entry));
            at += (s.hot.len() as u64 + 15) & !15;
        }
        assert!(code.link(hot, cold, data, &|id| stub_at[id as usize]), "linked");
        if let Some(k) = slot {
            code.data[k as usize..k as usize + 8].copy_from_slice(&cold.to_le_bytes());
        }
        // SAFETY: inside the mapping, which nothing else uses.
        unsafe {
            std::ptr::copy_nonoverlapping(code.hot.as_ptr(), hot as *mut u8, code.hot.len());
            std::ptr::copy_nonoverlapping(code.cold.as_ptr(), cold as *mut u8, code.cold.len());
            std::ptr::copy_nonoverlapping(code.data.as_ptr(), data as *mut u8, code.data.len());
            assert_eq!(libc::mprotect(base.cast(), len, libc::PROT_READ | libc::PROT_EXEC), 0);
        }
        let entry = if code.entry_cold { cold } else { hot } + u64::from(code.entry);
        Exec { base, len, entry: entry as *const u8 }
    }

    /// Runs the code as `fn(*mut [u64; 8]) -> u64`.
    fn run(&self, io: &mut [u64; 8]) -> u64 {
        // SAFETY: the code was written as this signature by `frame`, and keeps to the System V convention.
        let f: extern "C" fn(*mut [u64; 8]) -> u64 = unsafe { std::mem::transmute(self.entry) };
        f(io)
    }
}

impl Drop for Exec {
    fn drop(&mut self) {
        // SAFETY: the mapping made in `new`, no longer called.
        unsafe { libc::munmap(self.base.cast(), self.len) };
    }
}

/// A small generator, so that a failure reproduces.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    /// Values with edges: small, signed limits, 32-bit limits, random.
    fn value(&mut self) -> u64 {
        const EDGES: [u64; 10] = [0, 1, 2, 31, 32, 63, 0x7FFF_FFFF, 0x8000_0000, 0xFFFF_FFFF, 0x8000_0000_0000_0000];
        match self.next() % 4 {
            0 => EDGES[(self.next() % 10) as usize],
            1 => EDGES[(self.next() % 10) as usize].wrapping_neg(),
            _ => self.next(),
        }
    }
    fn reg(&mut self) -> Reg {
        x64::ALL[(self.next() % 15) as usize]
    }
}

const SAVED: [Reg; 6] = [RBX, RBP, R12, R13, R14, R15];

/// A function `fn(io) -> u64`: the registers `regs` loaded from io[0..], `body` run, `out` returned.
fn frame(regs: &[Reg], body: impl FnOnce(&mut X64), out: Reg) -> Code {
    let mut a = X64::new();
    let entry = a.label();
    a.bind(entry);
    for x in SAVED {
        a.push(x);
    }
    a.push(RDI);
    // rdi last, since it holds io.
    let mut order: Vec<(usize, Reg)> = regs.iter().copied().enumerate().collect();
    order.sort_by_key(|&(_, x)| x == RDI);
    for (k, x) in order {
        a.load(8, false, x, at(RDI, 8 * k as i32));
    }
    body(&mut a);
    a.mov(Width::W64, RAX, out);
    a.pop(RDI);
    for x in SAVED.iter().rev() {
        a.pop(*x);
    }
    a.ret();
    a.finish(entry)
}

fn distinct(rng: &mut Rng, n: usize) -> Vec<Reg> {
    let mut v: Vec<Reg> = Vec::new();
    while v.len() < n {
        let x = rng.reg();
        if !v.contains(&x) {
            v.push(x);
        }
    }
    v
}

fn width_of(w: Width, v: u64) -> u64 {
    match w {
        Width::W32 => v & 0xFFFF_FFFF,
        Width::W64 => v,
    }
}

fn alu_model(op: Op, w: Width, a: u64, b: u64) -> u64 {
    width_of(
        w,
        match op {
            Op::Add => a.wrapping_add(b),
            Op::Sub => a.wrapping_sub(b),
            Op::And => a & b,
            Op::Or => a | b,
            Op::Xor => a ^ b,
        },
    )
}

const ROUNDS: usize = 300;

#[test]
fn every_alu_operation_matches_its_model_in_every_register_and_form() {
    let mut rng = Rng(0x1234_5678_9ABC_DEF1);
    for op in [Op::Add, Op::Sub, Op::And, Op::Or, Op::Xor] {
        for w in [Width::W32, Width::W64] {
            for _ in 0..ROUNDS {
                let rs = distinct(&mut rng, 2);
                let d = if rng.next() % 3 == 0 { rs[(rng.next() % 2) as usize] } else { rng.reg() };
                let imm = rng.next() % 2 == 0;
                let k = match rng.next() % 3 {
                    0 => (rng.next() as i8) as i32,
                    _ => rng.next() as i32,
                };
                let (va, vb) = (rng.value(), rng.value());
                let code = frame(&rs, |a| a.alu(op, w, d, rs[0], if imm { Src::Imm(k) } else { Src::Reg(rs[1]) }), d);
                let mut io = [0; 8];
                io[0] = va;
                io[1] = vb;
                let got = Exec::new(code, &[]).run(&mut io);
                let b = if imm { k as i64 as u64 } else { vb };
                assert_eq!(got, alu_model(op, w, va, b), "{op:?} {w:?} d={d:?} a={:?} b={:?} imm={imm} {va:x} {b:x}", rs[0], rs[1]);
            }
        }
    }
}

#[test]
fn shifts_negation_and_extension_match_their_models() {
    let mut rng = Rng(0xDEAD_BEEF_0BAD_F00D);
    for _ in 0..ROUNDS * 4 {
        let rs = distinct(&mut rng, 2);
        let d = if rng.next() % 3 == 0 { rs[0] } else { rng.reg() };
        let w = if rng.next() % 2 == 0 { Width::W32 } else { Width::W64 };
        let (va, vb) = (rng.value(), rng.value());
        let bits = if w == Width::W32 { 31 } else { 63 };
        let kind = rng.next() % 9;
        let k = (rng.next() % (bits + 1)) as u8;
        let (code, want) = match kind {
            0..=2 => {
                let op = [ShiftOp::Shl, ShiftOp::Shr, ShiftOp::Sar][kind as usize];
                let n = (vb & bits) as u32;
                let want = match (op, w) {
                    (ShiftOp::Shl, _) => width_of(w, va << n),
                    (ShiftOp::Shr, _) => width_of(w, width_of(w, va) >> n),
                    (ShiftOp::Sar, Width::W32) => ((va as i32) >> n) as u32 as u64,
                    (ShiftOp::Sar, Width::W64) => ((va as i64) >> n) as u64,
                };
                (frame(&rs, |a| a.shift(op, w, d, rs[0], By::Reg(rs[1])), d), want)
            }
            3..=5 => {
                let op = [ShiftOp::Shl, ShiftOp::Shr, ShiftOp::Sar][kind as usize - 3];
                let n = u32::from(k);
                let want = match (op, w) {
                    (ShiftOp::Shl, _) => width_of(w, va << n),
                    (ShiftOp::Shr, _) => width_of(w, width_of(w, va) >> n),
                    (ShiftOp::Sar, Width::W32) => ((va as i32) >> n) as u32 as u64,
                    (ShiftOp::Sar, Width::W64) => ((va as i64) >> n) as u64,
                };
                (frame(&rs, |a| a.shift(op, w, d, rs[0], By::Imm(k)), d), want)
            }
            6 => (frame(&rs, |a| a.not(w, d, rs[0]), d), width_of(w, !va)),
            7 => (frame(&rs, |a| a.neg(w, d, rs[0]), d), width_of(w, va.wrapping_neg())),
            _ => (frame(&rs, |a| a.sext32(d, rs[0]), d), va as i32 as i64 as u64),
        };
        let mut io = [va, vb, 0, 0, 0, 0, 0, 0];
        assert_eq!(Exec::new(code, &[]).run(&mut io), want, "kind {kind} {w:?} d={d:?} rs={rs:?} {va:x} {vb:x} k={k}");
    }
}

fn holds(c: Cond, w: Width, a: u64, b: u64) -> bool {
    let (a, b) = (width_of(w, a), width_of(w, b));
    let (sa, sb) = match w {
        Width::W32 => (i64::from(a as u32 as i32), i64::from(b as u32 as i32)),
        Width::W64 => (a as i64, b as i64),
    };
    match c {
        Cond::Eq => a == b,
        Cond::Ne => a != b,
        Cond::Lt => sa < sb,
        Cond::Ge => sa >= sb,
        Cond::Le => sa <= sb,
        Cond::Gt => sa > sb,
        Cond::Ltu => a < b,
        Cond::Geu => a >= b,
        Cond::Leu => a <= b,
        Cond::Gtu => a > b,
        Cond::Overflow => unreachable!("not a compare"),
    }
}

const CONDS: [Cond; 10] = [Cond::Eq, Cond::Ne, Cond::Lt, Cond::Ge, Cond::Le, Cond::Gt, Cond::Ltu, Cond::Geu, Cond::Leu, Cond::Gtu];

#[test]
fn compares_set_select_and_branch_as_their_models() {
    let mut rng = Rng(0x0F0F_1234_ABCD_9876);
    for _ in 0..ROUNDS * 6 {
        let rs = distinct(&mut rng, 3);
        let c = CONDS[(rng.next() % 10) as usize];
        let w = if rng.next() % 2 == 0 { Width::W32 } else { Width::W64 };
        let mut va = rng.value();
        let mut vb = rng.value();
        if rng.next() % 4 == 0 {
            vb = va;
        }
        let imm = rng.next() % 3 == 0;
        let k = if rng.next() % 2 == 0 { (rng.next() as i8) as i32 } else { rng.next() as i32 };
        if imm {
            vb = match w {
                Width::W32 => k as u32 as u64,
                Width::W64 => k as i64 as u64,
            };
            if rng.next() % 4 == 0 {
                va = vb;
            }
        }
        let src = if imm { Src::Imm(k) } else { Src::Reg(rs[1]) };
        let want = holds(c, w, va, vb);
        let form = rng.next() % 4;
        let code = frame(
            &rs,
            |a| match form {
                1 if !rs.contains(&RAX) => {
                    a.mov_imm(rs[2], 7);
                    a.mov_imm(RAX, 9);
                    a.cmp(w, rs[0], src);
                    a.select(c, rs[2], RAX);
                }
                0 | 1 => {
                    a.cmp(w, rs[0], src);
                    a.set(c, rs[2]);
                }
                2 => {
                    let (yes, end) = (a.label(), a.label());
                    a.cmp(w, rs[0], src);
                    a.branch(c, yes);
                    a.mov_imm(rs[2], 0);
                    a.jump(end);
                    a.bind(yes);
                    a.mov_imm(rs[2], 1);
                    a.bind(end);
                }
                _ => {
                    // The taken side in the cold section, far away: rel32 and a relocation.
                    let (yes, back) = (a.label(), a.label());
                    a.cmp(w, rs[0], src);
                    a.mov_imm(rs[2], 0);
                    a.branch(c, yes);
                    a.bind(back);
                    a.section(Section::Cold);
                    for _ in 0..40 {
                        a.mov_imm(R11, 0x1234_5678_9ABC);
                    }
                    a.bind(yes);
                    a.mov_imm(rs[2], 1);
                    a.jump(back);
                    a.section(Section::Hot);
                }
            },
            rs[2],
        );
        let mut io = [va, vb, 0, 0, 0, 0, 0, 0];
        let got = Exec::new(code, &[]).run(&mut io);
        let expect = match form {
            1 if !rs.contains(&RAX) => {
                if want { 9 } else { 7 }
            }
            _ => u64::from(want),
        };
        assert_eq!(got, expect, "form {form} {c:?} {w:?} imm={imm} {va:x} {vb:x} rs={rs:?}");
    }
}

#[test]
fn overflow_is_the_signed_overflow_of_add_and_sub() {
    let mut rng = Rng(0x5555_AAAA_1111_2222);
    for _ in 0..ROUNDS * 2 {
        let rs = distinct(&mut rng, 3);
        let w = if rng.next() % 2 == 0 { Width::W32 } else { Width::W64 };
        let sub = rng.next() % 2 == 0;
        let (va, vb) = (rng.value(), rng.value());
        let code = frame(
            &rs,
            |a| {
                a.alu(if sub { Op::Sub } else { Op::Add }, w, rs[0], rs[0], Src::Reg(rs[1]));
                a.set(Cond::Overflow, rs[2]);
            },
            rs[2],
        );
        let want = match (w, sub) {
            (Width::W32, false) => (va as i32).checked_add(vb as i32).is_none(),
            (Width::W32, true) => (va as i32).checked_sub(vb as i32).is_none(),
            (Width::W64, false) => (va as i64).checked_add(vb as i64).is_none(),
            (Width::W64, true) => (va as i64).checked_sub(vb as i64).is_none(),
        };
        let mut io = [va, vb, 0, 0, 0, 0, 0, 0];
        assert_eq!(Exec::new(code, &[]).run(&mut io), u64::from(want), "{w:?} sub={sub} {va:x} {vb:x}");
    }
}

#[test]
fn loads_stores_and_memory_operations_reach_base_plus_displacement() {
    let mut rng = Rng(0x9999_0000_7777_3333);
    let mut buf = vec![0u8; 4096];
    for _ in 0..ROUNDS * 4 {
        for b in buf.iter_mut() {
            *b = rng.next() as u8;
        }
        let before = buf.clone();
        let rs = distinct(&mut rng, 3);
        let (base, val, d) = (rs[0], rs[1], rs[2]);
        let bytes = [1u8, 2, 4, 8][(rng.next() % 4) as usize];
        let disp = (rng.next() % 2048) as i32 - 1024;
        let p = buf.as_mut_ptr() as u64 + 2048;
        let v = rng.value();
        let kind = rng.next() % 6;
        let signed = rng.next() % 2 == 0;
        let at_ = (2048 + disp) as usize;
        let read = |b: &[u8]| -> u64 {
            let mut x = [0u8; 8];
            x[..bytes as usize].copy_from_slice(&b[at_..at_ + bytes as usize]);
            u64::from_le_bytes(x)
        };
        let extend = |x: u64| -> u64 {
            let sh = 64 - 8 * u32::from(bytes);
            if signed && bytes < 8 { (((x << sh) as i64) >> sh) as u64 } else { x }
        };
        let k = rng.next() as i32;
        let op = [Op::Add, Op::Sub, Op::And, Op::Or, Op::Xor][(rng.next() % 5) as usize];
        let mb = if bytes == 2 { 4 } else { bytes };
        let code = frame(
            &[base, val],
            |a| match kind {
                0 => a.load(bytes, signed, d, at(base, disp)),
                1 => {
                    a.store(bytes, val, at(base, disp));
                    a.mov_imm(d, 0);
                }
                2 => {
                    a.store_imm(bytes, k, at(base, disp));
                    a.mov_imm(d, 0);
                }
                3 => {
                    a.alu_mem(op, mb, at(base, disp), Src::Reg(val));
                    a.mov_imm(d, 0);
                }
                4 => {
                    a.cmp_mem(mb, at(base, disp), Src::Reg(val));
                    a.set(Cond::Ltu, d);
                }
                _ => a.lea(Width::W64, d, at(base, disp)),
            },
            d,
        );
        let mut io = [p, v, 0, 0, 0, 0, 0, 0];
        let got = Exec::new(code, &[]).run(&mut io);
        let mask = if bytes == 8 { u64::MAX } else { (1u64 << (8 * bytes)) - 1 };
        match kind {
            0 => assert_eq!(got, extend(read(&before)), "load {bytes} signed={signed} disp={disp}"),
            1 => assert_eq!(read(&buf), v & mask, "store {bytes}"),
            2 => {
                let want = if bytes == 8 { k as i64 as u64 } else { k as u64 & mask };
                assert_eq!(read(&buf), want, "store_imm {bytes}");
            }
            3 => {
                let m2 = if mb == 8 { u64::MAX } else { (1u64 << (8 * mb)) - 1 };
                let saved = bytes;
                let _ = saved;
                let mut x = [0u8; 8];
                x[..mb as usize].copy_from_slice(&before[at_..at_ + mb as usize]);
                let old = u64::from_le_bytes(x);
                let mut y = [0u8; 8];
                y[..mb as usize].copy_from_slice(&buf[at_..at_ + mb as usize]);
                assert_eq!(u64::from_le_bytes(y), alu_model(op, Width::W64, old, v) & m2, "alu_mem {op:?} {mb}");
            }
            4 => {
                let m2 = if mb == 8 { u64::MAX } else { (1u64 << (8 * mb)) - 1 };
                let mut x = [0u8; 8];
                x[..mb as usize].copy_from_slice(&before[at_..at_ + mb as usize]);
                assert_eq!(got, u64::from(u64::from_le_bytes(x) < v & m2), "cmp_mem {mb}");
            }
            _ => assert_eq!(got, p.wrapping_add(disp as i64 as u64), "lea"),
        }
        if kind == 0 || kind >= 4 {
            assert_eq!(buf, before, "nothing written");
        }
    }
}

#[test]
fn immediates_of_every_width_load_exactly() {
    let mut rng = Rng(0xABAB_CDCD_EFEF_0101);
    for _ in 0..ROUNDS {
        let d = rng.reg();
        let v = rng.value() as i64;
        let code = frame(&[], |a| a.mov_imm(d, v), d);
        assert_eq!(Exec::new(code, &[]).run(&mut [0; 8]) as i64, v, "{d:?} {v:x}");
    }
}

#[test]
fn data_stubs_and_the_trampoline_link_and_preserve_every_register() {
    extern "C" fn helper(ctx: u64, a: u64, b: u64, c: u64) -> u64 {
        let answer = ctx ^ a.wrapping_mul(3) ^ b.rotate_left(7) ^ c.wrapping_add(11);
        // A System V callee may clobber every caller-saved register: this one does, so the trampoline must save them. SAFETY: writes only registers declared clobbered.
        unsafe {
            std::arch::asm!(
                "mov rcx, -1", "mov rdx, -1", "mov rsi, -1", "mov rdi, -1", "mov r8, -1", "mov r9, -1", "mov r10, -1", "mov r11, -1",
                out("rcx") _, out("rdx") _, out("rsi") _, out("rdi") _, out("r8") _, out("r9") _, out("r10") _, out("r11") _,
            );
        }
        answer
    }
    let mut stub = X64::new();
    let entry = stub.label();
    stub.bind(entry);
    x64::trampoline(&mut stub, R15, helper as *const () as u64);
    let stub = stub.finish(entry);
    let mut rng = Rng(0x7070_7070_1212_3434);
    for _ in 0..20 {
        let words: Vec<u64> = (0..14).map(|_| rng.value()).collect();
        // regs[13] is r15 (the context), [5] rdi, [4] rsi, [1] rdx.
        let want = helper(words[13], words[5], words[4], words[1]);
        assert_eq!(call_trampoline_check(&stub, &words), want, "the answer, every register preserved");
    }
    // The block's data read by address, and a jump through a word of it.
    let mut a = X64::new();
    let e = a.label();
    a.bind(e);
    let target = a.label();
    let k = a.data(&0x0123_4567_89AB_CDEFu64.to_le_bytes());
    let slot = a.data(&[0; 8]);
    a.lea_data(RAX, k);
    a.load(8, false, RAX, at(RAX, 0));
    a.jump_via_data(slot);
    a.section(Section::Cold);
    a.bind(target);
    a.alu(Op::Xor, Width::W64, RAX, RAX, Src::Imm(-1));
    a.ret();
    let exec = Exec::with(a.finish(e), &[], Some(slot));
    // SAFETY: the code is `fn() -> u64` and keeps to the System V convention.
    let f: extern "C" fn() -> u64 = unsafe { std::mem::transmute(exec.entry) };
    assert_eq!(f(), !0x0123_4567_89AB_CDEFu64);
}

/// The trampoline called with every register set from `words`; the helper's answer, after checking every register.
fn call_trampoline_check(stub: &Code, words: &[u64]) -> u64 {
    let regs = [RCX, RDX, RBX, RBP, RSI, RDI, R8, R9, R10, R11, R12, R13, R14, R15];
    let mut a = X64::new();
    let e = a.label();
    a.bind(e);
    for x in SAVED {
        a.push(x);
    }
    a.push(RDI);
    a.mov(Width::W64, RAX, RDI);
    for (n, x) in regs.iter().enumerate() {
        a.load(8, false, *x, at(RAX, 8 * n as i32));
    }
    a.alu(Op::Sub, Width::W64, RSP, RSP, Src::Imm(8));
    a.call_stub(0);
    a.alu(Op::Add, Width::W64, RSP, RSP, Src::Imm(8));
    // The answer kept; each register compared with what it held, a mismatch counted in r11's place.
    a.push(RAX);
    a.load(8, false, RAX, at(RSP, 8));
    let ok = a.label();
    let bad = a.label();
    for (n, x) in regs.iter().enumerate() {
        a.cmp_mem(8, at(RAX, 8 * n as i32), Src::Reg(*x));
        a.branch(Cond::Ne, bad);
    }
    a.pop(RAX);
    a.jump(ok);
    a.bind(bad);
    a.pop(RAX);
    a.mov_imm(RAX, -1);
    a.bind(ok);
    a.pop(RDI);
    for x in SAVED.iter().rev() {
        a.pop(*x);
    }
    a.ret();
    let exec = Exec::new(a.finish(e), &[stub.clone()]);
    let mut io = [0u64; 8];
    let mut all = words.to_vec();
    all.resize(16, 0);
    // SAFETY: as `run`, with a longer buffer.
    let f: extern "C" fn(*mut u64) -> u64 = unsafe { std::mem::transmute(exec.entry) };
    let _ = &mut io;
    f(all.as_mut_ptr())
}

#[test]
fn a_long_block_relaxes_its_jumps_and_keeps_its_purposes() {
    // A loop whose body grows past rel8's reach: the back jump must become rel32 and still count down exactly.
    for body in [1usize, 10, 40, 200] {
        let code = frame(
            &[RCX],
            |a| {
                let top = a.label();
                a.mov_imm(RDX, 0);
                a.bind(top);
                a.purpose(3);
                for _ in 0..body {
                    a.alu(Op::Add, Width::W64, RDX, RDX, Src::Imm(1));
                }
                a.purpose(0);
                a.alu(Op::Sub, Width::W64, RCX, RCX, Src::Imm(1));
                a.branch(Cond::Ne, top);
            },
            RDX,
        );
        let counted = code.hot_by_purpose[3] as usize;
        assert_eq!(counted, body * 4, "four bytes an add of an imm8");
        let mut io = [5, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(Exec::new(code, &[]).run(&mut io), 5 * body as u64);
    }
}

#[test]
fn a_label_s_address_reaches_it_in_either_section() {
    for cold in [false, true] {
        let code = frame(
            &[],
            |a| {
                let (to, back) = (a.label(), a.label());
                a.lea_label(R11, to);
                a.jump_reg(R11);
                if cold {
                    a.section(Section::Cold);
                }
                a.bind(to);
                a.mov_imm(RDX, 77);
                a.jump(back);
                a.section(Section::Hot);
                a.bind(back);
            },
            RDX,
        );
        assert_eq!(Exec::new(code, &[]).run(&mut [0; 8]), 77, "cold={cold}");
    }
}

#[test]
fn multiply_and_divide_match_their_models_in_every_register() {
    let mut rng = Rng(0x0DD_BA11_5EED_0042);
    let pick = |rng: &mut Rng, ok: &dyn Fn(Reg) -> bool| loop {
        let x = rng.reg();
        if ok(x) {
            break x;
        }
    };
    for _ in 0..ROUNDS * 4 {
        let rs = distinct(&mut rng, 2);
        let (va, mut vb) = (rng.value(), rng.value());
        let d = if rng.next() % 3 == 0 { rs[(rng.next() % 2) as usize] } else { rng.reg() };
        let mut io = [va, vb, 0, 0, 0, 0, 0, 0];
        let got = Exec::new(frame(&rs, |a| a.mul(d, rs[0], rs[1]), d), &[]).run(&mut io);
        assert_eq!(got, va.wrapping_mul(vb), "mul d={d:?} rs={rs:?} {va:x} {vb:x}");
        let w = if rng.next() % 2 == 0 { Width::W32 } else { Width::W64 };
        let signed = rng.next() % 2 == 0;
        if width_of(w, vb) == 0 {
            vb = 7;
        }
        let min = (w == Width::W32 && va as u32 == 0x8000_0000) || (w == Width::W64 && va == 1 << 63);
        if signed && min && width_of(w, vb) == width_of(w, u64::MAX) {
            vb = 3;
        }
        let b = pick(&mut rng, &|x| !x64::DIV_REGS.contains(&x));
        let a = pick(&mut rng, &|x| x != b);
        let q = rng.reg();
        let r = pick(&mut rng, &|x| x != q && !(q == RDX && x == RAX));
        let want = match (signed, w) {
            (true, Width::W32) => ((va as i32).wrapping_div(vb as i32) as u32 as u64, (va as i32).wrapping_rem(vb as i32) as u32 as u64),
            (true, Width::W64) => ((va as i64).wrapping_div(vb as i64) as u64, (va as i64).wrapping_rem(vb as i64) as u64),
            (false, Width::W32) => (u64::from(va as u32 / vb as u32), u64::from(va as u32 % vb as u32)),
            (false, Width::W64) => (va / vb, va % vb),
        };
        for (out, wanted) in [(q, want.0), (r, want.1)] {
            let mut io = [va, vb, 0, 0, 0, 0, 0, 0];
            let got = Exec::new(frame(&[a, b], |m| m.divrem(signed, w, q, r, a, b), out), &[]).run(&mut io);
            assert_eq!(width_of(w, got), wanted, "divrem signed={signed} {w:?} q={q:?} r={r:?} a={a:?} b={b:?} {va:x} {vb:x}");
        }
    }
}
