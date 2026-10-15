//! CD-DA: the samples of each audio sector the drive plays, spread over the sector's 1/75 s at 44.1 kHz, scaled by the
//! fader and handed to the mix through a step synthesiser of their own (Nephrite_Native.md §53). The fader's ramp,
//! de-emphasis and the mix's measured levels are stage 9's.

use std::collections::VecDeque;

use emusen_native::steps::Steps;
use emusen_native::{StateReader, StateWriter, Truncated};

/// Samples a sector.
pub const FRAMES: usize = 588;
/// The fader's full volume, `$400` in FD10-FD0 (the Hardware Manual §3-6), and the mix's level for a full-scale
/// sample against the Genesis's output (argued; D-35).
pub const FULL: i32 = 0x400;
pub const GAIN: (i32, i32) = (1, 2);
/// The PCM chip's limited 16-bit sum against the same range: half of it, as CD-DA's (argued; D-38).
pub const PCM_GAIN: (i32, i32) = (1, 2);

#[derive(Clone)]
pub struct Cdda {
    pub steps: Steps,
    /// Samples not yet given to the synthesiser, each at its master clock.
    pub queue: VecDeque<(u64, [i32; 2])>,
    /// The two sources' levels now, CD-DA's and the PCM chip's, which the synthesiser holds the sum of.
    pub levels: [[i32; 2]; 2],
}

impl Cdda {
    pub fn new(num: u64, den: u64) -> Cdda {
        Cdda { steps: Steps::new(num, den, crate::sound::RATE), queue: VecDeque::with_capacity(2 * FRAMES), levels: [[0; 2]; 2] }
    }

    /// A sector's samples from master clock `start`, `period` master clocks long, at `volume` of `FULL`.
    pub fn sector(&mut self, raw: &[u8; 2352], start: u64, period: u64, volume: i32) {
        for (j, (l, r)) in emusen_disc::samples(raw).enumerate() {
            let at = start + period * j as u64 / FRAMES as u64;
            let level = |s: i16| s as i32 * volume / FULL * GAIN.0 / GAIN.1;
            self.queue.push_back((at, [level(l), level(r)]));
        }
    }

    /// The samples due by master clock `t`, CD-DA's queued and the PCM chip's in `pcm` (each at its master clock, in
    /// order), given to the synthesiser in time order as the sum of the two; the PCM's taken are removed.
    pub fn feed(&mut self, t: u64, pcm: &mut Vec<(u64, [i32; 2])>) {
        let mut i = 0;
        loop {
            let cd = self.queue.front().copied().filter(|e| e.0 <= t);
            let pc = pcm.get(i).copied().filter(|e| e.0 <= t);
            let (at, source, level) = match (cd, pc) {
                (Some(c), Some(p)) if c.0 <= p.0 => (c.0, 0, c.1),
                (Some(c), None) => (c.0, 0, c.1),
                (_, Some(p)) => (p.0, 1, [p.1[0] * PCM_GAIN.0 / PCM_GAIN.1, p.1[1] * PCM_GAIN.0 / PCM_GAIN.1]),
                (None, None) => break,
            };
            if source == 0 {
                self.queue.pop_front();
            } else {
                i += 1;
            }
            self.levels[source] = level;
            let [a, b] = self.levels;
            self.steps.set(at, [a[0] + b[0], a[1] + b[1]]);
        }
        pcm.drain(..i);
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        self.steps.write_state(w);
        let q: Vec<u64> = self.queue.iter().flat_map(|&(at, [l, r])| [at, l as u32 as u64 | (r as u32 as u64) << 32]).collect();
        w.i32s("MixLevels", &[self.levels[0][0], self.levels[0][1], self.levels[1][0], self.levels[1][1]]);
        w.u32("CddaQueued", self.queue.len() as u32);
        w.u64s("CddaQueue", &q);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        self.steps.read_state(r)?;
        let mut l = [0i32; 4];
        r.i32s(&mut l)?;
        self.levels = [[l[0], l[1]], [l[2], l[3]]];
        let n = r.u32()? as usize;
        let mut q = vec![0u64; 2 * n.min(4 * FRAMES)];
        r.u64s(&mut q)?;
        self.queue = q.chunks_exact(2).map(|p| (p[0], [p[1] as u32 as i32, (p[1] >> 32) as u32 as i32])).collect();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::mcd::MegaCd;
    use crate::mcd::cdd::{SECTOR, status};

    /// A play of the audio track reaches the mix: each sector's 588 samples at the fader's volume, a sector apart.
    #[test]
    fn an_audio_track_played_reaches_the_mix_at_the_faders_volume() {
        let disc = crate::mcd::cdd::tests::disc(300, 150);
        let mut m = MegaCd::new((4_725_000_000, 88), Vec::new(), 0, Some(disc));
        let h = &mut m.hw;
        h.fader = 0x400 << 4;
        h.cdd.set_hock(true);
        h.cdd.toc_read = true;
        // Play 00:06:10, sector 460, inside track 2 (from 450).
        let mut n = [3u8, 0, 0, 0, 0, 6, 1, 0, 0, 0];
        n[9] = !n[..9].iter().fold(0u8, |a, &b| a.wrapping_add(b)) & 15;
        let master = |s: u64| s * 189 / 44;
        m.run_to(master(81 * SECTOR[0]));
        for k in 0..5 {
            m.hw.cdd.write(0x0C + 2 * k as u32, (n[2 * k] as u16) << 8 | n[2 * k + 1] as u16, 0xFFFF, 0);
        }
        m.run_to(master(200 * SECTOR[0]));
        assert_eq!(m.hw.cdd.status[0], status::PLAYING);
        let q = &m.hw.cdda.queue;
        assert!(q.len() >= 588, "samples queued");
        let first = q[0];
        // The first sector played is 457, three short: audio sector 7 of the track; its frame 0 is (7, 0) halved.
        assert_eq!(first.1, [3, 0]);
        assert_eq!(q[1].1, [3, 0]);
        let k = q.iter().position(|e| e.1 == [4, 0]).expect("the next sector");
        assert_eq!(k, 588);
        let period = q[588].0 - q[0].0;
        assert!((period as i64 - master(SECTOR[0]) as i64).abs() <= 1, "a sector apart: {period}");
        m.hw.fader = 0;
        let before = m.hw.cdda.queue.len();
        m.run_to(master(210 * SECTOR[0]));
        assert!(m.hw.cdda.queue.iter().skip(before).all(|e| e.1 == [0, 0]), "the fader at 0 silences it");
    }
}
