//! The console (stage S24, Sapphire_Native.md §35): `board::System`, the EE core and every unit, with the player's
//! BIOS at 1FC00000h for both processors, the EE and the IOP started at their reset vectors, the drive and the pads
//! and cards on the IOP's bus, the EEPROM the drive keeps, and a frame that ends at each field's V-blank. Without a
//! BIOS nothing is at either reset vector, so both processors are held and the units alone run (black, silent).

use crate::board::System;

use crate::cdvd::nvm;
use crate::clock;
use crate::media::Media;
use crate::sio2::card::MemoryCard;
use crate::sio2::pad::{DualShock2, Input};

/// A memory the machine shows, by its space id and name (Sapphire_Plan.md §4.3).
pub const RAM_ID: u32 = 1;
pub const BIOS_ID: u32 = 10;

/// Each memory by id, name, size and whether it is read-only [R ps2tek "Memory Map", "VU", "GS", "SPU2", by the plan's
/// citations]: the EE's RDRAM, its scratchpad, the GS's eDRAM, the VUs' micro and data memories, the IOP's RAM, the
/// SPU2's sound RAM, and the boot ROM, the player's image when given.
pub const MEMORIES: [(u32, &str, usize, bool); 10] = [
    (RAM_ID, "RAM", 32 << 20, false),
    (2, "SCRATCH", 16 << 10, false),
    (3, "GSMEM", 4 << 20, false),
    (4, "VU0MICRO", 4 << 10, false),
    (5, "VU0DATA", 4 << 10, false),
    (6, "VU1MICRO", 16 << 10, false),
    (7, "VU1DATA", 16 << 10, false),
    (8, "IOPRAM", 2 << 20, false),
    (9, "SPU2RAM", 2 << 20, false),
    (BIOS_ID, "BIOS", BIOS_BYTES, true),
];

/// The boot ROM's size, every BIOS image's (Sapphire_Plan.md §5.9).
pub const BIOS_BYTES: usize = 4 << 20;

/// The output rate, the SPU2's 48,000 Hz (Sapphire_Plan.md §5.5).
pub const AUDIO_RATE: i32 = 48_000;

/// The picture before the first field, 640 by 448 NTSC and 512 PAL, the modes' own with neither circuit on.
pub const WIDTH: u32 = 640;
pub const NTSC_HEIGHT: u32 = 448;
pub const PAL_HEIGHT: u32 = 512;

/// EE cycles a step of the board while no processor runs.
const IDLE_STEP: u64 = 16_384;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Market {
    Japan,
    Americas,
    Europe,
}

/// Why the EE stopped, in the machine's words: an instruction the interpreter does not implement. Neither the address
/// nor the word is kept (Sapphire_Plan.md §1.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    Unimplemented,
}

