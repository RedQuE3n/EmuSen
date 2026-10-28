//! Micro mode, both units (Sapphire_Plan.md §7, S13): the pairs of upper and lower instructions issued one a cycle at
//! their T stage, held back by the hazards of the VU User's Manual's §3.4 and nothing else. Every result is computed
//! as its pair issues; what the pipelines make visible later is modelled where no hazard check hides it: the MAC,
//! status and clipping flags written at the S stage, 4 cycles on (§3.4.4); Q at the divider's F stage, 7 or 13 cycles
//! on (§3.4.5); P at the EFU's, its latency on (§3.4.6, §7.4); and an integer result a branch in the very next pair does
//! not yet see (§3.4.8). A register read waits for the FMAC write before it field by field, and an integer read for
//! ILW's and ILWR's load; ACC, I, Q, P and R are never waited on (§3.4.1). Sapphire_Native.md §23 records each reading
//! and Sapphire_Disputes.md D-80 to D-85 the ones the manual leaves open.

use std::collections::VecDeque;

use emusen_mips::vu::{self as table, Form, I_BIT, Operation};

use super::micro::Micro;
use super::ops::{self, Class, Ctx, Deferred, Entry};
use super::{Landing, Units, Vu, fbrst, vpu_stat};

/// A microprogram's first pair issues 3 cycles after the instruction that starts it: VCALLMS's I Q R A D, then the
/// pair's M and T (§5.4.5's figures).
pub const START: u64 = 3;
/// An FMAC-pipeline result and the flags reach a T stage 4 cycles after their own (§3.4.4: written at S).
pub const FMAC: u64 = 4;
/// The pairs one `finish` or `release` runs before it stops waiting for an end that may never come.
const BOUND: u64 = 1 << 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum State {
    Ready = 0,
    Run = 1,
    Stop = 2,
}

/// What one pair leaves for its S stage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct FlagWrite {
    pub(super) at: u64,
    pub(super) d: Deferred,
}

/// A cycle not yet set: no pair run, no pair with M; and an integer register with no integer-unit write to remember.
pub(super) const NONE: u64 = u64::MAX;
/// No branch taken past its delay slot.
pub(super) const NO_JUMP: u32 = u32::MAX;

/// One unit's micro mode. The fields a compiled microprogram reads and writes hold plain numbers rather than `Option`s, so
/// that the code can address them (Sapphire_Native.md §45.3); each means what the comment beside it says.
pub struct Pipe {
    pub(super) unit: usize,
    pub(super) state: State,
    pub(super) pc: u16,
    /// The earliest T stage of the next pair.
    pub(super) next_t: u64,
    /// The T stage of the last pair run since a reset, `NONE` before the first.
    pub(super) last_t: u64,
    /// A taken branch's target, which the pair after its delay slot starts at; `NO_JUMP` for none.
    pub(super) jump: u32,
    /// The E bit was seen: the next pair is its delay slot, then the unit stops (§3.5.3).
    pub(super) ending: bool,
    /// A D or T bit on a branch: the unit stops after the delay slot with this VPU-STAT bit (§7.3.1); 0 for none.
    pub(super) halting: u32,
    /// The cycle each register field may be read: the T stage of the FMAC write before it plus 4.
    pub(super) vf_ready: [[u64; 4]; 32],
    /// The cycle each integer register may be read after ILW or ILWR.
    pub(super) vi_ready: [u64; 16],
    /// Each integer register's last integer-unit write: its T stage (`NONE` for none) and the value before it (0 then).
    pub(super) vi_last_at: [u64; 16],
    pub(super) vi_last_old: [u16; 16],
    pub(super) flags: VecDeque<FlagWrite>,
    pub(super) path1_free: u64,
    /// The pair after an XGKICK that found PATH1 busy issues no earlier than this (§3.4.9).
    pub(super) kick_wait: u64,
    /// The T stage of the run's first pair with M set (§5.4.6), `NONE` before it.
    pub(super) release_t: u64,
    /// VDS, VTS and VFS.
    pub(super) stopped_by: u32,
    pub(super) now: u64,
    /// Each pair's address and T stage, when a test asks.
    pub trace: Option<Vec<(u16, u64)>>,
    /// The debugger's probe, asked before each pair runs (Sapphire_Native.md §41).
    pub probe: Option<Box<crate::debugger::Probe>>,
    /// Pairs run since the unit was made, kept across FBRST's reset: the bench's count.
    run: u64,
    /// The recompiler, when the unit runs its microprograms compiled (Sapphire_Native.md §45); none for the interpreter.
    pub(super) jit: Option<Box<super::jit::Tier>>,
    /// The bench's clock: time-stamp counter ticks running pairs, calls, pairs (Sapphire_Native.md §45.7).
    clock: Option<[u64; 3]>,
}

