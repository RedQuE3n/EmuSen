//! The machine (Endou_Native.md §9.6, §10, §11, §12): Beryl's Z80 on the bus, run a frame of the master clock at a time,
//! the Pause button's edge on NMI at the frame's start, the VDP's picture in the console's colours, the PSG's samples
//! queued for the host, and the spaces a host reads.

use beryl_sms_vdp::Vdp;
use beryl_sn76489::Sn76489;
use beryl_z80::Z80;
use emusen_native::samples::SampleQueue;

use crate::board::{Board, identify};
use crate::bus::Bus;
use crate::cart::Cartridge;
use crate::io::PAUSE;
use crate::media::{Console, Market, Media};
use crate::model::Model;
use crate::sound::Sound;

pub const RAM_ID: u32 = 1;
pub const VRAM_ID: u32 = 2;
pub const CRAM_ID: u32 = 3;
pub const ROM_ID: u32 = 4;
pub const CARTRAM_ID: u32 = 5;

pub const CRAM: usize = 32;

/// The output rate (`sound::RATE`).
pub const AUDIO_RATE: i32 = crate::sound::RATE as i32;

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
    /// The last picture in RGBA, `WIDTH` by `height` of it in use.
    pub picture: Vec<u8>,
    pub height: u32,
    pub skip: bool,
    pub pads: [u32; 2],
    /// The samples made and not yet drained, the oldest dropped past the host's limit.
    pub audio: SampleQueue,
}

impl Machine {
    /// A machine for the image, its cartridge the bytes after any copier's header, on the board the image names.
    pub fn new(media: Media, market: Market, model: Model, image: &[u8]) -> Machine {
        let rom = image[media.skipped..].to_vec();
        let board = identify(&rom, media.console);
        Machine::with_board(media, market, model, board, rom)
    }

