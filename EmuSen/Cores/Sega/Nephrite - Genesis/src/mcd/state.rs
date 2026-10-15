//! The Sega CD's block of a state (version 20 on): the sub CPU, its clock and the gate array, Word RAM's switches, the
//! sub CPU's wait, the PCM chip, the drive, the CDC with its buffer, the fader, CD-DA's samples in flight,
//! the RAM cartridge's switch and the disc's fingerprint. The memories are the machine's spaces (Nephrite_Native.md §53).

use beryl_m68k::{M68000, Size};
use emusen_native::{State, StateReader, StateWriter, Truncated};

use super::cdc::Cdc;
use super::cdd::Mode;
use super::{Gate, MegaCd};

/// What a load reads before anything is applied.
pub struct Saved {
    cpu: M68000,
    clock: u64,
    g: Gate,
    word: [u8; 5],
    parked: bool,
    owed: Vec<(u32, Size, u16)>,
    pcm: super::pcm::Pcm,
    fader: u16,
    cdd: CddSaved,
    cdc: Cdc,
    cdda: super::cdda::Cdda,
    cart_writable: bool,
    asic: super::asic::Asic,
    pub print: u64,
}

struct CddSaved {
    mode: [u32; 7],
    words: [u64; 5],
    nibbles: [u8; 20],
    flags: [bool; 4],
    report: [u8; 2],
}

fn mode_words(m: Mode) -> [u32; 7] {
    match m {
        Mode::Idle => [0; 7],
        Mode::Spinning(n) => [1, n, 0, 0, 0, 0, 0],
        Mode::ReadingToc(n) => [2, n, 0, 0, 0, 0, 0],
        Mode::Stopped(s) => [3, s as u32, 0, 0, 0, 0, 0],
        Mode::Seeking { target, play, busy, left, total, from } => [4, target, play as u32, busy, left, total, from],
        Mode::Approach { target, play, status } => [5, target, play as u32, status as u32, 0, 0, 0],
        Mode::Playing => [6, 0, 0, 0, 0, 0, 0],
        Mode::Paused { low, high, gap } => [7, low, high, gap as u32, 0, 0, 0],
        Mode::Scanning { forward } => [8, forward as u32, 0, 0, 0, 0, 0],
        Mode::TrayOpen => [9, 0, 0, 0, 0, 0, 0],
    }
}

fn mode_from(w: [u32; 7]) -> Mode {
    match w[0] {
        1 => Mode::Spinning(w[1]),
        2 => Mode::ReadingToc(w[1]),
        3 => Mode::Stopped(w[1] as u8),
        4 => Mode::Seeking { target: w[1], play: w[2] != 0, busy: w[3], left: w[4], total: w[5].max(1), from: w[6] },
        5 => Mode::Approach { target: w[1], play: w[2] != 0, status: w[3] as u8 },
        6 => Mode::Playing,
        7 => Mode::Paused { low: w[1], high: w[2], gap: w[3] != 0 },
        8 => Mode::Scanning { forward: w[1] != 0 },
        9 => Mode::TrayOpen,
        _ => Mode::Idle,
    }
}