/// The time-stamp counter, for the bench's clock.
fn ticks() -> u64 {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: RDTSC has no preconditions.
    unsafe {
        core::arch::x86_64::_rdtsc()
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        static ORIGIN: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        ORIGIN.get_or_init(std::time::Instant::now).elapsed().as_nanos() as u64
    }
}

fn some(v: u64) -> Option<u64> {
    (v != NONE).then_some(v)
}

/// What a pair reads and writes, for the hazards and the write priority.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Access {
    /// Up to three register reads, each a register and its lanes (bit 0 x).
    pub(super) vf: [(usize, u8); 3],
    pub(super) vi: [usize; 2],
    /// The register an FMAC-pipeline write goes to, with its lanes.
    pub(super) vf_write: Option<(usize, u8)>,
}

/// dest's lanes, bit 0 x (§4.1.3: bit 24 x).
fn lanes(w: u32) -> u8 {
    let d = (w >> 21 & 0xF) as u8;
    (d >> 3 & 1) | (d >> 1 & 2) | (d << 1 & 4) | (d << 3 & 8)
}
fn ft(w: u32) -> usize {
    (w >> 16 & 31) as usize
}
fn fs(w: u32) -> usize {
    (w >> 11 & 31) as usize
}
fn fd(w: u32) -> usize {
    (w >> 6 & 31) as usize
}
fn lane(f: u32) -> u8 {
    1 << f
}
fn fsf(w: u32) -> u8 {
    lane(w >> 21 & 3)
}
fn ftf(w: u32) -> u8 {
    lane(w >> 23 & 3)
}
const XYZ: u8 = 7;

/// The broadcast field of an FMAC word, none for a vector form (ps2tek's tables: Special1 below 1Ch, Special2's rows
/// 0000, 0001 and 0011's first four cells).
fn broadcast(w: u32) -> Option<u32> {
    if w & 0x3C != 0x3C {
        (w & 0x3F < 0x1C).then_some(w & 3)
    } else {
        let i = (w >> 6 & 0x1F) * 4 + (w & 3);
        (i < 16 || (24..28).contains(&i)).then_some(w & 3)
    }
}

pub(super) fn upper_access(o: &Operation, w: u32) -> Access {
    let m = lanes(w);
    let mut a = Access::default();
    let second = |a: &mut Access| a.vf[1] = (ft(w), broadcast(w).map_or(m, lane));
    match o.form {
        Form::Fmac | Form::Acc => {
            if o.name.starts_with("op") {
                a.vf = [(fs(w), XYZ), (ft(w), XYZ), (0, 0)];
            } else {
                a.vf[0] = (fs(w), m);
                second(&mut a);
            }
        }
        Form::FmacQ | Form::FmacI | Form::AccQ | Form::AccI | Form::FtFs => a.vf[0] = (fs(w), m),
        Form::Clip => a.vf = [(fs(w), XYZ), (ft(w), lane(3)), (0, 0)],
        _ => {}
    }
    a.vf_write = match o.form {
        Form::Fmac | Form::FmacQ | Form::FmacI if !o.name.starts_with("op") || o.name == "opmsub" => {
            Some((fd(w), if o.name == "opmsub" { XYZ } else { m }))
        }
        Form::FtFs => Some((ft(w), m)),
        _ => None,
    };
    a
}

pub(super) fn lower_access(o: &Operation, w: u32) -> Access {
    let m = lanes(w);
    let mut a = Access::default();
    let rotated = m >> 3 & 1 | m << 1 & 0xE;
    match o.form {
        Form::FtFs => {
            a.vf[0] = (fs(w), if o.name == "mr32" { rotated } else { m });
            a.vf_write = Some((ft(w), m));
        }
        Form::LoadInc | Form::LoadDec | Form::LoadOff => {
            a.vi[0] = fs(w);
            a.vf_write = Some((ft(w), m));
        }
        Form::StoreInc | Form::StoreDec | Form::StoreOff => {
            a.vf[0] = (fs(w), m);
            a.vi[0] = ft(w);
        }
        Form::Fdiv => a.vf = [(fs(w), fsf(w)), (ft(w), ftf(w)), (0, 0)],
        Form::Sqrt => a.vf[0] = (ft(w), ftf(w)),
        Form::Mtir | Form::RFs | Form::EfuF => a.vf[0] = (fs(w), fsf(w)),
        Form::Mfir => {
            a.vi[0] = fs(w);
            a.vf_write = Some((ft(w), m));
        }
        Form::Int3 => a.vi = [fs(w), ft(w)],
        Form::Imm5 | Form::Imm15 | Form::IntMem | Form::IntOff | Form::Is => {
            a.vi[0] = fs(w);
            if matches!(o.name, "iswr" | "isw") {
                a.vi[1] = ft(w);
            }
        }
        Form::ItIs => a.vi[0] = fs(w),
        Form::Branch2 => a.vi = [ft(w), fs(w)],
        Form::Branch1 => a.vi[0] = fs(w),
        Form::Efu => {
            let l = match o.name {
                "eatanxy" => 3,
                "eatanxz" => 5,
                "esum" => 15,
                _ => XYZ,
            };
            a.vf[0] = (fs(w), l);
        }
        Form::FtR | Form::Mfp => a.vf_write = Some((ft(w), m)),
        _ => {}
    }
    a
}

