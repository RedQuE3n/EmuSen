//! Chapter 3 of the EE Core Instruction Set Manual, the multimedia instructions' Operation sections, transcribed as
//! plain functions on bit strings apart from the interpreter (`src/ee/mmi.rs`): the expected record of every S8 program
//! is computed here (Sapphire_Native.md §18.5). Each instruction is written as its page lists its assignments, field
//! by field, in the pseudocode's vocabulary (X(hi..lo), ||, sign and zero extension) with i128 arithmetic; where an
//! Operation section cannot be read literally the Description decides, as §18.2 records.

use crate::s6::manual::bits;

/// The registers an instruction may change: GPR[rd], HI and LO, each 128 bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub rd: u128,
    pub hi: u128,
    pub lo: u128,
}

/// The state an instruction reads: GPR[rs], GPR[rt], GPR[rd] before it, HI, LO, the sa field and the SA register.
#[derive(Clone, Copy, Debug)]
pub struct Input {
    pub rs: u128,
    pub rt: u128,
    pub rd: u128,
    pub hi: u128,
    pub lo: u128,
    pub sa: u32,
    pub sa_register: u32,
}

/// X(hi..lo) as an integer, unsigned or signed.
fn field(x: u128, hi: u32, lo: u32) -> i128 {
    bits(x, hi, lo) as i128
}
fn sfield(x: u128, hi: u32, lo: u32) -> i128 {
    let n = hi - lo + 1;
    let v = bits(x, hi, lo) as i128;
    if v >> (n - 1) & 1 == 1 { v - (1i128 << n) } else { v }
}
/// Writes the n-bit string `v` into bits hi..lo of `x`.
fn put(x: &mut u128, hi: u32, lo: u32, v: i128) {
    let n = hi - lo + 1;
    let m = if n == 128 { u128::MAX } else { ((1u128 << n) - 1) << lo };
    *x = *x & !m | ((v as u128) << lo) & m;
}
/// (b)^n || b... : sign extension of the n-bit string `v` to `to` bits, as a raw value.
fn extend(v: i128, n: u32, to: u32) -> i128 {
    let m = (1i128 << n) - 1;
    let v = v & m;
    let s = if v >> (n - 1) & 1 == 1 { v - (1i128 << n) } else { v };
    s & ((1i128 << to) - 1)
}

/// Lane k of width w: bits (k+1)w-1 .. kw.
fn hi_of(k: u32, w: u32) -> u32 {
    (k + 1) * w - 1
}

/// The saturating pages: the Description's ranges (the Operation sections write unsigned sums against them).
fn saturate_signed(v: i128, w: u32) -> i128 {
    v.clamp(-(1i128 << (w - 1)), (1i128 << (w - 1)) - 1)
}
fn saturate_unsigned(v: i128, w: u32) -> i128 {
    v.clamp(0, (1i128 << w) - 1)
}

