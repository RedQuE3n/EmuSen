//! One voice (psx-spx "SPU ADPCM Samples", "SPU ADPCM Pitch", "SPU Volume and ADSR Generator"): SPU-ADPCM decoded from
//! sound RAM a word at a time as the pitch counter reaches it, the 4-point Gaussian interpolation, the ADSR envelope and
//! the two volumes with their sweeps. Diamond_SPU.md §3 and §4 record each rule and its source; BlackMoon_SPU.md §2
//! what a core gives a voice.

use crate::Touch;
use crate::envelope::{Envelope, Rate};
use crate::mask;
use crate::tables::{GAUSS, NEG, POS};

/// The samples of one 16-byte block.
pub const BLOCK_SAMPLES: u32 = 28;

/// The highest step the pitch counter takes a sample (psx-spx "Pitch Counter").
pub const MAX_STEP: u32 = 0x4000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    Attack,
    Decay,
    Sustain,
    #[default]
    Release,
}

impl Phase {
    pub fn code(self) -> u8 {
        self as u8
    }

    pub fn of(code: u8) -> Phase {
        [Phase::Attack, Phase::Decay, Phase::Sustain, Phase::Release][code as usize & 3]
    }
}

#[derive(Clone, Debug, Default)]
pub struct Voice {
    /// The left and right volume registers, the pitch, and ADSR1 and ADSR2 as written.
    pub volume: [u16; 2],
    pub pitch: u16,
    pub adsr: [u16; 2],
    /// The start address a key on plays from, and the loop address the loop end flag jumps to, in 8-byte units.
    pub start: u32,
    pub repeat: u32,
    /// The block playing, in bytes, the pitch counter, and how many of the block's samples are decoded.
    pub address: u32,
    pub counter: u32,
    pub decoded: u8,
    /// The previous block's last three samples, then this block's 28.
    pub samples: [i16; 31],
    /// The decoder's two samples before, newest first.
    pub history: [i16; 2],
    pub envelope: Envelope,
    pub phase: Phase,
    /// The two volumes as the sweeps leave them.
    pub current: [Envelope; 2],
    /// The voice's output after its envelope, before its volumes (what the next voice's pitch modulation takes).
    pub out: i16,
}

/// The pitch counter's step for one sample: `pitch`, or with pitch modulation by the voice before (whose output this
/// sample is `modulator`) `pitch` sign-extended times the output plus 8000h, shifted right by 15 and masked to 16 bits,
/// psx-spx's "hardware glitch" included; then at most 4000h (psx-spx "Pitch Counter", Diamond_SPU.md §3.3).
#[inline]
pub fn step(pitch: u16, modulator: Option<i16>) -> u32 {
    let mut step = pitch as u32;
    if let Some(previous) = modulator {
        let factor = previous as i32 + 0x8000;
        step = ((step as i16 as i64 * factor as i64) >> 15) as u32 & 0xFFFF;
    }
    step.min(MAX_STEP)
}

impl Voice {
    /// Key on: the block at the start address from its start, the decoder's and the interpolation's history cleared,
    /// the envelope at zero in its attack (Diamond_SPU.md §3.5).
    pub fn key_on(&mut self, ram: &[u8]) {
        self.address = (self.start * 8) & mask(ram) as u32;
        self.counter = 0;
        self.decoded = 0;
        self.samples = [0; 31];
        self.history = [0; 2];
        self.envelope = Envelope::default();
        self.phase = Phase::Attack;
        self.enter(ram);
    }

    pub fn key_off(&mut self) {
        self.phase = Phase::Release;
    }

    /// A block's loop start flag copies its address to the loop address as the block begins.
    fn enter(&mut self, ram: &[u8]) {
        if ram[(self.address + 1) as usize & mask(ram)] & 4 != 0 {
            self.repeat = self.address >> 3;
        }
    }

    /// The four samples of the block's word `w`, from the header's shift and filter as they are now.
    pub fn decode_word(&mut self, ram: &[u8], w: usize) {
        let m = mask(ram);
        let base = self.address as usize;
        let header = ram[base & m];
        let shift = match header & 0xF {
            s @ 0..=12 => s,
            _ => 9,
        } as u32;
        let filter = (header >> 4 & 7) as usize;
        let (f0, f1) = if filter < 5 { (POS[filter], NEG[filter]) } else { (0, 0) };
        let at = (base + 2 + 2 * w) & m & !1;
        let word = u16::from_le_bytes([ram[at], ram[at + 1]]);
        for k in 0..4 {
            let t = (((word >> (4 * k)) & 0xF) as i32) << 28 >> 16;
            let s = ((t >> shift) + (self.history[0] as i32 * f0 >> 6) + (self.history[1] as i32 * f1 >> 6)).clamp(-0x8000, 0x7FFF) as i16;
            self.history = [s, self.history[0]];
            self.samples[3 + 4 * w + k] = s;
        }
        self.decoded = (4 * w + 4) as u8;
    }

