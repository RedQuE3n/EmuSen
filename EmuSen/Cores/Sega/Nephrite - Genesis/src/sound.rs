//! The Genesis's sound: the YM2612 (`ym2612.rs`) and the PSG in the VDP (Beryl's SN76489, Sega's variant), each
//! caught up when it is touched and at the frame's end, each held between its changes and resampled to the output
//! rate by its own step synthesiser (emusen-native's `steps`), and the two summed. The mix's levels and the analogue
//! path are Nephrite_Native.md §18's.

use emusen_native::steps::Steps;
use emusen_native::{StateReader, StateWriter, Truncated};
use beryl_sn76489::{Sn76489, Variant};

use crate::ym2612::Ym2612;

/// Master clocks a PSG clock: the Z80's clock, over 16 inside the chip.
pub const PSG_CLOCK: u64 = 15 * 16;
/// The output rate.
pub const RATE: u32 = 48_000;
/// The output's units for one step of the YM2612's nine-bit output, and for one of the PSG's levels (a channel at
/// full volume is 8,192): argued from the references' mix, Nephrite_Native.md §18.
pub const FM_GAIN: i32 = 16;
pub const PSG_GAIN_DIV: i32 = 4;

#[derive(Clone)]
pub struct Sound {
    pub ym: Ym2612,
    pub psg: Sn76489,
    /// The master clock the PSG has reached, on its clock's boundaries.
    pub psg_time: u64,
    fm_steps: Steps,
    psg_steps: Steps,
    fm_out: Vec<[i32; 2]>,
    /// The latest master clock an access or a frame's end has brought the sound to: the Z80 runs up to an
    /// instruction past the 68000's clock, and an access from behind it is taken as made then.
    pub time: u64,
}

impl Sound {
    /// The sound of a board whose master clock is `num / den` Hz, its YM2612 discrete (model 1) or not.
    pub fn new(num: u64, den: u64, discrete: bool) -> Sound {
        Sound {
            ym: Ym2612::new(discrete),
            psg: Sn76489::new(Variant::SEGA),
            psg_time: 0,
            fm_steps: Steps::new(num, den, RATE),
            psg_steps: Steps::new(num, den, RATE),
            fm_out: Vec::new(),
            time: 0,
        }
    }

    fn now(&mut self, t: u64) -> u64 {
        self.time = self.time.max(t);
        self.time
    }

    /// The YM2612 up to `t`.
    pub fn run_ym(&mut self, t: u64) {
        let steps = &mut self.fm_steps;
        self.ym.run(t, |at, out| steps.set(at, [out[0] * FM_GAIN, out[1] * FM_GAIN]));
    }

    /// The PSG up to the last of its clocks at or before `t`.
    pub fn run_psg(&mut self, t: u64) {
        if t <= self.psg_time {
            return;
        }
        let ticks = ((t - self.psg_time) / PSG_CLOCK) as u32;
        let (start, steps) = (self.psg_time, &mut self.psg_steps);
        self.psg.advance(ticks, |k, out| steps.set(start + k as u64 * PSG_CLOCK, [out[0] / PSG_GAIN_DIV, out[1] / PSG_GAIN_DIV]));
        self.psg_time += ticks as u64 * PSG_CLOCK;
    }

    /// A write takes effect at once: a level it changes is the output from `t`, after the PSG's last clock.
    pub fn psg_write(&mut self, v: u8, t: u64) {
        let t = self.now(t);
        self.run_psg(t);
        self.psg.write(v);
        let out = self.psg.output();
        self.psg_steps.set(t, [out[0] / PSG_GAIN_DIV, out[1] / PSG_GAIN_DIV]);
    }

    pub fn ym_write(&mut self, port: u16, v: u8, t: u64) {
        let t = self.now(t);
        self.run_ym(t);
        self.ym.write(port, v, t);
    }

    /// The reset line the YM2612 shares with the Z80, changed at `t`.
    pub fn ym_reset(&mut self, held: bool, t: u64) {
        let t = self.now(t);
        self.run_ym(t);
        let steps = &mut self.fm_steps;
        self.ym.flush(t, |at, out| steps.set(at, [out[0] * FM_GAIN, out[1] * FM_GAIN]));
        self.ym.reset_line(held, t);
    }

    pub fn ym_read(&mut self, port: u16, t: u64) -> u8 {
        let t = self.now(t);
        self.run_ym(t);
        self.ym.read(port, t)
    }

