//! The SPU's block of the state (Diamond_SPU.md §9): the registers, each voice's decoder, counter, envelope and
//! volumes, the noise, the reverb's address and filters, the capture position, the transfer with its FIFO, and the
//! clock of the next sample. Sound RAM is the state's own block; the samples not yet taken by the host are not kept.

use emusen_native::{StateReader, StateWriter, Truncated};

use super::envelope::Envelope;
use super::voice::Phase;
use super::{FIFO, Spu, VOICES};

fn halves(e: &[Envelope]) -> Vec<u16> {
    e.iter().flat_map(|e| [e.level as u16, e.counter]).collect()
}

fn set_halves(e: &mut [Envelope], v: &[u16]) {
    for (e, p) in e.iter_mut().zip(v.chunks(2)) {
        *e = Envelope { level: p[0] as i16, counter: p[1] };
    }
}

impl Spu {
    pub fn write_state(&self, w: &mut StateWriter) {
        let v = &self.voices;
        w.u16s("Registers", &self.regs);
        w.u16s("VoiceRegisters", &v.iter().flat_map(|v| [v.volume[0], v.volume[1], v.pitch, v.start, v.adsr[0], v.adsr[1], v.repeat]).collect::<Vec<_>>());
        w.u32s("VoiceAddresses", &v.iter().map(|v| v.address).collect::<Vec<_>>());
        w.u32s("VoiceCounters", &v.iter().map(|v| v.counter).collect::<Vec<_>>());
        w.bytes("VoiceDecoded", &v.iter().map(|v| v.decoded).collect::<Vec<_>>());
        w.bytes("VoicePhases", &v.iter().map(|v| v.phase.code()).collect::<Vec<_>>());
        w.u16s("VoiceSamples", &v.iter().flat_map(|v| v.samples.map(|s| s as u16)).collect::<Vec<_>>());
        w.u16s("VoiceHistory", &v.iter().flat_map(|v| v.history.map(|s| s as u16)).collect::<Vec<_>>());
        w.u16s("VoiceEnvelopes", &halves(&v.iter().flat_map(|v| [v.envelope, v.current[0], v.current[1]]).collect::<Vec<_>>()));
        w.u16s("VoiceOut", &v.iter().map(|v| v.out as u16).collect::<Vec<_>>());
        w.u32s("Flags", &[self.endx, self.key_on, self.key_off, self.mode as u32, self.irq as u32, self.noise as u16 as u32, self.noise_timer as u32, self.capture as u32]);
        w.u16s("Main", &halves(&self.main));
        let r = &self.reverb;
        w.u32("ReverbAddress", r.address);
        w.u16s("ReverbInput", &r.input.as_flattened().iter().map(|&s| s as u16).collect::<Vec<_>>());
        w.u16s("ReverbOutput", &r.output.as_flattened().iter().map(|&s| s as u16).collect::<Vec<_>>());
        w.bytes("ReverbPhase", &[r.at, r.right as u8]);
        w.u32("Cursor", self.cursor);
        w.u16s("Fifo", &self.fifo);
        w.bytes("FifoAt", &[self.fifo_head, self.fifo_len]);
        w.u64s("Times", &[self.next_slot, self.next_tick]);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        r.u16s(&mut self.regs)?;
        let mut regs = [0u16; VOICES * 7];
        r.u16s(&mut regs)?;
        let mut addresses = [0u32; VOICES];
        r.u32s(&mut addresses)?;
        let mut counters = [0u32; VOICES];
        r.u32s(&mut counters)?;
        let mut decoded = [0u8; VOICES];
        r.bytes(&mut decoded)?;
        let mut phases = [0u8; VOICES];
        r.bytes(&mut phases)?;
        let mut samples = [0u16; VOICES * 31];
        r.u16s(&mut samples)?;
        let mut history = [0u16; VOICES * 2];
        r.u16s(&mut history)?;
        let mut envelopes = [0u16; VOICES * 6];
        r.u16s(&mut envelopes)?;
        let mut out = [0u16; VOICES];
        r.u16s(&mut out)?;
        for (n, v) in self.voices.iter_mut().enumerate() {
            let g = &regs[7 * n..7 * n + 7];
            (v.volume, v.pitch, v.start, v.adsr, v.repeat) = ([g[0], g[1]], g[2], g[3], [g[4], g[5]], g[6]);
            v.address = addresses[n] & 0x7FFFF;
            v.counter = counters[n] % (super::voice::BLOCK_SAMPLES << 12);
            v.decoded = decoded[n].min(28) & !3;
            v.phase = Phase::of(phases[n]);
            for (s, &x) in v.samples.iter_mut().zip(&samples[31 * n..31 * n + 31]) {
                *s = x as i16;
            }
            v.history = [history[2 * n] as i16, history[2 * n + 1] as i16];
            let mut e = [Envelope::default(); 3];
            set_halves(&mut e, &envelopes[6 * n..6 * n + 6]);
            (v.envelope, v.current) = (e[0], [e[1], e[2]]);
            v.out = out[n] as i16;
        }
        let mut f = [0u32; 8];
        r.u32s(&mut f)?;
        (self.endx, self.key_on, self.key_off) = (f[0] & 0xFF_FFFF, f[1] & 0xFF_FFFF, f[2] & 0xFF_FFFF);
        (self.mode, self.irq, self.noise, self.noise_timer, self.capture) = (f[3] as u16 & 0x3F, f[4] != 0, f[5] as u16 as i16, f[6] as i32, f[7] as u16 & 0x1FF);
        let mut main = [0u16; 4];
        r.u16s(&mut main)?;
        set_halves(&mut self.main, &main);
        let rv = &mut self.reverb;
        rv.address = r.u32()? & 0x7FFFE;
        let mut ring = [0u16; 128];
        r.u16s(&mut ring)?;
        for (s, &x) in rv.input.as_flattened_mut().iter_mut().zip(&ring) {
            *s = x as i16;
        }
        r.u16s(&mut ring)?;
        for (s, &x) in rv.output.as_flattened_mut().iter_mut().zip(&ring) {
            *s = x as i16;
        }
        let mut phase = [0u8; 2];
        r.bytes(&mut phase)?;
        (rv.at, rv.right) = (phase[0] & 63, phase[1] != 0);
        self.cursor = r.u32()? & 0x7FFFF;
        r.u16s(&mut self.fifo)?;
        let mut at = [0u8; 2];
        r.bytes(&mut at)?;
        (self.fifo_head, self.fifo_len) = (at[0] % FIFO as u8, at[1].min(FIFO as u8));
        let mut t = [0u64; 2];
        r.u64s(&mut t)?;
        [self.next_slot, self.next_tick] = t;
        self.out.clear();
        Ok(())
    }
}
