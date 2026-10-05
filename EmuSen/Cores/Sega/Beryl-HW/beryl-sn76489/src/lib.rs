//! The SN76489 programmable sound generator: three tone channels and a noise channel, each with a 2 dB attenuator,
//! written through one byte port, and the variant a console has (the noise register's width and taps). Sega's copy,
//! inside the Master System's, the Game Gear's and the Genesis's VDP, is `Variant::SEGA`. The chip's clock is the
//! caller's: `advance` counts internal clocks (the input clock over 16). Beryl_SN76489.md is the record; SMS
//! Power's "SN76489" document the source.

use emusen_native::{StateReader, StateWriter, Truncated};

/// The noise register: its width in bits and the bits fed back for white noise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Variant {
    pub width: u8,
    pub taps: u16,
}

impl Variant {
    /// The Master System's, the Game Gear's and the Genesis's: 16 bits, bits 0 and 3.
    pub const SEGA: Variant = Variant { width: 16, taps: 0x0009 };
    /// The discrete chip of the SG-1000, the ColecoVision and the BBC Micro: 15 bits, bits 0 and 1.
    pub const DISCRETE: Variant = Variant { width: 15, taps: 0x0003 };
}

/// A channel's level at each attenuation, 2 dB a step from 8,192, the last one silent.
pub const VOLUME: [i32; 16] = [8192, 6507, 5169, 4106, 3261, 2591, 2058, 1635, 1298, 1031, 819, 651, 517, 411, 326, 0];

/// The noise counter's reload for the shift rates 0-2; rate 3 takes tone 2's register.
const NOISE_RATES: [u16; 3] = [0x10, 0x20, 0x40];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sn76489 {
    pub variant: Variant,
    /// Tone 0-2's half-periods, the four attenuations, the noise register (mode bit 2, rate bits 0-1).
    pub tone: [u16; 3],
    pub volume: [u8; 4],
    pub noise: u8,
    /// The latched register: channel times two, plus one for its attenuation.
    pub latch: u8,
    /// Each channel's counter and output bit; the noise channel's bit drives its shift register.
    pub counter: [u16; 4],
    pub bit: [bool; 4],
    pub shift: u16,
    /// The Game Gear's stereo register: bits 0-3 put channels 0-3 on the right, bits 4-7 on the left.
    pub stereo: u8,
}

impl Sn76489 {
    /// The chip as Sega's integrated copy starts: tones and noise zero, every channel silent.
    pub fn new(variant: Variant) -> Sn76489 {
        Sn76489 { variant, tone: [0; 3], volume: [0xF; 4], noise: 0, latch: 0, counter: [0; 4], bit: [false; 4], shift: 1 << (variant.width - 1), stereo: 0xFF }
    }

    /// A byte at the port: a latch byte (bit 7 set) names a register and gives its low four bits; a data byte gives
    /// a tone's high six bits, or the low bits of an attenuation or of the noise register.
    pub fn write(&mut self, v: u8) {
        let d = if v & 0x80 != 0 {
            self.latch = (v >> 4) & 7;
            v & 0xF
        } else {
            v & 0x3F
        };
        let ch = (self.latch >> 1) as usize;
        let tone_data = v & 0x80 == 0;
        match (ch, self.latch & 1) {
            (_, 1) => self.volume[ch] = d & 0xF,
            (3, _) => {
                self.noise = d & 7;
                self.shift = 1 << (self.variant.width - 1);
            }
            _ if tone_data => self.tone[ch] = (self.tone[ch] & 0xF) | (d as u16) << 4,
            _ => self.tone[ch] = (self.tone[ch] & 0x3F0) | d as u16,
        }
    }

    /// The Game Gear's port $06.
    pub fn write_stereo(&mut self, v: u8) {
        self.stereo = v;
    }

    fn reload(&self, ch: usize) -> u16 {
        if ch < 3 {
            self.tone[ch]
        } else {
            match self.noise & 3 {
                3 => self.tone[2],
                r => NOISE_RATES[r as usize],
            }
        }
    }

    /// A tone channel whose half-period is 0 or 1 holds its output high.
    fn held(&self, ch: usize) -> bool {
        self.reload(ch) <= 1
    }

    /// The noise channel's bit went from 0 to 1: the register shifts right, its new top bit the taps' parity in white
    /// noise or bit 0 in "periodic" noise.
    fn shift_noise(&mut self) {
        let feedback = if self.noise & 4 != 0 { (self.shift & self.variant.taps).count_ones() as u16 & 1 } else { self.shift & 1 };
        self.shift = (self.shift >> 1) | feedback << (self.variant.width - 1);
    }

