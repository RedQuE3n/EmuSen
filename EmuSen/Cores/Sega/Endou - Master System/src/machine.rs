//! The stage 1 machine (Endou_Native.md §2): the memories allocated and zeroed, Beryl's Z80 and SN76489 at reset in
//! the variant each console carries, a frame counter, a black picture and silence at the output rate. Nothing runs.

use beryl_sn76489::{Sn76489, Variant};
use beryl_z80::Z80;

use crate::media::{Console, Market, Media};

/// A memory the machine holds, by its space id and name (Endou_Plan.md §4.3).
pub struct Memory {
    pub id: u32,
    pub name: &'static str,
    pub bytes: Vec<u8>,
    pub read_only: bool,
}

pub const RAM_ID: u32 = 1;
pub const VRAM_ID: u32 = 2;
pub const CRAM_ID: u32 = 3;
pub const ROM_ID: u32 = 4;

/// The Mark III's and the Master System's 8 KiB of RAM, the SG-1000's 1 KiB [D: `smstech` §2; `sc3000h`].
pub const SMS_RAM: usize = 0x2000;
pub const SG1000_RAM: usize = 0x400;
pub const VRAM: usize = 0x4000;
pub const CRAM: usize = 32;

/// The output rate, at which the silence is made; the mix resamples to it from stage 5.
pub const AUDIO_RATE: i32 = 48_000;

/// A frame as the master clock over 15 makes it: 228 Z80 cycles a line, 262 lines NTSC and 313 PAL, from 53.693175
/// and 53.203424 MHz (Endou_Plan.md §5.1, Endou_Native.md §2.4).
pub const NTSC_FRAME: (u64, u64) = (53_693_175, 15 * 228 * 262);
pub const PAL_FRAME: (u64, u64) = (53_203_424, 15 * 228 * 313);

/// Mode 4's picture and the 315-5246's tallest: 256 by 192, at most 256 by 240.
pub const WIDTH: u32 = 256;
pub const HEIGHT: u32 = 192;
pub const MAX_HEIGHT: u32 = 240;

pub struct Machine {
    pub media: Media,
    pub market: Market,
    pub memories: Vec<Memory>,
    pub cpu: Z80,
    pub psg: Sn76489,
    pub frames: i64,
    pub picture: Vec<u8>,
    pub skip: bool,
    pub pads: [u32; 2],
    /// Silent stereo frames made and not yet drained, and the bound on them in samples.
    pub silence: usize,
    pub audio_limit: usize,
}

impl Machine {
    /// A machine for the image, its cartridge the bytes after any copier's header.
    pub fn new(media: Media, market: Market, image: &[u8]) -> Machine {
        let rom = image[media.skipped..].to_vec();
        let mut memories = vec![Memory { id: RAM_ID, name: "RAM", bytes: vec![0; ram_bytes(media.console)], read_only: false }, Memory { id: VRAM_ID, name: "VRAM", bytes: vec![0; VRAM], read_only: false }];
        if media.console == Console::Sms {
            memories.push(Memory { id: CRAM_ID, name: "CRAM", bytes: vec![0; CRAM], read_only: false });
        }
        memories.push(Memory { id: ROM_ID, name: "ROM", bytes: rom, read_only: true });
        let variant = if media.console == Console::Sg1000 { Variant::DISCRETE } else { Variant::SEGA };
        Machine {
            media,
            market,
            memories,
            cpu: Z80::new(),
            psg: Sn76489::new(variant),
            frames: 0,
            picture: [0, 0, 0, 255].repeat((WIDTH * HEIGHT) as usize),
            skip: false,
            pads: [0; 2],
            silence: 0,
            audio_limit: usize::MAX,
        }
    }

    pub fn console(&self) -> Console {
        self.media.console
    }

    pub fn pal(&self) -> bool {
        self.market == Market::Europe
    }

    pub fn frame_rate(&self) -> (u64, u64) {
        if self.pal() { PAL_FRAME } else { NTSC_FRAME }
    }

    pub fn memory(&self, id: u32) -> Option<&Memory> {
        self.memories.iter().find(|m| m.id == id)
    }

    pub fn memory_mut(&mut self, id: u32) -> Option<&mut Memory> {
        self.memories.iter_mut().find(|m| m.id == id)
    }

    /// The stereo frames of silence frame `n` makes, so that every frame's count sums to the rate exactly.
    pub fn samples_in(&self, n: i64) -> usize {
        let (num, den) = self.frame_rate();
        let upto = |k: i64| (AUDIO_RATE as u128 * k as u128 * den as u128 / num as u128) as usize;
        upto(n + 1) - upto(n)
    }

    /// One frame: the count moves and the frame's silence is queued; nothing else does.
    pub fn advance(&mut self) {
        self.silence = (self.silence + self.samples_in(self.frames)).min(self.audio_limit / 2);
        self.frames += 1;
    }
}

pub fn ram_bytes(console: Console) -> usize {
    if console == Console::Sg1000 { SG1000_RAM } else { SMS_RAM }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::sample::cartridge;

    fn machine(image: &[u8], market: Market) -> Machine {
        Machine::new(Media::read(image).unwrap(), market, image)
    }

    #[test]
    fn the_rates_are_the_master_clock_over_a_frame() {
        let hz = |(n, d): (u64, u64)| n as f64 / d as f64;
        assert!((hz(NTSC_FRAME) - 59.922743).abs() < 1e-6, "{}", hz(NTSC_FRAME));
        assert!((hz(PAL_FRAME) - 49.701459).abs() < 1e-6, "{}", hz(PAL_FRAME));
    }

    #[test]
    fn a_seconds_frames_make_a_seconds_silence_exactly() {
        for market in [Market::Americas, Market::Europe] {
            let mut m = machine(&cartridge(0x8000, 4), market);
            let (num, den) = m.frame_rate();
            let frames = (num * 10 / den) as i64;
            let made: usize = (0..frames).map(|n| m.samples_in(n)).sum();
            assert_eq!(made as u128, AUDIO_RATE as u128 * frames as u128 * den as u128 / num as u128);
            assert!((800..=966).contains(&m.samples_in(0)), "{}", m.samples_in(0));
            m.audio_limit = 1000;
            m.advance();
            m.advance();
            assert_eq!(m.silence, 500, "the bound drops the oldest");
        }
    }

    #[test]
    fn each_console_has_its_memories_and_its_psg() {
        let sms = machine(&cartridge(0x8000, 4), Market::Americas);
        assert_eq!(sms.memories.iter().map(|m| (m.name, m.bytes.len(), m.read_only)).collect::<Vec<_>>(), [("RAM", 0x2000, false), ("VRAM", 0x4000, false), ("CRAM", 32, false), ("ROM", 0x8000, true)]);
        let sg = machine(&[0x00; 0x2000], Market::Japan);
        assert_eq!(sg.memories.iter().map(|m| (m.name, m.bytes.len())).collect::<Vec<_>>(), [("RAM", 0x400), ("VRAM", 0x4000), ("ROM", 0x2000)]);
        assert_eq!((sms.psg.variant, sg.psg.variant), (Variant::SEGA, Variant::DISCRETE));
        assert!(sms.picture.chunks(4).all(|p| p == [0, 0, 0, 255]) && sms.picture.len() == 256 * 192 * 4);
        assert_eq!(sms.cpu.regs, beryl_z80::Registers::default());
    }
}
