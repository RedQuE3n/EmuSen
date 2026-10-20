//! The machine (Endou_Native.md §9.6, §10, §11, §12): Beryl's Z80 on the bus, run a frame of the master clock at a time,
//! the Pause button's edge on NMI at the frame's start, the VDP's picture in the console's colours or on the Game Gear's
//! LCD (`gg`), the PSG's samples queued for the host, and the spaces a host reads.

use beryl_sms_vdp::Vdp;
use beryl_sn76489::Sn76489;
use beryl_z80::{Step, Z80};
use emusen_native::debug::{Hooks, run, stop};
use emusen_native::samples::SampleQueue;

use crate::board::{Board, identify};
use crate::boot::Boot;
use crate::bus::Bus;
use crate::cart::Cartridge;
use crate::gg::{self, Shrink};
use crate::io::PAUSE;
use crate::media::{Console, Market, Media};
use crate::model::Model;
use crate::sound::Sound;
use beryl_sms_colours::{SMS_COLOURS, SMS1_COLOURS, TMS_COLOURS};

/// The Z80's bus, a view numbered 0 as the plan numbers it, so that ids never move (Endou_Plan.md §4.3).
pub const Z80BUS_ID: u32 = 0;
pub const RAM_ID: u32 = 1;
pub const VRAM_ID: u32 = 2;
pub const CRAM_ID: u32 = 3;
pub const ROM_ID: u32 = 4;
pub const CARTRAM_ID: u32 = 5;
pub const BIOS_ID: u32 = 6;

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
    /// What the console ran before the game: none, the open boot program or the player's BIOS.
    pub boot: Boot,
    /// The CRC-32 of the BIOS's image, 0 for none, which a state is held to.
    pub bios_crc: u32,
    pub cpu: Z80,
    pub bus: Bus,
    pub frames: i64,
    /// The bus clock at which the frame being run ends.
    pub frame_end: u64,
    /// Whether the Pause button was held at the last frame's start, so that a press is one edge.
    pub pause_held: bool,
    /// The last picture in RGBA, `width` by `height` of it in use.
    pub picture: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// The Game Gear's LCD in a Master System cartridge's mode; none on another console.
    pub lcd: Option<Box<Shrink>>,
    pub skip: bool,
    pub pads: [u32; 2],
    /// The samples made and not yet drained, the oldest dropped past the host's limit.
    pub audio: SampleQueue,
    /// The debugger's tables and logs (`debug`).
    pub hooks: Hooks,
    /// A frame a debugger's stop left open, which the next run goes on with.
    pub debug_open: bool,
}

impl Machine {
    /// A machine for the image, its cartridge the bytes after any copier's header, on the board the image names.
    pub fn new(media: Media, market: Market, model: Model, image: &[u8]) -> Machine {
        let rom = image[media.skipped..].to_vec();
        let board = identify(&rom, media.console);
        Machine::with_board(media, market, model, board, rom)
    }

    /// A machine with no BIOS, the cartridge run from reset.
    pub fn with_board(media: Media, market: Market, model: Model, board: Board, rom: Vec<u8>) -> Machine {
        Machine::with_boot(media, market, model, board, rom, Boot::None, Vec::new())
    }