    /// What each channel puts out now: its bit (a held tone's is high; the noise channel's is the shift register's
    /// bit 0) at its attenuation.
    pub fn levels(&self) -> [i32; 4] {
        let mut l = [0; 4];
        for (ch, v) in l.iter_mut().enumerate() {
            let on = if ch == 3 { self.shift & 1 != 0 } else { self.bit[ch] || self.held(ch) };
            *v = if on { VOLUME[self.volume[ch] as usize] } else { 0 };
        }
        l
    }

    /// The left and right sums, each channel on the sides the stereo register gives it (both, but on a Game Gear).
    pub fn output(&self) -> [i32; 2] {
        let l = self.levels();
        let mut out = [0; 2];
        for (ch, &v) in l.iter().enumerate() {
            if self.stereo >> (4 + ch) & 1 != 0 {
                out[0] += v;
            }
            if self.stereo >> ch & 1 != 0 {
                out[1] += v;
            }
        }
        out
    }

    /// One internal clock: each counter counts down if it is not zero, and on reaching zero reloads and flips its bit.
    /// A held tone's counter rests, its output high whatever its bit.
    pub fn tick(&mut self) {
        for ch in 0..4 {
            if ch < 3 && self.held(ch) {
                continue;
            }
            if self.counter[ch] != 0 {
                self.counter[ch] -= 1;
            }
            if self.counter[ch] == 0 {
                self.counter[ch] = self.reload(ch);
                self.bit[ch] = !self.bit[ch];
                if ch == 3 && self.bit[3] {
                    self.shift_noise();
                }
            }
        }
    }

