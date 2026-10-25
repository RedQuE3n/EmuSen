//! One of the SPU2's two cores: 24 voices of `blackmoon-spu`, its registers, its noise, key on and off, its mix of
//! voices, sound data input and external input, dry and wet by side, its reverb in its own effects area, and its
//! output through its master volume (Sapphire_Native.md §32.4). Addresses are halfword addresses, 20 bits, as the
//! SPU2's registers hold them.

use blackmoon_spu::envelope::Envelope;
use blackmoon_spu::noise::Noise;
use blackmoon_spu::reverb::{Area, Params, Reverb};
use blackmoon_spu::voice::{self, Voice, sweep};

use super::adma::Adma;
use super::regs::{self, attr, vol};
use super::Watch;

pub const VOICES: usize = 24;
/// The halfword addresses' width: 2 MiB of sound RAM [R ps2tek "Memory Map"].
pub const ADDRESS_MASK: u32 = 0xF_FFFF;

#[derive(Clone)]
pub struct Core {
    pub index: usize,
    /// The block's halfwords from 000h to 3FFh as written; the voices' live registers are in `voices`.
    pub regs: [u16; 0x200],
    /// The core's volumes and reverb coefficients from 760h + 28h × core, as written.
    pub vol: [u16; 20],
    pub voices: [Voice; VOICES],
    /// Each voice's start address as written, in halfwords; the voice takes it in 8-byte units (D-268).
    pub ssa: [u32; VOICES],
    /// The voices whose loop end was met since their key on, and the key ons and offs written since the last sample.
    pub endx: u32,
    pub key_on: u32,
    pub key_off: u32,
    pub noise: Noise,
    pub main: [Envelope; 2],
    pub reverb: Reverb,
    /// IRQ address's flag, set on a hit while armed (Sapphire_Disputes.md D-265).
    pub irq: bool,
    /// The transfer's address, in halfwords.
    pub tsa: u32,
    pub adma: Adma,
    /// The input volumes as their sweeps leave them: AVOL left and right, then BVOL.
    pub inputs: [Envelope; 4],
    /// This sample's voice sums, dry left and right, then wet, as the output areas record them.
    pub sums: [i32; 4],
    /// The core's output this sample, after its master volume; core 1 takes core 0's.
    pub out: [i32; 2],
}

fn pair(regs: &[u16; 0x200], at: u32) -> u32 {
    let i = at as usize / 2;
    (regs[i] as u32 & 0xF) << 16 | regs[i + 1] as u32
}

fn mask24(regs: &[u16; 0x200], at: u32) -> u32 {
    regs[at as usize / 2] as u32 | (regs[at as usize / 2 + 1] as u32 & 0xFF) << 16
}

impl Core {
    pub fn new(index: usize) -> Core {
        Core {
            index,
            regs: [0; 0x200],
            vol: [0; 20],
            voices: core::array::from_fn(|_| Voice::default()),
            ssa: [0; VOICES],
            endx: 0,
            key_on: 0,
            key_off: 0,
            noise: Noise::default(),
            main: [Envelope::default(); 2],
            reverb: Reverb::default(),
            irq: false,
            tsa: 0,
            adma: Adma::default(),
            inputs: [Envelope::default(); 4],
            sums: [0; 4],
            out: [0; 2],
        }
    }

    pub fn attr(&self) -> u16 {
        self.regs[regs::ATTR as usize / 2]
    }

    pub fn irq_address(&self) -> u32 {
        pair(&self.regs, regs::IRQAH)
    }

    /// Whether a hit on the IRQ address now sets the flag: ATTR's IRQ enable, the flag not yet up.
    pub fn armed(&self) -> bool {
        self.attr() & attr::IRQ != 0 && !self.irq
    }

    pub fn adma_on(&self) -> bool {
        self.regs[regs::ADMA as usize / 2] & regs::adma_bit(self.index) != 0
    }

    // --- the registers, `r` the byte offset in the core's block ---

