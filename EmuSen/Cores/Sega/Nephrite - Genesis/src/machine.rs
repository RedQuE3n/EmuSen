//! The machine the ABI drives: the Genesis (`genesis/`) and what is fitted to it, the Sega CD for a disc and the 32X's
//! stub for a 32X image, held in `Fitted` and matched once per call, so that each configuration's frame is compiled
//! apart (Nephrite_Attachments.md §4). A Mega Drive image and a disc run; a 32X image is recognised and held, and
//! counts its frames (Nephrite_Native.md §1, §9).

use crate::genesis::cart::Cart;
use crate::genesis::{Genesis, Hw, Model};
use crate::interface::{Attachment, AttachmentView, Bare};
#[cfg(feature = "mcd")]
use crate::mcd::MegaCd;
#[cfg(feature = "s32x")]
use crate::s32x::S32x;
use crate::media::{Media, System};
use crate::genesis::debugger::{M68K, M68K_REGISTERS, M68KBUS_ID, Z80, Z80_REGISTERS, Z80BUS_ID};
use emusen_native::SampleQueue;
use emusen_native::core::{Instruction, Processor};
use emusen_native::debug::Hooks;

pub use crate::interface::SpaceRef;
#[cfg(feature = "mcd")]
pub use crate::mcd::{BRAM_ID, CARTRAM_ID};

pub use crate::genesis::{CRAM_ID, ROM_ID, SRAM_ID, VRAM_ID, VSRAM_ID, WRAM_ID, Z80RAM_ID};

/// The picture's size before the first frame: H40, 28 rows of tiles; each frame then has the VDP's width and height.
pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 224;

/// The Genesis as fitted: alone, or with the Sega CD for a disc's machine.
pub(crate) enum Fitted {
    Md(Genesis<Bare>),
    #[cfg(feature = "mcd")]
    Mcd(Genesis<MegaCd>),
    #[cfg(feature = "s32x")]
    S32x(Genesis<S32x>),
}

/// `$e` with `$g` bound to the Genesis fitted, its attachment's type known in each arm: the one match an ABI call makes.
macro_rules! fitted {
    ($f:expr, |$g:ident| $e:expr) => {
        match $f {
            Fitted::Md($g) => $e,
            #[cfg(feature = "mcd")]
            Fitted::Mcd($g) => $e,
            #[cfg(feature = "s32x")]
            Fitted::S32x($g) => $e,
        }
    };
}
pub(crate) use fitted;

pub struct Machine {
    pub media: Media,
    pub(crate) fitted: Fitted,
    pub frames: i64,
    pub picture: Vec<u8>,
    pub skip: bool,
    /// Each pad's buttons in the players' order: port 1's pads, then port 2's (`io::Io::first_pad`).
    pub pads: [u32; crate::genesis::io::PADS],
    /// What each port holds, and whether its pads are six-button ones (the settings `pad1` and `pad2`).
    pub plugged: [(crate::genesis::io::Plug, bool); 2],
    /// The firmware files given at create, by number; none is used yet.
    pub firmware: Vec<u32>,
    /// The samples made and not yet drained, stereo at `sound::RATE`.
    pub audio: SampleQueue,
    /// A save RAM's battery file as the host is given it, remade when its bytes change; not part of a state.
    pub battery_file: Vec<u8>,
    /// The debugger's tables and logs, the 68000's breakpoints among them; the Z80's breakpoints; the processor the
    /// last stop was on; and whether a stop left the frame open (Nephrite_Native.md §42).
    pub hooks: Hooks,
    pub z80_breakpoints: Vec<(i32, i32)>,
    pub debug_stopped: u32,
    pub debug_open: bool,
    /// Since core ABI 1.8, the host's create flags: the RAM cartridge (removable) and the backup RAM (the console's) kept
    /// apart from the game, so a state load leaves each as it stands (EmuSen_SaveMedia.md §7.3). Never part of a state.
    pub kept_apart: KeptApart,
}

/// Which of the Sega CD's save memories the host keeps apart from the game.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeptApart {
    pub cartridge: bool,
    pub backup: bool,
}