    /// Both chips up to `t`, and every finished sample, the two sources summed and clamped.
    pub fn take(&mut self, t: u64, mut out: impl FnMut(i16, i16)) {
        let t = self.now(t);
        self.run_ym(t);
        self.run_psg(t);
        let fm = &mut self.fm_out;
        fm.clear();
        self.fm_steps.take(t, |l, r| fm.push([l, r]));
        let mut i = 0;
        self.psg_steps.take(t, |l, r| {
            let [fl, fr] = fm[i];
            i += 1;
            out((fl + l).clamp(-32768, 32767) as i16, (fr + r).clamp(-32768, 32767) as i16);
        });
        debug_assert_eq!(i, fm.len(), "both synthesisers finish the same samples");
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        self.ym.write_state(w);
        self.psg.write_state(w);
        w.u64s("PsgTimeAndTime", &[self.psg_time, self.time]);
        self.fm_steps.write_state(w);
        self.psg_steps.write_state(w);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        self.ym.read_state(r)?;
        self.psg.read_state(r)?;
        let mut t = [0u64; 2];
        r.u64s(&mut t)?;
        (self.psg_time, self.time) = (t[0], t[1]);
        self.fm_steps.read_state(r)?;
        self.psg_steps.read_state(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NTSC: (u64, u64) = (4_725_000_000, 88);

    /// The frequency of the strongest of a few candidates, by their amplitudes in `x` at 48 kHz.
    fn amplitude(x: &[f64], f: f64) -> f64 {
        let (mut re, mut im) = (0.0, 0.0);
        for (n, &v) in x.iter().enumerate() {
            let a = 2.0 * std::f64::consts::PI * f * n as f64 / RATE as f64;
            re += v * a.cos();
            im += v * a.sin();
        }
        2.0 * (re * re + im * im).sqrt() / x.len() as f64
    }

    /// A PSG tone of half-period $0FE is the Z80's clock over 16 × 2 × 254, 440.4 Hz on NTSC, in the output.
    #[test]
    fn a_psg_tone_comes_out_at_its_frequency() {
        let mut s = Sound::new(NTSC.0, NTSC.1, true);
        for v in [0x8E, 0x0F, 0x90] {
            s.psg_write(v, 0);
        }
        let mut left = Vec::new();
        let second = NTSC.0 / NTSC.1;
        s.take(second, |l, _| left.push(l as f64));
        let mean = left.iter().sum::<f64>() / left.len() as f64;
        let x: Vec<f64> = left.iter().map(|v| v - mean).collect();
        let f = 3_579_545.454_5 / 16.0 / 2.0 / 254.0;
        let at = amplitude(&x, f);
        assert!(at > 8192.0 / PSG_GAIN_DIV as f64 * 0.6, "{at} at {f} Hz");
        assert!(amplitude(&x, f * 1.05) < at * 0.05);
    }

    /// The Z80's write after the 68000's clock, then the frame's end at that earlier clock: both synthesisers finish
    /// the same samples, the frame ending where the write was.
    #[test]
    fn an_access_ahead_of_the_frames_end_moves_the_end_with_it() {
        let mut s = Sound::new(NTSC.0, NTSC.1, true);
        s.ym_write(0, 0x2B, 50_000);
        s.psg_write(0x90, 60_000);
        let mut n = 0;
        s.take(40_000, |_, _| n += 1);
        let mut more = 0;
        s.take(100_000, |_, _| more += 1);
        assert_eq!((n > 0, s.time), (true, 100_000));
        assert!(more > 0);
    }

    #[test]
    fn the_dac_and_the_psg_add() {
        let mut s = Sound::new(NTSC.0, NTSC.1, true);
        for (a, v) in [(0x2B, 0x80), (0x2A, 0xFF)] {
            s.ym_write(0, a, 0);
            s.ym_write(1, v, 0);
        }
        s.psg_write(0x81, 0);
        s.psg_write(0x00, 0);
        s.psg_write(0x90, 0);
        let mut last = (0, 0);
        s.take(200_000, |l, r| last = (l, r));
        let dac = 254 * FM_GAIN;
        let psg = 8192 / PSG_GAIN_DIV;
        assert_eq!(last, ((dac + psg) as i16, (dac + psg) as i16));
    }
}
