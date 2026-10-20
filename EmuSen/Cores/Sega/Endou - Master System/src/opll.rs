//! The YM2413 (OPLL), the FM unit of the Japanese Master System and of the Mark III's FM Sound Unit (Endou_Opll.md).
//! A chip that is not a processor: the caller owns the clock and runs it a sample at a time, 72 of its clocks each;
//! writes are its two ports; its output is the melody and rhythm sums a sample, in the DAC's nine-bit units. Each rule
//! cites the page's section, documented by Yamaha's application manual or measured on the black box (Endou_Opll.md §3).

use emusen_native::{StateReader, StateWriter, Truncated};

mod tables;

pub use tables::{EXP, INSTRUMENTS, LOGSIN, RHYTHM_INSTRUMENTS};

/// Master clocks a sample: 18 slots of four [D the manual, (III-1-6): fsam = fM / 72].
pub const CLOCKS_PER_SAMPLE: u64 = 72;

/// The register file's size: addresses $00-$3F, of which the manual's map names $00-$07, $0E, $0F, $10-$18, $20-$28
/// and $30-$38 [D (II-7)].
pub const REGISTERS: usize = 0x40;

/// The envelope's stages (Endou_Opll.md §3.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// At key-on, the level is taken to silence at rate 48 before the attack starts [M §3.5].
    Damp,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl Stage {
    fn byte(self) -> u8 {
        self as u8
    }

    fn of_byte(b: u8) -> Option<Stage> {
        [Stage::Damp, Stage::Attack, Stage::Decay, Stage::Sustain, Stage::Release].into_iter().find(|s| s.byte() == b)
    }
}

/// The silent level: the envelope's seven bits all set [D (III-3): 48 dB in steps of 0.375 dB].
pub const SILENT: u8 = 127;

/// From this envelope level a slot sounds nothing [M §3.5].
pub const QUIET: u8 = 124;

/// An attenuation past the log-sine's range, which `operator` gives as 0.
const MUTE: u32 = 0x100;

/// One operator slot: its phase, its envelope and its key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    /// The phase accumulator, 19 bits a cycle [M §3.2].
    pub phase: u32,
    /// The envelope's attenuation, 0 loud to 127 silent, in 0.375 dB.
    pub level: u8,
    pub stage: Stage,
    pub key: bool,
    /// The last two outputs, newest first: the modulator's feedback averages them [D Table III-6].
    pub out: [i16; 2],
}

impl Slot {
    const OFF: Slot = Slot { phase: 0, level: SILENT, stage: Stage::Release, key: false, out: [0; 2] };
}

/// One operator's eight parameters, as a patch's two bytes of each register give them [D (III-1-2) to (III-1-5)].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Operator {
    am: bool,
    vib: bool,
    sustained: bool,
    ksr: bool,
    mul: u8,
    ksl: u8,
    ar: u8,
    dr: u8,
    sl: u8,
    rr: u8,
    /// The half-wave (rectified) sine: DM for the modulator, DC for the carrier.
    half: bool,
}

impl Operator {
    /// Operator `op` (0 the modulator, 1 the carrier) of an eight-byte patch.
    fn of(p: &[u8; 8], op: usize) -> Operator {
        let a = p[op];
        Operator {
            am: a & 0x80 != 0,
            vib: a & 0x40 != 0,
            sustained: a & 0x20 != 0,
            ksr: a & 0x10 != 0,
            mul: a & 0x0F,
            ksl: p[2 + op] >> 6,
            ar: p[4 + op] >> 4,
            dr: p[4 + op] & 0x0F,
            sl: p[6 + op] >> 4,
            rr: p[6 + op] & 0x0F,
            half: p[3] & (0x08 << op) != 0,
        }
    }
}

/// Twice the multiple of each MUL code [D Table III-1].
const MUL2: [u32; 16] = [1, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 20, 24, 24, 30, 30];

/// The level key scale at 3 dB an octave in block 7, by the F-number's four high bits, in 0.375 dB; each lower block is
/// 3 dB (8) less, never below 0 [D Table III-5].
const KSL: [u8; 16] = [0, 24, 32, 37, 40, 43, 45, 47, 48, 50, 51, 52, 53, 54, 55, 56];

/// Which of eight envelope ticks step, by the rate's low two bits: 4, 5, 6 or 7 of eight [M §3.5].
const STEPS: [[u8; 8]; 4] = [[0, 1, 0, 1, 0, 1, 0, 1], [0, 1, 0, 1, 1, 1, 0, 1], [0, 1, 1, 1, 0, 1, 1, 1], [0, 1, 1, 1, 1, 1, 1, 1]];