/// The integer register a lower operation writes, and whether it is a load (ready 4 cycles on), a flag check (seen by a
/// branch at once) or the integer unit's (seen by a branch a pair later).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViWrite {
    Load(usize),
    Check(usize),
    Alu(usize),
}

pub(super) fn vi_write(o: &Operation, e: &Entry, w: u32) -> Option<ViWrite> {
    let r = match o.form {
        Form::Int3 => fd(w),
        Form::Imm5 | Form::Imm15 | Form::Mtir | Form::Fs | Form::ItIs | Form::It | Form::Bal => ft(w),
        Form::LoadInc | Form::LoadDec => fs(w),
        Form::StoreInc | Form::StoreDec => ft(w),
        Form::IntMem | Form::IntOff if matches!(o.name, "ilwr" | "ilw") => return Some(ViWrite::Load(ft(w))),
        Form::Fc => 1,
        _ => return None,
    };
    Some(if e.class == Class::Fcheck { ViWrite::Check(r) } else { ViWrite::Alu(r) })
}

/// Lands the flags due by cycle `t`.
pub(super) fn drain(flags: &mut VecDeque<FlagWrite>, vu: &mut Vu, t: u64) {
    while let Some(f) = flags.front().copied().filter(|f| f.at <= t) {
        flags.pop_front();
        apply(vu, f.d);
    }
}

/// One pair's flag write as its S stage makes it.
pub(super) fn apply(vu: &mut Vu, d: Deferred) {
    // FSSET's pair writes the status flag as FSSET says, its upper instruction's effect ignored (§3.3.4).
    match (d.sticky_set, d.mac) {
        (Some(sticky), mac) => {
            vu.status = vu.status & 0x3F | sticky << super::status::STICKY;
            if let Some(mac) = mac {
                vu.mac = mac;
            }
        }
        (None, Some(mac)) => ops::apply_fmac_flags(vu, mac, d.zsuo, d.sticky),
        (None, None) => {}
    }
    match (d.clip_set, d.clip_shift) {
        (Some(c), _) => vu.clip = c,
        (None, Some(j)) => vu.clip = ops::shift_clip(vu.clip, j),
        _ => {}
    }
}

/// The upper word's operation, none for a word no page draws (bits 26-25 set, or a lower operation's code).
pub(super) fn upper_op(w: u32) -> Option<(&'static Operation, &'static Entry)> {
    ops::of_word(w).filter(|(o, _)| o.unit == table::Unit::Upper && w >> 25 & 3 == 0)
}

impl Pipe {
    /// A unit's micro mode on the tier the environment names (`super::jit::Tier::from_environment`), the interpreter
    /// unless it names the recompiler.
    pub fn new(unit: usize) -> Pipe {
        Pipe { jit: super::jit::Tier::from_environment(unit), ..Pipe::new_interpreted(unit) }
    }

    /// A unit's micro mode on the interpreter.
    pub fn new_interpreted(unit: usize) -> Pipe {
        Pipe {
            unit,
            state: State::Ready,
            pc: 0,
            next_t: 0,
            last_t: NONE,
            jump: NO_JUMP,
            ending: false,
            halting: 0,
            vf_ready: [[0; 4]; 32],
            vi_ready: [0; 16],
            vi_last_at: [NONE; 16],
            vi_last_old: [0; 16],
            flags: VecDeque::new(),
            path1_free: 0,
            kick_wait: 0,
            release_t: NONE,
            stopped_by: 0,
            now: 0,
            trace: None,
            probe: None,
            run: 0,
            jit: None,
            clock: None,
        }
    }

    /// Everything `step` reads and writes made equal to `other`'s, for the verifier's copy.
    pub(super) fn copy_from(&mut self, other: &Pipe) {
        let jit = self.jit.take();
        *self = Pipe { trace: None, probe: None, flags: other.flags.clone(), ..Pipe::new_interpreted(other.unit) };
        let o = other;
        (self.state, self.pc, self.next_t, self.last_t, self.jump, self.ending, self.halting) = (o.state, o.pc, o.next_t, o.last_t, o.jump, o.ending, o.halting);
        (self.vf_ready, self.vi_ready, self.vi_last_at, self.vi_last_old) = (o.vf_ready, o.vi_ready, o.vi_last_at, o.vi_last_old);
        (self.path1_free, self.kick_wait, self.release_t, self.stopped_by, self.now) = (o.path1_free, o.kick_wait, o.release_t, o.stopped_by, o.now);
        self.jit = jit;
    }

