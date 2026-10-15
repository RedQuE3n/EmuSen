//! CD-DA: the samples of each audio sector the drive plays, spread over the sector's 1/75 s at 44.1 kHz, through the
//! LC7883's de-emphasis and attenuator (the fader, which moves a step a sample toward the volume asked for), and handed
//! to the mix through a step synthesiser of their own with the PCM chip's (Nephrite_Native.md §53, §59; D-39).

use std::collections::VecDeque;

use emusen_native::steps::Steps;
use emusen_native::{StateReader, StateWriter, Truncated};

/// Samples a sector.
pub const FRAMES: usize = 588;
/// The fader's full volume, `$400` in FD10-FD0 (the Hardware Manual §3-6).
pub const FULL: i32 = 0x400;
/// CD-DA's full-scale sample, and the PCM chip's limited sum, against the output's range: fitted to recordings of two
/// model 1 consoles with a model 1 Sega CD playing the 240p suite's MDFourier sequence, each level held to the
/// Genesis's FM notes (D-35, D-38; Nephrite_Native.md §59).
pub const GAIN: (i32, i32) = (158, 1024);
pub const PCM_GAIN: (i32, i32) = (267, 1024);

#[derive(Clone)]
pub struct Cdda {
    pub steps: Steps,
    /// Samples not yet given to the synthesiser, each at its master clock.
    pub queue: VecDeque<(u64, [i32; 2])>,
    /// The two sources' levels now, CD-DA's and the PCM chip's, which the synthesiser holds the sum of.
    pub levels: [[i32; 2]; 2],
    /// The attenuator's level now, in the fader's units (`FULL` the whole), which moves a step a sample to the one asked.
    pub att: i32,
    /// The de-emphasis filter's last input and output, each side.
    pub emph: [[f64; 2]; 2],
    /// The last four samples each side, for undoing the droop of the synthesiser's held steps (`UNHOLD`).
    pub hist: [[i32; 4]; 2],
}

/// A five-tap filter at 44.1 kHz, in 16,384ths summing to one, that undoes the droop of samples held as steps to within 0.4 dB up to
/// 20 kHz: the LC7883 oversamples CD-DA through its digital filter, flat across the band (Nephrite_Native.md §59).
pub const UNHOLD: [i32; 5] = [365, -1623, 18900, -1623, 365];

/// The LC7883's de-emphasis, 50 µs and 15 µs, as a first-order filter by the bilinear transform at `fs`, the rate
/// DEF1-DEF0 names (D-39): the coefficients b0, b1 and a1.
fn deemphasis(fs: f64) -> (f64, f64, f64) {
    let (k, t1, t2) = (2.0 * fs, 50e-6, 15e-6);
    let d = 1.0 + k * t1;
    ((1.0 + k * t2) / d, (1.0 - k * t2) / d, (1.0 - k * t1) / d)
}

/// The attenuator's gain for a level in the fader's units: its upper bits count above 4, the lower two below (the
/// LC7883's attenuation table).
fn gain(att: i32) -> i32 {
    if att >= 4 { att & !3 } else { att }
}

impl Cdda {
    pub fn new(num: u64, den: u64) -> Cdda {
        Cdda { steps: Steps::new(num, den, crate::sound::RATE), queue: VecDeque::with_capacity(2 * FRAMES), levels: [[0; 2]; 2], att: 0, emph: [[0.0; 2]; 2], hist: [[0; 4]; 2] }
    }