    pub fn with_board(media: Media, market: Market, model: Model, board: Board, rom: Vec<u8>) -> Machine {
        let pal = market == Market::Europe;
        let (num, _) = if pal { PAL_FRAME } else { NTSC_FRAME };
        let sound = Sound::new(Sn76489::new(model.psg()), num, 15);
        let vdp = Vdp::new(model.vdp(), pal);
        let bus = Bus::new(model, Cartridge::new(board, rom), model.ram(), sound, vdp);
        Machine {
            media,
            market,
            cpu: Z80::power_on(),
            bus,
            frames: 0,
            frame_end: 0,
            pause_held: false,
            picture: [0, 0, 0, 255].repeat((WIDTH * MAX_HEIGHT) as usize),
            height: HEIGHT,
            skip: false,
            pads: [0; 2],
            audio: SampleQueue::default(),
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

    /// Whether the console has CRAM: every one but the SG-1000, whose TMS9918A has none.
    pub fn has_cram(&self) -> bool {
        self.model() != Model::Sg1000
    }

    /// The spaces in id order: (id, name, read-only).
    pub fn spaces(&self) -> Vec<(u32, &'static str, bool)> {
        let mut s = vec![(RAM_ID, "RAM", false), (VRAM_ID, "VRAM", false)];
        if self.has_cram() {
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
            VRAM_ID => &b.vdp.vram,
            CRAM_ID if self.has_cram() => &b.vdp.cram[..CRAM],
            ROM_ID => &b.cart.rom,
            CARTRAM_ID if !b.cart.ram.is_empty() => &b.cart.ram,
            _ => return None,
        })
    }

    /// A writable space; the ROM is none.
    pub fn space_mut(&mut self, id: u32) -> Option<&mut [u8]> {
        let sms = self.has_cram();
        let b = &mut self.bus;
        Some(match id {
            RAM_ID => &mut b.ram,
            VRAM_ID => &mut b.vdp.vram,
            CRAM_ID if sms => &mut b.vdp.cram[..CRAM],
            CARTRAM_ID if !b.cart.ram.is_empty() => {
                b.cart.dirty = true;
                &mut b.cart.ram
            }
            _ => return None,
        })
    }

    /// One frame: the pads latched and a Pause press made an NMI edge at its start, then the Z80 run until the bus's
    /// clock passes the frame's end, the overshoot carried into the next frame, the VDP's picture taken and the sound's
    /// samples up to the bus's clock queued.
    pub fn advance(&mut self) {
        self.bus.vdp.skip = self.skip;
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
        if !self.skip {
            self.take_picture();
        }
        let audio = &mut self.audio;
        self.bus.sound.take(self.bus.clock, |l, r| audio.push_pair(l, r));
        self.frames += 1;
    }

    /// The VDP's colour codes in the console's colours: the 315-5124's or the 315-5246's 64 (Endou_Disputes.md D-1), or
    /// the TMS9918A's 16 (D-11).
    fn take_picture(&mut self) {
        let vdp = &self.bus.vdp;
        self.height = vdp.height;
        let (colours, mask): (&[[u8; 4]], usize) = match self.model() {
            Model::MarkIII | Model::Sms1 => (&SMS1_COLOURS, 0x3F),
            Model::Sms2 => (&SMS_COLOURS, 0x3F),
            Model::Sg1000 => (&TMS_COLOURS, 0x0F),
        };
        for (out, &e) in self.picture.as_chunks_mut::<4>().0.iter_mut().zip(vdp.picture()) {
            out.copy_from_slice(&colours[e as usize & mask]);
        }
    }

    /// The picture's bytes in use.
    pub fn frame_bytes(&self) -> &[u8] {
        &self.picture[..(WIDTH * self.height) as usize * 4]
    }
}

/// A two-bit component's level on the 315-5246, four equal steps (Endou_Disputes.md D-1).
pub const LEVELS: [u8; 4] = [0, 85, 170, 255];

/// The 315-5124's blue, its first level boosted to #68, read from SMS Power!'s "Palette", not measured (D-1, decided
/// 2026-10-19).
pub const BLUE_5124: [u8; 4] = [0, 0x68, 170, 255];

const fn colours(blue: [u8; 4]) -> [[u8; 4]; 64] {
    let mut t = [[0u8; 4]; 64];
    let mut i = 0;
    while i < 64 {
        t[i] = [LEVELS[i & 3], LEVELS[(i >> 2) & 3], blue[(i >> 4) & 3], 255];
        i += 1;
    }
    t
}

/// The 64 colours of a `--BBGGRR` entry as RGBA: the 315-5246's (the Master System II), and the 315-5124's (the Mark III
/// and the first Master System).
pub const SMS_COLOURS: [[u8; 4]; 64] = colours(LEVELS);
pub const SMS1_COLOURS: [[u8; 4]; 64] = colours(BLUE_5124);

/// The TMS9918A's 16 colours: TI's colour difference levels (TMS9918A data manual, Table 2-3) as RGB, each difference
/// taken from 0.47, rounded to the nearest of 255 steps, held by a test (Endou_Disputes.md D-11).
pub const TMS_COLOURS: [[u8; 4]; 16] = [
    [0x00, 0x00, 0x00, 255],
    [0x00, 0x00, 0x00, 255],
    [0x21, 0xC8, 0x42, 255],
    [0x5E, 0xDC, 0x78, 255],
    [0x54, 0x55, 0xED, 255],
    [0x7D, 0x76, 0xFC, 255],
    [0xD4, 0x52, 0x4D, 255],
    [0x42, 0xEC, 0xF5, 255],
    [0xFC, 0x55, 0x54, 255],
    [0xFF, 0x79, 0x78, 255],
    [0xD4, 0xC1, 0x54, 255],
    [0xE6, 0xCE, 0x80, 255],
    [0x21, 0xB0, 0x3B, 255],
    [0xC9, 0x5B, 0xBA, 255],
    [0xCC, 0xCC, 0xCC, 255],
    [0xFF, 0xFF, 0xFF, 255],
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::sample::cartridge;

    fn machine(image: &[u8], market: Market) -> Machine {
        let media = Media::read(image).unwrap();
        let model = if media.console == Console::Sg1000 { Model::Sg1000 } else { Model::Sms2 };
        Machine::new(media, market, model, image)
    }

    /// TI's Table 2-3, the TMS9928A's luminance and colour difference levels in volts for codes 1-15.
    const TI_LEVELS: [(f64, f64, f64); 15] = [
        (0.00, 0.47, 0.47),
        (0.53, 0.07, 0.20),
        (0.67, 0.17, 0.27),
        (0.40, 0.40, 1.00),
        (0.53, 0.43, 0.93),
        (0.47, 0.83, 0.30),
        (0.73, 0.00, 0.70),
        (0.53, 0.93, 0.27),
        (0.67, 0.93, 0.27),
        (0.73, 0.57, 0.07),
        (0.80, 0.57, 0.17),
        (0.47, 0.13, 0.23),
        (0.53, 0.73, 0.67),
        (0.80, 0.47, 0.47),
        (1.00, 0.47, 0.47),
    ];

    #[test]
    fn the_tms9918a_s_colours_are_ti_s_levels_as_rgb() {
        for (code, &(y, ry, by)) in TI_LEVELS.iter().enumerate() {
            let (r, b) = (y + ry - 0.47, y + by - 0.47);
            let g = (y - 0.299 * r - 0.114 * b) / 0.587;
            let q = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            assert_eq!(TMS_COLOURS[code + 1], [q(r), q(g), q(b), 255], "code {}", code + 1);
        }
        assert_eq!(TMS_COLOURS[0], TMS_COLOURS[1], "transparent over no external video is black");
    }

    #[test]
    fn the_315_5124_boosts_only_its_first_blue_level() {
        let differ: Vec<usize> = (0..64).filter(|&i| SMS1_COLOURS[i] != SMS_COLOURS[i]).collect();
        assert_eq!(differ, (16..32).collect::<Vec<_>>());
        assert_eq!((SMS1_COLOURS[0x10], SMS_COLOURS[0x10]), ([0, 0, 0x68, 255], [0, 0, 0x55, 255]));
    }

    #[test]
    fn the_rates_are_the_master_clock_over_a_frame() {
        let hz = |(n, d): (u64, u64)| n as f64 / d as f64;
        assert!((hz(NTSC_FRAME) - 59.922743).abs() < 1e-6, "{}", hz(NTSC_FRAME));
        assert!((hz(PAL_FRAME) - 49.701459).abs() < 1e-6, "{}", hz(PAL_FRAME));
        assert_eq!((machine(&cartridge(0x8000, 4), Market::Americas).frame_clocks(), machine(&cartridge(0x8000, 4), Market::Europe).frame_clocks()), (59_736, 71_364));
    }

    /// Ten seconds of frames make ten seconds of samples, to the one the synthesiser holds back and the instruction a
    /// frame overruns by; the queue's bound drops the oldest.
    #[test]
    fn a_seconds_frames_make_a_seconds_samples() {
        for market in [Market::Americas, Market::Europe] {
            let mut m = machine(&cartridge(0x8000, 4), market);
            let (num, den) = m.frame_rate();
            let frames = (num * 10 / den) as i64;
            let mut made = 0usize;
            let mut out = vec![0i16; 4096];
            for _ in 0..frames {
                m.advance();
                made += m.audio.drain(&mut out, usize::MAX) / 2;
            }
            let want = AUDIO_RATE as f64 * frames as f64 * den as f64 / num as f64;
            assert!((made as f64 - want).abs() < 2.0, "{made} for {want}");
            m.audio.set_limit(1000);
            m.advance();
            m.advance();
            assert_eq!(m.audio.len(), 1000, "the bound drops the oldest");
        }
    }

    #[test]
    fn each_console_has_its_memories_and_its_psg() {
        let sms = machine(&cartridge(0x8000, 4), Market::Americas);
        assert_eq!(sms.spaces(), [(RAM_ID, "RAM", false), (VRAM_ID, "VRAM", false), (CRAM_ID, "CRAM", false), (ROM_ID, "ROM", true)]);
        assert_eq!([RAM_ID, VRAM_ID, CRAM_ID, ROM_ID].map(|i| sms.space(i).unwrap().len()), [0x2000, 0x4000, 32, 0x8000]);
        let sg = machine(&[0x00; 0x2000], Market::Japan);
        assert_eq!(sg.spaces().iter().map(|s| (s.1, sg.space(s.0).unwrap().len())).collect::<Vec<_>>(), [("RAM", 0x400), ("VRAM", 0x4000), ("ROM", 0x2000)]);
        assert_eq!((sms.bus.sound.psg.variant, sg.bus.sound.psg.variant), (beryl_sn76489::Variant::SEGA, beryl_sn76489::Variant::DISCRETE));
        assert!(sms.frame_bytes().chunks(4).all(|p| p == [0, 0, 0, 255]) && sms.frame_bytes().len() == 256 * 192 * 4);
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
