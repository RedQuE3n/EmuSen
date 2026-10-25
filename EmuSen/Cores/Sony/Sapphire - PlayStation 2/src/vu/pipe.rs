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
enum State {
    Ready,
    Run,
    Stop,
}

/// What one pair leaves for its S stage.
#[derive(Clone, Copy, Debug)]
struct FlagWrite {
    at: u64,
    d: Deferred,
}

/// One unit's micro mode.
pub struct Pipe {
    unit: usize,
    state: State,
    pc: u16,
    /// The earliest T stage of the next pair.
    next_t: u64,
    /// The T stage of the last pair run, since a reset.
    last_t: Option<u64>,
    /// A taken branch's target, which the pair after its delay slot starts at.
    jump: Option<u16>,
    /// The E bit was seen: the next pair is its delay slot, then the unit stops (§3.5.3).
    ending: bool,
    /// A D or T bit on a branch: the unit stops after the delay slot with this VPU-STAT bit (§7.3.1).
    halting: Option<u32>,
    /// The cycle each register field may be read: the T stage of the FMAC write before it plus 4.
    vf_ready: [[u64; 4]; 32],
    /// The cycle each integer register may be read after ILW or ILWR.
    vi_ready: [u64; 16],
    /// Each integer register's last integer-unit write: its T stage and the value before it.
    vi_last: [Option<(u64, u16)>; 16],
    flags: VecDeque<FlagWrite>,
    path1_free: u64,
    /// The pair after an XGKICK that found PATH1 busy issues no earlier than this (§3.4.9).
    kick_wait: u64,
    /// The T stage of the run's first pair with M set (§5.4.6).
    release_t: Option<u64>,
    /// VDS, VTS and VFS.
    stopped_by: u32,
    now: u64,
    /// Each pair's address and T stage, when a test asks.
    pub trace: Option<Vec<(u16, u64)>>,
}