pub struct Machine {
    pub media: Media,
    pub market: Market,
    pub sys: Box<System>,
    pub frames: i64,
    pub skip: bool,
    pub pads: [u32; 2],
    pub ports: [Option<&'static str>; 2],
    /// Firmware files given, by number.
    pub firmware: Vec<u32>,
    /// The sticks the frontend set, by port, in the pad's order: right X, right Y, left X, left Y (S25).
    pub sticks: [[u8; 4]; 2],
    /// A BIOS is in the boot ROM and the processors run from reset.
    pub booted: bool,
    pub stop: Option<Stop>,
}

impl Machine {
    pub fn new(media: Media, market: Market, ports: [Option<&'static str>; 2]) -> Machine {
        let sys = Machine::console(market, ports);
        let mut m = Machine { media, market, sys, frames: 0, skip: false, pads: [0; 2], ports, firmware: Vec::new(), sticks: [[0x80; 4]; 2], booted: false, stop: None };
        for slot in &mut m.sys.board.iop.bus.sio2.slots {
            slot.cards[0] = Some(MemoryCard::erased());
        }
        m
    }

    /// The console at power-on in the market's standard, with a DualShock 2 in each port named.
    fn console(market: Market, ports: [Option<&'static str>; 2]) -> Box<System> {
        let mode = if market == Market::Europe { clock::PAL } else { clock::NTSC };
        let mut sys = Box::new(System { ee: crate::ee::Ee::reset(), board: crate::board::Board::new(mode) });
        // SMODE2 interlaced in frame mode from the first field, as the BIOS sets it, so the picture is the standard's
        // whole frame before the BIOS programs the PCRTC, and with no BIOS (Sapphire_Native.md §35.6) [C].
        crate::ee::Bus::write(&mut sys.board, 0x1200_0020, 8, 3);
        sys.board.iop.bus.cdvd.region = Some(match market {
            Market::Japan => crate::cdvd::REGION_JAPAN,
            Market::Americas => crate::cdvd::REGION_AMERICAS,
            Market::Europe => crate::cdvd::REGION_EUROPE,
        });
        for (n, plug) in ports.iter().enumerate() {
            sys.board.iop.bus.sio2.slots[n].pads[0] = plug.map(|_| DualShock2::default());
        }
        sys
    }

    /// The player's BIOS in the boot ROM, the same image at 1FC00000h for the EE and the IOP, and both processors at
    /// their reset vectors (Sapphire_Plan.md §5.9).
    pub fn insert_bios(&mut self, image: &[u8]) {
        let b = &mut self.sys.board;
        b.map.rom = image.to_vec();
        b.iop.bus.rom = image.to_vec();
        self.sys.ee = crate::ee::Ee::reset();
        b.iop.reset();
        self.booted = true;
    }

    /// The drive's tray opened or closed by the player's hand, now (G10, the core ABI's `MEDIA`).
    pub fn set_tray(&mut self, open: bool) {
        let b = &mut self.sys.board;
        b.iop.bus.tray(open, b.now);
    }

    /// A disc, or none, put in the drive; refused while the tray is closed, as a hand cannot.
    pub fn insert_disc(&mut self, disc: Option<crate::cdvd::Disc>) -> bool {
        self.sys.board.iop.bus.cdvd.insert(disc)
    }

    pub fn pal(&self) -> bool {
        self.market == Market::Europe
    }

    pub fn height(&self) -> u32 {
        if self.pal() { PAL_HEIGHT } else { NTSC_HEIGHT }
    }

    /// The field rate of the market's standard, interlaced, as the BIOS sets it [A, Sapphire_Native.md §35.6].
    pub fn frame_rate(&self) -> (u64, u64) {
        let mode = if self.pal() { clock::PAL } else { clock::NTSC };
        mode.field_rate(true)
    }

    /// The console's EEPROM, the medium `ps2.nvm` (Sapphire_Plan.md §4.4).
    pub fn nvm(&self) -> &nvm::Nvm {
        &self.sys.board.iop.bus.cdvd.nvm
    }

    pub fn nvm_mut(&mut self) -> &mut nvm::Nvm {
        &mut self.sys.board.iop.bus.cdvd.nvm
    }

    pub fn card(&self, slot: usize) -> Option<&MemoryCard> {
        self.sys.board.iop.bus.sio2.slots.get(slot)?.cards[0].as_ref()
    }

    pub fn card_mut(&mut self, slot: usize) -> Option<&mut MemoryCard> {
        self.sys.board.iop.bus.sio2.card_mut(slot, 0)
    }

    /// The frontend's buttons and sticks handed to the pad in port `port`.
    pub fn sync_pad(&mut self, port: usize) {
        let input = Input::from_digital(self.pads[port] as u16, self.sticks[port]);
        if let Some(p) = self.sys.board.iop.bus.sio2.pad_mut(port, 0) {
            p.input = input;
        }
    }

    /// Memory `id`'s bytes, read one at a time from where the board keeps them.
    pub fn space_byte(&self, id: u32, at: usize) -> Option<u8> {
        let b = &self.sys.board;
        let vu = |unit: usize| if unit == 0 { &b.vpu.units.vu0 } else { &b.vpu.units.vu1 };
        match id {
            RAM_ID => b.map.rdram.get(at).copied(),
            2 => self.sys.ee.spram.get(at).copied(),
            3 => b.gs.local.words.get(at / 4).map(|w| (w >> (8 * (at % 4))) as u8),
            4 => vu(0).micro.get(at).copied(),
            5 => vu(0).data.get(at).copied(),
            6 => vu(1).micro.get(at).copied(),
            7 => vu(1).data.get(at).copied(),
            8 => b.iop.bus.ram.get(at).copied(),
            9 => b.iop.bus.spu2.ram.get(at).copied(),
            BIOS_ID => (at < BIOS_BYTES).then(|| b.map.rom.get(at).copied().unwrap_or(0)),
            _ => None,
        }
    }

    /// Writes one byte of a writable memory; false past its end.
    pub fn set_space_byte(&mut self, id: u32, at: usize, v: u8) -> bool {
        let b = &mut self.sys.board;
        let slot: Option<&mut u8> = match id {
            RAM_ID => b.map.rdram.get_mut(at),
            2 => self.sys.ee.spram.get_mut(at),
            3 => {
                let Some(w) = b.gs.local.words.get_mut(at / 4) else { return false };
                let s = 8 * (at % 4);
                *w = *w & !(0xFF << s) | u32::from(v) << s;
                return true;
            }
            4 => b.vpu.units.vu0.micro.get_mut(at),
            5 => b.vpu.units.vu0.data.get_mut(at),
            6 => b.vpu.units.vu1.micro.get_mut(at),
            7 => b.vpu.units.vu1.data.get_mut(at),
            8 => b.iop.bus.ram.get_mut(at),
            9 => b.iop.bus.spu2.ram.get_mut(at),
            _ => None,
        };
        slot.map(|s| *s = v).is_some()
    }

    /// One frame: the console run until the PCRTC begins its next field, which is when V-blank rises.
    pub fn advance(&mut self) {
        let target = self.sys.board.pcrtc.fields + 1;
        while self.sys.board.pcrtc.fields < target {
            if self.booted && self.stop.is_none() {
                for _ in 0..1024 {
                    if self.sys.step().is_err() {
                        self.stop = Some(Stop::Unimplemented);
                        break;
                    }
                }
            } else {
                let t = self.sys.board.now + IDLE_STEP;
                let System { ee, board } = &mut *self.sys;
                board.run_to(t, &mut ee.spram[..]);
            }
        }
        self.frames += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::sample::elf;

    fn machine(market: Market) -> Machine {
        Machine::new(Media::read(&elf()).unwrap(), market, [Some("ps2.dualshock2"), None])
    }

    #[test]
    fn without_a_bios_both_processors_are_held_and_a_frame_is_one_field() {
        let mut m = machine(Market::Americas);
        assert!(!m.booted && m.sys.board.iop.held);
        let t0 = m.sys.board.now;
        m.advance();
        m.advance();
        let rate = m.frame_rate();
        let field = clock::EE_HZ * rate.1 / rate.0;
        let took = m.sys.board.now - t0;
        assert!(took.abs_diff(2 * field) < 2 * IDLE_STEP, "{took} for two frames of {field}: the first field is the reset's non-interlaced one");
        assert_eq!(m.frames, 2);
    }

    /// The drive answers the region query with the console's own market (D-348).
    #[test]
    fn the_drive_reports_the_consoles_market() {
        let want = [(Market::Japan, crate::cdvd::REGION_JAPAN), (Market::Americas, crate::cdvd::REGION_AMERICAS), (Market::Europe, crate::cdvd::REGION_EUROPE)];
        for (market, region) in want {
            assert_eq!(machine(market).sys.board.iop.bus.cdvd.region, Some(region), "{market:?}");
        }
    }

    #[test]
    fn the_memories_are_the_plans_and_the_bios_is_empty_until_given() {
        let m = machine(Market::Japan);
        for &(id, _, size, _) in &MEMORIES {
            assert!(m.space_byte(id, size - 1).is_some() && m.space_byte(id, size).is_none(), "{id}");
        }
        assert_eq!(m.space_byte(BIOS_ID, 0), Some(0));
        let mut m = machine(Market::Europe);
        assert!(m.set_space_byte(3, 5, 0xA7) && m.sys.board.gs.local.words[1] == 0xA700, "GSMEM is the GS's local memory");
        assert!(!m.set_space_byte(BIOS_ID, 0, 1));
        m.insert_bios(&vec![0; BIOS_BYTES]);
        assert!(m.booted && !m.sys.board.iop.held && m.sys.board.iop.cpu.regs.pc == crate::iop::RESET && m.sys.ee.pc == 0xBFC0_0000);
    }
}