impl KeptApart {
    /// Whether a state load leaves the memory of space `id` as it stands.
    #[cfg(feature = "mcd")]
    pub fn keeps(&self, id: u32) -> bool {
        (id == CARTRAM_ID && self.cartridge) || (id == BRAM_ID && self.backup)
    }

    /// Without the Sega CD built, there is no memory to keep apart.
    #[cfg(not(feature = "mcd"))]
    pub fn keeps(&self, _: u32) -> bool {
        false
    }
}

/// The model a machine starts as: the first of the Americas, Japan and Europe the header allows (Nephrite_Plan.md §9,
/// Q6), the Americas where it names none, and no TMSS (Nephrite_Native.md §9, §45.4).
pub fn default_model(media: &Media) -> Model {
    let m = media.header.as_ref().map(|h| h.markets).unwrap_or_default();
    let overseas = m.americas || !m.japan;
    Model { overseas, pal: media.pal(), version: 0, model2: false }
}

impl Machine {
    pub fn new(image: &[u8], media: Media) -> Machine {
        Machine::with_model(image, media.clone(), default_model(&media))
    }

    pub fn with_model(image: &[u8], media: Media, model: Model) -> Machine {
        Machine::with_bios(image, media, model, None)
    }

    /// A machine with `bios` in a Sega CD's boot ROM and the image, when it is a disc, in its drive.
    #[cfg_attr(not(feature = "mcd"), allow(unused_variables))]
    pub fn with_bios(image: &[u8], media: Media, model: Model, bios: Option<Vec<u8>>) -> Machine {
        match media.system {
            #[cfg(feature = "mcd")]
            System::Mcd => {
                let disc = crate::mcd::open_disc(emusen_native::core::Kept::from(image.to_vec()), Vec::new()).ok();
                Machine::with_disc(media, model, bios, disc)
            }
            #[cfg(feature = "s32x")]
            System::S32x => {
                let cart = cartridge(image.to_vec(), &media);
                Machine::assembled(media, Fitted::S32x(Genesis::with(cart, model, S32x::new())))
            }
            System::Md => {
                let cart = cartridge(image.to_vec(), &media);
                Machine::assembled(media, Fitted::Md(Genesis::with(cart, model, Bare)))
            }
        }
    }

    /// A disc's machine with `disc` in its Sega CD's drive, which has its Sega CD whether a boot ROM is given or not.
    #[cfg(feature = "mcd")]
    pub fn with_disc(media: Media, model: Model, bios: Option<Vec<u8>>, disc: Option<crate::mcd::cdd::Disc>) -> Machine {
        let cart = cartridge(Vec::new(), &media);
        let board = crate::interface::Board { master_hz: crate::genesis::master_hz(model.pal), pal: model.pal, start: crate::genesis::START, rate: crate::genesis::sound::RATE };
        Machine::assembled(media, Fitted::Mcd(Genesis::with(cart, model, MegaCd::new(board, bios.unwrap_or_default(), disc))))
    }

    fn assembled(media: Media, fitted: Fitted) -> Machine {
        Machine { media, fitted, frames: 0, picture: blank(), skip: false, pads: [0; crate::genesis::io::PADS], plugged: [(crate::genesis::io::Plug::Pad, false); 2], firmware: Vec::new(), audio: SampleQueue::default(), battery_file: Vec::new(), hooks: Hooks::new(&[24, 16]), z80_breakpoints: Vec::new(), debug_stopped: 0, debug_open: false, kept_apart: KeptApart::default() }
    }

    /// The Genesis's devices, whatever is fitted.
    pub fn hw(&self) -> &Hw {
        fitted!(&self.fitted, |g| &g.hw.dev)
    }

    pub fn hw_mut(&mut self) -> &mut Hw {
        fitted!(&mut self.fitted, |g| &mut g.hw.dev)
    }

    pub fn cpu(&self) -> &beryl_m68k::M68000 {
        fitted!(&self.fitted, |g| &g.cpu)
    }

    pub fn z80(&self) -> &beryl_z80::Z80 {
        fitted!(&self.fitted, |g| &g.z80)
    }