/// The rhythm instruments' channels: the bass drum on channel 7, the high hat and snare on 8, the tom-tom and top cymbal
/// on 9 [D Table III-9], numbered from 0 here.
const BD: usize = 6;
const HH_SD: usize = 7;
const TOM_TC: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opll {
    /// The address last written to port 0.
    pub address: u8,
    pub regs: [u8; REGISTERS],
    /// Slot `2c` is channel c's modulator and `2c + 1` its carrier.
    pub slots: [Slot; 18],
    /// Samples since reset, which the envelope's ticks and the LFOs count.
    pub counter: u32,
    /// The tremolo's triangle, 0-104, and its direction [M §3.6].
    pub am: u8,
    pub am_down: bool,
    /// The rhythm's noise register, 23 bits [A §3.8].
    pub noise: u32,
}

impl Default for Opll {
    fn default() -> Opll {
        Opll::new()
    }
}

impl Opll {
    /// The chip after its IC pin's reset: every register 0 [D (III-1)], every slot silent.
    pub fn new() -> Opll {
        Opll { address: 0, regs: [0; REGISTERS], slots: [Slot::OFF; 18], counter: 0, am: 0, am_down: false, noise: 1 }
    }

    /// A write to port 0, the address [D Table II-1].
    pub fn write_address(&mut self, v: u8) {
        self.address = v;
    }

    /// A write to port 1, the data for the address last written; addresses the map does not name take nothing [A].
    pub fn write_data(&mut self, v: u8) {
        let a = self.address as usize;
        if a < REGISTERS {
            self.regs[a] = v;
        }
    }

    fn rhythm(&self) -> bool {
        self.regs[0x0E] & 0x20 != 0
    }

    fn fnum(&self, ch: usize) -> u32 {
        self.regs[0x10 + ch] as u32 | (self.regs[0x20 + ch] as u32 & 1) << 8
    }

    fn block(&self, ch: usize) -> u32 {
        (self.regs[0x20 + ch] as u32 >> 1) & 7
    }

    /// The patch a slot plays: the rhythm's own where rhythm mode takes the slot, else its channel's instrument, 0 the
    /// user's registers $00-$07.
    fn patch(&self, slot: usize) -> &[u8; 8] {
        let ch = slot / 2;
        if ch >= BD && self.rhythm() {
            return &RHYTHM_INSTRUMENTS[ch - BD];
        }
        match self.regs[0x30 + ch] >> 4 {
            0 => self.regs[..8].try_into().expect("eight registers"),
            n => &INSTRUMENTS[n as usize - 1],
        }
    }

    /// Whether a slot is keyed: its channel's KEY bit, or in rhythm mode the rhythm bits as well for the slots they
    /// take [D (III-1-7)].
    fn keyed(&self, slot: usize) -> bool {
        let ch = slot / 2;
        let key = self.regs[0x20 + ch] & 0x10 != 0;
        if ch < BD || !self.rhythm() {
            return key;
        }
        let r = self.regs[0x0E];
        key | match slot {
            12 | 13 => r & 0x10 != 0,
            14 => r & 0x01 != 0,
            15 => r & 0x08 != 0,
            16 => r & 0x04 != 0,
            17 => r & 0x02 != 0,
            _ => unreachable!(),
        }
    }

    /// The attenuation a slot's volume adds, in 0.375 dB: the modulator's TL in 0.75 dB steps, the carrier's VOL in 3 dB
    /// steps; in rhythm mode the high hat's and tom-tom's modulator slots take a rhythm volume as a carrier does [D
    /// (III-1-3), (III-1-8)].
    fn volume(&self, slot: usize, patch: &[u8; 8]) -> u32 {
        let ch = slot / 2;
        let rhythm = ch >= BD && self.rhythm();
        if slot & 1 == 1 || (rhythm && ch != BD) {
            let v = self.regs[0x30 + ch];
            let vol = if slot & 1 == 0 { v >> 4 } else { v & 0x0F };
            (vol as u32) << 3
        } else {
            (patch[2] as u32 & 0x3F) << 1
        }
    }

    /// The level key scale's attenuation for a channel's frequency: KSL 1, 2 and 3 are 1.5, 3 and 6 dB an octave [D
    /// Table III-5; M §3.4 for which code is which, the manual's Table III-4 giving bits 7 and 6 the other way].
    fn ksl(&self, ch: usize, ksl: u8) -> u32 {
        if ksl == 0 {
            return 0;
        }
        let base = (KSL[(self.fnum(ch) >> 5) as usize] as i32 - 8 * (7 - self.block(ch) as i32)).max(0) as u32;
        match ksl {
            1 => base >> 1,
            2 => base,
            _ => base << 1,
        }
    }