    /// The first field in which two units' micro modes differ, for the verifier.
    pub(super) fn differs(&self, o: &Pipe) -> Option<String> {
        let a = (self.state, self.pc, self.next_t, self.last_t, self.jump, self.ending, self.halting);
        let b = (o.state, o.pc, o.next_t, o.last_t, o.jump, o.ending, o.halting);
        if a != b {
            return Some(format!("state, PC, next T, last T, jump, ending, halting {a:?} against {b:?}"));
        }
        if (self.vf_ready, self.vi_ready) != (o.vf_ready, o.vi_ready) {
            return Some("the registers' ready cycles".into());
        }
        if (self.vi_last_at, self.vi_last_old) != (o.vi_last_at, o.vi_last_old) {
            return Some(format!("the integer writes a branch sees {:?} against {:?}", self.vi_last_at, o.vi_last_at));
        }
        if self.flags != o.flags {
            return Some(format!("the flag writes in flight {:?} against {:?}", self.flags, o.flags));
        }
        let c = (self.path1_free, self.kick_wait, self.release_t, self.stopped_by, self.now);
        let d = (o.path1_free, o.kick_wait, o.release_t, o.stopped_by, o.now);
        (c != d).then(|| format!("PATH1 free, kick wait, release, stopped by, now {c:?} against {d:?}"))
    }

