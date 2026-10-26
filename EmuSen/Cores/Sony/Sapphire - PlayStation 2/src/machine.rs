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
pub const TTY_ID: u32 = 11;

/// Each memory by id, name, size and whether it is read-only [R ps2tek "Memory Map", "VU", "GS", "SPU2", by the plan's
/// citations]: the EE's RDRAM, its scratchpad, the GS's eDRAM, the VUs' micro and data memories, the IOP's RAM, the
/// SPU2's sound RAM, the boot ROM, the player's image when given, and the IOP's console text (S10).
pub const MEMORIES: [(u32, &str, usize, bool); 11] = [
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
    (TTY_ID, "TTY", crate::iop::tty::TTY_BYTES, true),
];

/// SIF's SMFLAG bit the IOP sets once it has finished booting (ps2tek "SIF Registers"; "BIOS IOP EESYNC").
pub const IOP_BOOTED: u32 = 0x4_0000;

/// ExecPS2, the EE kernel's call that clears the kernel's state and starts a program already in memory as the main
/// thread (ps2tek "BIOS EE Syscalls", 07h): its number, given in v1.
pub const EXEC_PS2: u64 = 7;

/// The program's one argument, its name as a loader gives it in argv[0] [C], and where the argument block is put for
/// the kernel to copy: past the program and its heap's start, below the main thread's stack (Sapphire_Native.md §36.2).
pub const PROGRAM_NAME: &[u8] = b"host:program.elf\0";
pub const ARGS: usize = 0x01F0_0000;

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
    /// The EE program the fast boot enters once the BIOS is up (S10), kept so that a state made before its entry still
    /// enters it.
    pub program: Option<Vec<u8>>,
    /// The frame at whose run the program was entered; none while it waits.
    pub entered: Option<i64>,
    /// The debugger's hooks and its open frame (S31, Sapphire_Native.md §41).
    pub debug: crate::debugger::Debugger,
    /// The EE's recompiled tier, when chosen (S36, Sapphire_Native.md §44); the interpreter alone without it.
    pub blocks: Option<Box<crate::blocks::Blocks>>,
    /// Idle skipping (S33, Sapphire_Native.md §47): off unless chosen, so that a machine made here is the plain run.
    pub idle: crate::idle::Idle,
}

/// What is left of a frame's batch of 1,024 steps after `n` more, when `n` may run past its end: a skip crosses a
/// batch's end only where the plain run's check of the field would find no new field, no event falling inside it.
fn batch_left(left: u64, n: u64) -> u64 {
    if n <= left {
        return left - n;
    }
    match (n - left) % 1024 {
        0 => 0,
        over => 1024 - over,
    }
}

