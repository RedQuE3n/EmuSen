//! The machine (Endou_Native.md §9.6): Beryl's Z80 on the bus, run a frame of the master clock at a time, the Pause
//! button's edge on NMI at the frame's start, and the spaces a host reads. No picture or sound is made before stages 3
//! and 5: the picture stays black and the output silent.

use beryl_sn76489::{Sn76489, Variant};
use beryl_z80::Z80;

use crate::board::{Board, identify};
use crate::bus::Bus;
use crate::cart::Cartridge;
use crate::io::PAUSE;
use crate::media::{Console, Market, Media};
use crate::model::Model;

pub const RAM_ID: u32 = 1;
pub const VRAM_ID: u32 = 2;
pub const CRAM_ID: u32 = 3;
pub const ROM_ID: u32 = 4;
pub const CARTRAM_ID: u32 = 5;

/// The Mark III's and the Master System's 8 KiB of RAM, the SG-1000's 1 KiB [D: `smstech` §2; `sc3000h`].
pub const SMS_RAM: usize = 0x2000;
pub const SG1000_RAM: usize = 0x400;
pub const VRAM: usize = 0x4000;
pub const CRAM: usize = 32;

/// The output rate, at which the silence is made; the mix resamples to it from stage 5.
pub const AUDIO_RATE: i32 = 48_000;

/// Z80 clocks a line, and lines a frame [D: `msvdp.txt` §1; Endou_Plan.md §5.1].
pub const LINE: u64 = 228;
pub const NTSC_LINES: u64 = 262;
pub const PAL_LINES: u64 = 313;

/// A frame as the master clock over 15 makes it: 228 Z80 cycles a line, 262 lines NTSC and 313 PAL, from 53.693175
/// and 53.203424 MHz (Endou_Plan.md §5.1, Endou_Native.md §2.4).
pub const NTSC_FRAME: (u64, u64) = (53_693_175, 15 * LINE * NTSC_LINES);
pub const PAL_FRAME: (u64, u64) = (53_203_424, 15 * LINE * PAL_LINES);

/// Mode 4's picture and the 315-5246's tallest: 256 by 192, at most 256 by 240.
pub const WIDTH: u32 = 256;
pub const HEIGHT: u32 = 192;
pub const MAX_HEIGHT: u32 = 240;

pub struct Machine {
    pub media: Media,
    pub market: Market,
    pub cpu: Z80,
    pub bus: Bus,
    pub frames: i64,
    /// The bus clock at which the frame being run ends.
    pub frame_end: u64,
    /// Whether the Pause button was held at the last frame's start, so that a press is one edge.
    pub pause_held: bool,
    pub picture: Vec<u8>,
    pub skip: bool,
    pub pads: [u32; 2],
    /// Silent stereo frames made and not yet drained, and the bound on them in samples.
    pub silence: usize,
    pub audio_limit: usize,
}

impl Machine {
    /// A machine for the image, its cartridge the bytes after any copier's header, on the board the image names.
    pub fn new(media: Media, market: Market, model: Model, image: &[u8]) -> Machine {
        let rom = image[media.skipped..].to_vec();
        let board = identify(&rom, media.console);
        Machine::with_board(media, market, model, board, rom)
    }