    fn vu<'a>(&self, units: &'a mut Units) -> &'a mut Vu {
        if self.unit == 0 { &mut units.vu0 } else { &mut units.vu1 }
    }

    pub(super) fn pairs(&self, units: &Units) -> u16 {
        let len = if self.unit == 0 { units.vu0.micro.len() } else { units.vu1.micro.len() };
        (len / 8) as u16
    }

    /// The pair at the PC: its upper word (bits 63-32) and its lower word.
    fn fetch(&self, units: &Units) -> (u32, u32) {
        let mem = if self.unit == 0 { &units.vu0.micro } else { &units.vu1.micro };
        let at = self.pc as usize * 8;
        let word = |i: usize| u32::from_le_bytes(mem[at + i..at + i + 4].try_into().unwrap());
        (word(4), word(0))
    }

    /// An integer register as a branch at `t` reads it: as it was before an integer-unit write in the pair just before
    /// (§3.4.8: "a one-instruction slot is necessary"; Sapphire_Disputes.md D-80).
    fn branch_vi(&self, vu: &Vu, r: usize, t: u64) -> u16 {
        match self.vi_last_at.get(r) {
            Some(&at) if at != NONE && at + 1 == t => self.vi_last_old[r],
            _ => vu.vi(r),
        }
    }

    /// The earliest T stage of the pair `upper | lower`: one after the last, held by its register and integer reads,
    /// the divider and the EFU busy, WAITQ and WAITP, and a busy PATH1 behind an XGKICK.
    fn plan(&self, units: &Units, upper: u32, lower: u32) -> u64 {
        let vu = if self.unit == 0 { &units.vu0 } else { &units.vu1 };
        let mut t = self.next_t.max(self.kick_wait);
        let mut wait = |a: &Access| {
            for &(r, l) in &a.vf {
                if r != 0 {
                    for (f, ready) in self.vf_ready[r].iter().enumerate() {
                        if l >> f & 1 == 1 {
                            t = t.max(*ready);
                        }
                    }
                }
            }
            for &r in &a.vi {
                if (1..16).contains(&r) {
                    t = t.max(self.vi_ready[r]);
                }
            }
        };
        if let Some((o, _)) = upper_op(upper) {
            wait(&upper_access(o, upper));
        }
        if upper & I_BIT == 0
            && let Some((o, e)) = ops::of_lower(lower)
        {
            wait(&lower_access(o, lower));
            t = match e.class {
                Class::Div(_) => t.max(vu.div_free),
                Class::WaitQ => t.max(vu.q_next.map_or(0, |l| l.at)),
                Class::Efu(_) if self.unit == 1 => t.max(vu.efu_free),
                Class::WaitP => t.max(vu.p_last().map_or(0, |l| l.at - 1)),
                _ => t,
            };
        }
        t
    }

    /// Runs the pair at the PC if it issues by cycle `limit`.
    pub(super) fn step(&mut self, units: &mut Units, limit: u64) -> bool {
        if self.state != State::Run {
            return false;
        }
        let (upper, lower) = self.fetch(units);
        let t = self.plan(units, upper, lower);
        if t > limit {
            return false;
        }
        if let Some(p) = &mut self.probe
            && !p.admit(self.pc as u32 * 8, units.halts)
        {
            return false;
        }
        self.execute(units, upper, lower, t);
        true
    }

    fn execute(&mut self, units: &mut Units, upper: u32, lower: u32, t: u64) {
        let unit = self.unit;
        let mask = self.pairs(units) - 1;
        let pc = self.pc;
        if let Some(tr) = &mut self.trace {
            tr.push((pc, t));
        }
        let up = upper_op(upper);
        let low = if upper & I_BIT == 0 { ops::of_lower(lower) } else { None };
        let (vu, mut window) = match unit {
            0 => (&mut units.vu0, Some(&mut units.vu1)),
            _ => (&mut units.vu1, None),
        };
        let link = &mut units.link;
        vu.settle(t);
        drain(&mut self.flags, vu, t);

        // The lower instruction first, its register write held back so that the upper reads what both read at T.
        let mut held: Option<(usize, [u32; 4], u8)> = None;
        let mut low_out = Deferred::default();
        let mut taken = None;
        if let Some((o, e)) = low {
            match e.class {
                Class::Branch => taken = self.branch_of(vu, o, lower, pc, t, mask),
                Class::External => match o.name {
                    "xgkick" if unit == 1 => {
                        let addr = vu.vi(fs(lower)) & 0x3FF;
                        let start = t.max(self.path1_free);
                        if self.path1_free > t {
                            self.kick_wait = self.path1_free;
                        }
                        self.path1_free = link.path1.kick(&vu.data, addr, start);
                    }
                    "xtop" if unit == 1 => self.write_alu(vu, ft(lower), link.top[1], t),
                    "xitop" => self.write_alu(vu, ft(lower), link.itop[unit], t),
                    _ => {}
                },
                Class::WaitQ | Class::WaitP => {}
                Class::Efu(_) if unit == 0 => {}
                _ => {
                    let access = lower_access(o, lower);
                    let written = vi_write(o, e, lower);
                    let before = written.map(|w| match w {
                        ViWrite::Load(r) | ViWrite::Check(r) | ViWrite::Alu(r) => (w, vu.vi(r)),
                    });
                    let dest = access.vf_write.map(|(r, l)| (r, l, vu.vf(r)));
                    let mut c = Ctx::micro(vu, window.as_deref_mut());
                    (e.run)(&mut c, lower);
                    low_out = c.out;
                    let quotient = c.q_out;
                    if let Some((r, l, old)) = dest {
                        held = Some((r, vu.vf(r), l));
                        vu.set_vf(r, old);
                    }
                    match before {
                        Some((ViWrite::Load(r), _)) if (1..16).contains(&r) => self.vi_ready[r] = t + FMAC,
                        Some((ViWrite::Alu(r), old)) if r < 16 => (self.vi_last_at[r], self.vi_last_old[r]) = (t, old),
                        Some((ViWrite::Check(r), _)) if r < 16 => (self.vi_last_at[r], self.vi_last_old[r]) = (NONE, 0),
                        _ => {}
                    }
                    if let (Class::Div(latency), Some(q)) = (e.class, quotient) {
                        let at = t + u64::from(latency);
                        vu.q_next = Some(Landing { at, value: q, flags: low_out.divide });
                        vu.div_free = at;
                    }
                    if let (Class::Efu(latency), Some(p)) = (e.class, low_out.p) {
                        let at = t + u64::from(latency);
                        vu.send_p(Landing { at, value: p, flags: None });
                        vu.efu_free = at - 1;
                    }
                }
            }
        }

        // The upper instruction, then the I register, then the held lower write unless the upper wrote that register
        // (§3.4.3: "priority is given to the Upper instruction", whole registers).
        let mut up_out = Deferred::default();
        let mut up_write = None;
        if let Some((o, e)) = up {
            let mut c = Ctx::micro(vu, window);
            (e.run)(&mut c, upper);
            up_out = c.out;
            up_write = upper_access(o, upper).vf_write;
        }
        if upper & I_BIT != 0 {
            vu.i = lower;
        }
        let ready = |vf_ready: &mut [[u64; 4]; 32], r: usize, l: u8| {
            for (f, slot) in vf_ready[r].iter_mut().enumerate() {
                if l >> f & 1 == 1 {
                    *slot = t + FMAC;
                }
            }
        };
        if let Some((r, l)) = up_write {
            ready(&mut self.vf_ready, r, l);
        }
        if let Some((r, v, l)) = held
            && up_write.is_none_or(|(u, _)| u != r)
        {
            vu.set_vf(r, v);
            ready(&mut self.vf_ready, r, l);
        }
        let d = Deferred {
            mac: up_out.mac,
            zsuo: up_out.zsuo,
            sticky: up_out.sticky,
            clip_shift: up_out.clip_shift,
            clip_set: low_out.clip_set,
            sticky_set: low_out.sticky_set,
            ..Deferred::default()
        };
        if d.mac.is_some() || d.clip_shift.is_some() || d.clip_set.is_some() || d.sticky_set.is_some() {
            self.flags.push_back(FlagWrite { at: t + FMAC, d });
        }

        // The flow: the M bit; a branch taken past its delay slot; the D and T bits, which stop after the pair (after
        // the delay slot on a branch) with no delay slot of their own (§7.3.1); the E bit and its delay slot (§3.5.3).
        if upper >> 29 & 1 == 1 && self.release_t == NONE {
            self.release_t = t;
        }
        self.pc = if self.jump != NO_JUMP { self.jump as u16 } else { (pc + 1) & mask };
        self.jump = taken.map_or(NO_JUMP, u32::from);
        self.next_t = t + 1;
        self.last_t = t;
        let enables = link.fbrst >> (fbrst::VU1 * unit as u32);
        let halt = if upper >> 28 & 1 == 1 && enables & fbrst::DE0 != 0 {
            Some(vpu_stat::VDS)
        } else if upper >> 27 & 1 == 1 && enables & fbrst::TE0 != 0 {
            Some(vpu_stat::VTS)
        } else {
            None
        };
        let mut stop = None;
        if self.halting != 0 {
            stop = Some(std::mem::take(&mut self.halting));
        } else if let Some(bit) = halt {
            if taken.is_some() { self.halting = bit } else { stop = Some(bit) }
        } else if self.ending {
            self.ending = false;
            self.state = State::Ready;
        } else if upper >> 30 & 1 == 1 {
            self.ending = true;
        }
        if let Some(bit) = stop {
            self.state = State::Stop;
            self.stopped_by = bit;
            link.irq |= 1 << unit;
        }
        if self.state != State::Run {
            if self.jump != NO_JUMP {
                self.pc = std::mem::replace(&mut self.jump, NO_JUMP) as u16;
            }
            vu.tpc = self.pc;
        }
    }

    pub(super) fn write_alu(&mut self, vu: &mut Vu, r: usize, v: u16, t: u64) {
        if r < 16 {
            (self.vi_last_at[r], self.vi_last_old[r]) = (t, vu.vi(r));
        }
        vu.set_vi(r, v);
    }

    /// A branch at `pc` issued at `t`: its target when taken; BAL and JALR write the address after the delay slot
    /// (§4.3: "PC + 2 × 8"), JALR having read its target first; JR and JALR read the target as it now is, a write in the
    /// pair just before included (D-80 as amended by the `vu/lower/branch` console text).
    fn branch_of(&mut self, vu: &mut Vu, o: &Operation, w: u32, pc: u16, t: u64, mask: u16) -> Option<u16> {
        let rel = pc.wrapping_add(1).wrapping_add(table::imm11(w) as u16) & mask;
        let (a, b) = (self.branch_vi(vu, ft(w), t), self.branch_vi(vu, fs(w), t));
        let link = pc.wrapping_add(2) & mask;
        match o.name {
            "b" => Some(rel),
            "bal" => {
                self.write_alu(vu, ft(w), link, t);
                Some(rel)
            }
            "jr" => Some(vu.vi(fs(w)) & mask),
            "jalr" => {
                let target = vu.vi(fs(w));
                self.write_alu(vu, ft(w), link, t);
                Some(target & mask)
            }
            "ibeq" => (a == b).then_some(rel),
            "ibne" => (a != b).then_some(rel),
            "ibltz" => ((b as i16) < 0).then_some(rel),
            "ibgtz" => ((b as i16) > 0).then_some(rel),
            "iblez" => ((b as i16) <= 0).then_some(rel),
            "ibgez" => ((b as i16) >= 0).then_some(rel),
            _ => None,
        }
    }

    /// Runs pairs while the unit runs and each issues by `limit`; with `release`, until a pair with M has run; at most
    /// `bound` pairs. The interpreter's loop, or the recompiler's, which leaves the unit as this loop would.
    fn run_pairs(&mut self, units: &mut Units, limit: u64, release: bool, bound: u64) {
        let from = self.clock.map(|_| ticks());
        let n = if self.jit.is_some() {
            super::jit::run(self, units, limit, release, bound)
        } else {
            let mut n = 0;
            while self.state == State::Run && !(release && self.release_t != NONE) && n < bound && self.step(units, limit) {
                n += 1;
            }
            n
        };
        self.run += n;
        if let (Some(c), Some(from)) = (&mut self.clock, from) {
            c[0] += ticks() - from;
            c[1] += 1;
            c[2] += n;
        }
    }

    /// The earliest cycle by which `run_to` could change more than the unit's clock: its next pair's earliest T stage
    /// while it runs, and the first flag, quotient or EFU result to land (Sapphire_Speed.md §57.3).
    fn due(&self, units: &Units) -> u64 {
        let vu = if self.unit == 0 { &units.vu0 } else { &units.vu1 };
        let land = super::jit::pending(self, vu);
        if self.state != State::Run {
            return land;
        }
        let stall = self.jit.as_ref().map_or(0, |j| j.stalled_until(self.pc, vu.micro.lent()));
        land.min(self.next_t.max(self.kick_wait).max(stall))
    }

    /// Lands every flag and result due by `t` in this unit.
    fn land(&mut self, units: &mut Units, t: u64) {
        let vu = if self.unit == 0 { &mut units.vu0 } else { &mut units.vu1 };
        drain(&mut self.flags, vu, t);
        vu.settle(t);
    }
}

