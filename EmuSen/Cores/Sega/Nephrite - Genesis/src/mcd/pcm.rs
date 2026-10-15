//! The RF5C164, the Sega CD's PCM sound source: eight channels reading 8-bit sign-magnitude samples from its 64 KiB of
//! wave RAM at a step each of 11 fraction bits, looping at a sample of `$FF`, each scaled by its envelope and its pan
//! and summed into a 16-bit pair with a limiter, a sample every 384 of its clocks. The Mega-CD Hardware Manual's "PCM
//! Sound Source" (§4) and Ricoh's RF5C68A data sheet are the sources; Nephrite_Native.md §58 is the record, D-38 what
//! they leave open.

/// Sub clocks a sample: the chip runs on the sub CPU's 12.5 MHz and makes a sample every 384 of them.
pub const SAMPLE: u64 = 384;
/// A sample's address: 16 integer bits above 11 of fraction.
const FRACTION: u32 = 11;
const LOOP: u8 = 0xFF;

/// One channel's registers, as the bank of `$07` selects them: ENV, PAN, FD, LS and ST.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Channel {
    pub env: u8,
    pub pan: u8,
    pub fd: u16,
    pub ls: u16,
    pub st: u8,
    /// The read address, 16.11; and whether the next sample reads ST's without a step (the first after it starts).
    pub addr: u32,
    pub fresh: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pcm {
    pub ch: [Channel; 8],
    /// `$07`: ON (sounding), and the two banks MOD writes: the wave RAM's 4 KiB window (WB) and the channel (CB).
    pub on: bool,
    pub wave_bank: u8,
    pub chan_bank: u8,
    /// `$08`: a channel is off while its bit is set.
    pub off: u8,
    /// The sub clock of the next sample, and the samples made and not yet mixed, each at its sub clock.
    pub next: u64,
    pub out: Vec<(u64, [i32; 2])>,
}

impl Default for Pcm {
    fn default() -> Pcm {
        Pcm { ch: [Channel { fresh: true, ..Channel::default() }; 8], on: false, wave_bank: 0, chan_bank: 0, off: 0xFF, next: SAMPLE, out: Vec::with_capacity(2048) }
    }
}

impl Pcm {
    /// The wave RAM's byte behind window offset `i` (0 to `$FFF`).
    pub fn window(&self, i: usize) -> usize {
        (self.wave_bank as usize & 15) << 12 | (i & 0xFFF)
    }

    /// The samples due by sub clock `now`, from `ram`.
    pub fn run(&mut self, now: u64, ram: &[u8]) {
        while self.next <= now {
            let t = self.next;
            let s = self.sample(ram);
            self.out.push((t, s));
            self.next += SAMPLE;
        }
    }

    /// One sample period: every channel's address moved or held, and the sounding channels summed.
    fn sample(&mut self, ram: &[u8]) -> [i32; 2] {
        if !self.on {
            return [0, 0];
        }
        let (mut l, mut r) = (0i32, 0i32);
        for (k, c) in self.ch.iter_mut().enumerate() {
            if self.off & 1 << k != 0 {
                // A channel not sounding holds its address at ST (the data sheet's "address pointer").
                (c.addr, c.fresh) = ((c.st as u32) << (8 + FRACTION), true);
                continue;
            }
            if !std::mem::take(&mut c.fresh) {
                c.addr = (c.addr + c.fd as u32) & ((1 << (16 + FRACTION)) - 1);
            }
            let mut b = ram[(c.addr >> FRACTION) as usize];
            if b == LOOP {
                // The loop's end: the address goes to LS and the RAM is read again there.
                c.addr = (c.ls as u32) << FRACTION;
                b = ram[c.ls as usize];
                if b == LOOP {
                    continue;
                }
            }
            // Sign and magnitude, bit 7 set for a positive sample; times ENV, times each side's PAN nibble, the top 14
            // of the 19 bits added to or taken from the sum (D-38).
            let m = (b & 0x7F) as i32 * c.env as i32;
            let (a, bside) = (m * (c.pan & 15) as i32 >> 5, m * (c.pan >> 4) as i32 >> 5);
            if b & 0x80 != 0 {
                (l, r) = (l + a, r + bside);
            } else {
                (l, r) = (l - a, r - bside);
            }
        }
        [l.clamp(-0x8000, 0x7FFF), r.clamp(-0x8000, 0x7FFF)]
    }