    pub fn read(&self, r: u32) -> u16 {
        if r < 0x180 {
            let v = &self.voices[r as usize >> 4];
            return match (r as usize >> 1) & 7 {
                regs::voice::VOLL => v.volume[0],
                regs::voice::VOLR => v.volume[1],
                regs::voice::PITCH => v.pitch,
                regs::voice::ADSR1 => v.adsr[0],
                regs::voice::ADSR2 => v.adsr[1],
                regs::voice::ENVX => v.envelope.level as u16,
                regs::voice::VOLXL => v.current[0].level as u16,
                _ => v.current[1].level as u16,
            };
        }
        if (regs::ADDRESSES..regs::ADDRESSES_END).contains(&r) {
            let k = r - regs::ADDRESSES;
            let v = &self.voices[k as usize / 12];
            let hw = match k % 12 / 4 {
                0 => self.ssa[k as usize / 12],
                1 => v.repeat * 4,
                _ => v.address / 2 + 1 + (v.counter >> 12).min(voice::BLOCK_SAMPLES - 1) / 4,
            };
            return if k.is_multiple_of(4) { (hw >> 16) as u16 } else { hw as u16 };
        }
        match r {
            regs::ENDX => self.endx as u16,
            0x342 => (self.endx >> 16) as u16,
            regs::STATX => self.status(),
            regs::TSAH => (self.tsa >> 16) as u16,
            regs::TSAL => self.tsa as u16,
            _ => self.regs[r as usize / 2],
        }
    }

    /// STATX: zero, as the referee reads it at rest and with the IRQ flag up (Sapphire_Disputes.md D-266).
    pub fn status(&self) -> u16 {
        0
    }

    pub fn write(&mut self, r: u32, value: u16) {
        if r < 0x180 {
            let v = &mut self.voices[r as usize >> 4];
            match (r as usize >> 1) & 7 {
                regs::voice::VOLL => v.volume[0] = value,
                regs::voice::VOLR => v.volume[1] = value,
                regs::voice::PITCH => v.pitch = value,
                regs::voice::ADSR1 => v.adsr[0] = value,
                regs::voice::ADSR2 => v.adsr[1] = value,
                regs::voice::ENVX => v.envelope.level = value as i16,
                _ => {}
            }
            return;
        }
        if (regs::ADDRESSES..regs::ADDRESSES_END).contains(&r) {
            let k = r - regs::ADDRESSES;
            let n = k as usize / 12;
            let v = &mut self.voices[n];
            let high = k.is_multiple_of(4);
            let set = |hw: u32| if high { (value as u32 & 0xF) << 16 | (hw & 0xFFFF) } else { (hw & 0xF_0000) | value as u32 };
            match k % 12 / 4 {
                0 => {
                    self.ssa[n] = set(self.ssa[n]);
                    v.start = self.ssa[n] >> 2;
                }
                1 => v.repeat = set(v.repeat * 4) >> 2,
                _ => v.address = set(v.address / 2) * 2,
            }
            return;
        }
        match r {
            regs::KON => self.key_on |= value as u32,
            0x1A2 => self.key_on |= (value as u32 & 0xFF) << 16,
            regs::KOFF => self.key_off |= value as u32,
            0x1A6 => self.key_off |= (value as u32 & 0xFF) << 16,
            regs::ENDX | 0x342 | regs::STATX => return,
            regs::TSAH => self.tsa = (value as u32 & 0xF) << 16 | (self.tsa & 0xFFFF),
            regs::TSAL => self.tsa = (self.tsa & 0xF_0000) | value as u32,
            regs::ATTR => {
                if value & attr::IRQ == 0 {
                    self.irq = false;
                }
                if (value ^ self.attr()) & 0x3F00 != 0 {
                    self.noise.timer = 0;
                }
            }
            regs::ESAH | 0x2E2 => {
                self.regs[r as usize / 2] = value;
                self.reverb.address = pair(&self.regs, regs::ESAH) * 2;
                return;
            }
            regs::ADMA => {
                let was = self.adma_on();
                self.regs[r as usize / 2] = value;
                if self.adma_on() && !was {
                    self.adma.start();
                }
                return;
            }
            _ => {}
        }
        self.regs[r as usize / 2] = value;
    }

    // --- the sample ---

    fn bits(&self, at: u32) -> u32 {
        mask24(&self.regs, at)
    }

    /// The reverb's offsets and coefficients this sample: halfword offsets to bytes, the volumes sign-extended.
    fn reverb_params(&self) -> Params {
        let m = |k: u32| pair(&self.regs, regs::REVERB + 4 * k) as i32 * 2;
        let two = |k: u32| [m(k), m(k + 1)];
        let s = |i: usize| self.vol[i] as i16 as i32;
        Params {
            input: [s(vol::IN), s(vol::IN + 1)],
            iir: s(vol::IIR),
            wall: s(vol::WALL),
            comb: [s(vol::COMB), s(vol::COMB + 1), s(vol::COMB + 2), s(vol::COMB + 3)],
            apf: [s(vol::APF), s(vol::APF + 1)],
            output: [s(vol::EVOL), s(vol::EVOL + 1)],
            d_apf: two(0),
            same: two(2),
            comb_at: [two(4), two(6), two(12), two(14)],
            d_same: two(8),
            diff: two(10),
            d_diff: two(16),
            apf_at: [two(18), two(20)],
        }
    }

