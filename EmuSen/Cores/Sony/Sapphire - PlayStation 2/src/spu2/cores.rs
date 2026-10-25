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

    /// STATX as psx-spx gives the PlayStation SPU's ("SPUSTAT"): ATTR's bits 5-0 (applied at once [C]), the IRQ flag in
    /// bit 6 and ATTR's bit 5 again in bit 7; the transfers being immediate, the busy and request bits read 0. The
    /// player's BIOS waits on it after a transfer; PCSX2 reads it as 0 throughout (Sapphire_Disputes.md D-266, D-349).
    pub fn status(&self) -> u16 {
        let a = self.attr();
        a & 0x3F | u16::from(self.irq) << 6 | (a >> 5 & 1) << 7
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

fn halves(e: &[Envelope]) -> impl Iterator<Item = u16> + '_ {
    e.iter().flat_map(|e| [e.level as u16, e.counter])
}

fn envelope(level: u16, counter: u16) -> Envelope {
    Envelope { level: level as i16, counter }
}

/// A core's block of `SPH2` (Sapphire_Native.md §38.2): its registers and volumes as written, each voice's registers,
/// decoder, counter, envelope and output, the key ons and offs, the noise, the main volumes' sweeps, the reverb's
/// address and filters, the IRQ flag, the transfer address, AutoDMA, the input sweeps and this sample's sums and output.
impl emusen_native::State for Core {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        w.u16s("Registers", &self.regs);
        w.u16s("Volumes", &self.vol);
        let v = &self.voices;
        let mut regs = [0u16; VOICES * 5];
        let mut words = [0u32; VOICES * 4];
        let mut samples = [0u16; VOICES * 34];
        let mut envelopes = [0u16; VOICES * 6];
        for (n, v) in v.iter().enumerate() {
            regs[5 * n..5 * n + 5].copy_from_slice(&[v.volume[0], v.volume[1], v.pitch, v.adsr[0], v.adsr[1]]);
            words[4 * n..4 * n + 4].copy_from_slice(&[v.start, v.repeat, v.address, v.counter]);
            for (d, s) in samples[34 * n..34 * n + 34].iter_mut().zip(v.samples.iter().chain(&v.history).chain([&v.out])) {
                *d = *s as u16;
            }
            for (d, s) in envelopes[6 * n..6 * n + 6].iter_mut().zip(halves(&[v.envelope, v.current[0], v.current[1]])) {
                *d = s;
            }
        }
        w.u16s("VoiceRegisters", &regs);
        w.u32s("VoiceWords", &words);
        w.bytes("VoiceDecoded", &v.each_ref().map(|v| v.decoded));
        w.bytes("VoicePhases", &v.each_ref().map(|v| v.phase.code()));
        w.u16s("VoiceSamples", &samples);
        w.u16s("VoiceEnvelopes", &envelopes);
        w.u32s("VoiceStarts", &self.ssa);
        w.u32s("Keys", &[self.endx, self.key_on, self.key_off, self.tsa]);
        w.i32s("Noise", &[i32::from(self.noise.level), self.noise.timer]);
        let mut env = [0u16; 12];
        for (d, s) in env.iter_mut().zip(halves(&self.main).chain(halves(&self.inputs))) {
            *d = s;
        }
        w.u16s("Sweeps", &env);
        let r = &self.reverb;
        w.u32("ReverbAddress", r.address);
        let mut ring = [0u16; 4 * blackmoon_spu::reverb::RING];
        for (d, s) in ring.iter_mut().zip(r.input.as_flattened().iter().chain(r.output.as_flattened())) {
            *d = *s as u16;
        }
        w.u16s("ReverbRings", &ring);
        w.bytes("ReverbPhase", &[r.at, u8::from(r.right)]);
        w.bools("Flags", &[self.irq, self.adma.filled[0], self.adma.filled[1]]);
        w.u32s("Adma", &[self.adma.play, self.adma.buffer, self.adma.words]);
        w.i32s("Sums", &self.sums);
        w.i32s("Out", &self.out);
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        r.u16s(&mut self.regs)?;
        r.u16s(&mut self.vol)?;
        let mut regs = [0u16; VOICES * 5];
        let mut words = [0u32; VOICES * 4];
        let mut decoded = [0u8; VOICES];
        let mut phases = [0u8; VOICES];
        let mut samples = [0u16; VOICES * 34];
        let mut envelopes = [0u16; VOICES * 6];
        r.u16s(&mut regs)?;
        r.u32s(&mut words)?;
        r.bytes(&mut decoded)?;
        r.bytes(&mut phases)?;
        r.u16s(&mut samples)?;
        r.u16s(&mut envelopes)?;
        for (n, v) in self.voices.iter_mut().enumerate() {
            let g = &regs[5 * n..5 * n + 5];
            (v.volume, v.pitch, v.adsr) = ([g[0], g[1]], g[2], [g[3], g[4]]);
            let k = &words[4 * n..4 * n + 4];
            (v.start, v.repeat, v.address, v.counter) = (k[0] & ADDRESS_MASK, k[1] & ADDRESS_MASK, k[2], k[3]);
            v.decoded = decoded[n].min(28);
            v.phase = voice::Phase::of(phases[n]);
            let s = &samples[34 * n..34 * n + 34];
            for (d, x) in v.samples.iter_mut().zip(s) {
                *d = *x as i16;
            }
            (v.history, v.out) = ([s[31] as i16, s[32] as i16], s[33] as i16);
            let e = &envelopes[6 * n..6 * n + 6];
            (v.envelope, v.current) = (envelope(e[0], e[1]), [envelope(e[2], e[3]), envelope(e[4], e[5])]);
        }
        r.u32s(&mut self.ssa)?;
        let mut k = [0u32; 4];
        r.u32s(&mut k)?;
        [self.endx, self.key_on, self.key_off, self.tsa] = k;
        let mut n = [0i32; 2];
        r.i32s(&mut n)?;
        self.noise = Noise { level: n[0] as i16, timer: n[1] };
        let mut env = [0u16; 12];
        r.u16s(&mut env)?;
        self.main = [envelope(env[0], env[1]), envelope(env[2], env[3])];
        self.inputs = std::array::from_fn(|i| envelope(env[4 + 2 * i], env[5 + 2 * i]));
        let rv = &mut self.reverb;
        rv.address = r.u32()?;
        let mut ring = [0u16; 4 * blackmoon_spu::reverb::RING];
        r.u16s(&mut ring)?;
        for (d, s) in rv.input.as_flattened_mut().iter_mut().chain(rv.output.as_flattened_mut()).zip(ring) {
            *d = s as i16;
        }
        let mut phase = [0u8; 2];
        r.bytes(&mut phase)?;
        (rv.at, rv.right) = (phase[0] % blackmoon_spu::reverb::RING as u8, phase[1] != 0);
        let mut f = [false; 3];
        r.bools(&mut f)?;
        (self.irq, self.adma.filled) = (f[0], [f[1], f[2]]);
        let mut a = [0u32; 3];
        r.u32s(&mut a)?;
        (self.adma.play, self.adma.buffer, self.adma.words) = (a[0] % 512, a[1] & 1, a[2]);
        r.i32s(&mut self.sums)?;
        r.i32s(&mut self.out)
    }
}