    /// The vibrato's step this sample, in halves of an F-number unit: eight steps of 1,024 samples, 0, half, full, half,
    /// 0, then the same below [M §3.6].
    fn vibrato(&self, fnum: u32) -> i32 {
        let top = (fnum >> 6) as i32;
        match (self.counter >> 10) & 7 {
            0 | 4 => 0,
            1 | 3 => top >> 1,
            2 => top,
            5 | 7 => -(top >> 1),
            _ => -top,
        }
    }

    /// A slot's phase step a sample, in its 19-bit cycle: ((2F + vibrato) × 2^block × 2·MUL) / 4 [M §3.2].
    fn increment(&self, ch: usize, op: &Operator) -> u32 {
        let fnum = self.fnum(ch);
        let f2 = 2 * fnum as i32 + if op.vib { self.vibrato(fnum) } else { 0 };
        (((f2 as u32) << self.block(ch)) * MUL2[op.mul as usize]) >> 2
    }

    /// A rate's step this sample, by the envelope's tick counter [M §3.5]: rates 4-51 step 4-7 times in eight ticks of
    /// 2^(13 - rate / 4) samples, 52-55 the same each sample, 56-59 one more each sample, 60-63 two each sample.
    fn step(&self, rate: u32) -> u8 {
        if rate < 4 {
            return 0;
        }
        let (high, low) = (rate >> 2, (rate & 3) as usize);
        match high {
            1..=12 => {
                let shift = 13 - high;
                if self.counter & ((1 << shift) - 1) != 0 {
                    0
                } else {
                    STEPS[low][((self.counter >> shift) & 7) as usize]
                }
            }
            13 => STEPS[low][(self.counter & 7) as usize],
            14 => 1 + STEPS[low][(self.counter & 7) as usize],
            _ => 2,
        }
    }

    /// The rate with its key scale: 4 × R + Rks, 0 for an R of 0 [D (III-1-2), Table III-2].
    fn rate(&self, ch: usize, op: &Operator, r: u8) -> u32 {
        if r == 0 {
            return 0;
        }
        let ks = (self.block(ch) << 1 | self.fnum(ch) >> 8) >> if op.ksr { 0 } else { 2 };
        (4 * r as u32 + ks).min(63)
    }

    /// One sample: every slot's output, then its envelope and its phase moved on (Endou_Opll.md §3.7). Returns the
    /// melody's and the rhythm's sums, each channel's carrier in nine bits; the rhythm instruments sound twice [D (III-4)].
    pub fn sample(&mut self) -> (i32, i32) {
        let rhythm = self.rhythm();
        let am = (self.am >> 3) as u32;
        let (mut melody, mut drums) = (0i32, 0i32);
        let mut ops = [Operator::of(&[0; 8], 0); 18];
        for ch in 0..9 {
            let (m, c) = (2 * ch, 2 * ch + 1);
            let patch = *self.patch(m);
            let (mo, co) = (Operator::of(&patch, 0), Operator::of(&patch, 1));
            (ops[m], ops[c]) = (mo, co);
            let att_m = self.attenuation(m, &patch, &mo, am);
            let att_c = self.attenuation(c, &patch, &co, am);
            if rhythm && ch >= BD {
                drums += 2 * self.rhythm_channel(ch, &patch, att_m, att_c, &mo, &co);
            } else {
                let fb = patch[3] & 7;
                let s = &self.slots[m];
                let feedback = if fb == 0 { 0 } else { (s.out[0] as i32 + s.out[1] as i32) >> (9 - fb) };
                let mod_out = operator((s.phase >> 9) as i32 + feedback, att_m, mo.half);
                // The carrier takes the modulator's output of the sample before [M §3.7].
                let sm = &mut self.slots[m];
                let used = sm.out[0];
                sm.out = [mod_out, sm.out[0]];
                let car = operator((self.slots[c].phase >> 9) as i32 + used as i32, att_c, co.half);
                self.slots[c].out = [car, self.slots[c].out[0]];
                melody += car as i32 >> 3;
            }
        }
        self.envelopes(&ops);
        self.phases(&ops);
        self.lfo();
        (melody, drums)
    }

    /// A slot's whole attenuation: its envelope, its volume or TL, its key scale and the tremolo, in 0.375 dB up to
    /// silence; past `QUIET` an attenuation no operator sounds at.
    fn attenuation(&self, slot: usize, patch: &[u8; 8], op: &Operator, am: u32) -> u32 {
        let level = self.slots[slot].level;
        if level >= QUIET {
            return MUTE;
        }
        let a = level as u32 + self.volume(slot, patch) + self.ksl(slot / 2, op.ksl) + if op.am { am } else { 0 };
        a.min(SILENT as u32)
    }