/// The instruction `name` on `i`. The undefined inputs a page names are the caller's to avoid or to grade as such.
pub fn mmi(name: &str, i: &Input) -> Outcome {
    let (rs, rt) = (i.rs, i.rt);
    let mut o = Outcome { rd: i.rd, hi: i.hi, lo: i.lo };
    let per_lane = |w: u32, f: &dyn Fn(i128, i128, i128, i128) -> i128| -> u128 {
        let mut r = 0u128;
        for k in 0..128 / w {
            let (h, l) = (hi_of(k, w), k * w);
            put(&mut r, h, l, f(field(rs, h, l), field(rt, h, l), sfield(rs, h, l), sfield(rt, h, l)));
        }
        r
    };
    match name {
        "paddb" => o.rd = per_lane(8, &|a, b, _, _| a + b),
        "paddh" => o.rd = per_lane(16, &|a, b, _, _| a + b),
        "paddw" => o.rd = per_lane(32, &|a, b, _, _| a + b),
        "psubb" => o.rd = per_lane(8, &|a, b, _, _| a - b),
        "psubh" => o.rd = per_lane(16, &|a, b, _, _| a - b),
        "psubw" => o.rd = per_lane(32, &|a, b, _, _| a - b),
        "paddsb" => o.rd = per_lane(8, &|_, _, a, b| saturate_signed(a + b, 8)),
        "paddsh" => o.rd = per_lane(16, &|_, _, a, b| saturate_signed(a + b, 16)),
        "paddsw" => o.rd = per_lane(32, &|_, _, a, b| saturate_signed(a + b, 32)),
        "psubsb" => o.rd = per_lane(8, &|_, _, a, b| saturate_signed(a - b, 8)),
        "psubsh" => o.rd = per_lane(16, &|_, _, a, b| saturate_signed(a - b, 16)),
        "psubsw" => o.rd = per_lane(32, &|_, _, a, b| saturate_signed(a - b, 32)),
        "paddub" => o.rd = per_lane(8, &|a, b, _, _| saturate_unsigned(a + b, 8)),
        "padduh" => o.rd = per_lane(16, &|a, b, _, _| saturate_unsigned(a + b, 16)),
        "padduw" => o.rd = per_lane(32, &|a, b, _, _| saturate_unsigned(a + b, 32)),
        "psubub" => o.rd = per_lane(8, &|a, b, _, _| saturate_unsigned(a - b, 8)),
        "psubuh" => o.rd = per_lane(16, &|a, b, _, _| saturate_unsigned(a - b, 16)),
        "psubuw" => o.rd = per_lane(32, &|a, b, _, _| saturate_unsigned(a - b, 32)),
        "padsbh" => {
            let mut r = 0u128;
            for k in 0..8 {
                let (h, l) = (hi_of(k, 16), 16 * k);
                let v = if k < 4 { field(rs, h, l) - field(rt, h, l) } else { field(rs, h, l) + field(rt, h, l) };
                put(&mut r, h, l, v);
            }
            o.rd = r;
        }
        "pceqb" => o.rd = per_lane(8, &|a, b, _, _| if a == b { -1 } else { 0 }),
        "pceqh" => o.rd = per_lane(16, &|a, b, _, _| if a == b { -1 } else { 0 }),
        "pceqw" => o.rd = per_lane(32, &|a, b, _, _| if a == b { -1 } else { 0 }),
        "pcgtb" => o.rd = per_lane(8, &|_, _, a, b| if a > b { -1 } else { 0 }),
        "pcgth" => o.rd = per_lane(16, &|_, _, a, b| if a > b { -1 } else { 0 }),
        "pcgtw" => o.rd = per_lane(32, &|_, _, a, b| if a > b { -1 } else { 0 }),
        "pmaxh" => o.rd = per_lane(16, &|_, _, a, b| if a > b { a } else { b }),
        "pmaxw" => o.rd = per_lane(32, &|_, _, a, b| if a > b { a } else { b }),
        "pminh" => o.rd = per_lane(16, &|_, _, a, b| if a > b { b } else { a }),
        "pminw" => o.rd = per_lane(32, &|_, _, a, b| if a > b { b } else { a }),
        "pabsh" => o.rd = per_lane(16, &|_, _, _, b| if b == -0x8000 { 0x7FFF } else { b.abs() }),
        "pabsw" => o.rd = per_lane(32, &|_, _, _, b| if b == -0x8000_0000 { 0x7FFF_FFFF } else { b.abs() }),
        "pand" => o.rd = rs & rt,
        "por" => o.rd = rs | rt,
        "pxor" => o.rd = rs ^ rt,
        "pnor" => o.rd = !(rs | rt),
        "psllh" | "psrlh" | "psrah" => {
            let s = bits(i.sa as u128, 3, 0) as u32;
            let mut r = 0u128;
            for k in 0..8 {
                let l = 16 * k;
                let v = match name {
                    "psllh" => if s == 0 { field(rt, l + 15, l) } else { field(rt, l + 15 - s, l) << s },
                    "psrlh" => field(rt, l + 15, l + s),
                    _ => extend(field(rt, l + 15, l + s), 16 - s, 16),
                };
                put(&mut r, l + 15, l, v);
            }
            o.rd = r;
        }
        "psllw" | "psrlw" | "psraw" => {
            let s = i.sa;
            let mut r = 0u128;
            for k in 0..4 {
                let l = 32 * k;
                let v = match name {
                    "psllw" => if s == 0 { field(rt, l + 31, l) } else { field(rt, l + 31 - s, l) << s },
                    "psrlw" => field(rt, l + 31, l + s),
                    _ => extend(field(rt, l + 31, l + s), 32 - s, 32),
                };
                put(&mut r, l + 31, l, v);
            }
            o.rd = r;
        }
        "psllvw" | "psrlvw" | "psravw" => {
            let (s, t) = (field(rs, 4, 0) as u32, field(rs, 68, 64) as u32);
            let mut r = 0u128;
            for (base, amount) in [(0u32, s), (64, t)] {
                let temp = match name {
                    "psllvw" => if amount == 0 { field(rt, base + 31, base) } else { field(rt, base + 31 - amount, base) << amount },
                    "psrlvw" => field(rt, base + 31, base + amount),
                    _ => extend(field(rt, base + 31, base + amount), 32 - amount, 32),
                };
                put(&mut r, base + 63, base, extend(temp, 32, 64));
            }
            o.rd = r;
        }
        "pextlb" | "pextub" => {
            let from = if name == "pextlb" { 0 } else { 64 };
            let mut r = 0u128;
            for k in 0..8 {
                let src = from + 8 * k;
                put(&mut r, 16 * k + 15, 16 * k, field(rs, src + 7, src) << 8 | field(rt, src + 7, src));
            }
            o.rd = r;
        }
        "pextlh" | "pextuh" => {
            let from = if name == "pextlh" { 0 } else { 64 };
            let mut r = 0u128;
            for k in 0..4 {
                let src = from + 16 * k;
                put(&mut r, 32 * k + 31, 32 * k, field(rs, src + 15, src) << 16 | field(rt, src + 15, src));
            }
            o.rd = r;
        }
        "pextlw" => o.rd = bits(rs, 63, 32) << 96 | bits(rt, 63, 32) << 64 | bits(rs, 31, 0) << 32 | bits(rt, 31, 0),
        "pextuw" => o.rd = bits(rs, 127, 96) << 96 | bits(rt, 127, 96) << 64 | bits(rs, 95, 64) << 32 | bits(rt, 95, 64),
        "ppacb" => {
            let mut r = 0u128;
            for k in 0..8 {
                put(&mut r, 8 * k + 7, 8 * k, field(rt, 16 * k + 7, 16 * k));
                put(&mut r, 64 + 8 * k + 7, 64 + 8 * k, field(rs, 16 * k + 7, 16 * k));
            }
            o.rd = r;
        }
        "ppach" => {
            let mut r = 0u128;
            for k in 0..4 {
                put(&mut r, 16 * k + 15, 16 * k, field(rt, 32 * k + 15, 32 * k));
                put(&mut r, 64 + 16 * k + 15, 64 + 16 * k, field(rs, 32 * k + 15, 32 * k));
            }
            o.rd = r;
        }
        "ppacw" => o.rd = bits(rs, 95, 64) << 96 | bits(rs, 31, 0) << 64 | bits(rt, 95, 64) << 32 | bits(rt, 31, 0),
        "pinth" => {
            o.rd = bits(rs, 127, 112) << 112
                | bits(rt, 63, 48) << 96
                | bits(rs, 111, 96) << 80
                | bits(rt, 47, 32) << 64
                | bits(rs, 95, 80) << 48
                | bits(rt, 31, 16) << 32
                | bits(rs, 79, 64) << 16
                | bits(rt, 15, 0)
        }
        "pinteh" => {
            o.rd = bits(rs, 111, 96) << 112
                | bits(rt, 111, 96) << 96
                | bits(rs, 79, 64) << 80
                | bits(rt, 79, 64) << 64
                | bits(rs, 47, 32) << 48
                | bits(rt, 47, 32) << 32
                | bits(rs, 15, 0) << 16
                | bits(rt, 15, 0)
        }
        "pcpyld" => o.rd = bits(rs, 63, 0) << 64 | bits(rt, 63, 0),
        "pcpyud" => o.rd = bits(rt, 127, 64) << 64 | bits(rs, 127, 64),
        "pcpyh" => {
            let (a0, a1) = (bits(rt, 15, 0), bits(rt, 79, 64));
            o.rd = a1 << 112 | a1 << 96 | a1 << 80 | a1 << 64 | a0 << 48 | a0 << 32 | a0 << 16 | a0;
        }
        "pexch" | "pexeh" | "prevh" => {
            // GPR[rd]'s halfwords 0..7, each as the page names its source halfword of GPR[rt].
            let src: [u32; 8] = match name {
                "pexch" => [0, 2, 1, 3, 4, 6, 5, 7],
                "pexeh" => [2, 1, 0, 3, 6, 5, 4, 7],
                _ => [3, 2, 1, 0, 7, 6, 5, 4],
            };
            let mut r = 0u128;
            for (k, &s) in src.iter().enumerate() {
                put(&mut r, 16 * k as u32 + 15, 16 * k as u32, field(rt, 16 * s + 15, 16 * s));
            }
            o.rd = r;
        }
        "pexcw" => o.rd = bits(rt, 127, 96) << 96 | bits(rt, 63, 32) << 64 | bits(rt, 95, 64) << 32 | bits(rt, 31, 0),
        "pexew" => o.rd = bits(rt, 127, 96) << 96 | bits(rt, 31, 0) << 64 | bits(rt, 63, 32) << 32 | bits(rt, 95, 64),
        "prot3w" => o.rd = bits(rt, 127, 96) << 96 | bits(rt, 31, 0) << 64 | bits(rt, 95, 64) << 32 | bits(rt, 63, 32),
        "pext5" => {
            let mut r = 0u128;
            for k in 0..4 {
                let b = 32 * k;
                let v = bits(rt, b + 15, b + 15) << 31 | bits(rt, b + 14, b + 10) << 19 | bits(rt, b + 9, b + 5) << 11 | bits(rt, b + 4, b) << 3;
                put(&mut r, b + 31, b, v as i128);
            }
            o.rd = r;
        }
        "ppac5" => {
            let mut r = 0u128;
            for k in 0..4 {
                let b = 32 * k;
                let v = bits(rt, b + 31, b + 31) << 15 | bits(rt, b + 23, b + 19) << 10 | bits(rt, b + 15, b + 11) << 5 | bits(rt, b + 7, b + 3);
                put(&mut r, b + 31, b, v as i128);
            }
            o.rd = r;
        }
        "pmfhi" => o.rd = i.hi,
        "pmflo" => o.rd = i.lo,
        "pmthi" => o.hi = rs,
        "pmtlo" => o.lo = rs,
        "pmfhl.lw" => o.rd = bits(i.hi, 95, 64) << 96 | bits(i.lo, 95, 64) << 64 | bits(i.hi, 31, 0) << 32 | bits(i.lo, 31, 0),
        "pmfhl.uw" => o.rd = bits(i.hi, 127, 96) << 96 | bits(i.lo, 127, 96) << 64 | bits(i.hi, 63, 32) << 32 | bits(i.lo, 63, 32),
        "pmfhl.lh" => {
            o.rd = bits(i.hi, 111, 96) << 112
                | bits(i.hi, 79, 64) << 96
                | bits(i.lo, 111, 96) << 80
                | bits(i.lo, 79, 64) << 64
                | bits(i.hi, 47, 32) << 48
                | bits(i.hi, 15, 0) << 32
                | bits(i.lo, 47, 32) << 16
                | bits(i.lo, 15, 0)
        }
        "pmfhl.sh" => {
            // clamp(X): X > 0x00007FFF gives 0x7FFF, X < 0xFFFF8000 gives 0x8000, both read as signed words.
            let clamp = |x: u128, h: u32| -> i128 { sfield(x, h, h - 31).clamp(-0x8000, 0x7FFF) };
            let order = [(i.lo, 31), (i.lo, 63), (i.hi, 31), (i.hi, 63), (i.lo, 95), (i.lo, 127), (i.hi, 95), (i.hi, 127)];
            let mut r = 0u128;
            for (k, &(x, h)) in order.iter().enumerate() {
                put(&mut r, 16 * k as u32 + 15, 16 * k as u32, clamp(x, h));
            }
            o.rd = r;
        }
        "pmfhl.slw" => {
            let mut r = 0u128;
            for b in [0u32, 64] {
                // (HI || LO) of the doubleword's low words, compared as a signed 64-bit value.
                let joined = sfield(i.hi, b + 31, b) << 32 | field(i.lo, b + 31, b);
                let v = if joined >= 0x7FFF_FFFF {
                    0x7FFF_FFFF
                } else if joined <= -0x8000_0000 {
                    extend(0x8000_0000, 32, 64)
                } else {
                    extend(field(i.lo, b + 31, b), 32, 64)
                };
                put(&mut r, b + 63, b, v);
            }
            o.rd = r;
        }
        "pmthl.lw" => {
            put(&mut o.lo, 31, 0, field(rs, 31, 0));
            put(&mut o.hi, 31, 0, field(rs, 63, 32));
            put(&mut o.lo, 95, 64, field(rs, 95, 64));
            put(&mut o.hi, 95, 64, field(rs, 127, 96));
        }
        "plzcw" => {
            let lzc = |w: i128| -> i128 {
                let top = w >> 31 & 1;
                (0..32).rev().take_while(|&b| w >> b & 1 == top).count() as i128
            };
            put(&mut o.rd, 31, 0, lzc(field(rs, 31, 0)) - 1);
            put(&mut o.rd, 63, 32, lzc(field(rs, 63, 32)) - 1);
        }
        "qfsrv" => {
            // SA holds the byte count (Sapphire_Disputes.md D-9); the page's SA is the bit count 8 × that.
            let s = 8 * i.sa_register;
            o.rd = if s == 0 { rt } else { bits(rs, s - 1, 0) << (128 - s) | bits(rt, 127, s) };
        }
        "pmulth" | "pmaddh" | "pmsubh" => {
            // prod0 → LO31..0 and GPR[rd]31..0; prod1 → LO63..32; prod2 → HI31..0 and GPR[rd]63..32; prod3 →
            // HI63..32; prod4 → LO95..64 and GPR[rd]95..64; prod5 → LO127..96; prod6 → HI95..64 and GPR[rd]127..96;
            // prod7 → HI127..96.
            let dest: [(bool, u32, Option<u32>); 8] =
                [(false, 0, Some(0)), (false, 32, None), (true, 0, Some(32)), (true, 32, None), (false, 64, Some(64)), (false, 96, None), (true, 64, Some(96)), (true, 96, None)];
            for (k, &(to_hi, at, rd_at)) in dest.iter().enumerate() {
                let l = 16 * k as u32;
                let product = sfield(rs, l + 15, l) * sfield(rt, l + 15, l);
                let reg = if to_hi { &mut o.hi } else { &mut o.lo };
                let acc = field(*reg, at + 31, at);
                let v = match name {
                    "pmulth" => product,
                    "pmaddh" => acc + product,
                    _ => acc - product,
                };
                put(reg, at + 31, at, v);
                if let Some(d) = rd_at {
                    put(&mut o.rd, d + 31, d, v);
                }
            }
        }
        "phmadh" | "phmsbh" => {
            // prod0 → LO31..0, prod1 → HI31..0, prod2 → LO95..64, prod3 → HI95..64, and GPR[rd]'s words in order.
            let dest = [(false, 0), (true, 0), (false, 64), (true, 64)];
            for (j, &(to_hi, at)) in dest.iter().enumerate() {
                let l = 32 * j as u32;
                let high = sfield(rs, l + 31, l + 16) * sfield(rt, l + 31, l + 16);
                let low = sfield(rs, l + 15, l) * sfield(rt, l + 15, l);
                let v = if name == "phmadh" { high + low } else { high - low };
                put(if to_hi { &mut o.hi } else { &mut o.lo }, at + 31, at, v);
                put(&mut o.rd, l + 31, l, v);
            }
        }
        "pmultw" | "pmultuw" | "pmaddw" | "pmadduw" | "pmsubw" => {
            for b in [0u32, 64] {
                let unsigned = matches!(name, "pmultuw" | "pmadduw");
                let (a, c) = if unsigned { (field(rs, b + 31, b), field(rt, b + 31, b)) } else { (sfield(rs, b + 31, b), sfield(rt, b + 31, b)) };
                let acc = field(i.hi, b + 31, b) << 32 | field(i.lo, b + 31, b);
                let prod = match name {
                    "pmultw" | "pmultuw" => a * c,
                    "pmaddw" | "pmadduw" => acc + a * c,
                    _ => acc - a * c,
                };
                put(&mut o.lo, b + 63, b, extend(prod, 32, 64));
                put(&mut o.hi, b + 63, b, extend(prod >> 32, 32, 64));
                put(&mut o.rd, b + 63, b, prod);
            }
        }
        "pdivw" | "pdivuw" => {
            for b in [0u32, 64] {
                let (n, d) = if name == "pdivw" { (sfield(rs, b + 31, b), sfield(rt, b + 31, b)) } else { (field(rs, b + 31, b), field(rt, b + 31, b)) };
                assert!(d != 0, "a zero divisor is undefined");
                let q = n / d;
                let r = n - q * d;
                put(&mut o.lo, b + 63, b, extend(q, 32, 64));
                put(&mut o.hi, b + 63, b, extend(r, 32, 64));
            }
        }
        "pdivbw" => {
            let d = sfield(rt, 15, 0);
            assert!(d != 0, "a zero divisor is undefined");
            for k in 0..4 {
                let l = 32 * k;
                let n = sfield(rs, l + 31, l);
                let q = n / d;
                let r = n - q * d;
                put(&mut o.lo, l + 31, l, q);
                // HI ← (r15)^16 || r15..0: the Operation section's sign extension (Sapphire_Disputes.md D-20).
                put(&mut o.hi, l + 31, l, extend(r, 16, 32));
            }
        }
        _ => panic!("not a multimedia instruction: {name}"),
    }
    o
}