    /// The envelope's rate in its phase (psx-spx "ADSR1", "ADSR2").
    fn rate(&self) -> Rate {
        let [a1, a2] = self.adsr;
        match self.phase {
            Phase::Attack => Rate { shift: (a1 >> 10 & 0x1F) as u8, step: (a1 >> 8 & 3) as u8, decreasing: false, exponential: a1 & 0x8000 != 0, negative: false },
            Phase::Decay => Rate { shift: (a1 >> 4 & 0xF) as u8, step: 0, decreasing: true, exponential: true, negative: false },
            Phase::Sustain => Rate { shift: (a2 >> 8 & 0x1F) as u8, step: (a2 >> 6 & 3) as u8, decreasing: a2 & 0x4000 != 0, exponential: a2 & 0x8000 != 0, negative: false },
            Phase::Release => Rate { shift: (a2 & 0x1F) as u8, step: 0, decreasing: true, exponential: a2 & 0x20 != 0, negative: false },
        }
    }

    /// The envelope's cycle, and its move to the next phase once its level is reached.
    fn envelope_tick(&mut self) {
        self.envelope.tick(self.rate());
        let level = self.envelope.level as i32;
        match self.phase {
            Phase::Attack if level >= 0x7FFF => self.phase = Phase::Decay,
            Phase::Decay if level <= ((self.adsr[0] & 0xF) as i32 + 1) * 0x800 => self.phase = Phase::Sustain,
            _ => {}
        }
    }

    /// One sample: the sample at the counter, interpolated (or the noise), through the envelope as it stood, which then
    /// takes its step (Diamond's D-36); the counter advanced by `step`, past the block's end to the next or to the loop.
    /// Returns whether a loop end flag was met.
    pub fn tick<W: Touch>(&mut self, ram: &[u8], step: u32, noise: Option<i16>, watch: &mut W) -> bool {
        let m = mask(ram) as u32;
        let i = (self.counter >> 12) as usize;
        let w = i / 4;
        if self.decoded as usize <= i {
            for k in self.decoded as usize / 4..=w {
                self.decode_word(ram, k);
            }
        }
        watch.touch(self.address);
        watch.touch(self.address + 2 + 2 * w as u32);
        let sample = match noise {
            Some(n) => n as i32,
            None => {
                let g = ((self.counter >> 4) & 0xFF) as usize;
                let s = &self.samples[i..i + 4];
                (GAUSS[0xFF - g] * s[0] as i32 + GAUSS[0x1FF - g] * s[1] as i32 + GAUSS[0x100 + g] * s[2] as i32 + GAUSS[g] * s[3] as i32) >> 15
            }
        };
        self.out = (sample * self.envelope.level as i32 >> 15).clamp(-0x8000, 0x7FFF) as i16;
        self.envelope_tick();
        self.counter += step;
        if self.counter < BLOCK_SAMPLES << 12 {
            return false;
        }
        self.counter -= BLOCK_SAMPLES << 12;
        if (self.decoded as u32) < BLOCK_SAMPLES {
            for k in self.decoded as usize / 4..7 {
                self.decode_word(ram, k);
            }
        }
        let flags = ram[((self.address + 1) & m) as usize];
        let end = flags & 1 != 0;
        if end {
            self.address = (self.repeat * 8) & m;
            if flags & 2 == 0 {
                self.phase = Phase::Release;
                self.envelope.level = 0;
            }
        } else {
            self.address = (self.address + 16) & m;
        }
        self.samples.copy_within(28..31, 0);
        self.decoded = 0;
        self.enter(ram);
        end
    }

    /// The two volumes' cycle, direct or swept (psx-spx "Sweep Volume Mode"), and the output through them.
    #[inline]
    pub fn volumes(&mut self) -> [i32; 2] {
        let out = self.out as i32;
        let mut lr = [0; 2];
        for (side, v) in lr.iter_mut().enumerate() {
            sweep(&mut self.current[side], self.volume[side]);
            *v = out * self.current[side].level as i32 >> 15;
        }
        lr
    }
}

/// A volume register's cycle: bit 15 clear sets the level to bits 0-14 doubled, set sweeps it.
#[inline]
pub fn sweep(current: &mut Envelope, reg: u16) {
    if reg & 0x8000 == 0 {
        current.level = (reg << 1) as i16;
    } else {
        current.tick(Rate::sweep(reg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each filter's prediction is the two samples before, weighted by psx-spx's "Pos/neg Tables" over 64, each product
    /// shifted apart (Diamond's D-34): a block whose first nibble is 7 and the rest 0 decays by the filter's own weights.
    #[test]
    fn each_filter_predicts_from_psx_spx_weights() {
        let weights = [(0, 0), (60, 0), (115, -52), (98, -55), (122, -60)];
        for (f, (w0, w1)) in weights.into_iter().enumerate() {
            let mut ram = [0u8; 64];
            ram[0] = (f as u8) << 4 | 2;
            ram[2] = 7;
            let mut v = Voice::default();
            v.key_on(&ram);
            v.decode_word(&ram, 0);
            let first = (7i32 << 28 >> 16) >> 2;
            let mut want = [first, 0, 0, 0];
            want[1] = first * w0 >> 6;
            want[2] = (want[1] * w0 >> 6) + (first * w1 >> 6);
            want[3] = (want[2] * w0 >> 6) + (want[1] * w1 >> 6);
            assert_eq!(v.samples[3..7].iter().map(|&s| s as i32).collect::<std::vec::Vec<_>>(), want, "filter {f}");
        }
    }
}