impl Micro for Pipe {
    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        self.write(w, 0);
    }

    fn write_state_resting(&self, w: &mut emusen_native::StateWriter, rest: u64) {
        self.write(w, rest);
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        self.read(r)
    }

    fn fits(&self) -> bool {
        self.flags.len() <= FLAG_SLOTS
    }

    fn start(&mut self, units: &mut Units, addr: u16, first: u64) -> bool {
        let mask = self.pairs(units) - 1;
        self.state = State::Run;
        self.pc = addr & mask;
        self.next_t = first.max(self.next_t);
        self.jump = NO_JUMP;
        self.ending = false;
        self.halting = 0;
        self.release_t = NONE;
        self.stopped_by = 0;
        if let Some(j) = &mut self.jit {
            j.entered(self.pc);
        }
        true
    }

    fn run_to(&mut self, units: &mut Units, t: u64) {
        if t < self.due(units) {
            self.now = self.now.max(t);
            return;
        }
        self.run_pairs(units, t, false, u64::MAX);
        self.now = self.now.max(t);
        self.land(units, t);
    }

    fn pairs_run(&self) -> u64 {
        self.run
    }

    fn busy(&self) -> bool {
        self.state == State::Run
    }

    fn idle(&self, units: &Units) -> bool {
        let vu = if self.unit == 0 { &units.vu0 } else { &units.vu1 };
        self.state != State::Run && self.flags.is_empty() && vu.q_next.is_none() && vu.p_next.is_none() && vu.p_later.is_none()
    }

    fn idle_to(&mut self, units: &mut Units, t: u64) -> bool {
        let idle = self.idle(units);
        if idle {
            self.now = self.now.max(t);
        }
        idle
    }

    fn finish(&mut self, units: &mut Units, _: u64) -> Option<u64> {
        self.run_pairs(units, u64::MAX, false, BOUND);
        let vu = if self.unit == 0 { &mut units.vu0 } else { &mut units.vu1 };
        drain(&mut self.flags, vu, u64::MAX);
        some(self.last_t)
    }

    fn release(&mut self, units: &mut Units, _: u64) -> Option<u64> {
        self.run_pairs(units, u64::MAX, true, BOUND);
        match some(self.release_t) {
            Some(m) if self.state == State::Run => Some(m + 1),
            _ => some(self.last_t).map(|t| t + FMAC),
        }
    }

    fn force_break(&mut self, units: &mut Units, at: u64) {
        self.run_to(units, at);
        if self.state == State::Run {
            self.state = State::Stop;
            self.stopped_by = vpu_stat::VFS;
            let pc = self.pc;
            self.vu(units).tpc = pc;
        }
    }

    fn reset(&mut self) {
        *self = Pipe { trace: self.trace.take().map(|_| Vec::new()), probe: self.probe.take(), jit: self.jit.take(), clock: self.clock, run: self.run, ..Pipe::new_interpreted(self.unit) };
    }

    fn last(&self) -> Option<u64> {
        some(self.last_t)
    }

    fn pc(&self) -> u16 {
        self.pc
    }

    fn probe(&mut self) -> Option<&mut Option<Box<crate::debugger::Probe>>> {
        Some(&mut self.probe)
    }

    fn set_tier(&mut self, tier: Option<super::jit::Settings>) {
        self.jit = tier.map(|s| Box::new(super::jit::Tier::new(self.unit, s)));
    }

    fn jit_counts(&self) -> Option<super::jit::Counts> {
        self.jit.as_ref().map(|j| j.counts())
    }

    fn set_clock(&mut self, on: bool) {
        self.clock = on.then_some([0; 3]);
    }

    fn clock(&self) -> Option<[u64; 3]> {
        self.clock
    }

    fn stat(&self) -> u32 {
        let run = self.state == State::Run;
        let gw = if run && self.kick_wait > self.now { vpu_stat::VGW } else { 0 };
        (u32::from(run) * vpu_stat::VBS) | self.stopped_by | gw
    }
}