    /// The effects area in bytes: from ESA to the end of EEA's 64 Ki halfwords, at least one halfword [A; D-264].
    fn area(&self) -> Area {
        let start = pair(&self.regs, regs::ESAH) * 2;
        let end = ((self.regs[regs::EEA as usize / 2] as u32 & 0xF) << 16 | 0xFFFF) * 2 + 2;
        Area { start, end: end.max(start + 2) }
    }

    /// One sample: key offs and ons applied, the noise, the sound data input, the 24 voices, the external input `ext`,
    /// the reverb and the mix through the master volume; the output, also kept in `out`.
    pub fn tick(&mut self, ram: &mut [u8], watch: &mut Watch, ext: [i32; 2]) -> [i32; 2] {
        let a = self.attr();
        let (off, on) = (core::mem::take(&mut self.key_off), core::mem::take(&mut self.key_on));
        for n in 0..VOICES {
            if off & 1 << n != 0 {
                self.voices[n].key_off();
            }
            if on & 1 << n != 0 {
                self.voices[n].key_on(ram);
            }
        }
        self.endx &= !on;
        self.noise.tick(a >> 10, a >> 8);

        let (l, r) = self.adma.take(self.index);
        let half = |ram: &[u8], hw: u32, watch: &mut Watch| {
            let b = (hw as usize * 2) & (ram.len() - 2);
            blackmoon_spu::Touch::touch(watch, b as u32);
            i16::from_le_bytes([ram[b], ram[b + 1]]) as i32
        };
        let input = [half(ram, l, watch), half(ram, r, watch)];

        let (pmon, non) = (self.bits(regs::PMON), self.bits(regs::NON));
        let mix = [self.bits(regs::VMIXL), self.bits(regs::VMIXR)];
        let wet_mix = [self.bits(regs::VMIXEL), self.bits(regs::VMIXER)];
        let (mut dry, mut wet) = ([0i32; 2], [0i32; 2]);
        let mut previous = 0i16;
        for n in 0..VOICES {
            let v = &mut self.voices[n];
            let step = voice::step(v.pitch, (n > 0 && pmon & 1 << n != 0).then_some(previous));
            let noise = (non & 1 << n != 0).then_some(self.noise.level);
            if v.tick(ram, step, noise, watch) {
                self.endx |= 1 << n;
            }
            previous = v.out;
            let lr = v.volumes();
            for s in 0..2 {
                if mix[s] & 1 << n != 0 {
                    dry[s] += lr[s];
                }
                if wet_mix[s] & 1 << n != 0 {
                    wet[s] += lr[s];
                }
            }
        }

        self.sums = [dry[0].clamp(-0x8000, 0x7FFF), dry[1].clamp(-0x8000, 0x7FFF), wet[0].clamp(-0x8000, 0x7FFF), wet[1].clamp(-0x8000, 0x7FFF)];

        // The external input through AVOL, the sound data input through BVOL, each a volume register.
        let mut through = [[0i32; 2]; 2];
        for (k, (x, at)) in [(ext, vol::AVOL), (input, vol::BVOL)].into_iter().enumerate() {
            for s in 0..2 {
                sweep(&mut self.inputs[2 * k + s], self.vol[at + s]);
                through[k][s] = (x[s] * self.inputs[2 * k + s].level as i32) >> 15;
            }
        }
        // MMIX's gates, a right then a left bit each: the external input wet (0-1) and dry (2-3), the sound data input
        // wet (4-5) and dry (6-7), the voices wet (8-9) and dry (10-11).
        let mmix = self.regs[regs::MMIX as usize / 2] as u32;
        let gate = |bit: u32, s: usize, x: i32| if mmix >> (bit + 1 - s as u32) & 1 != 0 { x } else { 0 };
        let mut to_reverb = [0i32; 2];
        let mut out = [0i32; 2];
        for s in 0..2 {
            to_reverb[s] = gate(0, s, through[0][s]) + gate(4, s, through[1][s]) + gate(8, s, wet[s]);
            out[s] = gate(2, s, through[0][s]) + gate(6, s, through[1][s]) + gate(10, s, dry[s]);
        }

        let params = self.reverb_params();
        let area = self.area();
        let rev = self.reverb.tick(ram, watch, &params, area, a & attr::REVERB != 0, to_reverb);
        for s in 0..2 {
            let m = (out[s] + rev[s]).clamp(-0x8000, 0x7FFF);
            sweep(&mut self.main[s], self.vol[vol::MVOL + s]);
            self.vol[vol::MVOLX + s] = self.main[s].level as u16;
            self.out[s] = ((m * self.main[s].level as i32) >> 15).clamp(-0x8000, 0x7FFF);
        }
        self.out
    }
}
