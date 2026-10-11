//! The SPU (psx-spx "Sound Processing Unit"): 24 voices of SPU-ADPCM in 512 KiB of sound RAM, the noise generator,
//! the reverb unit, the capture buffers, the IRQ address, the transfer FIFO that DMA4 and the CPU fill and empty, CD
//! audio in, and the mix at one sample every 768 system cycles. It is brought to the present at each access and each
//! frame's end, sample by sample. Diamond_SPU.md is its record: what each rule is written from, and what was chosen.

pub mod envelope;
pub mod reverb;
pub mod state;
pub mod tables;
pub mod voice;

#[cfg(test)]
mod tests;

use emusen_native::samples::SampleQueue;

use envelope::Envelope;
use reverb::Reverb;
use voice::{Voice, Watch, sweep};

/// Sound RAM.
pub const RAM_BYTES: usize = 0x8_0000;
pub const VOICES: usize = 24;
/// The system clock's cycles a sample (psx-spx "SPU Internal State Machine": 768 × 44,100 Hz).
pub const CYCLES_PER_SAMPLE: u64 = 768;
/// The transfer's cycles a halfword: one of the 24 voices' spare RAM slots each (Diamond_SPU.md §6).
pub const CYCLES_PER_HALFWORD: u64 = 32;
/// The transfer FIFO's halfwords (psx-spx "DATA?": "max 32 halfwords"; "DMA Channels": a 16-word buffer).
pub const FIFO: usize = 32;

/// The control registers' halfword indices from 1F801C00h.
pub mod reg {
    pub const MVOL: usize = 0x180 / 2;
    pub const EVOL: usize = 0x184 / 2;
    pub const KON: usize = 0x188 / 2;
    pub const KOFF: usize = 0x18C / 2;
    pub const PMON: usize = 0x190 / 2;
    pub const NON: usize = 0x194 / 2;
    pub const EON: usize = 0x198 / 2;
    pub const ENDX: usize = 0x19C / 2;
    pub const ESA: usize = 0x1A2 / 2;
    pub const IRQA: usize = 0x1A4 / 2;
    pub const TSA: usize = 0x1A6 / 2;
    pub const DATA: usize = 0x1A8 / 2;
    pub const ATTR: usize = 0x1AA / 2;
    pub const RAM_CTRL: usize = 0x1AC / 2;
    pub const STATX: usize = 0x1AE / 2;
    pub const AVOL: usize = 0x1B0 / 2;
    pub const MVOLX: usize = 0x1B8 / 2;
    pub const REVERB: usize = 0x1C0 / 2;
}

/// ATTR's bits (psx-spx "ATTR").
pub mod attr {
    pub const ENABLE: u16 = 1 << 15;
    pub const UNMUTE: u16 = 1 << 14;
    pub const REVERB: u16 = 1 << 7;
    pub const IRQ: u16 = 1 << 6;
    pub const CD_REVERB: u16 = 1 << 2;
    pub const CD_IN: u16 = 1 << 0;
}

#[derive(Clone)]
pub struct Spu {
    /// The registers from 1F801C00h to 1F801FFFh as halfwords, as written; the voices' own are in `voices`.
    pub regs: [u16; 512],
    pub ram: Vec<u8>,
    pub voices: [Voice; VOICES],
    /// The voices whose loop end was met since their key on (ENDX), and the key ons and offs written since the last
    /// sample, which act at the next.
    pub endx: u32,
    pub key_on: u32,
    pub key_off: u32,
    /// ATTR's bits 0-5 as the SPU has applied them, which STATX shows (psx-spx "STATX").
    pub mode: u16,
    pub irq: bool,
    pub noise: i16,
    pub noise_timer: i32,
    pub main: [Envelope; 2],
    pub reverb: Reverb,
    /// The halfword the four capture buffers write next.
    pub capture: u16,
    /// The transfer: its address in bytes, the FIFO as a ring, and the cycle of its next RAM slot.
    pub cursor: u32,
    pub fifo: [u16; FIFO],
    pub fifo_head: u8,
    pub fifo_len: u8,
    pub next_slot: u64,
    /// The cycle at which the next sample is made.
    pub next_tick: u64,
    /// The samples made and not yet taken by the host; not machine state.
    pub out: SampleQueue,
}

impl Default for Spu {
    fn default() -> Spu {
        Spu::new()
    }
}

impl Spu {
    pub fn new() -> Spu {
        Spu {
            regs: [0; 512],
            ram: vec![0; RAM_BYTES],
            voices: std::array::from_fn(|_| Voice::default()),
            endx: 0,
            key_on: 0,
            key_off: 0,
            mode: 0,
            irq: false,
            noise: 0,
            noise_timer: 0,
            main: [Envelope::default(); 2],
            reverb: Reverb::default(),
            capture: 0,
            cursor: 0,
            fifo: [0; FIFO],
            fifo_head: 0,
            fifo_len: 0,
            next_slot: 0,
            next_tick: CYCLES_PER_SAMPLE,
            out: SampleQueue::with_capacity(emusen_native::samples::DEFAULT_LIMIT),
        }
    }

