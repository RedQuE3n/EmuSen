//! The machine the ABI drives: the Genesis (`genesis.rs`), whose memories are the spaces, and the Sega CD's and the
//! 32X's memories beside it until their stages. A Mega Drive image runs; the attachments' images are recognised and
//! held (Nephrite_Native.md §1, §9).

use crate::cart::Cart;
use crate::genesis::{Genesis, Model};
use crate::media::{Media, System};

/// A memory of the attachments, by its space name and id (Nephrite_Plan.md §4.3).
pub struct Memory {
    pub id: u32,
    pub name: &'static str,
    pub bytes: Vec<u8>,
}

/// One space as the ABI lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpaceRef {
    pub id: u32,
    pub name: &'static str,
    pub read_only: bool,
}

pub const WRAM_ID: u32 = 2;
pub const Z80RAM_ID: u32 = 3;
pub const VRAM_ID: u32 = 4;
pub const CRAM_ID: u32 = 5;
pub const VSRAM_ID: u32 = 6;
pub const SRAM_ID: u32 = 7;
pub const ROM_ID: u32 = 8;
pub const MCD_MEMORIES: [(u32, &str, usize); 4] = [(17, "PRGRAM", 0x8_0000), (18, "WORDRAM", 0x4_0000), (19, "PCMRAM", 0x1_0000), (20, "BRAM", 0x2000)];
pub const S32X_MEMORIES: [(u32, &str, usize); 3] = [(34, "SDRAM", 0x4_0000), (35, "FRAMEBUFFER", 0x4_0000), (36, "PALETTE", 512)];

/// The picture's size before the first frame: H40, 28 rows of tiles; each frame then has the VDP's width and height.
pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 224;

pub struct Machine {
    pub media: Media,
    pub genesis: Genesis,
    pub extra: Vec<Memory>,
    pub frames: i64,
    pub picture: Vec<u8>,
    pub skip: bool,
    pub pads: [u32; 2],
    /// Each port's pad: the six-button one when true (the settings `pad1` and `pad2`).
    pub six_button: [bool; 2],
    /// The firmware files given at create, by number; none is used yet.
    pub firmware: Vec<u32>,
}

/// The model a machine starts as: the header's market, overseas first, and no TMSS (Nephrite_Native.md §9).
pub fn default_model(media: &Media) -> Model {
    let m = media.header.as_ref().map(|h| h.markets).unwrap_or_default();
    let overseas = m.americas || m.europe || !m.japan;
    Model { overseas, pal: media.pal(), version: 0 }
}

impl Machine {
    pub fn new(image: &[u8], media: Media) -> Machine {
        Machine::with_model(image, media.clone(), default_model(&media))
    }

    pub fn with_model(image: &[u8], media: Media, model: Model) -> Machine {
        let header = media.header.clone().unwrap_or_default();
        let rom = if media.system == System::Mcd { Vec::new() } else { image.to_vec() };
        let (rom, top, patch) = split_lock_on(rom, &header.serial);
        let mut cart = Cart::new(rom, header.save, &header.system_type, &header.serial);
        if let Some(top) = top {
            cart.lock_on(top, patch);
        }
        let mut extra = Vec::new();
        let list: &[(u32, &'static str, usize)] = match media.system {
            System::Mcd => &MCD_MEMORIES,
            System::S32x => &S32X_MEMORIES,
            System::Md => &[],
        };
        for &(id, name, size) in list {
            extra.push(Memory { id, name, bytes: vec![0; size] });
        }
        Machine { media, genesis: Genesis::new(cart, model), extra, frames: 0, picture: blank(), skip: false, pads: [0; 2], six_button: [false; 2], firmware: Vec::new() }
    }

    /// The spaces in id order: the Genesis's memories, the battery's save RAM, the ROM, then the attachment's.
    pub fn spaces(&self) -> Vec<SpaceRef> {
        let mut v = [(WRAM_ID, "WRAM"), (Z80RAM_ID, "Z80RAM"), (VRAM_ID, "VRAM"), (CRAM_ID, "CRAM"), (VSRAM_ID, "VSRAM")]
            .map(|(id, name)| SpaceRef { id, name, read_only: false })
            .to_vec();
        if self.genesis.hw.cart.has_battery() {
            v.push(SpaceRef { id: SRAM_ID, name: "SRAM", read_only: false });
        }
        if self.media.system != System::Mcd {
            v.push(SpaceRef { id: ROM_ID, name: "ROM", read_only: true });
        }
        v.extend(self.extra.iter().map(|m| SpaceRef { id: m.id, name: m.name, read_only: false }));
        v
    }

    pub fn bytes(&self, id: u32) -> Option<&[u8]> {
        let hw = &self.genesis.hw;
        Some(match id {
            WRAM_ID => &hw.wram,
            Z80RAM_ID => &hw.zram,
            VRAM_ID => &hw.vdp.vram,
            CRAM_ID => &hw.vdp.cram,
            VSRAM_ID => &hw.vdp.vsram,
            SRAM_ID if hw.cart.has_battery() => hw.cart.battery(),
            ROM_ID if self.media.system != System::Mcd => &hw.cart.rom,
            _ => return self.extra.iter().find(|m| m.id == id).map(|m| m.bytes.as_slice()),
        })
    }

    pub fn bytes_mut(&mut self, id: u32) -> Option<&mut [u8]> {
        let hw = &mut self.genesis.hw;
        let battery = hw.cart.has_battery();
        Some(match id {
            WRAM_ID => &mut hw.wram,
            Z80RAM_ID => &mut hw.zram,
            VRAM_ID => &mut hw.vdp.vram,
            CRAM_ID => &mut hw.vdp.cram,
            VSRAM_ID => &mut hw.vdp.vsram,
            SRAM_ID if battery => hw.cart.battery_mut(),
            _ => return self.extra.iter_mut().find(|m| m.id == id).map(|m| m.bytes.as_mut_slice()),
        })
    }

    /// The memory the battery file holds: the cartridge's save RAM, or the Sega CD's internal backup RAM.
    pub fn battery_id(&self) -> Option<u32> {
        match self.media.system {
            System::Mcd => Some(20),
            _ => self.genesis.hw.cart.has_battery().then_some(SRAM_ID),
        }
    }

    /// One frame: a Mega Drive image runs one frame of the Genesis with the pads as set; the attachments only count.
    pub fn advance(&mut self) {
        if self.media.system == System::Md {
            for (i, pad) in self.genesis.hw.io.pads.iter_mut().enumerate() {
                pad.buttons = self.pads[i];
                pad.six = self.six_button[i];
            }
            self.genesis.hw.draw = !self.skip;
            self.genesis.run_frame();
            if !self.skip {
                let f = &self.genesis.hw.frame;
                self.picture.clear();
                for y in 0..f.height {
                    self.picture.extend_from_slice(&f.rgba[y * crate::render::MAX_W * 4..][..f.width * 4]);
                }
            }
        }
        self.frames += 1;
    }
}

/// A combined Sonic & Knuckles image, its 2 MiB followed by the cartridge on top and, when the rest is a whole number
/// of mebibytes and 256 KiB, the patch ROM last.
pub fn split_lock_on(image: Vec<u8>, serial: &str) -> (Vec<u8>, Option<Vec<u8>>, Vec<u8>) {
    const SK: usize = 0x20_0000;
    if !serial.starts_with(crate::cart::LOCK_ON_SERIAL) || image.len() <= SK {
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