    /// A machine whose BIOS slot holds `bios`, the open boot program's image or the player's, run from reset; with
    /// `Boot::None`, none.
    pub fn with_boot(media: Media, market: Market, model: Model, board: Board, rom: Vec<u8>, boot: Boot, bios: Vec<u8>) -> Machine {
        let pal = market == Market::Europe && model.has_pal();
        let (num, _) = if pal { PAL_FRAME } else { NTSC_FRAME };
        let sound = Sound::with_fm(Sn76489::new(model.psg()), model.fm(), num, 15);
        let mut vdp = Vdp::new(model.vdp(), pal);
        vdp.gg_mode = model == Model::GameGear && media.gg_mode();
        let (width, height) = if model != Model::GameGear { (WIDTH, HEIGHT) } else if vdp.gg_mode { (gg::WIDTH, gg::WINDOW_HEIGHT) } else { (gg::WIDTH, gg::PANEL_HEIGHT) };
        let lcd = (model == Model::GameGear && !vdp.gg_mode).then(Box::default);
        let bios = if boot == Boot::None { Vec::new() } else { bios };
        let bios_crc = if bios.is_empty() { 0 } else { crate::board::crc32(&bios) };
        let bus = Bus::with_bios(model, Cartridge::new(board, rom), model.ram(), sound, vdp, bios);
        Machine {
            media,
            market,
            boot,
            bios_crc,
            cpu: Z80::power_on(),
            bus,
            frames: 0,
            frame_end: 0,
            pause_held: false,
            picture: [0, 0, 0, 255].repeat((WIDTH * MAX_HEIGHT) as usize),
            width,
            height,
            lcd,
            skip: false,
            pads: [0; 2],
            audio: SampleQueue::default(),
            hooks: Hooks::new(&[16]),
            debug_open: false,
        }
    }

    pub fn console(&self) -> Console {
        self.media.console
    }

    pub fn model(&self) -> Model {
        self.bus.model
    }