impl MegaCd {
    pub fn write_state(&self, w: &mut StateWriter, print: u64) {
        w.group("SubCpu", |w| self.cpu.write_state(w));
        let h = &self.hw;
        let g = &h.g;
        w.u64("SubClock", h.clock);
        w.bools("GateLines", &[g.reset_held, g.busreq, g.reset_due]);
        w.bytes("GateBytes", &[g.leds, g.wp, g.bank, g.cfm, g.cfs, g.timer, g.timer_count, g.mask, g.pending, g.font_color, g.forced]);
        w.u16s("GateWords", &[g.hivect, g.font_bits]);
        w.u16s("Commands", &g.cmd);
        w.u16s("Statuses", &g.sta);
        w.u64s("GateTimes", &[g.res0_at, g.stopwatch, g.timer_next, g.taken2]);
        let wr = &h.word;
        w.bytes("WordRamSwitches", &[wr.one_m as u8, wr.sub_owns as u8, wr.ret as u8, wr.request as u8, wr.priority]);
        w.bool("SubParked", h.parked);
        w.u32("SubOwed", h.owed.len() as u32);
        let owed: Vec<u64> = h.owed.iter().map(|&(a, s, v)| a as u64 | ((s == Size::Word) as u64) << 32 | (v as u64) << 40).collect();
        w.u64s("SubOwedWrites", &owed);
        let p = &h.pcm;
        let bytes: Vec<u8> = p.ch.iter().flat_map(|c| [c.env, c.pan, c.st, c.fresh as u8]).chain([p.on as u8, p.wave_bank, p.chan_bank, p.off]).collect();
        w.bytes("PcmBytes", &bytes);
        let words: Vec<u16> = p.ch.iter().flat_map(|c| [c.fd, c.ls]).collect();
        w.u16s("PcmWords", &words);
        let addrs: Vec<u32> = p.ch.iter().map(|c| c.addr).collect();
        w.u32s("PcmAddresses", &addrs);
        w.u64("PcmNext", p.next);
        w.u32("PcmQueued", p.out.len() as u32);
        let q: Vec<u64> = p.out.iter().flat_map(|&(at, [l, r])| [at, l as u32 as u64 | (r as u32 as u64) << 32]).collect();
        w.u64s("PcmQueue", &q);
        w.u16("Fader", h.fader);
        let c = &h.cdd;
        w.u32s("DriveMode", &mode_words(c.mode));
        w.u64s("DriveWords", &[c.dts_until, c.head as u64, c.laps, c.next, c.phase as u64]);
        w.bytes("DriveNibbles", &[c.status, c.command].concat());
        w.bools("DriveFlags", &[c.hock, c.command_new, c.data, c.toc_read]);
        w.bytes("DriveReport", &[c.report, c.report_track]);
        let k = &h.cdc;
        w.bytes("CdcBuffer", &k.buffer);
        w.bytes("CdcBytes", &[k.ar, k.ifctrl, k.ctrl[0], k.ctrl[1], k.dd]);
        w.u16s("CdcCounters", &[k.dbc, k.dac, k.wa, k.pt, k.host]);
        w.bytes("CdcHeader", &[k.head, k.stat].concat());
        w.bools("CdcFlags", &[k.dtei, k.deci, k.pin, k.edt, k.dsr, k.busy]);
        w.u64s("CdcTimes", &[k.deci_until, k.dma_next, k.dma as u64]);
        w.group("Cdda", |w| h.cdda.write_state(w));
        w.bool("CartWritable", h.cart.as_ref().is_some_and(|c| c.writable));
        let a = &h.asic;
        w.bytes("AsicBytes", &[a.size, a.vcells, a.offset, a.vdots, a.busy as u8]);
        w.u16s("AsicWords", &[a.map, a.buffer, a.hdots, a.vector]);
        w.u64s("AsicTimes", &[a.line as u64, a.next, a.refresh]);
        w.u64("DiscFingerprint", print);
    }