    fn attr(&self) -> u16 {
        self.regs[reg::ATTR]
    }

    /// Whether a hit on the IRQ address now raises IRQ9: ATTR's enable with the SPU on, the flag not yet up.
    fn armed(&self) -> bool {
        self.attr() & (attr::ENABLE | attr::IRQ) == attr::ENABLE | attr::IRQ && !self.irq
    }

    fn watch(&self) -> Watch {
        Watch { unit: self.regs[reg::IRQA] as u32, armed: self.armed(), hit: false }
    }

    // --- the registers ---

    pub fn read16(&self, offset: u32) -> u16 {
        let i = (offset as usize >> 1) & 511;
        match i {
            0..0xC0 => {
                let v = &self.voices[i >> 3];
                match i & 7 {
                    0 => v.volume[0],
                    1 => v.volume[1],
                    2 => v.pitch,
                    3 => v.start,
                    4 => v.adsr[0],
                    5 => v.adsr[1],
                    6 => v.envelope.level as u16,
                    _ => v.repeat,
                }
            }
            reg::ENDX => self.endx as u16,
            0xCF => (self.endx >> 16) as u16,
            reg::STATX => self.status(),
            reg::MVOLX => self.main[0].level as u16,
            0xDD => self.main[1].level as u16,
            0x100..0x130 => self.voices[(i - 0x100) >> 1].current[i & 1].level as u16,
            0x140.. => 0xFFFF,
            _ => self.regs[i],
        }
    }

    /// STATX (psx-spx "STATX"): the applied mode, IRQ9's flag, the transfer's requests and busy flag, and which half of
    /// the capture buffers is being written.
    pub fn status(&self) -> u16 {
        let request = self.dreq() as u16;
        let write = (self.mode >> 4 & 3 == 2) as u16 * request;
        let read = (self.mode >> 4 & 3 == 3) as u16 * request;
        let half = (self.regs[reg::RAM_CTRL] & 0xC != 0 && self.capture >= 0x100) as u16;
        self.mode | (self.irq as u16) << 6 | (write | read) << 7 | write << 8 | read << 9 | (self.busy() as u16) << 10 | half << 11
    }

    /// A halfword written at `now`, the SPU already brought to it.
    pub fn write16(&mut self, offset: u32, value: u16, now: u64) {
        let i = (offset as usize >> 1) & 511;
        match i {
            0..0xC0 => {
                let v = &mut self.voices[i >> 3];
                match i & 7 {
                    0 => v.volume[0] = value,
                    1 => v.volume[1] = value,
                    2 => v.pitch = value,
                    3 => v.start = value,
                    4 => v.adsr[0] = value,
                    5 => v.adsr[1] = value,
                    6 => v.envelope.level = value as i16,
                    _ => v.repeat = value,
                }
                return;
            }
            reg::KON => self.key_on |= value as u32,
            0xC5 => self.key_on |= (value as u32 & 0xFF) << 16,
            reg::KOFF => self.key_off |= value as u32,
            0xC7 => self.key_off |= (value as u32 & 0xFF) << 16,
            reg::ENDX | 0xCF | reg::STATX | reg::MVOLX | 0xDD => return,
            0x100..0x130 => return,
            reg::ESA => self.reverb.set_base(value),
            reg::TSA => self.cursor = value as u32 * 8,
            reg::DATA => {
                self.push(value);
                self.wake(now);
                return;
            }
            reg::ATTR => {
                if value & attr::IRQ == 0 || value & attr::ENABLE == 0 {
                    self.irq = false;
                }
            }
            _ => {}
        }
        self.regs[i] = value;
    }

    // --- the transfer ---

    /// Whether the transfer has a RAM slot to use: a write mode with halfwords to write, the read mode with room.
    fn transferring(&self) -> bool {
        match self.mode >> 4 & 3 {
            1 | 2 => self.fifo_len > 0,
            3 => (self.fifo_len as usize) < FIFO,
            _ => false,
        }
    }

    fn busy(&self) -> bool {
        self.transferring()
    }

    /// DMA4's request (STATX bits 8 and 9): in DMA write mode the FIFO empty, in DMA read mode full.
    pub fn dreq(&self) -> bool {
        match self.mode >> 4 & 3 {
            2 => self.fifo_len == 0,
            3 => self.fifo_len as usize == FIFO,
            _ => false,
        }
    }

