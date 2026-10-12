//! The machine: the R3000A stepping against the bus, a frame ending as a field's vblank begins with the SPU brought to
//! it, a PS-EXE side-loaded on the kernel it runs or started directly, the TTY, and port 1's pad (Diamond_Native.md
//! §11, §12; Diamond_Pad.md; Diamond_SPU.md).

use blackmoon_r3000::{R3000, Step};

use crate::bus::{BIOS_BYTES, Board, RAM_BYTES, SCRATCH_BYTES, power_on_memctl};
use crate::devices::TTY_BYTES;
use crate::spu::RAM_BYTES as SPU_RAM_BYTES;
use crate::gpu::display;
use crate::gpu::vram::BYTES as VRAM_BYTES;
use crate::icache::KERNEL_BCC;
use crate::kernel::{self, Kernel};
use crate::media::{Kind, Market, Media};
use crate::memctl;
use crate::card::Card;
use crate::multitap::Multitap;
use crate::pad;
use crate::sio0::{Port, Sio0};
use crate::video::Video;
use crate::exe;

/// The spaces as (id, name, size, read-only) (Diamond_Plan.md §4.3), the TTY's text last.
pub const SPACES: [(u32, &str, usize, bool); 6] =
    [(1, "RAM", RAM_BYTES, false), (2, "SCRATCH", SCRATCH_BYTES, false), (3, "VRAM", VRAM_BYTES, false), (4, "SPURAM", SPU_RAM_BYTES, false), (5, "BIOS", BIOS_BYTES, true), (6, "TTY", TTY_BYTES, true)];
pub const RAM_ID: u32 = 1;
pub const BIOS_ID: u32 = 5;
pub const TTY_ID: u32 = 6;

/// The SPU's rate, 33.8688 MHz over 768 (psx-spx "SPU").
pub const AUDIO_RATE: i32 = 44_100;
pub const CYCLES_PER_SAMPLE: u64 = crate::spu::CYCLES_PER_SAMPLE;

/// The nominal picture, 320 wide by 240 lines NTSC and 256 PAL, and the largest the display makes (Diamond_GPU.md §7).
pub const WIDTH: u32 = 320;
pub const MAX_WIDTH: u32 = display::MAX_WIDTH;
pub const MAX_HEIGHT: u32 = display::MAX_HEIGHT;

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
    pub cpu: R3000,
    pub board: Board,
    pub kernel: Kernel,
    /// The PS-EXE the kernel's shell entry is to receive, while it waits, and the program kept for a state that loads
    /// from before the side-load.
    pub exe: Option<Vec<u8>>,
    pub image: Option<Vec<u8>>,
    pub frames: i64,
    pub skip: bool,
    /// Whether each card's change was reported to the host since it last saved the card.
    pub card_reported: [bool; 2],
    pub ports: [Option<&'static str>; 2],
    /// The firmware files given at create, by number.
    pub firmware: Vec<u32>,
    /// The fingerprint of the disc in the drive, 0 for none, which a state records and a load is held to (§6.23 of
    /// EmuSen_CoreAPI.md).
    pub disc_print: u64,
}

/// The console's standard field rate, by its market: NTSC on an NTSC clock, PAL on a PAL one (D-0, provisional).
pub fn standard_rate(pal: bool) -> (u64, u64) {
    let mut v = Video::new(pal);
    v.mode = (pal as u32) << 3;
    v.field_rate()
}