    /// The Sega CD, on a disc's machine.
    #[cfg(feature = "mcd")]
    pub fn mcd(&self) -> Option<&MegaCd> {
        match &self.fitted {
            Fitted::Mcd(g) => Some(&g.hw.att),
            _ => None,
        }
    }

    #[cfg(feature = "mcd")]
    pub fn mcd_mut(&mut self) -> Option<&mut MegaCd> {
        match &mut self.fitted {
            Fitted::Mcd(g) => Some(&mut g.hw.att),
            _ => None,
        }
    }

    /// The spaces in id order: the two buses, the Genesis's memories, the battery's save RAM, the ROM, then the attachment's.
    pub fn spaces(&self) -> Vec<SpaceRef> {
        let mut v = vec![
            SpaceRef { id: crate::genesis::debugger::M68KBUS_ID, name: "M68KBUS", read_only: true },
            SpaceRef { id: crate::genesis::debugger::Z80BUS_ID, name: "Z80BUS", read_only: true },
        ];
        v.extend([(WRAM_ID, "WRAM"), (Z80RAM_ID, "Z80RAM"), (VRAM_ID, "VRAM"), (CRAM_ID, "CRAM"), (VSRAM_ID, "VSRAM")]
            .map(|(id, name)| SpaceRef { id, name, read_only: false }));
        if self.hw().cart.has_battery() {
            v.push(SpaceRef { id: SRAM_ID, name: "SRAM", read_only: false });
        }
        if !self.media.system.is_disc() {
            v.push(SpaceRef { id: ROM_ID, name: "ROM", read_only: true });
        }
        fitted!(&self.fitted, |g| g.hw.att.spaces(&mut v));
        v
    }

    pub fn bytes(&self, id: u32) -> Option<&[u8]> {
        let hw = self.hw();
        Some(match id {
            WRAM_ID => &hw.wram,
            Z80RAM_ID => &hw.zram,
            VRAM_ID => &hw.vdp.vram,
            CRAM_ID => &hw.vdp.cram,
            VSRAM_ID => &hw.vdp.vsram,
            SRAM_ID if hw.cart.has_battery() => hw.cart.battery(),
            ROM_ID if !self.media.system.is_disc() => &hw.cart.rom,
            _ => return fitted!(&self.fitted, |g| g.hw.att.bytes(id)),
        })
    }

    pub fn bytes_mut(&mut self, id: u32) -> Option<&mut [u8]> {
        fitted!(&mut self.fitted, |g| {
            let (hw, att) = (&mut g.hw.dev, &mut g.hw.att);
            let battery = hw.cart.has_battery();
            Some(match id {
                WRAM_ID => &mut hw.wram,
                Z80RAM_ID => &mut hw.zram,
                VRAM_ID => &mut hw.vdp.vram,
                CRAM_ID => &mut hw.vdp.cram,
                VSRAM_ID => &mut hw.vdp.vsram,
                SRAM_ID if battery => hw.cart.battery_mut(),
                _ => return att.bytes_mut(id),
            })
        })
    }

    /// The memory the battery file holds: the cartridge's save RAM, or the Sega CD's internal backup RAM.
    pub fn battery_id(&self) -> Option<u32> {
        match self.media.system {
            #[cfg(feature = "mcd")]
            System::Mcd => Some(BRAM_ID),
            _ => self.hw().cart.has_battery().then_some(SRAM_ID),
        }
    }

    /// One frame: a Mega Drive image runs one frame of the Genesis with the pads as set; the attachments only count.
    pub fn advance(&mut self) {
        if self.runs() {
            self.begin_frame();
            fitted!(&mut self.fitted, |g| g.run_frame());
        }
        self.end_frame();
    }

    /// The frame through the observed loop: the stop's reasons, zero at the frame's end. A stop leaves the frame open,
    /// and the next call goes on from it; `run::UNCHECKED` runs the stopped processor's next instruction unasked.
    pub fn run_frame_debug(&mut self, flags: u32) -> u32 {
        let skip = (flags & emusen_native::debug::run::UNCHECKED != 0).then_some(self.debug_stopped);
        if self.runs() {
            if !self.debug_open {
                self.begin_frame();
                self.debug_open = true;
            }
            let (why, processor) = fitted!(&mut self.fitted, |g| g.run_frame_observed(&mut self.hooks, &self.z80_breakpoints, skip));
            if why != 0 {
                self.debug_stopped = processor;
                return why;
            }
        }
        self.debug_stopped = 0;
        self.end_frame();
        0
    }

