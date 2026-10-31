//! COP2 from the EE: VU0 in macro mode (Sapphire_Plan.md §7, S12). The transfers QMFC2, QMTC2, CFC2, CTC2, LQC2 and SQC2
//! with their interlock bit, the branches on VU1's activity, and every VU operation the EE issues, which `crate::vu::ops`
//! computes; VCALLMS and VCALLMSR start micro mode through `crate::vu::micro::Micro`. Each written from the VU User's
//! Manual's chapters 5 and 6; the interlocks are §5.4.5's and §5.4.6's, charged to the timing model as the cycle the
//! instruction may issue in. Sapphire_Native.md §21 records every reading.

use crate::vu::ops::{self as vops, Class, Ctx};
use crate::vu::{Landing, Vpu, ccr, pipe};
use emusen_mips::vu::Form;

use super::ops::{Kind, Op, R, Stop, aligned, cond, likely, load_address, rd, rs, rt, sext32, store_address_of};
use super::timing::{Category, Issue, VU0R, rows};
use super::{Bus, Ee, status};

/// Coprocessor 2 is usable when Status.CU2 is set (EE Core User's Manual §3.2; Sapphire_Disputes.md D-8).
fn usable(e: &Ee) -> R {
    if e.status() & status::CU2 != 0 { Ok(()) } else { Err(Stop::Exception(super::Exception::Unusable(2))) }
}

/// The bus's vector units; a bus without them (the tests' RDRAM alone) halts the interpreter at the instruction.
fn vpu(b: &mut dyn Bus) -> Result<&mut Vpu, Stop> {
    b.vpu().ok_or(Stop::Halt)
}

/// `vpu` for an instruction that reaches VU0 alone.
fn vpu0(b: &mut dyn Bus) -> Result<&mut Vpu, Stop> {
    b.vpu0().ok_or(Stop::Halt)
}

/// The control registers whose access reaches VU1: FBRST, VPU-STAT and CMSAR1 (§5.1.3).
fn reaches_vu1(n: usize) -> bool {
    matches!(n, ccr::FBRST | ccr::VPU_STAT | ccr::CMSAR1)
}

/// The interlock bit of a transfer, bit 0 (VU User's Manual §6.1.1, type 11).
fn interlocked(w: u32) -> bool {
    w & 1 == 1
}

/// How far behind the T stage of a microprogram's last pair an instruction that waits for its end issues (§5.4.5,
/// §5.4.6): a macro instruction's T stage meets that pair's S stage, so it issues 3 cycles on; QMFC2.I and CFC2.I read
/// as they issue, 4 on; another VCALLMS's first pair follows the last by `pipe::START` (the second VCALLMS's D stage
/// stalls), so it issues at once.
const MACRO: u64 = 3;
const TRANSFER: u64 = pipe::FMAC;
const CALL: u64 = 0;

/// Waits for VU0's microprogram to end, `behind` cycles after its last pair's T stage.
fn wait_end(e: &mut Ee, vpu: &mut Vpu, behind: u64) {
    if let Some(last) = vpu.finish0(e.planned()) {
        e.wait(last + behind);
    }
}

/// QMFC2: GPR[rt] ← VF[fd], all 128 bits; with the interlock, once VU0's microprogram has ended (§6.2).
fn qmfc2(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    let vpu = vpu0(b)?;
    if interlocked(w) {
        wait_end(e, vpu, TRANSFER);
    }
    e.set_quad(rt(w), vpu.units.vu0.vf_quad(rd(w)));
    Ok(())
}
/// QMTC2: VF[fd] ← GPR[rt]; with the interlock, once the microprogram ends or reaches a pair with M set (§5.4.6).
fn qmtc2(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    let vpu = vpu0(b)?;
    if interlocked(w) {
        let end = vpu.release0(e.planned());
        e.wait(end);
    }
    vpu.units.vu0.set_vf_quad(rd(w), e.gpr[rt(w)]);
    Ok(())
}
/// CFC2: GPR[rt] ← CCR[2,id] sign-extended into bits 63..0, bits 127..64 kept (the `transfer` console text's
/// `cfc2: 12345678 12345678 ffffffff ffffffff`, Sapphire_Disputes.md D-45).
fn cfc2(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    let vpu = if reaches_vu1(rd(w)) { vpu(b)? } else { vpu0(b)? };
    if interlocked(w) {
        wait_end(e, vpu, TRANSFER);
    }
    let v = vpu.ccr(rd(w), e.planned());
    e.set(rt(w), sext32(v));
    Ok(())
}
/// CTC2: CCR[2,id] ← GPR[rt]'s bits 31..0, each register as `Vpu::set_ccr` keeps it.
fn ctc2(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    let vpu = if reaches_vu1(rd(w)) { vpu(b)? } else { vpu0(b)? };
    if interlocked(w) {
        let end = vpu.release0(e.planned());
        e.wait(end);
    }
    let t = e.planned();
    if vpu.set_ccr(rd(w), e.get(rt(w)) as u32, t) { Ok(()) } else { Err(Stop::Halt) }
}
/// LQC2: VF[ft] ← the quadword at GPR[base] + offset, an address error unless it is aligned (§6.2's Remarks).
fn lqc2(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    vpu0(b)?;
    let a = load_address(e, w)?;
    aligned(a, 16, false)?;
    let v = e.read(b, a, 16)?;
    vpu0(b)?.units.vu0.set_vf_quad(rt(w), v);
    Ok(())
}
/// SQC2: the quadword at GPR[base] + offset ← VF[ft], its data breakpoint judged on x.
fn sqc2(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    let v = vpu0(b)?.units.vu0.vf_quad(rt(w));
    let a = store_address_of(e, w, v as u32)?;
    aligned(a, 16, true)?;
    e.write(b, a, 16, v)?;
    Ok(())
}