impl Machine {
    /// A machine on `bios`, the kernel image to run (a player's or the open kernel's), or on no kernel.
    pub fn new(media: Media, config: Config, ports: [Option<&'static str>; 2], image: &[u8], bios: Option<(&[u8], Kernel)>) -> Machine {
        let pal = config.market == Market::Europe;
        let version = if config.gpu == Gpu::Pin208 { 2 } else { 0 };
        let kernel = bios.map_or(Kernel::None, |b| b.1);
        let mut board = if kernel.boots() {
            Board::new(pal, version, power_on_memctl(), memctl::KERNEL_RAM_SIZE, 0)
        } else {
            Board::new(pal, version, memctl::KERNEL_VALUES, memctl::KERNEL_RAM_SIZE, KERNEL_BCC)
        };
        board.sio0 = Sio0::new(ports.map(|k| Port { controller: k.and_then(pad::controller), ..Port::default() }));
        let mut cpu = R3000::new();
        let is_exe = matches!(media.kind, Kind::Exe { .. });
        let mut exe = None;
        if let Some((rom, _)) = bios {
            board.bios.copy_from_slice(rom);
            cpu.regs.pc = 0xBFC0_0000;
            cpu.regs.sr = 1 << 22;
            if is_exe {
                exe = Some(image.to_vec());
            }
        } else if is_exe {
            exe::load(image, &mut board.ram, &mut cpu);
        } else {
            cpu.regs.pc = 0xBFC0_0000;
            cpu.regs.sr = 1 << 22;
        }
        Machine {
            media,
            config,
            cpu,
            board,
            kernel,
            image: exe.clone(),
            exe,
            frames: 0,
            skip: false,
            card_reported: [false; 2],
            ports,
            firmware: Vec::new(),
            disc_print: 0,
        }
    }

    /// Puts `disc` in the drive, its lid closed, as at power-on.
    pub fn insert_disc(&mut self, mut disc: emusen_disc::Disc) {
        self.disc_print = crate::medium::fingerprint(&mut disc);
        self.board.cdrom = crate::cdrom::Cdrom::new(Some(disc), self.config.market);
        self.board.schedule();
    }

    /// Replaces the disc in the drive, or empties it, whatever the lid: the lid open, as a player would, or for a state
    /// about to load, which brings the drive's own registers.
    pub fn change_disc(&mut self, disc: Option<emusen_disc::Disc>) {
        let now = self.board.now;
        self.board.cdrom.run(now);
        self.disc_print = disc.map_or(0, |mut d| {
            let print = crate::medium::fingerprint(&mut d);
            self.board.cdrom.disc = Some(crate::cdrom::DiscSlot::new(d));
            print
        });
        if self.disc_print == 0 {
            self.board.cdrom.disc = None;
        }
        self.board.sync_cdrom();
        self.board.schedule();
    }

    /// Opens the lid, or closes it on `disc` (another, or the same one when `None`).
    pub fn set_lid(&mut self, open: bool, disc: Option<emusen_disc::Disc>) {
        let now = self.board.now;
        if open {
            self.board.cdrom.open_lid(now);
        } else {
            self.board.cdrom.close_lid(now, disc);
        }
        self.board.sync_cdrom();
        self.board.schedule();
    }

    pub fn pal(&self) -> bool {
        self.config.market == Market::Europe
    }

    pub fn frame_rate(&self) -> (u64, u64) {
        standard_rate(self.pal())
    }

    pub fn height(&self) -> u32 {
        height(self.config.market)
    }

    pub fn space(&self, id: u32) -> Option<&[u8]> {
        let b = &self.board;
        Some(match id {
            1 => &b.ram,
            2 => &b.scratch,
            3 => &b.gpu.vram.bytes,
            4 => &b.spu.ram,
            5 => &b.bios,
            6 => &b.tty.text,
            _ => return None,
        })
    }

    pub fn space_mut(&mut self, id: u32) -> Option<&mut [u8]> {
        let b = &mut self.board;
        Some(match id {
            1 => &mut b.ram,
            2 => &mut b.scratch,
            3 => &mut b.gpu.vram.bytes,
            4 => &mut b.spu.ram,
            _ => return None,
        })
    }

    /// One step at the boundary, with the kernel's seams: the side-load at the shell's entry and the putchar tap.
    pub fn step(&mut self) {
        let pc = self.cpu.regs.pc;
        if self.exe.is_some() && pc & 0x1FFF_FFFF == kernel::SHELL_ENTRY && !self.cpu.regs.branch.slot {
            let image = self.exe.take().unwrap_or_default();
            exe::load(&image, &mut self.board.ram, &mut self.cpu);
            self.board.cache.tags = [0; 256];
        }
        let tapped = if self.kernel.taps_putchar() { kernel::putchar(self.cpu.regs.pc, self.cpu.regs.r[9], self.cpu.regs.r[4]) } else { None };
        let step = self.cpu.step(&mut self.board);
        if let (Some(c), Step::Instruction) = (tapped, step) {
            self.board.tty.put(c);
        }
        if self.board.now >= self.board.next_event {
            self.board.events();
        }
    }

    /// One frame: the machine runs until a field's vblank begins, its picture complete, or to the field's end when the
    /// display range leaves no vblank; the SPU's samples are made up to that cycle.
    pub fn advance(&mut self) {
        self.board.field_done = false;
        while !self.board.field_done {
            self.step();
        }
        self.board.sync_spu();
        self.frames += 1;
    }
}

impl Machine {
    /// A card in port `n`'s slot (or its multitap's slot A), holding `data` or unformatted.
    pub fn insert_card(&mut self, n: usize, data: Option<&[u8]>) {
        let p = &mut self.board.sio0.ports[n];
        match &mut p.tap {
            Some(t) => t.cards[0] = Some(Card::new(data)),
            None => p.card = Some(Card::new(data)),
        }
    }

    /// The card in port `n`'s slot, or in its multitap's slot A.
    pub fn card_in(&self, n: usize) -> Option<&Card> {
        let p = &self.board.sio0.ports[n];
        p.tap.as_ref().map_or(p.card.as_ref(), |t| t.cards[0].as_ref())
    }