/// Flag writes in flight in the state: an FMAC pair's land 4 cycles on, one pair a cycle.
const FLAG_SLOTS: usize = 16;

/// A flag write as the state holds it: which of its six optional values are present, and its eight words.
fn deferred_words(d: &Deferred) -> (u8, [u32; 8]) {
    let opts = [d.mac, d.clip_shift, d.clip_set, d.sticky_set, d.divide, d.p];
    let some = opts.iter().enumerate().fold(0, |b, (i, o)| b | u8::from(o.is_some()) << i);
    (some, [d.mac.unwrap_or(0), d.clip_shift.unwrap_or(0), d.clip_set.unwrap_or(0), d.sticky_set.unwrap_or(0), d.divide.unwrap_or(0), d.p.unwrap_or(0), d.zsuo, d.sticky])
}

fn deferred_of(some: u8, v: &[u32]) -> Deferred {
    let o = |i: usize| (some >> i & 1 != 0).then_some(v[i]);
    Deferred { mac: o(0), clip_shift: o(1), clip_set: o(2), sticky_set: o(3), divide: o(4), p: o(5), zsuo: v[6], sticky: v[7] }
}

/// A unit's micro mode in `SPH2` (Sapphire_Native.md §38.2): its state and program counter, the next pair's time, the
/// branch, end and halt in progress, the hazards' ready cycles and the flag writes in flight. The trace is a test's.
impl Pipe {
    fn write(&self, w: &mut emusen_native::StateWriter, rest: u64) {
        w.u8("State", match self.state {
            State::Ready => 0,
            State::Run => 1,
            State::Stop => 2,
        });
        w.u16("Pc", self.pc);
        w.u64("NextT", self.next_t);
        crate::state::opt_u64(w, "LastT", some(self.last_t));
        crate::state::opt_u32(w, "Jump", (self.jump != NO_JUMP).then_some(self.jump));
        w.bool("Ending", self.ending);
        crate::state::opt_u32(w, "Halting", (self.halting != 0).then_some(self.halting));
        w.u64s("VfReady", self.vf_ready.as_flattened());
        w.u64s("ViReady", &self.vi_ready);
        w.u16("ViLastSome", self.vi_last_at.iter().enumerate().fold(0, |b, (i, &at)| b | u16::from(at != NONE) << i));
        w.u64s("ViLastAt", &self.vi_last_at.map(|at| if at == NONE { 0 } else { at }));
        w.u16s("ViLastValue", &self.vi_last_old);
        w.u32("FlagsLength", self.flags.len() as u32);
        let slots = crate::state::slots::<_, FLAG_SLOTS>(&self.flags);
        let words: [(u8, [u32; 8]); FLAG_SLOTS] = slots.map(|f| deferred_words(&f.d));
        w.u64s("FlagsAt", &slots.map(|f| f.at));
        w.bytes("FlagsSome", &words.map(|d| d.0));
        w.u32s("FlagsValues", words.map(|d| d.1).as_flattened());
        w.u64s("Times", &[self.path1_free, self.kick_wait, self.now.max(rest)]);
        crate::state::opt_u64(w, "ReleaseT", some(self.release_t));
        w.u32("StoppedBy", self.stopped_by);
    }