/// BC2F, BC2T and their likely forms test the COP2 condition signal, VU1 running a microprogram (§5.5.1).
fn bc2(e: &mut Ee, b: &mut dyn Bus, w: u32, pc: u32, on_true: bool, is_likely: bool) -> R {
    usable(e)?;
    let running = vpu(b)?.vu1_busy(e.planned());
    let t = running == on_true;
    if is_likely { likely(e, w, pc, t) } else { cond(e, w, pc, t) }
}
fn bc2f(e: &mut Ee, b: &mut dyn Bus, w: u32, pc: u32) -> R {
    bc2(e, b, w, pc, false, false)
}
fn bc2t(e: &mut Ee, b: &mut dyn Bus, w: u32, pc: u32) -> R {
    bc2(e, b, w, pc, true, false)
}
fn bc2fl(e: &mut Ee, b: &mut dyn Bus, w: u32, pc: u32) -> R {
    bc2(e, b, w, pc, false, true)
}
fn bc2tl(e: &mut Ee, b: &mut dyn Bus, w: u32, pc: u32) -> R {
    bc2(e, b, w, pc, true, true)
}

/// A VU operation issued as a macro instruction: it waits for VU0's microprogram to end (§1.3.2), then runs on VU0
/// with the window onto VU1; a divide's quotient lands in Q at the divider's latency, VWAITQ waits for it, and VCALLMS
/// and VCALLMSR start micro mode at Imm15 or CMSAR0 (§5.4.5, §5.4.7, §6.2).
fn vu_op(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    let vpu = vpu0(b)?;
    let Some((o, entry)) = vops::of_word(w) else { return Err(Stop::Halt) };
    match entry.class {
        Class::Callms => {
            wait_end(e, vpu, CALL);
            // A VCALLMS after a macro instruction stalls one cycle (§5.4.5).
            if vpu.last_macro.is_some_and(|m| m + 1 >= e.planned()) {
                e.wait(e.planned() + 1);
            }
            let addr = if o.form == Form::Callms { (w >> 6 & 0x7FFF) as u16 } else { vpu.cmsar0 };
            if !vpu.start(0, addr, e.planned() + pipe::START) {
                return Err(Stop::Halt);
            }
        }
        Class::WaitQ => {
            wait_end(e, vpu, MACRO);
            if let Some(l) = vpu.units.vu0.q_next {
                e.wait(l.at - 1);
            }
        }
        _ => {
            wait_end(e, vpu, MACRO);
            // The divider may still be busy with a microprogram's divide (§3.4.1, "DIV resource hazards").
            if matches!(entry.class, Class::Div(_)) {
                e.wait(vpu.units.vu0.div_free.saturating_sub(1));
            }
            let t = e.planned();
            vpu.settle_q(t);
            vpu.last_macro = Some(t);
            let units = &mut vpu.units;
            let mut c = Ctx::new(&mut units.vu0, Some(&mut units.vu1));
            (entry.run)(&mut c, w);
            if let (Class::Div(latency), Some(q)) = (entry.class, c.q_out) {
                let at = t + 1 + u64::from(latency);
                units.vu0.q_next = Some(Landing { at, value: q, flags: None });
                units.vu0.div_free = at;
            }
        }
    }
    Ok(())
}

