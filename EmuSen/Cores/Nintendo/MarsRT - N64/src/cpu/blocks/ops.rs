//! Each instruction's handler, chosen once when its block is shaped: the interpreter's own method, behind a C ABI that compiled code calls too.

use crate::cpu::interp::{NOT_STORED, rd, rs, rt, sa, signed_immediate, immediate};
use crate::cpu::{Cpu, Exec};
use crate::memory::bus::MemoryBus;

/// A handler: what `Cpu::execute` does for one word, with a store's landing reported.
pub type Handler = unsafe extern "C" fn(*mut Cpu, *mut MemoryBus, u32) -> u64;

/// A handler's answer: the physical address a store landed at, `THROUGH_BUS`, `NOT_STORED`, or `RAISED`.
pub const RAISED: u64 = 1 << 32;
pub const PLAIN: u64 = NOT_STORED as u64;

#[inline(always)]
fn done(r: Exec<u32>) -> u64 {
    match r {
        Ok(v) => v as u64,
        Err(_) => RAISED,
    }
}

macro_rules! handler {
    ($name:ident, |$c:ident, $b:ident, $i:ident| $body:expr) => {
        unsafe extern "C" fn $name(c: *mut Cpu, b: *mut MemoryBus, $i: u32) -> u64 {
            // SAFETY: the caller lends the machine's two halves for the call and touches neither meanwhile.
            let ($c, $b) = unsafe { (&mut *c, &mut *b) };
            let _ = &$b;
            done($body)
        }
    };
}

macro_rules! plain {
    ($name:ident, |$c:ident, $i:ident| $body:expr) => {
        handler!($name, |$c, _b, $i| {
            $body;
            Ok(NOT_STORED)
        });
    };
}

handler!(any, |c, b, i| c.execute::<false>(b, i).map(|_| NOT_STORED));

plain!(sll, |c, i| c.write32(rd(i), (c.read(rt(i)) as u32) << sa(i)));
plain!(srl, |c, i| c.write32(rd(i), (c.read(rt(i)) as u32) >> sa(i)));
plain!(sra, |c, i| c.shift_right_arithmetic(i, sa(i)));
plain!(sllv, |c, i| c.write32(rd(i), (c.read(rt(i)) as u32) << (c.read(rs(i)) & 0x1F)));
plain!(srlv, |c, i| c.write32(rd(i), (c.read(rt(i)) as u32) >> (c.read(rs(i)) & 0x1F)));
plain!(srav, |c, i| c.shift_right_arithmetic(i, (c.read(rs(i)) & 0x1F) as u32));
plain!(jr, |c, i| c.jump_register::<false>(i, false));
plain!(jalr, |c, i| c.jump_register::<false>(i, true));
plain!(mfhi, |c, i| c.write(rd(i), c.hi));
plain!(mflo, |c, i| c.write(rd(i), c.lo));
plain!(addu, |c, i| c.write32(rd(i), (c.read(rs(i)) as u32).wrapping_add(c.read(rt(i)) as u32)));
plain!(subu, |c, i| c.write32(rd(i), (c.read(rs(i)) as u32).wrapping_sub(c.read(rt(i)) as u32)));
plain!(and, |c, i| c.write(rd(i), c.read(rs(i)) & c.read(rt(i))));
plain!(or, |c, i| c.write(rd(i), c.read(rs(i)) | c.read(rt(i))));
plain!(xor, |c, i| c.write(rd(i), c.read(rs(i)) ^ c.read(rt(i))));
plain!(nor, |c, i| c.write(rd(i), !(c.read(rs(i)) | c.read(rt(i)))));
plain!(slt, |c, i| c.write(rd(i), ((c.read(rs(i)) as i64) < (c.read(rt(i)) as i64)) as u64));
plain!(sltu, |c, i| c.write(rd(i), (c.read(rs(i)) < c.read(rt(i))) as u64));
plain!(daddu, |c, i| c.write(rd(i), c.read(rs(i)).wrapping_add(c.read(rt(i)))));
plain!(dsubu, |c, i| c.write(rd(i), c.read(rs(i)).wrapping_sub(c.read(rt(i)))));
plain!(dsll32, |c, i| c.write(rd(i), c.read(rt(i)) << (sa(i) + 32)));
plain!(dsra32, |c, i| c.write(rd(i), ((c.read(rt(i)) as i64) >> (sa(i) + 32)) as u64));