    /// When the request is next to rise, the transfer running on alone: at the next sample if a written mode is still
    /// to be applied, else when the FIFO empties or fills.
    pub fn dreq_change(&self) -> u64 {
        if self.attr() & 0x3F != self.mode {
            return self.next_tick;
        }
        match self.mode >> 4 & 3 {
            2 => self.next_slot + CYCLES_PER_HALFWORD * (self.fifo_len as u64).saturating_sub(1),
            3 => self.next_slot + CYCLES_PER_HALFWORD * (FIFO as u64 - 1 - self.fifo_len as u64),
            _ => u64::MAX,
        }
    }

    /// The first slot after `now` for a transfer that was idle.
    fn wake(&mut self, now: u64) {
        let next = (now / CYCLES_PER_HALFWORD + 1) * CYCLES_PER_HALFWORD;
        self.next_slot = self.next_slot.max(next);
    }

    /// A halfword into the FIFO; a full FIFO first writes its oldest at once (Diamond_SPU.md §6).
    fn push(&mut self, value: u16) {
        if self.fifo_len as usize == FIFO {
            let v = self.pop();
            self.store(v, &mut Watch::default());
        }
        self.fifo[(self.fifo_head as usize + self.fifo_len as usize) % FIFO] = value;
        self.fifo_len += 1;
    }

    fn pop(&mut self) -> u16 {
        let v = self.fifo[self.fifo_head as usize];
        self.fifo_head = ((self.fifo_head as usize + 1) % FIFO) as u8;
        self.fifo_len -= 1;
        v
    }

    fn store(&mut self, value: u16, watch: &mut Watch) {
        let a = self.cursor as usize & 0x7FFFE;
        watch.touch(a as u32);
        self.ram[a..a + 2].copy_from_slice(&value.to_le_bytes());
        self.cursor = (self.cursor + 2) & 0x7FFFF;
    }

    fn load(&mut self, watch: &mut Watch) -> u16 {
        let a = self.cursor as usize & 0x7FFFE;
        watch.touch(a as u32);
        self.cursor = (self.cursor + 2) & 0x7FFFF;
        u16::from_le_bytes([self.ram[a], self.ram[a + 1]])
    }

    /// One RAM slot of the transfer.
    fn slot(&mut self, watch: &mut Watch) {
        if self.mode >> 4 & 3 == 3 {
            let v = self.load(watch);
            self.fifo[(self.fifo_head as usize + self.fifo_len as usize) % FIFO] = v;
            self.fifo_len += 1;
        } else {
            let v = self.pop();
            self.store(v, watch);
        }
    }

    /// A word from DMA4 into the FIFO, low halfword first.
    pub fn dma_write(&mut self, word: u32, now: u64) {
        self.push(word as u16);
        self.push((word >> 16) as u16);
        self.wake(now);
    }

    /// A word for DMA4 from the FIFO; an empty FIFO reads sound RAM at once (Diamond_SPU.md §6).
    pub fn dma_read(&mut self, now: u64) -> u32 {
        let mut half = || if self.fifo_len > 0 { self.pop() } else { self.load(&mut Watch::default()) };
        let w = half() as u32 | (half() as u32) << 16;
        self.wake(now);
        w
    }

    // --- time ---

    /// The next cycle at which the SPU must run unasked: while IRQ9 can be raised, each sample, and each transfer slot
    /// while one runs.
    pub fn next_event(&self) -> u64 {
        if !self.armed() {
            return u64::MAX;
        }
        if self.transferring() { self.next_slot.min(self.next_tick) } else { self.next_tick }
    }

    /// Brings the SPU to `now`: the transfer's slots and the samples due, in their order; `cd` gives the CD-ROM
    /// controller's next frame. Returns whether IRQ9 rose.
    pub fn run(&mut self, now: u64, cd: &mut impl FnMut() -> [i16; 2]) -> bool {
        let mut rose = false;
        loop {
            let until = now.min(self.next_tick);
            if self.transferring() && self.next_slot <= until {
                let mut watch = self.watch();
                while self.next_slot <= until && self.transferring() {
                    self.slot(&mut watch);
                    self.next_slot += CYCLES_PER_HALFWORD;
                }
                rose |= self.hit(watch);
            }
            if self.next_tick > now {
                return rose;
            }
            let at = self.next_tick;
            rose |= self.tick(cd());
            self.next_tick += CYCLES_PER_SAMPLE;
            if self.transferring() {
                self.wake(at - 1);
            }
        }
    }

    fn hit(&mut self, watch: Watch) -> bool {
        if watch.hit && self.armed() {
            self.irq = true;
            return true;
        }
        false
    }