    /// A sector's samples from master clock `start`, `period` master clocks long, toward `volume` of `FULL`, through
    /// the de-emphasis DEF1-DEF0 (`def`) asks for; None for a sector that gives no sound, the fader moving on all the
    /// same and the output falling silent.
    pub fn sector(&mut self, raw: Option<&[u8; 2352]>, start: u64, period: u64, volume: i32, def: u8) {
        let Some(raw) = raw else {
            self.att += (volume - self.att).clamp(-(FRAMES as i32), FRAMES as i32);
            if self.queue.back().map_or(self.levels[0], |e| e.1) != [0, 0] {
                self.queue.push_back((start, [0, 0]));
            }
            self.hist = [[0; 4]; 2];
            return;
        };
        let filter = match def & 3 {
            0 => None,
            1 => Some(deemphasis(44_100.0)),
            2 => Some(deemphasis(32_000.0)),
            _ => Some(deemphasis(48_000.0)),
        };
        for (j, (l, r)) in emusen_disc::samples(raw).enumerate() {
            let at = start + period * j as u64 / FRAMES as u64;
            self.att += (volume - self.att).signum();
            let mut out = [0i32; 2];
            for (side, x) in [l, r].into_iter().enumerate() {
                let x = match filter {
                    Some((b0, b1, a1)) => {
                        let [x1, y1] = self.emph[side];
                        let y = b0 * x as f64 + b1 * x1 - a1 * y1;
                        self.emph[side] = [x as f64, y];
                        y.round().clamp(-32768.0, 32767.0) as i32
                    }
                    None => x as i32,
                };
                let v = x * gain(self.att) / FULL * GAIN.0 / GAIN.1;
                let h = &mut self.hist[side];
                out[side] = (UNHOLD[0] * (v + h[3]) + UNHOLD[1] * (h[0] + h[2]) + UNHOLD[2] * h[1]) >> 14;
                *h = [v, h[0], h[1], h[2]];
            }
            self.queue.push_back((at, out));
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
        w.i32s("MixLevels", &[self.levels[0][0], self.levels[0][1], self.levels[1][0], self.levels[1][1], self.att]);
        w.u64s("Deemphasis", &[self.emph[0][0].to_bits(), self.emph[0][1].to_bits(), self.emph[1][0].to_bits(), self.emph[1][1].to_bits()]);
        w.i32s("CddaHistory", &[self.hist[0], self.hist[1]].concat());
        w.u32("CddaQueued", self.queue.len() as u32);
        w.u64s("CddaQueue", &q);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        self.steps.read_state(r)?;
        let mut l = [0i32; 5];
        r.i32s(&mut l)?;
        (self.levels, self.att) = ([[l[0], l[1]], [l[2], l[3]]], l[4].clamp(0, FULL));
        let mut e = [0u64; 4];
        r.u64s(&mut e)?;
        self.emph = [[f64::from_bits(e[0]), f64::from_bits(e[1])], [f64::from_bits(e[2]), f64::from_bits(e[3])]];
        let mut h = [0i32; 8];
        r.i32s(&mut h)?;
        self.hist = [[h[0], h[1], h[2], h[3]], [h[4], h[5], h[6], h[7]]];
        let n = r.u32()? as usize;
        let mut q = vec![0u64; 2 * n.min(4 * FRAMES)];
        r.u64s(&mut q)?;
        self.queue = q.chunks_exact(2).map(|p| (p[0], [p[1] as u32 as i32, (p[1] >> 32) as u32 as i32])).collect();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    /// DEF1-DEF0 01: the 50/15 µs de-emphasis at 44.1 kHz, unity at DC and about 7.6 dB down at 10 kHz, as the analogue
    /// curve is.
    #[test]
    fn de_emphasis_follows_the_50_15_us_curve() {
        let gain = |hz: f64| {
            let mut c = super::Cdda::new(1, 1);
            c.att = super::FULL;
            let mut raw = [0u8; 2352];
            let mut peak = 0i32;
            for sector in 0..20 {
                for j in 0..588 {
                    let n = (sector * 588 + j) as f64;
                    let s = (10_000.0 * (2.0 * std::f64::consts::PI * hz * n / 44_100.0).sin()) as i16;
                    raw[4 * j..4 * j + 2].copy_from_slice(&s.to_le_bytes());
                    raw[4 * j + 2..4 * j + 4].copy_from_slice(&s.to_le_bytes());
                }
                c.queue.clear();
                c.sector(Some(&raw), 0, 588, super::FULL, 1);
                if sector >= 10 {
                    peak = peak.max(c.queue.iter().map(|e| e.1[0].abs()).max().unwrap());
                }
            }
            20.0 * (peak as f64 / (10_000.0 * super::GAIN.0 as f64 / super::GAIN.1 as f64)).log10()
        };
        let (low, high) = (gain(100.0), gain(10_000.0));
        assert!(low.abs() < 0.3, "{low}");
        assert!((high + 7.6).abs() < 0.6, "{high}");
    }

    use crate::interface::Attachment;
    use crate::mcd::MegaCd;
    use crate::mcd::cdd::{SECTOR, status};

    /// A play of the audio track reaches the mix: each sector's 588 samples at the fader's volume, a sector apart.
    #[test]
    fn an_audio_track_played_reaches_the_mix_at_the_faders_volume() {
        let disc = crate::mcd::cdd::tests::disc(300, 150);
        let board = crate::interface::Board { master_hz: (4_725_000_000, 88), pal: false, start: 0, rate: crate::sound::RATE };
        let mut m = MegaCd::new(board, Vec::new(), Some(disc));
        let h = &mut m.hw;
        h.fader = 0x400 << 4;
        h.cdda.att = super::FULL;
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
        // The first sector played is 457, three short: audio sector 7 of the track, each frame's left sample 7, at the
        // mix's gain and out of the droop filter two samples late; each later sector's a sector on.
        let level = |sector: i32| sector * super::GAIN.0 / super::GAIN.1;
        assert_eq!((q[4].1[0], q[5].1[0]), (level(7), level(7)));
        let first = |v: i32| q.iter().position(|e| e.1[0] == v).unwrap();
        let k = first(level(13));
        assert!((6 * 588..=6 * 588 + 2).contains(&k), "{k}");
        let period = q[588].0 - q[0].0;
        assert!((period as i64 - master(SECTOR[0]) as i64).abs() <= 1, "a sector apart: {period}");
        // The fader at 0: the attenuator moves a step a sample, 1,024 samples from full to silent (the LC7883's soft mute).
        m.hw.fader = 0;
        let before = m.hw.cdda.queue.len();
        m.run_to(master(210 * SECTOR[0]));
        assert_eq!(m.hw.cdda.att, 0);
        let after: Vec<i32> = m.hw.cdda.queue.iter().skip(before).map(|e| e.1[0]).collect();
        assert!(after[1100..].iter().all(|&l| l == 0), "silent once the fade is done");
        assert!(after[..900].iter().any(|&l| l != 0), "and not at once");
    }
}