const fn cop(name: &'static str, run: super::ops::Run, category: Category, row: usize) -> Op {
    Op { name, run, category, row, reads: 0, writes: 0, kind: Kind::Cop2 }
}

/// The COP2 instructions by the decoder's names, every VU operation as the one entry `vu` (Table 1-1's categories: COP2
/// Move for the transfers, COP2 Operate for the rest).
pub(super) static TABLE: &[Op] = &[
    cop("qmfc2", qmfc2, Category::Cop2Move, rows::VFROM),
    cop("qmtc2", qmtc2, Category::Cop2Move, rows::VTO),
    cop("cfc2", cfc2, Category::Cop2Move, rows::VFROM),
    cop("ctc2", ctc2, Category::Cop2Move, rows::VTO),
    cop("lqc2", lqc2, Category::Cop2Move, rows::VTO),
    cop("sqc2", sqc2, Category::Cop2Move, rows::VFROM),
    cop("bc2f", bc2f, Category::Branch, rows::BRANCH),
    cop("bc2t", bc2t, Category::Branch, rows::BRANCH),
    cop("bc2fl", bc2fl, Category::Branch, rows::BRANCH),
    cop("bc2tl", bc2tl, Category::Branch, rows::BRANCH),
    cop("vu", vu_op, Category::Cop2Operate, rows::VFMAC),
];

/// The index of `vu` in `TABLE`.
pub(super) const VU: usize = 10;

/// VU0's register `n` as a scoreboard resource; register 0 is none (§5.4.2: "VF00 and VI00 are not subject to hazard
/// checks").
fn vr(n: usize) -> u128 {
    if n == 0 { 0 } else { 1 << (VU0R + n - 1) }
}
fn gpr(n: usize) -> u128 {
    if n == 0 { 0 } else { 1 << n }
}

/// What the timing model is told of a COP2 instruction at `w`: its row and the registers it reads and writes, VU0's by
/// number.
pub(super) fn issue(op: &Op, w: u32) -> Issue {
    let (t, s, d) = (rt(w), rd(w), (w >> 6 & 31) as usize);
    let mut i = Issue::new(op.category, op.row);
    let (reads, writes) = match op.name {
        "qmfc2" | "cfc2" => (vr(s), gpr(t)),
        "qmtc2" | "ctc2" => (gpr(t), vr(s)),
        "lqc2" => (gpr(rs(w)), vr(t)),
        "sqc2" => (gpr(rs(w)) | vr(t), 0),
        "vu" => {
            let Some((o, entry)) = vops::of_word(w) else { return i };
            i.row = match entry.class {
                Class::Fmac => rows::VFMAC,
                Class::Div(13) => rows::VRSQRT,
                Class::Div(_) => rows::VDIV,
                _ => rows::VIALU,
            };
            i.alone = entry.class == Class::WaitQ;
            let flags = if entry.flags { vr(ccr::STATUS) | vr(ccr::MAC) } else { 0 };
            match o.form {
                Form::Fmac => (vr(s) | vr(t), vr(d) | flags),
                Form::FmacQ | Form::FmacI => (vr(s), vr(d) | flags),
                Form::Acc => (vr(s) | vr(t), flags),
                Form::AccQ | Form::AccI => (vr(s), flags),
                Form::FtFs => (vr(s), vr(t)),
                Form::Clip => (vr(s) | vr(t), vr(ccr::CLIP)),
                Form::Fdiv => (vr(s) | vr(t), vr(ccr::STATUS)),
                Form::Sqrt => (vr(t), vr(ccr::STATUS)),
                Form::Mtir => (vr(s), vr(t)),
                Form::Mfir => (vr(s), vr(t)),
                Form::Int3 => (vr(s) | vr(t), vr(d)),
                Form::Imm5 => (vr(s), vr(t)),
                Form::LoadInc | Form::LoadDec => (vr(s), vr(t) | vr(s)),
                Form::StoreInc | Form::StoreDec => (vr(s) | vr(t), vr(t)),
                Form::IntMem if o.name == "ilwr" => (vr(s), vr(t)),
                Form::IntMem => (vr(s) | vr(t), 0),
                Form::RFs => (vr(s), 0),
                Form::FtR => (0, vr(t)),
                Form::Callmsr => (vr(ccr::CMSAR0), 0),
                // The micro-only forms never reach COP2: the decoder leaves them out.
                _ => (0, 0),
            }
        }
        _ => (0, 0),
    };
    i.reads = reads;
    i.writes = writes;
    i
}