    /// One sample: the written mode and key ons applied, the noise, the capture buffers, the 24 voices, the reverb and
    /// the mix through the main volume.
    fn tick(&mut self, cd: [i16; 2]) -> bool {
        let a = self.attr();
        if (a >> 4 & 3 == 3) != (self.mode >> 4 & 3 == 3) {
            self.fifo_len = 0;
        }
        self.mode = a & 0x3F;
        let (off, on) = (std::mem::take(&mut self.key_off), std::mem::take(&mut self.key_on));
        for n in 0..VOICES {
            if off & 1 << n != 0 {
                self.voices[n].key_off();
            }
            if on & 1 << n != 0 {
                self.voices[n].key_on(&self.ram);
            }
        }
        self.endx &= !on;
        self.step_noise(a);
        let mut watch = self.watch();
        let capture = self.capture as usize * 2;
        let capturing = self.regs[reg::RAM_CTRL] & 0xC != 0;
        self.put(capture, cd[0], &mut watch, capturing);
        self.put(0x400 + capture, cd[1], &mut watch, capturing);

        let pmon = self.regs[reg::PMON] as u32 | (self.regs[reg::PMON + 1] as u32) << 16;
        let non = self.regs[reg::NON] as u32 | (self.regs[reg::NON + 1] as u32) << 16;
        let eon = self.regs[reg::EON] as u32 | (self.regs[reg::EON + 1] as u32) << 16;
        let (mut dry, mut wet) = ([0i32; 2], [0i32; 2]);
        let mut previous = 0i16;
        for n in 0..VOICES {
            let v = &mut self.voices[n];
            let mut step = v.pitch as u32;
            if n > 0 && pmon & 1 << n != 0 {
                let factor = previous as i32 + 0x8000;
                step = ((step as i16 as i64 * factor as i64) >> 15) as u32 & 0xFFFF;
            }
            let step = step.min(0x4000);
            let noise = (non & 1 << n != 0).then_some(self.noise);
            if v.tick(&self.ram, step, noise, &mut watch) {
                self.endx |= 1 << n;
            }
            previous = v.out;
            let lr = v.volumes();
            dry[0] += lr[0];
            dry[1] += lr[1];
            if eon & 1 << n != 0 {
                wet[0] += lr[0];
                wet[1] += lr[1];
            }
        }
        let (v1, v3) = (self.voices[1].out, self.voices[3].out);
        self.put(0x800 + capture, v1, &mut watch, capturing);
        self.put(0xC00 + capture, v3, &mut watch, capturing);
        self.capture = (self.capture + 1) & 0x1FF;

        if a & attr::CD_IN != 0 {
            for (s, &c) in cd.iter().enumerate() {
                let c = c as i32 * self.regs[reg::AVOL + s] as i16 as i32 >> 15;
                dry[s] += c;
                if a & attr::CD_REVERB != 0 {
                    wet[s] += c;
                }
            }
        }
        let reverb_regs: [u16; 32] = std::array::from_fn(|k| self.regs[reg::REVERB + k]);
        let evol = [self.regs[reg::EVOL], self.regs[reg::EVOL + 1]];
        let rev = self.reverb.tick(&mut self.ram, &mut watch, &reverb_regs, evol, self.regs[reg::ESA], a & attr::REVERB != 0, wet);
        let on = a & (attr::ENABLE | attr::UNMUTE) == attr::ENABLE | attr::UNMUTE;
        let mut frame = [0i16; 2];
        for s in 0..2 {
            let mut mix = if on { dry[s] + rev[s] } else if a & attr::CD_IN != 0 { cd[s] as i32 * self.regs[reg::AVOL + s] as i16 as i32 >> 15 } else { 0 };
            mix = mix.clamp(-0x8000, 0x7FFF);
            sweep(&mut self.main[s], self.regs[reg::MVOL + s]);
            frame[s] = (mix * self.main[s].level as i32 >> 15).clamp(-0x8000, 0x7FFF) as i16;
        }
        self.out.push_pair(frame[0], frame[1]);
        self.hit(watch)
    }

    /// A capture buffer's halfword (psx-spx "SPU Memory layout"), watched only when RAM_CTRL's bits 2-3 allow.
    fn put(&mut self, at: usize, value: i16, watch: &mut Watch, capturing: bool) {
        if capturing {
            watch.touch(at as u32);
        }
        self.ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }

    /// The noise generator's cycle (psx-spx "SPU Noise Generator"), at ATTR's shift and step.
    fn step_noise(&mut self, a: u16) {
        let step = (a >> 8 & 3) as i32 + 4;
        let shift = (a >> 10 & 0xF) as i32;
        self.noise_timer -= step;
        let n = self.noise as u16;
        let parity = (n >> 15 ^ n >> 12 ^ n >> 11 ^ n >> 10 ^ 1) & 1;
        if self.noise_timer < 0 {
            self.noise = (n << 1 | parity) as i16;
            self.noise_timer += 0x2_0000 >> shift;
            if self.noise_timer < 0 {
                self.noise_timer += 0x2_0000 >> shift;
            }
        }
    }
}