    /// A register read: `$10`-`$1F` give each channel's address, low byte first; the others read 0.
    pub fn read(&self, reg: u32) -> u8 {
        match reg {
            0x10..=0x1F => {
                let a = self.ch[(reg as usize - 0x10) / 2].addr >> FRACTION;
                if reg & 1 == 0 { a as u8 } else { (a >> 8) as u8 }
            }
            _ => 0,
        }
    }

    /// A register written: `$00`-`$06` to the channel CB selects, `$07` the control register, `$08` the channel
    /// switches.
    pub fn write(&mut self, reg: u32, v: u8) {
        let c = &mut self.ch[self.chan_bank as usize & 7];
        match reg {
            0 => c.env = v,
            1 => c.pan = v,
            2 => c.fd = (c.fd & 0xFF00) | v as u16,
            3 => c.fd = (c.fd & 0x00FF) | (v as u16) << 8,
            4 => c.ls = (c.ls & 0xFF00) | v as u16,
            5 => c.ls = (c.ls & 0x00FF) | (v as u16) << 8,
            6 => c.st = v,
            7 => {
                self.on = v & 0x80 != 0;
                if v & 0x40 != 0 {
                    self.chan_bank = v & 7;
                } else {
                    self.wave_bank = v & 15;
                }
            }
            8 => self.off = v,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chip(fd: u16) -> (Pcm, Vec<u8>) {
        let mut p = Pcm::default();
        p.write(7, 0x40);
        p.write(0, 0xFF);
        p.write(1, 0xFF);
        p.write(2, fd as u8);
        p.write(3, (fd >> 8) as u8);
        (p, vec![0u8; 0x1_0000])
    }

    #[test]
    fn the_address_steps_by_fd_from_st_and_loops_at_ff() {
        let (mut p, mut ram) = chip(0x0800);
        ram[..16].fill(0x80);
        ram[15] = LOOP;
        p.write(4, 3);
        p.write(8, 0xFE);
        p.write(7, 0x80);
        let addresses: Vec<u8> = (0..20).map(|k| {
            p.run(SAMPLE * (k + 1), &ram);
            p.read(0x10)
        }).collect();
        // The first sample reads ST's byte; then a byte a sample; the loop's marker sends it to LS, 3.
        assert_eq!(&addresses[..16], &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 3]);
        assert_eq!(&addresses[16..], &[4, 5, 6, 7]);
    }

    #[test]
    fn a_channel_off_holds_st_and_the_chip_off_holds_everything() {
        let (mut p, ram) = chip(0x0100);
        p.write(6, 2);
        p.write(8, 0xFF);
        p.write(7, 0x80);
        p.run(SAMPLE * 4, &ram);
        assert_eq!((p.read(0x10), p.read(0x11)), (0, 2), "off: at ST");
        p.write(8, 0xFE);
        p.run(SAMPLE * 4 + SAMPLE * 801, &ram);
        assert_eq!(p.read(0x10), 100, "800 steps of an eighth");
        p.write(7, 0x00);
        p.run(SAMPLE * 2000, &ram);
        assert_eq!(p.read(0x10), 100, "the chip off holds the address");
    }

    #[test]
    fn samples_are_sign_magnitude_scaled_by_env_and_pan_and_limited() {
        let (mut p, mut ram) = chip(0);
        ram[0] = 0x80 | 100;
        p.write(1, 0x0F);
        p.write(8, 0xFE);
        p.write(7, 0x80);
        p.run(SAMPLE, &ram);
        assert_eq!(p.out[0].1, [100 * 255 * 15 >> 5, 0], "left only");
        ram[0] = 100;
        p.run(SAMPLE * 2, &ram);
        assert_eq!(p.out[1].1, [-(100 * 255 * 15 >> 5), 0], "bit 7 clear is negative");
        // Eight channels at full scale overflow and are held at the limit.
        ram[0] = 0xFF - 1;
        for k in 0..8 {
            p.write(7, 0xC0 | k);
            p.write(0, 0xFF);
            p.write(1, 0xFF);
        }
        p.write(8, 0);
        p.run(SAMPLE * 3, &ram);
        assert_eq!(p.out[2].1, [0x7FFF, 0x7FFF]);
    }
}
