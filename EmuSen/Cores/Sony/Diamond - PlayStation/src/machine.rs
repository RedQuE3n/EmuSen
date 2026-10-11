//! The stage 1 machine: the image recognised, the memories allocated and zeroed, the two processors' registers, a frame
//! counter, a black picture and silence at the SPU's rate. Nothing is emulated (Diamond_Native.md §2).

use blackmoon_gte::Gte;
use blackmoon_r3000::R3000;

use crate::media::{Market, Media};

/// A memory the machine holds, by its space name and id (Diamond_Plan.md §4.3).
pub struct Memory {
    pub id: u32,
    pub name: &'static str,
    pub bytes: Vec<u8>,
    pub read_only: bool,
}

/// The memories as (id, name, size): main RAM, the scratchpad, VRAM, SPU RAM, and the BIOS ROM, read-only.
pub const MEMORIES: [(u32, &str, usize); 5] = [(1, "RAM", 0x20_0000), (2, "SCRATCH", 0x400), (3, "VRAM", 0x10_0000), (4, "SPURAM", 0x8_0000), (5, "BIOS", 0x8_0000)];
pub const RAM_ID: u32 = 1;
pub const BIOS_ID: u32 = 5;
pub const BIOS_BYTES: usize = 0x8_0000;

/// The SPU's rate, 33.8688 MHz over 768 (psx-spx "SPU"), at which the silence is made.
pub const AUDIO_RATE: i32 = 44_100;

/// The video clocks psx-spx states for each standard over a progressive frame's lines and clocks (263 of 3,413 NTSC,
/// 314 of 3,406 PAL); provisional until D-0 settles which clock an NTSC console runs (Diamond_Native.md §2.3).
pub const NTSC_FRAME: (u64, u64) = (53_693_175, 263 * 3413);
pub const PAL_FRAME: (u64, u64) = (53_203_425, 314 * 3406);

/// The picture before the GPU exists: 320 wide, 240 lines NTSC and 256 PAL; at most 640 by 512.
pub const WIDTH: u32 = 320;
pub const MAX_WIDTH: u32 = 640;
pub const MAX_HEIGHT: u32 = 512;

/// The two GPU families (psx-spx "GPU Versions").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpu {
    Pin208,
    Pin160,
}

/// What the player chose at create, resolved against the image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub market: Market,
    pub gpu: Gpu,
}

pub struct Machine {
    pub media: Media,
    pub config: Config,
    pub memories: Vec<Memory>,
    pub cpu: R3000,
    pub gte: Gte,
    pub frames: i64,
    pub picture: Vec<u8>,
    pub skip: bool,
    pub pads: [u32; 2],
    pub ports: [Option<&'static str>; 2],
    /// Silent stereo frames made and not yet drained, and the bound on them in samples.
    pub silence: usize,
    pub audio_limit: usize,
    /// The firmware files given at create, by number.
    pub firmware: Vec<u32>,
}

impl Machine {
    pub fn new(media: Media, config: Config, ports: [Option<&'static str>; 2]) -> Machine {
        let memories = MEMORIES.iter().map(|&(id, name, size)| Memory { id, name, bytes: vec![0; size], read_only: id == BIOS_ID }).collect();
        let picture = black(WIDTH, height(config.market));
        Machine {
            media,
            config,
            memories,
            cpu: R3000::new(),
            gte: Gte::new(),
            frames: 0,
            picture,
            skip: false,
            pads: [0; 2],
            ports,
            silence: 0,
            audio_limit: usize::MAX,
            firmware: Vec::new(),
        }
    }

    pub fn pal(&self) -> bool {
        self.config.market == Market::Europe
    }

    pub fn frame_rate(&self) -> (u64, u64) {
        if self.pal() { PAL_FRAME } else { NTSC_FRAME }
    }

    pub fn height(&self) -> u32 {
        height(self.config.market)
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

pub fn height(market: Market) -> u32 {
    if market == Market::Europe { 256 } else { 240 }
}

/// An opaque black frame, RGBA.
pub fn black(width: u32, height: u32) -> Vec<u8> {
    [0, 0, 0, 255].repeat((width * height) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::Kind;

    fn machine(market: Market) -> Machine {
        Machine::new(Media { kind: Kind::Disc { raw: false }, market: Some(market) }, Config { market, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None])
    }

    #[test]
    fn the_rates_are_the_documented_clocks_over_a_frame() {
        let hz = |(n, d): (u64, u64)| n as f64 / d as f64;
        assert!((hz(NTSC_FRAME) - 59.8173).abs() < 1e-4, "{}", hz(NTSC_FRAME));
        assert!((hz(PAL_FRAME) - 49.7468).abs() < 1e-4, "{}", hz(PAL_FRAME));
    }

    #[test]
    fn a_seconds_frames_make_a_seconds_silence_exactly() {
        for market in [Market::Americas, Market::Europe] {
            let mut m = machine(market);
            let (num, den) = m.frame_rate();
            let frames = (num * 10 / den) as i64;
            let made: usize = (0..frames).map(|n| m.samples_in(n)).sum();
            assert_eq!(made as u128, AUDIO_RATE as u128 * frames as u128 * den as u128 / num as u128);
            assert!((735..=887).contains(&m.samples_in(0)), "{}", m.samples_in(0));
            m.audio_limit = 1000;
            m.advance();
            m.advance();
            assert_eq!(m.silence, 500, "the bound drops the oldest");
        }
    }

    #[test]
    fn the_picture_is_black_and_the_bios_read_only() {
        let m = machine(Market::Europe);
        assert_eq!((m.picture.len(), m.height()), (320 * 256 * 4, 256));
        assert!(m.picture.chunks(4).all(|p| p == [0, 0, 0, 255]));
        assert_eq!(machine(Market::Japan).height(), 240);
        assert!(m.memory(BIOS_ID).unwrap().read_only && !m.memory(RAM_ID).unwrap().read_only);
        assert_eq!(m.memories.iter().map(|x| x.bytes.len()).sum::<usize>(), 0x20_0000 + 0x400 + 0x10_0000 + 0x8_0000 + 0x8_0000);
    }
}