    pub fn read_state(&self, r: &mut StateReader) -> Result<Saved, Truncated> {
        let mut cpu = self.cpu.clone();
        cpu.read_state(r)?;
        let clock = r.u64()?;
        let mut g = Gate::default();
        let mut lines = [false; 3];
        r.bools(&mut lines)?;
        [g.reset_held, g.busreq, g.reset_due] = lines;
        let mut b = [0u8; 11];
        r.bytes(&mut b)?;
        [g.leds, g.wp, g.bank, g.cfm, g.cfs, g.timer, g.timer_count, g.mask, g.pending, g.font_color, g.forced] = b;
        let mut words = [0u16; 2];
        r.u16s(&mut words)?;
        [g.hivect, g.font_bits] = words;
        r.u16s(&mut g.cmd)?;
        r.u16s(&mut g.sta)?;
        let mut t = [0u64; 4];
        r.u64s(&mut t)?;
        [g.res0_at, g.stopwatch, g.timer_next, g.taken2] = t;
        let mut word = [0u8; 5];
        r.bytes(&mut word)?;
        let parked = r.bool()?;
        let n = r.u32()? as usize;
        let mut owed = vec![0u64; n.min(64)];
        r.u64s(&mut owed)?;
        let owed = owed.iter().map(|&o| (o as u32, if o >> 32 & 1 != 0 { Size::Word } else { Size::Byte }, (o >> 40) as u16)).collect();
        let mut pcm = super::pcm::Pcm::default();
        let mut pb = [0u8; 36];
        r.bytes(&mut pb)?;
        let mut pw = [0u16; 16];
        r.u16s(&mut pw)?;
        let mut pa = [0u32; 8];
        r.u32s(&mut pa)?;
        for (k, c) in pcm.ch.iter_mut().enumerate() {
            (c.env, c.pan, c.st, c.fresh) = (pb[4 * k], pb[4 * k + 1], pb[4 * k + 2], pb[4 * k + 3] != 0);
            (c.fd, c.ls, c.addr) = (pw[2 * k], pw[2 * k + 1], pa[k] & 0x7FF_FFFF);
        }
        (pcm.on, pcm.wave_bank, pcm.chan_bank, pcm.off) = (pb[32] != 0, pb[33] & 15, pb[34] & 7, pb[35]);
        pcm.next = r.u64()?;
        let n = r.u32()? as usize;
        let mut q = vec![0u64; 2 * n.min(4096)];
        r.u64s(&mut q)?;
        pcm.out = q.chunks_exact(2).map(|e| (e[0], [e[1] as u32 as i32, (e[1] >> 32) as u32 as i32])).collect();
        let fader = r.u16()?;
        let mut mode = [0u32; 7];
        r.u32s(&mut mode)?;
        let mut dw = [0u64; 5];
        r.u64s(&mut dw)?;
        let mut nibbles = [0u8; 20];
        r.bytes(&mut nibbles)?;
        let mut flags = [false; 4];
        r.bools(&mut flags)?;
        let mut report = [0u8; 2];
        r.bytes(&mut report)?;
        let mut cdc = Cdc::default();
        r.bytes(&mut cdc.buffer)?;
        let mut cb = [0u8; 5];
        r.bytes(&mut cb)?;
        [cdc.ar, cdc.ifctrl, cdc.ctrl[0], cdc.ctrl[1], cdc.dd] = cb;
        let mut cw = [0u16; 5];
        r.u16s(&mut cw)?;
        [cdc.dbc, cdc.dac, cdc.wa, cdc.pt, cdc.host] = cw;
        let mut hs = [0u8; 8];
        r.bytes(&mut hs)?;
        cdc.head.copy_from_slice(&hs[..4]);
        cdc.stat.copy_from_slice(&hs[4..]);
        let mut cf = [false; 6];
        r.bools(&mut cf)?;
        [cdc.dtei, cdc.deci, cdc.pin, cdc.edt, cdc.dsr, cdc.busy] = cf;
        let mut ct = [0u64; 3];
        r.u64s(&mut ct)?;
        (cdc.deci_until, cdc.dma_next, cdc.dma) = (ct[0], ct[1], ct[2] as u32);
        let mut cdda = self.hw.cdda.clone();
        cdda.read_state(r)?;
        let cart_writable = r.bool()?;
        let mut ab = [0u8; 5];
        r.bytes(&mut ab)?;
        let mut aw = [0u16; 4];
        r.u16s(&mut aw)?;
        let mut at = [0u64; 3];
        r.u64s(&mut at)?;
        let asic = super::asic::Asic {
            size: ab[0] & 7, vcells: ab[1] & 0x1F, offset: ab[2] & 0x3F, vdots: ab[3], busy: ab[4] != 0,
            map: aw[0], buffer: aw[1], hdots: aw[2] & 0x1FF, vector: aw[3], line: at[0] as u32, next: at[1], refresh: at[2] % super::asic::REFRESH_EVERY,
        };
        let print = r.u64()?;
        let cdd = CddSaved { mode, words: dw, nibbles, flags, report };
        let s = Saved { cpu, clock, g, word, parked, owed, pcm, fader, cdd, cdc, cdda, cart_writable, asic, print };
        Ok(s)
    }

    /// A state read whole put in place; the disc stays the drive's own.
    pub fn apply_state(&mut self, s: Saved) {
        self.cpu = s.cpu;
        let h = &mut self.hw;
        (h.clock, h.g, h.parked, h.owed, h.pcm, h.fader, h.cdc, h.cdda, h.asic) = (s.clock, s.g, s.parked, s.owed, s.pcm, s.fader, s.cdc, s.cdda, s.asic);
        let wr = &mut h.word;
        (wr.one_m, wr.sub_owns, wr.ret, wr.request, wr.priority) = (s.word[0] != 0, s.word[1] != 0, s.word[2] != 0, s.word[3] != 0, s.word[4] & 3);
        let c = &mut h.cdd;
        c.mode = mode_from(s.cdd.mode);
        (c.dts_until, c.head, c.laps, c.next, c.phase) = (s.cdd.words[0], s.cdd.words[1] as u32, s.cdd.words[2], s.cdd.words[3], s.cdd.words[4] as u8 % 3);
        c.status.copy_from_slice(&s.cdd.nibbles[..10]);
        c.command.copy_from_slice(&s.cdd.nibbles[10..]);
        [c.hock, c.command_new, c.data, c.toc_read] = s.cdd.flags;
        [c.report, c.report_track] = s.cdd.report;
        c.named = None;
        if let Some(cart) = h.cart.as_mut() {
            cart.writable = s.cart_writable;
        }
    }
}