/// What a pair reads and writes, for the hazards and the write priority.
#[derive(Clone, Copy, Debug, Default)]
struct Access {
    /// Up to three register reads, each a register and its lanes (bit 0 x).
    vf: [(usize, u8); 3],
    vi: [usize; 2],
    /// The register an FMAC-pipeline write goes to, with its lanes.
    vf_write: Option<(usize, u8)>,
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

fn upper_access(o: &Operation, w: u32) -> Access {
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

fn lower_access(o: &Operation, w: u32) -> Access {
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
enum ViWrite {
    Load(usize),
    Check(usize),
    Alu(usize),
}

fn vi_write(o: &Operation, e: &Entry, w: u32) -> Option<ViWrite> {
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
fn drain(flags: &mut VecDeque<FlagWrite>, vu: &mut Vu, t: u64) {
    while let Some(f) = flags.front().copied().filter(|f| f.at <= t) {
        flags.pop_front();
        let d = f.d;
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
}

/// The upper word's operation, none for a word no page draws (bits 26-25 set, or a lower operation's code).
fn upper_op(w: u32) -> Option<(&'static Operation, &'static Entry)> {
    ops::of_word(w).filter(|(o, _)| o.unit == table::Unit::Upper && w >> 25 & 3 == 0)
}

impl Pipe {
    pub fn new(unit: usize) -> Pipe {
        Pipe {
            unit,
            state: State::Ready,
            pc: 0,
            next_t: 0,
            last_t: None,
            jump: None,
            ending: false,
            halting: None,
            vf_ready: [[0; 4]; 32],
            vi_ready: [0; 16],
            vi_last: [None; 16],
            flags: VecDeque::new(),
            path1_free: 0,
            kick_wait: 0,
            release_t: None,
            stopped_by: 0,
            now: 0,
            trace: None,
        }
    }

    fn vu<'a>(&self, units: &'a mut Units) -> &'a mut Vu {
        if self.unit == 0 { &mut units.vu0 } else { &mut units.vu1 }
    }

    fn pairs(&self, units: &Units) -> u16 {
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
        match self.vi_last.get(r).copied().flatten() {
            Some((at, old)) if at + 1 == t => old,
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
    fn step(&mut self, units: &mut Units, limit: u64) -> bool {
        if self.state != State::Run {
            return false;
        }
        let (upper, lower) = self.fetch(units);
        let t = self.plan(units, upper, lower);
        if t > limit {
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
                        Some((ViWrite::Alu(r), old)) if r < 16 => self.vi_last[r] = Some((t, old)),
                        Some((ViWrite::Check(r), _)) if r < 16 => self.vi_last[r] = None,
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
        if upper >> 29 & 1 == 1 && self.release_t.is_none() {
            self.release_t = Some(t);
        }
        self.pc = self.jump.take().unwrap_or((pc + 1) & mask);
        self.jump = taken;
        self.next_t = t + 1;
        self.last_t = Some(t);
        let enables = link.fbrst >> (fbrst::VU1 * unit as u32);
        let halt = if upper >> 28 & 1 == 1 && enables & fbrst::DE0 != 0 {
            Some(vpu_stat::VDS)
        } else if upper >> 27 & 1 == 1 && enables & fbrst::TE0 != 0 {
            Some(vpu_stat::VTS)
        } else {
            None
        };
        let mut stop = None;
        if let Some(bit) = self.halting.take() {
            stop = Some(bit);
        } else if let Some(bit) = halt {
            if taken.is_some() { self.halting = Some(bit) } else { stop = Some(bit) }
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
            self.pc = self.jump.take().unwrap_or(self.pc);
            vu.tpc = self.pc;
        }
    }

    fn write_alu(&mut self, vu: &mut Vu, r: usize, v: u16, t: u64) {
        if r < 16 {
            self.vi_last[r] = Some((t, vu.vi(r)));
        }
        vu.set_vi(r, v);
    }

    /// A branch at `pc` issued at `t`: its target when taken; BAL and JALR write the address after the delay slot
    /// (§4.3: "PC + 2 × 8"), JALR having read its target first (the `vu/lower/branch` console text).
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
            "jr" => Some(b & mask),
            "jalr" => {
                self.write_alu(vu, ft(w), link, t);
                Some(b & mask)
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

    fn run_bounded(&mut self, units: &mut Units, until: impl Fn(&Pipe) -> bool) {
        let mut n = 0;
        while self.state == State::Run && !until(self) && n < BOUND && self.step(units, u64::MAX) {
            n += 1;
        }
    }

    /// Lands every flag and result due by `t` in this unit.
    fn land(&mut self, units: &mut Units, t: u64) {
        let vu = if self.unit == 0 { &mut units.vu0 } else { &mut units.vu1 };
        drain(&mut self.flags, vu, t);
        vu.settle(t);
    }
}

impl Micro for Pipe {
    fn start(&mut self, units: &mut Units, addr: u16, first: u64) -> bool {
        let mask = self.pairs(units) - 1;
        self.state = State::Run;
        self.pc = addr & mask;
        self.next_t = first.max(self.next_t);
        self.jump = None;
        self.ending = false;
        self.halting = None;
        self.release_t = None;
        self.stopped_by = 0;
        true
    }

    fn run_to(&mut self, units: &mut Units, t: u64) {
        while self.step(units, t) {}
        self.now = self.now.max(t);
        self.land(units, t);
    }

    fn busy(&self) -> bool {
        self.state == State::Run
    }

    fn finish(&mut self, units: &mut Units, _: u64) -> Option<u64> {
        self.run_bounded(units, |_| false);
        let vu = if self.unit == 0 { &mut units.vu0 } else { &mut units.vu1 };
        drain(&mut self.flags, vu, u64::MAX);
        self.last_t
    }

    fn release(&mut self, units: &mut Units, _: u64) -> Option<u64> {
        self.run_bounded(units, |p| p.release_t.is_some());
        match self.release_t {
            Some(m) if self.state == State::Run => Some(m + 1),
            _ => self.last_t.map(|t| t + FMAC),
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
        *self = Pipe { trace: self.trace.take().map(|_| Vec::new()), ..Pipe::new(self.unit) };
    }

    fn last(&self) -> Option<u64> {
        self.last_t
    }

    fn stat(&self) -> u32 {
        let run = self.state == State::Run;
        let gw = if run && self.kick_wait > self.now { vpu_stat::VGW } else { 0 };
        (u32::from(run) * vpu_stat::VBS) | self.stopped_by | gw
    }
}