plain!(j, |c, i| c.branch(c.jump_target(i)));
plain!(jal, |c, i| {
    c.write(31, c.next_pc);
    c.branch(c.jump_target(i));
});
plain!(beq, |c, i| c.branch_if::<false>(c.read(rs(i)) == c.read(rt(i)), i, false, false));
plain!(bne, |c, i| c.branch_if::<false>(c.read(rs(i)) != c.read(rt(i)), i, false, false));
plain!(blez, |c, i| c.branch_if::<false>(c.read(rs(i)) as i64 <= 0, i, false, false));
plain!(bgtz, |c, i| c.branch_if::<false>(c.read(rs(i)) as i64 > 0, i, false, false));
plain!(beql, |c, i| c.branch_if::<false>(c.read(rs(i)) == c.read(rt(i)), i, true, false));
plain!(bnel, |c, i| c.branch_if::<false>(c.read(rs(i)) != c.read(rt(i)), i, true, false));
plain!(addiu, |c, i| c.write32(rt(i), (c.read(rs(i)) as u32).wrapping_add(signed_immediate(i) as u32)));
plain!(slti, |c, i| c.write(rt(i), ((c.read(rs(i)) as i64) < signed_immediate(i)) as u64));
plain!(sltiu, |c, i| c.write(rt(i), (c.read(rs(i)) < signed_immediate(i) as u64) as u64));
plain!(andi, |c, i| c.write(rt(i), c.read(rs(i)) & immediate(i)));
plain!(ori, |c, i| c.write(rt(i), c.read(rs(i)) | immediate(i)));
plain!(xori, |c, i| c.write(rt(i), c.read(rs(i)) ^ immediate(i)));
plain!(lui, |c, i| c.write32(rt(i), (immediate(i) << 16) as u32));
plain!(daddiu, |c, i| c.write(rt(i), (c.read(rs(i)) as i64).wrapping_add(signed_immediate(i)) as u64));

handler!(lb, |c, b, i| c.load(b, i, 1, true).map(|_| NOT_STORED));
handler!(lh, |c, b, i| c.load(b, i, 2, true).map(|_| NOT_STORED));
handler!(lw, |c, b, i| c.load(b, i, 4, true).map(|_| NOT_STORED));
handler!(lbu, |c, b, i| c.load(b, i, 1, false).map(|_| NOT_STORED));
handler!(lhu, |c, b, i| c.load(b, i, 2, false).map(|_| NOT_STORED));
handler!(lwu, |c, b, i| c.load(b, i, 4, false).map(|_| NOT_STORED));
handler!(ld, |c, b, i| c.load(b, i, 8, false).map(|_| NOT_STORED));
handler!(lwc1, |c, b, i| c.load_cop1(b, i, false).map(|_| NOT_STORED));
handler!(ldc1, |c, b, i| c.load_cop1(b, i, true).map(|_| NOT_STORED));
handler!(cop1, |c, _b, i| c.execute_cop1(i).map(|_| NOT_STORED));

handler!(sb, |c, b, i| c.store::<false>(b, i, 1));
handler!(sh, |c, b, i| c.store::<false>(b, i, 2));
handler!(sw, |c, b, i| c.store::<false>(b, i, 4));
handler!(sd, |c, b, i| c.store::<false>(b, i, 8));
handler!(swl, |c, b, i| c.store_word_left(b, i));
handler!(swr, |c, b, i| c.store_word_right(b, i));
handler!(sdl, |c, b, i| c.store_double_left(b, i));
handler!(sdr, |c, b, i| c.store_double_right(b, i));
handler!(sc, |c, b, i| c.store_conditional::<false>(b, i, 4));
handler!(scd, |c, b, i| c.store_conditional::<false>(b, i, 8));
handler!(swc1, |c, b, i| c.store_cop1(b, i, false));
handler!(sdc1, |c, b, i| c.store_cop1(b, i, true));

/// The handler for a word, as `Cpu::execute` would decode it; kernel mode is assumed, as a block only runs there.
pub fn handler(word: u32) -> Handler {
    match word >> 26 {
        0x00 => match word & 0x3F {
            0x00 => sll,
            0x02 => srl,
            0x03 => sra,
            0x04 => sllv,
            0x06 => srlv,
            0x07 => srav,
            0x08 => jr,
            0x09 => jalr,
            0x10 => mfhi,
            0x12 => mflo,
            0x21 => addu,
            0x23 => subu,
            0x24 => and,
            0x25 => or,
            0x26 => xor,
            0x27 => nor,
            0x2A => slt,
            0x2B => sltu,
            0x2D => daddu,
            0x2F => dsubu,
            0x3C => dsll32,
            0x3F => dsra32,
            _ => any,
        },
        0x02 => j,
        0x03 => jal,
        0x04 => beq,
        0x05 => bne,
        0x06 => blez,
        0x07 => bgtz,
        0x09 => addiu,
        0x0A => slti,
        0x0B => sltiu,
        0x0C => andi,
        0x0D => ori,
        0x0E => xori,
        0x0F => lui,
        0x11 => cop1,
        0x14 => beql,
        0x15 => bnel,
        0x19 => daddiu,
        0x20 => lb,
        0x21 => lh,
        0x23 => lw,
        0x24 => lbu,
        0x25 => lhu,
        0x27 => lwu,
        0x28 => sb,
        0x29 => sh,
        0x2A => swl,
        0x2B => sw,
        0x2C => sdl,
        0x2D => sdr,
        0x2E => swr,
        0x31 => lwc1,
        0x35 => ldc1,
        0x37 => ld,
        0x38 => sc,
        0x39 => swc1,
        0x3C => scd,
        0x3D => sdc1,
        0x3F => sd,
        _ => any,
    }
}