    pub fn with_board(media: Media, market: Market, model: Model, board: Board, rom: Vec<u8>) -> Machine {
        let sg = media.console == Console::Sg1000;
        let psg = Sn76489::new(if sg { Variant::DISCRETE } else { Variant::SEGA });
        let bus = Bus::new(model, Cartridge::new(board, rom), if sg { SG1000_RAM } else { SMS_RAM }, psg, if sg { 0 } else { CRAM });
        Machine {
            media,
            market,
            cpu: Z80::power_on(),
            bus,
            frames: 0,
            frame_end: 0,
            pause_held: false,
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

    pub fn model(&self) -> Model {
        self.bus.model
    }

    pub fn pal(&self) -> bool {
        self.market == Market::Europe
    }

    pub fn frame_rate(&self) -> (u64, u64) {
        if self.pal() { PAL_FRAME } else { NTSC_FRAME }
    }

    /// Z80 clocks a frame: 59,736 NTSC, 71,364 PAL.
    pub fn frame_clocks(&self) -> u64 {
        LINE * if self.pal() { PAL_LINES } else { NTSC_LINES }
    }

    /// The spaces in id order: (id, name, read-only).
    pub fn spaces(&self) -> Vec<(u32, &'static str, bool)> {
        let mut s = vec![(RAM_ID, "RAM", false), (VRAM_ID, "VRAM", false)];
        if !self.bus.cram.is_empty() {
            s.push((CRAM_ID, "CRAM", false));
        }
        s.push((ROM_ID, "ROM", true));
        if !self.bus.cart.ram.is_empty() {
            s.push((CARTRAM_ID, "CARTRAM", false));
        }
        s
    }

    pub fn space(&self, id: u32) -> Option<&[u8]> {
        let b = &self.bus;
        Some(match id {
            RAM_ID => &b.ram,
            VRAM_ID => &b.vram,
            CRAM_ID if !b.cram.is_empty() => &b.cram,
            ROM_ID => &b.cart.rom,
            CARTRAM_ID if !b.cart.ram.is_empty() => &b.cart.ram,
            _ => return None,
        })
    }

    /// A writable space; the ROM is none.
    pub fn space_mut(&mut self, id: u32) -> Option<&mut [u8]> {
        let b = &mut self.bus;
        Some(match id {
            RAM_ID => &mut b.ram,
            VRAM_ID => &mut b.vram,
            CRAM_ID if !b.cram.is_empty() => &mut b.cram,
            CARTRAM_ID if !b.cart.ram.is_empty() => {
                b.cart.dirty = true;
                &mut b.cart.ram
            }
            _ => return None,
        })
    }

    /// The stereo frames of silence frame `n` makes, so that every frame's count sums to the rate exactly.
    pub fn samples_in(&self, n: i64) -> usize {
        let (num, den) = self.frame_rate();
        let upto = |k: i64| (AUDIO_RATE as u128 * k as u128 * den as u128 / num as u128) as usize;
        upto(n + 1) - upto(n)
    }

    /// One frame: the pads latched and a Pause press made an NMI edge at its start, then the Z80 run until the bus's
    /// clock passes the frame's end, the overshoot carried into the next frame.
    pub fn advance(&mut self) {
        let pause = self.pads[0] as u8 & PAUSE != 0;
        if pause && !self.pause_held {
            self.bus.nmi = true;
        }
        self.pause_held = pause;
        self.bus.io.pads = [self.pads[0] as u8, self.pads[1] as u8];
        self.frame_end += self.frame_clocks();
        while self.bus.clock < self.frame_end {
            self.cpu.step(&mut self.bus);
        }
        self.bus.settle();
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
        let media = Media::read(image).unwrap();
        let model = if media.console == Console::Sg1000 { Model::Sg1000 } else { Model::Sms2 };
        Machine::new(media, market, model, image)
    }

    #[test]
    fn the_rates_are_the_master_clock_over_a_frame() {
        let hz = |(n, d): (u64, u64)| n as f64 / d as f64;
        assert!((hz(NTSC_FRAME) - 59.922743).abs() < 1e-6, "{}", hz(NTSC_FRAME));
        assert!((hz(PAL_FRAME) - 49.701459).abs() < 1e-6, "{}", hz(PAL_FRAME));
        assert_eq!((machine(&cartridge(0x8000, 4), Market::Americas).frame_clocks(), machine(&cartridge(0x8000, 4), Market::Europe).frame_clocks()), (59_736, 71_364));
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
        assert_eq!(sms.spaces(), [(RAM_ID, "RAM", false), (VRAM_ID, "VRAM", false), (CRAM_ID, "CRAM", false), (ROM_ID, "ROM", true)]);
        assert_eq!([RAM_ID, VRAM_ID, CRAM_ID, ROM_ID].map(|i| sms.space(i).unwrap().len()), [0x2000, 0x4000, 32, 0x8000]);
        let sg = machine(&[0x00; 0x2000], Market::Japan);
        assert_eq!(sg.spaces().iter().map(|s| (s.1, sg.space(s.0).unwrap().len())).collect::<Vec<_>>(), [("RAM", 0x400), ("VRAM", 0x4000), ("ROM", 0x2000)]);
        assert_eq!((sms.bus.psg.variant, sg.bus.psg.variant), (Variant::SEGA, Variant::DISCRETE));
        assert!(sms.picture.chunks(4).all(|p| p == [0, 0, 0, 255]) && sms.picture.len() == 256 * 192 * 4);
        assert_eq!(sms.cpu.regs, Z80::power_on().regs);
        let big = machine(&cartridge(0x20000, 4), Market::Americas);
        assert_eq!((big.spaces().last().copied(), big.space(CARTRAM_ID).map(<[u8]>::len)), (Some((CARTRAM_ID, "CARTRAM", false)), Some(0x8000)));
    }

    #[test]
    fn a_frame_runs_the_frame_s_clocks_and_carries_the_overshoot() {
        let mut m = machine(&cartridge(0x8000, 4), Market::Americas);
        m.advance();
        let first = m.bus.clock;
        assert!((59_736..59_736 + 23).contains(&first), "{first}");
        m.advance();
        assert!(m.bus.clock - 2 * 59_736 < 23, "{}", m.bus.clock);
        assert_eq!(m.frames, 2);
    }
}