    fn read(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        self.state = match r.u8()? {
            1 => State::Run,
            2 => State::Stop,
            _ => State::Ready,
        };
        self.pc = r.u16()?;
        self.next_t = r.u64()?;
        self.last_t = crate::state::read_opt_u64(r)?.unwrap_or(NONE);
        self.jump = crate::state::read_opt_u32(r)?.unwrap_or(NO_JUMP);
        self.ending = r.bool()?;
        self.halting = crate::state::read_opt_u32(r)?.unwrap_or(0);
        r.u64s(self.vf_ready.as_flattened_mut())?;
        r.u64s(&mut self.vi_ready)?;
        let some = r.u16()?;
        let mut at = [0u64; 16];
        r.u64s(&mut at)?;
        let mut value = [0u16; 16];
        r.u16s(&mut value)?;
        self.vi_last_at = std::array::from_fn(|i| if some >> i & 1 != 0 { at[i] } else { NONE });
        self.vi_last_old = std::array::from_fn(|i| if some >> i & 1 != 0 { value[i] } else { 0 });
        let len = r.u32()?;
        let mut at = [0u64; FLAG_SLOTS];
        let mut some = [0u8; FLAG_SLOTS];
        let mut values = [0u32; FLAG_SLOTS * 8];
        r.u64s(&mut at)?;
        r.bytes(&mut some)?;
        r.u32s(&mut values)?;
        let slots: [FlagWrite; FLAG_SLOTS] = std::array::from_fn(|i| FlagWrite { at: at[i], d: deferred_of(some[i], &values[8 * i..8 * i + 8]) });
        crate::state::refill(&mut self.flags, len, &slots);
        let mut t = [0u64; 3];
        r.u64s(&mut t)?;
        [self.path1_free, self.kick_wait, self.now] = t;
        self.release_t = crate::state::read_opt_u64(r)?.unwrap_or(NONE);
        self.stopped_by = r.u32()?;
        if let Some(j) = &mut self.jit {
            j.loaded(self.pc, self.state == State::Run);
        }
        Ok(())
    }
}