    /// `ticks` internal clocks, `changed(t, [left, right])` called at each clock after which the output differs, `t`
    /// counting from 1. Clocks where no counter reaches zero are passed over at once.
    pub fn advance(&mut self, ticks: u32, mut changed: impl FnMut(u32, [i32; 2])) {
        let mut done = 0;
        let mut last = self.output();
        while done < ticks {
            // The clocks until the first counter reaches zero: a counter at zero reaches it at the next clock.
            let next = (0..4).filter(|&ch| ch == 3 || !self.held(ch)).map(|ch| self.counter[ch].max(1) as u32).min().unwrap();
            let skip = (next - 1).min(ticks - done);
            if skip > 0 {
                for ch in 0..4 {
                    if ch == 3 || !self.held(ch) {
                        self.counter[ch] -= skip as u16;
                    }
                }
                done += skip;
                continue;
            }
            self.tick();
            done += 1;
            let now = self.output();
            if now != last {
                changed(done, now);
                last = now;
            }
        }
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        w.u16s("Tone", &self.tone);
        w.bytes("Volume", &self.volume);
        w.bytes("NoiseLatchStereo", &[self.noise, self.latch, self.stereo]);
        w.u16s("Counter", &self.counter);
        w.bools("Bit", &self.bit);
        w.u16("Shift", self.shift);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        r.u16s(&mut self.tone)?;
        r.bytes(&mut self.volume)?;
        let mut b = [0u8; 3];
        r.bytes(&mut b)?;
        (self.noise, self.latch, self.stereo) = (b[0], b[1], b[2]);
        r.u16s(&mut self.counter)?;
        r.bools(&mut self.bit)?;
        self.shift = r.u16()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn psg() -> Sn76489 {
        Sn76489::new(Variant::SEGA)
    }

    /// The document's examples of latch and data bytes.
    #[test]
    fn latch_and_data_bytes_fill_the_registers_as_the_document_shows() {
        let mut p = psg();
        p.write(0b1000_1110);
        p.write(0b0000_1111);
        assert_eq!(p.tone[0], 0x0FE);
        p.write(0b1011_1111);
        assert_eq!(p.volume[1], 0xF);
        p.write(0b1101_1111);
        p.write(0b0000_0000);
        assert_eq!(p.volume[2], 0, "a data byte after a volume latch is not ignored");
        p.write(0b1110_0101);
        assert_eq!(p.noise, 0b101);
        p.write(0b0000_0100);
        assert_eq!(p.noise, 0b100, "a data byte after a noise latch is not ignored");
    }

    /// The tone register changes at once: the latch byte's four bits, then the data byte's six.
    #[test]
    fn a_tone_takes_each_byte_as_it_comes() {
        let mut p = psg();
        p.tone[0] = 0x3FF;
        p.write(0b1000_0000);
        assert_eq!(p.tone[0], 0x3F0);
        p.write(0b0000_0000);
        assert_eq!(p.tone[0], 0x000);
        p.write(0b1000_1111);
        assert_eq!(p.tone[0], 0x00F);
        p.write(0b0011_1111);
        assert_eq!(p.tone[0], 0x3FF);
    }

    #[test]
    fn a_tone_flips_every_half_period_and_a_half_period_of_one_holds_it_high() {
        let mut p = psg();
        p.write(0x80 | 0x0E);
        p.write(0x0F);
        p.write(0x90);
        let mut flips = Vec::new();
        p.advance(254 * 6, |t, [l, _]| flips.push((t, l)));
        let times: Vec<u32> = flips.iter().map(|f| f.0).collect();
        assert_eq!(times, [1, 255, 509, 763, 1017, 1271], "the counter starts at zero and reloads at once");
        let mut q = psg();
        q.write(0x81);
        q.write(0x00);
        q.write(0x90);
        assert_eq!(q.output(), [8192, 8192]);
        let mut changes = 0;
        q.advance(1000, |_, _| changes += 1);
        assert_eq!(changes, 0);
    }

    #[test]
    fn writing_the_noise_register_resets_it_and_periodic_noise_is_one_in_sixteen() {
        let mut p = psg();
        p.shift = 0x1234;
        p.write(0xE0);
        assert_eq!(p.shift, 0x8000);
        let mut bits = Vec::new();
        for _ in 0..64 {
            p.shift_noise();
            bits.push(p.shift & 1);
        }
        assert_eq!(bits.iter().sum::<u16>(), 4);
        assert_eq!(bits.iter().position(|&b| b == 1), Some(14));
        // The noise channel's bit shifts the register at every second reload: rate 0 is a shift every 32 clocks.
        let mut q = psg();
        q.write(0xE0);
        q.write(0xF0);
        let shift = q.shift;
        q.advance(32, |_, _| {});
        assert_eq!(q.shift, shift >> 1);
        q.advance(32, |_, _| {});
        assert_eq!(q.shift, shift >> 2);
    }

    /// White noise: the top bit takes bits 0 and 3's parity.
    #[test]
    fn white_noise_feeds_back_bits_zero_and_three() {
        let mut p = psg();
        p.write(0xE4);
        p.shift = 0b1001;
        p.shift_noise();
        assert_eq!(p.shift, 0b0100);
        p.shift = 0b0001;
        p.shift_noise();
        assert_eq!(p.shift, 0x8000);
        let mut d = Sn76489::new(Variant::DISCRETE);
        d.write(0xE4);
        assert_eq!(d.shift, 0x4000);
        d.shift = 0b0011;
        d.shift_noise();
        assert_eq!(d.shift, 0b0001);
    }

    #[test]
    fn each_step_of_attenuation_is_two_decibels() {
        for k in 1..15 {
            let db = 20.0 * (VOLUME[k - 1] as f64 / VOLUME[k] as f64).log10();
            assert!((db - 2.0).abs() < 0.02, "step {k}: {db} dB");
        }
        assert_eq!(VOLUME[15], 0);
    }

    #[test]
    fn the_game_gear_stereo_register_picks_each_channels_sides() {
        let mut p = psg();
        p.write(0x81);
        p.write(0x90);
        p.write(0xA1);
        p.write(0xB4);
        p.write_stereo(0x21);
        assert_eq!(p.output(), [VOLUME[4], VOLUME[0]]);
    }

    #[test]
    fn passing_over_quiet_clocks_changes_nothing() {
        let mut a = psg();
        for v in [0x83, 0x05, 0x90, 0xA7, 0x1A, 0xB2, 0xC9, 0x02, 0xD5, 0xE5, 0xF3] {
            a.write(v);
        }
        let mut b = a.clone();
        let mut ea = Vec::new();
        a.advance(50_000, |t, o| ea.push((t, o)));
        let mut eb = Vec::new();
        let mut last = b.output();
        for t in 1..=50_000 {
            b.tick();
            if b.output() != last {
                last = b.output();
                eb.push((t, last));
            }
        }
        assert_eq!(ea, eb);
        assert_eq!(a, b);
    }
}