    /// Whether the machine runs: a Genesis, alone or with its Sega CD; the 32X only counts frames until its stage.
    pub fn runs(&self) -> bool {
        !self.media.system.is_32x()
    }

    /// The pads and the drawing switch as a frame takes them.
    fn begin_frame(&mut self) {
        let io = &mut fitted!(&mut self.fitted, |g| &mut g.hw.dev).io;
        io.plugs = [self.plugged[0].0, self.plugged[1].0];
        let (second, ports, count) = (io.first_pad(1), io.pad_count(), io.players());
        for (i, pad) in io.pads.iter_mut().enumerate() {
            pad.buttons = if i < count { self.pads[i] } else { 0 };
            // The J-Cart's ports take three-button pads (§48).
            pad.six = i < ports && self.plugged[(i >= second && self.plugged[0].0 != crate::genesis::io::Plug::FourWay) as usize].1;
        }
        let draw = !self.skip;
        self.hw_mut().draw = draw;
    }

    /// The frame's sound and picture taken, and the count and the battery file brought up to it.
    fn end_frame(&mut self) {
        self.debug_open = false;
        if self.runs() {
            let audio = &mut self.audio;
            fitted!(&mut self.fitted, |g| {
                let (hw, att) = (&mut g.hw.dev, &mut g.hw.att);
                hw.sound.take_with(hw.clock, att.sound(hw.clock), |l, r| audio.push_pair(l, r));
            });
            if !self.skip {
                let f = &fitted!(&self.fitted, |g| &g.hw.dev).vdp.frame;
                self.picture.clear();
                for y in 0..f.height {
                    self.picture.extend_from_slice(&f.rgba[y * crate::genesis::render::MAX_W * 4..][..f.width * 4]);
                }
            }
        }
        self.frames += 1;
        self.refresh_battery_file();
    }

    /// The battery file remade where the battery's bytes changed: a save RAM's in Genesis Plus GX's form, empty for an
    /// EEPROM, whose file is its bytes (Nephrite_Native.md §33).
    pub fn refresh_battery_file(&mut self) {
        let cart = &mut fitted!(&mut self.fitted, |g| &mut g.hw.dev).cart;
        if cart.take_dirty() && !cart.battery_file(&mut self.battery_file) {
            self.battery_file.clear();
        }
    }

    /// The battery file's bytes: the save RAM's file form, or the battery's own bytes.
    pub fn battery_bytes(&self) -> Option<&[u8]> {
        let id = self.battery_id()?;
        if id == SRAM_ID && !self.battery_file.is_empty() { Some(&self.battery_file) } else { self.bytes(id) }
    }
}

/// The cartridge in the slot from its image, Sonic & Knuckles' split from what is locked on top; a disc's is empty.
fn cartridge(rom: Vec<u8>, media: &Media) -> Cart {
    let header = media.header.clone().unwrap_or_default();
    let (rom, top, patch) = split_lock_on(rom, &header.serial);
    let mut cart = Cart::new(rom, header.save, &header.system_type, &header.serial);
    if let Some(top) = top {
        cart.lock_on(top, patch);
    }
    cart
}

/// A combined Sonic & Knuckles image, its 2 MiB followed by the cartridge on top and, when the rest is a whole number
/// of mebibytes and 256 KiB, the patch ROM last.
pub fn split_lock_on(image: Vec<u8>, serial: &str) -> (Vec<u8>, Option<Vec<u8>>, Vec<u8>) {
    const SK: usize = 0x20_0000;
    if !serial.starts_with(crate::genesis::cart::LOCK_ON_SERIAL) || image.len() <= SK {
        return (image, None, Vec::new());
    }
    let mut sk = image;
    let mut top = sk.split_off(SK);
    let patch = if top.len() % 0x10_0000 == 0x4_0000 { top.split_off(top.len() - 0x4_0000) } else { Vec::new() };
    (sk, Some(top), patch)
}