impl Machine {
    pub fn new(media: Media, market: Market, ports: [Option<&'static str>; 2]) -> Machine {
        let sys = Machine::console(market, ports);
        let mut m = Machine { media, market, sys, frames: 0, skip: false, pads: [0; 2], ports, firmware: Vec::new(), sticks: [[0x80; 4]; 2], booted: false, stop: None, program: None, entered: None, debug: crate::debugger::Debugger::new(), blocks: None, idle: crate::idle::Idle::new(false) };
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

    /// A disc in the drive when the console is switched on, before its first instruction: the drive spun up with it,
    /// as a console started with a game in it (Sapphire_Native.md §39.2).
    pub fn disc_at_power_on(&mut self, disc: crate::cdvd::Disc) {
        self.sys.board.iop.bus.cdvd.in_at_power_on(disc);
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
            TTY_ID => b.iop.tty.text.get(at).copied(),
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

    /// The fast boot (Sapphire_Native.md §36.2): an EE program given as the image is entered in OSDSYS's place once the
    /// BIOS is up, with no BIOS code read or changed. The IOP has finished booting (SMFLAG's bit 18, EESYNC's) and the EE
    /// runs a program in user mode, at an instruction boundary outside a delay slot.
    pub fn at_entry(&self) -> bool {
        self.sys.board.iop.bus.sif.smflg & IOP_BOOTED != 0 && !self.sys.ee.kernel() && !self.sys.ee.in_delay_slot()
    }

    /// Enters the program as its loader would: the EE's caches written back and emptied, the segments written into RAM
    /// with their bss cleared, and the kernel's ExecPS2 called in place of the running program's next instruction with
    /// the entry, `gp` from the program's register information and one argument, its name; the kernel clears its own
    /// state and starts the program as its main thread.
    pub fn enter_program(&mut self) {
        let Some(image) = self.program.take() else { return };
        self.entered = Some(self.frames + 1);
        self.enter(&image);
        self.program = Some(image);
    }

    /// Whether the fast boot still waits to enter its program.
    pub fn waiting(&self) -> bool {
        self.program.is_some() && self.entered.is_none()
    }

    fn enter(&mut self, image: &[u8]) {
        let Ok(crate::media::Media::Elf(elf)) = crate::media::Media::read(image) else { return };
        let System { ee, board } = &mut *self.sys;
        ee.flush_caches(board);
        for &(address, offset, file, memory) in &elf.segments {
            let ram = &mut board.map.rdram;
            for i in 0..memory {
                let at = (address.wrapping_add(i) & 0x1FFF_FFFF) as usize;
                if let Some(b) = ram.get_mut(at) {
                    *b = if i < file { image[(offset + i) as usize] } else { 0 };
                }
            }
        }
        let ram = &mut board.map.rdram;
        ram[ARGS..ARGS + 16].fill(0);
        ram[ARGS + 16..ARGS + 16 + PROGRAM_NAME.len()].copy_from_slice(PROGRAM_NAME);
        ram[ARGS..ARGS + 4].copy_from_slice(&((ARGS + 16) as u32).to_le_bytes());
        for (r, v) in [(3, EXEC_PS2 as u32), (4, elf.entry), (5, elf.gp), (6, 1), (7, ARGS as u32)] {
            ee.gpr[r] = v as i32 as i64 as u64 as u128;
        }
        let pc = ee.pc;
        ee.raise(crate::ee::Exception::Syscall, pc, false);
        board.iop.tty.device = true;
    }
    /// One interpreter step, which the verifier follows when one is attached.
    fn step(&mut self) -> Result<(), crate::ee::Halt> {
        match self.blocks.as_deref_mut() {
            Some(b) => crate::blocks::interpret(&mut self.sys, b).map(|_| ()),
            None => self.sys.step(),
        }
    }

    /// One frame: the console run until the PCRTC begins its next field, which is when V-blank rises.
    pub fn advance(&mut self) {
        let target = self.sys.board.pcrtc.fields + 1;
        while self.sys.board.pcrtc.fields < target {
            if self.booted && self.stop.is_none() && self.waiting() {
                if self.at_entry() {
                    self.enter_program();
                } else if self.step().is_err() {
                    self.stop = Some(Stop::Unimplemented);
                }
            } else if self.booted && self.stop.is_none() {
                let mut left = 1024;
                while left > 0 {
                    let from = self.sys.ee.pc;
                    let ran = match self.blocks.as_deref_mut() {
                        Some(b) => crate::blocks::run(&mut self.sys, b, left),
                        None => self.sys.step().map(|()| 1),
                    };
                    match ran {
                        Ok(n) => left -= n,
                        Err(_) => {
                            self.stop = Some(Stop::Unimplemented);
                            break;
                        }
                    }
                    // Once the field has begun the frame ends at this batch's end, which no skip may pass.
                    if left > 0 && self.idle.backward(from, self.sys.ee.pc, self.sys.ee.in_delay_slot()) && self.sys.board.pcrtc.fields < target {
                        let n = crate::idle::arrive(&mut self.sys, &mut self.idle, self.blocks.as_deref_mut(), left);
                        left = batch_left(left, n);
                    }
                }
            } else {
                let t = self.sys.board.now + IDLE_STEP;
                let System { ee, board } = &mut *self.sys;
                board.run_to(t, &mut ee.spram[..]);
            }
        }
        if self.booted {
            self.sys.board.iop.resolve_tty();
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

    /// The fast boot (S10): only once the IOP has finished booting and the EE runs in user mode is the program written
    /// past the caches, its bss cleared, every dirty line written back first, and ExecPS2 called in place of the next
    /// instruction with the entry, `gp` and the program's name as its one argument.
    #[test]
    fn the_fast_boot_enters_the_program_through_execps2_once_the_bios_is_up() {
        use crate::ee::cop0::{CAUSE, EPC, STATUS};
        let mut image = crate::media::program(&[0x1111_1111, 0x2222_2222], crate::media::PROGRAM_ORIGIN);
        let h = crate::media::ELF_HEADER;
        image[h + 20..h + 24].copy_from_slice(&64u32.to_le_bytes());
        let mut m = Machine::new(Media::read(&image).unwrap(), Market::Americas, [None, None]);
        m.insert_bios(&vec![0; BIOS_BYTES]);
        m.program = Some(image);
        m.sys.ee.cop0[STATUS] = 0x10;
        assert!(!m.at_entry(), "the IOP has not finished booting");
        m.sys.board.iop.bus.sif.smflg = IOP_BOOTED;
        m.sys.ee.cop0[STATUS] = 0;
        assert!(!m.at_entry(), "kernel mode");
        m.sys.ee.cop0[STATUS] = 0x10;
        assert!(m.at_entry());
        let base = crate::media::PROGRAM_ORIGIN;
        m.sys.board.map.rdram[base as usize + 8..base as usize + 64].fill(0x5A);
        let dc = &mut m.sys.ee.dcache;
        dc.tags[0][0] = crate::ee::cache::DTag { pfn: base >> 12, dirty: true, valid: true, lrf: false, lock: false };
        dc.data[0][0] = [0xEE; 64];
        dc.tags[0][1] = crate::ee::cache::DTag { pfn: 0x0020_0000 >> 12, dirty: true, valid: true, lrf: false, lock: false };
        dc.data[0][1] = [0xAB; 64];
        let pc = m.sys.ee.pc;
        m.enter_program();
        let ram = &m.sys.board.map.rdram;
        let at = base as usize;
        assert_eq!(&ram[at..at + 8], &[0x11, 0x11, 0x11, 0x11, 0x22, 0x22, 0x22, 0x22], "the segment over the line written back");
        assert!(ram[at + 8..at + 64].iter().all(|&b| b == 0), "the bss cleared");
        assert!(ram[0x20_0000..0x20_0040].iter().all(|&b| b == 0xAB), "a dirty line elsewhere written back");
        assert!(m.sys.ee.dcache.tags.iter().flatten().all(|t| !t.valid), "the data cache emptied");
        let g = |r: usize| m.sys.ee.gpr[r] as u32;
        assert_eq!((g(3), g(4), g(5), g(6), g(7)), (EXEC_PS2 as u32, base, 0, 1, ARGS as u32));
        let argv0 = u32::from_le_bytes(ram[ARGS..ARGS + 4].try_into().unwrap()) as usize;
        assert_eq!(&ram[argv0..argv0 + PROGRAM_NAME.len()], PROGRAM_NAME);
        assert_eq!(u32::from_le_bytes(ram[ARGS + 4..ARGS + 8].try_into().unwrap()), 0, "argv ends after the name");
        assert_eq!((m.sys.ee.cop0[CAUSE] >> 2 & 31, m.sys.ee.cop0[EPC], m.sys.ee.pc), (8, pc, 0x8000_0180), "SYSCALL taken at the next instruction");
        assert!(!m.waiting() && m.entered == Some(1));
    }
}