    /// The host's ports and what each holds: 0 and 1 the two ports, then 2-4 and 5-7 their multitaps' slots B-D.
    pub fn host_ports(&self) -> Vec<(u32, Option<&'static str>)> {
        let mut v = vec![(0, self.ports[0]), (1, self.ports[1])];
        for n in 0..2 {
            if self.board.sio0.ports[n].tap.is_some() {
                v.extend((0..3).map(|i| (2 + 3 * n as u32 + i, self.ports[n])));
            }
        }
        v
    }

    /// A multitap on port `n`, its four slots holding controllers of the port's kind and the port's card in slot A.
    pub fn plug_multitap(&mut self, n: usize) {
        let kind = self.ports[n];
        let p = &mut self.board.sio0.ports[n];
        let mut tap = Multitap::new(std::array::from_fn(|_| kind.and_then(pad::controller)));
        tap.cards[0] = p.card.take();
        p.controller = None;
        p.tap = Some(tap);
    }
}

pub fn height(market: Market) -> u32 {
    if market == Market::Europe { 256 } else { 240 }
}


#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::media::Kind;
    use crate::media::sample::exe;
    use emusen_mips::assemble;

    pub fn program(source: &str) -> Vec<u8> {
        let words = assemble(source, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
        let mut image = exe(None, words.len() * 4);
        for (i, w) in words.iter().enumerate() {
            image[0x800 + 4 * i..0x804 + 4 * i].copy_from_slice(&w.to_le_bytes());
        }
        image
    }

    pub fn machine_for(image: &[u8], market: Market) -> Machine {
        let media = Media::read(image).unwrap();
        Machine::new(media, Config { market, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], image, None)
    }

    #[test]
    fn the_rates_are_d0s_provisional_ones() {
        let hz = |(n, d): (u64, u64)| n as f64 / d as f64;
        assert!((hz(standard_rate(false)) - 59.8261).abs() < 1e-4);
        assert!((hz(standard_rate(true)) - 49.7614).abs() < 1e-4);
    }

    #[test]
    fn a_seconds_frames_make_a_seconds_samples_from_the_clock() {
        let mut m = machine_for(&program("loop: j loop\nnop"), Market::Americas);
        let (num, den) = m.frame_rate();
        let frames = (num / den) as i64;
        m.advance();
        let first = m.board.now;
        for _ in 0..frames {
            m.advance();
        }
        assert_eq!(m.board.spu.out.len() as u64, 2 * (m.board.now / CYCLES_PER_SAMPLE));
        let seconds = (m.board.now - first) as f64 / 33_868_800.0;
        assert!((seconds - frames as f64 * den as f64 / num as f64).abs() < 1e-5, "{seconds}");
        assert_eq!(m.board.video.position(first).0, 256, "a frame ends as vblank begins, at line 256 by GP1(07h)'s reset");
        m.board.spu.out.set_limit(1000);
        m.advance();
        assert_eq!(m.board.spu.out.len(), 1000, "the bound drops the oldest");
    }

    #[test]
    fn a_program_runs_and_its_stores_land() {
        let mut m = machine_for(&program("lui t0, 0x8001\nori t0, t0, 0x1000\naddiu t1, zero, 0x55\nsw t1, 0(t0)\nloop: j loop\nnop"), Market::Americas);
        assert_eq!(m.media.kind, Kind::Exe { pc: 0x8001_0000, load: 0x8001_0000, size: 0x800 });
        m.advance();
        assert_eq!(m.board.ram_word(0x11000), 0x55);
        assert!(m.board.gpu.display.frame().chunks(4).all(|p| p == [0, 0, 0, 255]));
        assert_eq!(m.space(BIOS_ID).unwrap().len(), 0x8_0000);
    }

    #[test]
    fn a_load_from_nothing_is_a_data_bus_error_and_a_fetch_from_the_scratchpad_an_instruction_one() {
        let mut m = machine_for(&program("lui t0, 0x1F80\nlw t1, 0x400(t0)\nnop"), Market::Americas);
        for _ in 0..3 {
            m.step();
        }
        assert_eq!((m.cpu.regs.cause >> 2 & 0x1F, m.cpu.regs.epc, m.cpu.regs.pc), (7, 0x8001_0004, 0x8000_0080 + 4));
        let mut j = machine_for(&program("lui t0, 0x1F80\njr t0\nnop"), Market::Americas);
        for _ in 0..4 {
            j.step();
        }
        assert_eq!((j.cpu.regs.cause >> 2 & 0x1F, j.cpu.regs.epc), (6, 0x1F80_0000));
    }

    #[test]
    fn the_duart_carries_text_to_the_tty() {
        let src = "lui t0, 0x1F80\nori t0, t0, 0x2023\naddiu t1, zero, 0x48\nsb t1, 0(t0)\naddiu t1, zero, 0x69\nsb t1, 0(t0)\nloop: j loop\nnop";
        let mut m = machine_for(&program(src), Market::Americas);
        m.advance();
        assert_eq!(&m.space(TTY_ID).unwrap()[..3], b"Hi\0");
    }
}