/// An opaque black frame, RGBA.
pub fn blank() -> Vec<u8> {
    (0..(WIDTH * HEIGHT) as usize).flat_map(|_| [0, 0, 0, 255]).collect()
}

/// The debugger's composition: the Genesis's processors and buses, read without side effects (Nephrite_Native.md §41).
impl Machine {
    pub fn processors(&self) -> Vec<Processor> {
        let named = |r: &[(&str, u32)]| r.iter().map(|&(n, b)| (n.to_owned(), b)).collect::<Vec<_>>();
        vec![
            Processor { id: M68K, name: "M68K".into(), pc_bits: 24, registers: named(&M68K_REGISTERS), code_space: Some(M68KBUS_ID) },
            Processor { id: Z80, name: "Z80".into(), pc_bits: 16, registers: named(&Z80_REGISTERS), code_space: Some(Z80BUS_ID) },
        ]
    }

    /// The bytes `address` on of a bus space; another space is the caller's.
    pub fn read_bus(&self, space: u32, address: u32, out: &mut [u8]) {
        let hw = self.hw();
        for (i, b) in out.iter_mut().enumerate() {
            let at = address.wrapping_add(i as u32);
            *b = match space {
                M68KBUS_ID => hw.peek_m68k(at),
                _ => hw.peek_z80(at as u16),
            };
        }
    }

    /// The address of the instruction `processor` stands in front of.
    pub fn debug_pc(&self, processor: u32) -> Option<u64> {
        match processor {
            M68K => Some((self.cpu().regs.pc & 0xFF_FFFF) as u64),
            Z80 => Some(self.z80().regs.pc as u64),
            _ => None,
        }
    }

    /// A processor's registers in the order its list gives them.
    pub fn debug_registers(&self, processor: u32) -> Option<Vec<i64>> {
        match processor {
            M68K => {
                let r = &self.cpu().regs;
                let mut v: Vec<i64> = r.d.iter().chain(r.a.iter()).map(|&x| x as i64).collect();
                v.extend([r.pc as i64, r.sr as i64, r.usp() as i64, r.ssp() as i64]);
                Some(v)
            }
            Z80 => {
                let r = &self.z80().regs;
                Some(vec![
                    r.af, r.bc, r.de, r.hl, r.af_, r.bc_, r.de_, r.hl_, r.ix, r.iy, r.sp, r.pc,
                    r.i as u16, r.r as u16, r.wz, r.im as u16, r.iff1 as u16, r.iff2 as u16,
                ].into_iter().map(i64::from).collect())
            }
            _ => None,
        }
    }

    /// `count` instructions from `address`, each in its processor's set: the 68000's on its bus, the Z80's on its own
    /// bus and in its RAM; another space is read as the processor asked would see it.
    pub fn debug_disassemble(&self, processor: u32, space: u32, address: u32, count: u32) -> Option<Vec<Instruction>> {
        let z80 = match space {
            Z80BUS_ID => true,
            M68KBUS_ID => false,
            Z80RAM_ID => true,
            _ => processor == Z80,
        };
        let byte = |a: u32| -> u8 {
            match space {
                M68KBUS_ID | Z80BUS_ID => {
                    let mut b = [0];
                    self.read_bus(space, a, &mut b);
                    b[0]
                }
                _ => self.bytes(space).and_then(|m| m.get(a as usize).copied()).unwrap_or(0xFF),
            }
        };
        if processor > Z80 || (space > 1 && self.bytes(space).is_none()) {
            return None;
        }
        let mut out = Vec::with_capacity(count as usize);
        let mut at = address;
        for _ in 0..count {
            let i = if z80 {
                beryl_z80::disasm::disassemble(&|a: u16| byte(a as u32), at as u16)
            } else {
                beryl_m68k::disasm::disassemble(&|a: u32| (byte(a) as u16) << 8 | byte(a.wrapping_add(1)) as u16, at & 0xFF_FFFE)
            };
            at = i.address.wrapping_add(i.bytes.len().max(1) as u32);
            out.push(i);
        }
        Some(out)
    }
}