    /// The rhythm's three channels (Endou_Opll.md §3.8): the bass drum as a melody channel; the high hat, snare drum
    /// and top cymbal on phases made of the noise and of the high hat's and top cymbal's phase bits; the tom-tom a
    /// plain sine.
    fn rhythm_channel(&mut self, ch: usize, patch: &[u8; 8], att_m: u32, att_c: u32, mo: &Operator, co: &Operator) -> i32 {
        let (m, c) = (2 * ch, 2 * ch + 1);
        match ch {
            BD => {
                let fb = patch[3] & 7;
                let s = &self.slots[m];
                let feedback = if fb == 0 { 0 } else { (s.out[0] as i32 + s.out[1] as i32) >> (9 - fb) };
                let mod_out = operator((s.phase >> 9) as i32 + feedback, att_m, mo.half);
                let used = self.slots[m].out[0];
                self.slots[m].out = [mod_out, used];
                let car = operator((self.slots[c].phase >> 9) as i32 + used as i32, att_c, co.half);
                car as i32 >> 3
            }
            HH_SD => {
                let noise = self.noise & 1 != 0;
                let hh = self.cymbal_bit();
                let hh_phase = if hh { 0x200 | if noise { 0xD0 } else { 0x34 } } else if noise { 0x34 } else { 0xD0 };
                let hh_out = operator(hh_phase, att_m, mo.half) as i32 >> 3;
                let sd_bit = (self.slots[m].phase >> 17) & 1 != 0;
                let sd_phase = if sd_bit { 0x200 } else { 0x100 } ^ if noise { 0x100 } else { 0 };
                let sd_out = operator(sd_phase as i32, att_c, co.half) as i32 >> 3;
                hh_out + sd_out
            }
            _ => {
                let tom = operator((self.slots[m].phase >> 9) as i32, att_m, mo.half) as i32 >> 3;
                let tc_phase = if self.cymbal_bit() { 0x300 } else { 0x100 };
                let tc = operator(tc_phase, att_c, co.half) as i32 >> 3;
                tom + tc
            }
        }
    }

    /// The bit the high hat and top cymbal share: bits of the high hat's slot phase and of the top cymbal's mixed [A
    /// §3.8].
    fn cymbal_bit(&self) -> bool {
        let hh = self.slots[2 * HH_SD].phase >> 9;
        let tc = self.slots[2 * TOM_TC + 1].phase >> 9;
        let a = ((hh >> 2) ^ (hh >> 7)) & 1 != 0 || (hh >> 3) & 1 != 0;
        let b = ((tc >> 3) ^ (tc >> 5)) & 1 != 0;
        a | b
    }

    /// Every slot's envelope: a key's edge starts its damp or its release, then its stage steps at its rate [M §3.5].
    fn envelopes(&mut self, ops: &[Operator; 18]) {
        for slot in 0..18 {
            let ch = slot / 2;
            let op = ops[slot];
            let keyed = self.keyed(slot);
            let s = self.slots[slot];
            if keyed && !s.key {
                self.slots[slot].stage = Stage::Damp;
            } else if !keyed && s.key {
                self.slots[slot].stage = Stage::Release;
            }
            self.slots[slot].key = keyed;
            let sus = self.regs[0x20 + ch] & 0x20 != 0;
            let s = self.slots[slot];
            let r = match s.stage {
                Stage::Damp => 12,
                Stage::Attack => op.ar,
                Stage::Decay => op.dr,
                Stage::Sustain => if op.sustained { 0 } else { op.rr },
                Stage::Release if !s.key => if sus { 5 } else if op.sustained { op.rr } else { 7 },
                Stage::Release => op.rr,
            };
            let rate = if s.stage == Stage::Damp { 48 } else { self.rate(ch, &op, r) };
            let inc = self.step(rate);
            let mut level = s.level;
            let mut stage = s.stage;
            match stage {
                Stage::Damp => {
                    level = level.saturating_add(inc).min(SILENT);
                    if level == SILENT {
                        // The attack begins with the phase restarted [M §3.5].
                        stage = Stage::Attack;
                        self.slots[slot].phase = 0;
                    }
                }
                Stage::Attack => {
                    if rate >= 60 {
                        level = 0;
                    } else if inc > 0 && level > 0 {
                        let d = ((level as u32 * inc as u32) >> 2) + 1;
                        level = level.saturating_sub(d as u8);
                    }
                    if level == 0 {
                        stage = Stage::Decay;
                    }
                }
                Stage::Decay => {
                    if level >= op.sl << 3 {
                        stage = Stage::Sustain;
                    } else {
                        level = level.saturating_add(inc).min(SILENT);
                    }
                }
                Stage::Sustain | Stage::Release => level = level.saturating_add(inc).min(SILENT),
            }
            self.slots[slot].level = level;
            self.slots[slot].stage = stage;
        }
        self.counter = self.counter.wrapping_add(1);
    }