    pub fn pal(&self) -> bool {
        self.bus.vdp.pal
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

    /// The controller ports: two, the Game Gear's one.
    pub fn ports(&self) -> u32 {
        if self.model() == Model::GameGear { 1 } else { 2 }
    }

    /// CRAM's bytes: 32, or the Game Gear's 64 in its own mode.
    pub fn cram_len(&self) -> usize {
        if self.bus.vdp.gg_mode { 2 * CRAM } else { CRAM }
    }

    /// The spaces in id order: (id, name, read-only). An empty cartridge slot has no ROM, a console without a BIOS run no
    /// BIOS.
    pub fn spaces(&self) -> Vec<(u32, &'static str, bool)> {
        let mut s = vec![(RAM_ID, "RAM", false), (VRAM_ID, "VRAM", false)];
        if self.has_cram() {
            s.push((CRAM_ID, "CRAM", false));
        }
        if !self.bus.cart.rom.is_empty() {
            s.push((ROM_ID, "ROM", true));
        }
        if !self.bus.cart.ram.is_empty() {
            s.push((CARTRAM_ID, "CARTRAM", false));
        }
        if !self.bus.bios.is_empty() {
            s.push((BIOS_ID, "BIOS", true));
        }
        s
    }

    pub fn space(&self, id: u32) -> Option<&[u8]> {
        let b = &self.bus;
        Some(match id {
            RAM_ID => &b.ram,
            VRAM_ID => &b.vdp.vram,
            CRAM_ID if self.has_cram() => &b.vdp.cram[..self.cram_len()],
            ROM_ID if !b.cart.rom.is_empty() => &b.cart.rom,
            CARTRAM_ID if !b.cart.ram.is_empty() => &b.cart.ram,
            BIOS_ID if !b.bios.is_empty() => &b.bios,
            _ => return None,
        })
    }

    /// A writable space; the ROM is none.
    pub fn space_mut(&mut self, id: u32) -> Option<&mut [u8]> {
        let (sms, cram) = (self.has_cram(), self.cram_len());
        let b = &mut self.bus;
        Some(match id {
            RAM_ID => &mut b.ram,
            VRAM_ID => &mut b.vdp.vram,
            CRAM_ID if sms => &mut b.vdp.cram[..cram],
            CARTRAM_ID if !b.cart.ram.is_empty() => {
                b.cart.dirty = true;
                &mut b.cart.ram
            }
            _ => return None,
        })
    }

    /// One frame: the pads latched and a Pause press made an NMI edge at its start, then the Z80 run until the bus's
    /// clock passes the frame's end, the overshoot carried into the next frame, the VDP's picture taken and the sound's
    /// samples up to the bus's clock queued. A frame a debugger's stop left open is finished.
    pub fn advance(&mut self) {
        if !std::mem::take(&mut self.debug_open) {
            self.begin_frame();
        }
        while self.bus.clock < self.frame_end {
            self.cpu.step(&mut self.bus);
        }
        self.end_frame();
    }

    /// The frame through the debugger's observer: the stop's reasons, zero at the frame's end. A stop leaves the frame
    /// open in front of the instruction it names, every device brought to its time, and the next call goes on from it;
    /// `run::UNCHECKED` runs that instruction unasked (Endou_Native.md §15.3).
    pub fn run_frame_debug(&mut self, flags: u32) -> u32 {
        if !std::mem::take(&mut self.debug_open) {
            self.begin_frame();
        }
        self.bus.watch = self.hooks.writes;
        let mut watch = crate::debug::Watch { hooks: &mut self.hooks, unchecked: flags & run::UNCHECKED != 0 };
        while self.bus.clock < self.frame_end {
            let at = self.cpu.regs.pc as u32;
            let step = self.cpu.step_observed(&mut self.bus, &mut watch);
            for (space, offset, value) in self.bus.landed.drain(..) {
                watch.hooks.note_write(space, offset, value, at);
            }
            if let Step::Observed(why) = step {
                self.bus.watch = false;
                self.bus.settle();
                self.debug_open = true;
                return why;
            }
        }
        self.bus.watch = false;
        self.end_frame();
        stop::FRAME
    }

    /// What a frame takes at its start: the drawing switch, a Pause press as an NMI edge, the pads, and its end.
    fn begin_frame(&mut self) {
        self.bus.vdp.skip = self.skip;
        // In the Game Gear's own mode Start is a port software reads, not NMI (Endou_Plan.md §2.1; stage J2).
        let pause = self.pads[0] as u8 & PAUSE != 0 && !self.bus.vdp.gg_mode;
        if pause && !self.pause_held {
            self.bus.nmi = true;
        }
        self.pause_held = pause;
        self.bus.io.pads = [self.pads[0] as u8, self.pads[1] as u8];
        self.frame_end += self.frame_clocks();
    }

    /// The frame's end: every device at the bus's time, the picture taken and the sound queued.
    fn end_frame(&mut self) {
        self.bus.settle();
        if !self.skip {
            self.take_picture();
        }
        let audio = &mut self.audio;
        self.bus.sound.take(self.bus.clock, |l, r| audio.push_pair(l, r));
        self.frames += 1;
    }

    /// The VDP's colour codes in the console's colours: the 315-5124's or the 315-5246's 64 (Endou_Disputes.md D-1), or
    /// the TMS9918A's 16 (D-11); on the Game Gear its LCD's picture (`gg`).
    fn take_picture(&mut self) {
        let vdp = &self.bus.vdp;
        if self.model() == Model::GameGear {
            match self.lcd.as_mut() {
                Some(lcd) => lcd.draw(vdp.picture(), vdp.height, vdp.border(), &mut self.picture),
                None => gg::window(vdp.picture(), vdp.height, &mut self.picture),
            }
            return;
        }
        self.height = vdp.height;
        let (colours, mask): (&[[u8; 4]], usize) = match self.model().vdp() {
            beryl_sms_vdp::Variant::SMS1 => (&SMS1_COLOURS, 0x3F),
            beryl_sms_vdp::Variant::TMS9918A => (&TMS_COLOURS, 0x0F),
            _ => (&SMS_COLOURS, 0x3F),
        };
        for (out, &e) in self.picture.as_chunks_mut::<4>().0.iter_mut().zip(vdp.picture()) {
            out.copy_from_slice(&colours[e as usize & mask]);
        }
    }

    /// The picture's bytes in use.
    pub fn frame_bytes(&self) -> &[u8] {
        &self.picture[..(self.width * self.height) as usize * 4]
    }
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