    fn phases(&mut self, ops: &[Operator; 18]) {
        for slot in 0..18 {
            let ch = slot / 2;
            let op = ops[slot];
            let inc = self.increment(ch, &op);
            let s = &mut self.slots[slot];
            s.phase = (s.phase + inc) & 0x7FFFF;
        }
        // The noise: a 23-bit register shifted each sample [A §3.8].
        let bit = (self.noise ^ (self.noise >> 14)) & 1;
        self.noise = (self.noise >> 1) | bit << 22;
    }

    /// The tremolo: its triangle steps every 64 samples between 0 and 104, each end held twice [M §3.6].
    fn lfo(&mut self) {
        if self.counter & 63 != 0 {
            return;
        }
        if self.am_down {
            if self.am == 0 {
                self.am_down = false;
            } else {
                self.am -= 1;
            }
        } else if self.am == 104 {
            self.am_down = true;
        } else {
            self.am += 1;
        }
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        w.u8("OpllAddress", self.address);
        w.bytes("OpllRegisters", &self.regs);
        w.u32s("OpllPhase", &self.slots.map(|s| s.phase));
        w.bytes("OpllLevel", &self.slots.map(|s| s.level));
        w.bytes("OpllStage", &self.slots.map(|s| s.stage.byte() | (s.key as u8) << 7));
        let out: Vec<u16> = self.slots.iter().flat_map(|s| s.out).map(|v| v as u16).collect();
        w.u16s("OpllOut", &out);
        w.u32("OpllCounter", self.counter);
        w.u8("OpllAm", self.am | (self.am_down as u8) << 7);
        w.u32("OpllNoise", self.noise);
    }

    /// Reads a state; `false` for one no chip could be in.
    pub fn read_state(&mut self, r: &mut StateReader) -> Result<bool, Truncated> {
        let address = r.u8()?;
        let mut regs = [0u8; REGISTERS];
        r.bytes(&mut regs)?;
        let mut phase = [0u32; 18];
        r.u32s(&mut phase)?;
        let mut level = [0u8; 18];
        r.bytes(&mut level)?;
        let mut stage = [0u8; 18];
        r.bytes(&mut stage)?;
        let mut out = [0u16; 36];
        r.u16s(&mut out)?;
        let counter = r.u32()?;
        let am = r.u8()?;
        let noise = r.u32()?;
        if level.iter().any(|&l| l > SILENT) || phase.iter().any(|&p| p > 0x7FFFF) || am & 0x7F > 104 || stage.iter().any(|&s| Stage::of_byte(s & 0x7F).is_none()) {
            return Ok(false);
        }
        self.address = address;
        self.regs = regs;
        for i in 0..18 {
            self.slots[i] = Slot { phase: phase[i], level: level[i], stage: Stage::of_byte(stage[i] & 0x7F).expect("checked"), key: stage[i] & 0x80 != 0, out: [out[2 * i] as i16, out[2 * i + 1] as i16] };
        }
        self.counter = counter;
        self.am = am & 0x7F;
        self.am_down = am & 0x80 != 0;
        self.noise = noise;
        Ok(true)
    }
}

/// An operator's output for a ten-bit phase (wrapped) and an attenuation in 0.375 dB: the log-sine of the phase's
/// quarter, the attenuation added in the same log units, and the exponent back to a 12-bit magnitude with its sign
/// [D "OPLx decapsulated"; Endou_Opll.md §3.3]. The half-wave sine is silent on its negative half [D (III-1-3)].
pub fn operator(phase: i32, attenuation: u32, half: bool) -> i16 {
    let p = (phase & 0x3FF) as usize;
    let negative = p & 0x200 != 0;
    if negative && half {
        return 0;
    }
    let q = if p & 0x100 != 0 { !p & 0xFF } else { p & 0xFF };
    let log = LOGSIN[q] as u32 + (attenuation << 4);
    if log >= 0x1000 {
        return 0;
    }
    let m = (((EXP[(!log & 0xFF) as usize] as u32 + 1024) << 1) >> (log >> 8)) as i16;
    if negative { -m } else { m }
}

#[cfg(test)]
mod tests;

