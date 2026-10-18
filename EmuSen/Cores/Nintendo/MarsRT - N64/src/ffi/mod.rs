//! MarsRT's machine and its host, the VI's scan-out, behind the core ABI v1 (`v1`), and C# Mars's native components (`rdp`,
//! `vi`). A negative return is a status. See Mars_Native.md §5.1, §5.2, §5.5 and §7.

pub mod blocks;
pub mod host;
pub mod multiple;
pub mod rdp;
pub mod threads;
pub mod v1;
pub mod vi;

use std::sync::Arc;

use emusen_native::core::status;

use crate::cpu::cop0::ENTRY_HI;
use crate::cpu::segments::{self, Mode, Segment};
use crate::cpu::tlb::TlbResult;
use crate::machine::Machine;
use crate::memory::bus_access::map;
use crate::memory::rom_patches::{RomPatch, RomPatches};
use crate::vi::scan::Scanout;

/// A null handle or buffer.
pub const STATUS_NULL: i32 = status::NULL;
/// An image that is not a Nintendo 64 cartridge.
pub const STATUS_NOT_A_ROM: i32 = -9;
/// A memory this library has no number for.
pub const STATUS_NO_SUCH_SPACE: i32 = status::NO_SUCH_SPACE;
/// A memory the host may read and not write.
pub const STATUS_READ_ONLY: i32 = status::READ_ONLY;

/// The memories a host names, as C#'s `MarsDebugSpaces` names them; `CPU` is the processor's kernel view of a virtual address.
pub mod space {
    pub const RDRAM: u32 = 0;
    pub const DMEM: u32 = 1;
    pub const IMEM: u32 = 2;
    pub const PIF_RAM: u32 = 3;
    pub const ROM: u32 = 4;
    pub const CPU: u32 = 5;
}

/// The machine and its host: the VI's scan-out, one for the machine's life as C#'s raster is, and whether a frame is scanned at all.
pub struct Core {
    pub machine: Machine,
    pub scanout: Scanout,
    /// Advanced wherever the shown frame is replaced, as C#'s `_frameSerial`.
    pub frame_serial: i64,
    pub skip_rendering: bool,
    pub shown: bool,
    /// C#'s `RenderScale`, `Antialiasing` and `Gpu` as asked for; what they got is the machine's scale and the scan-out's average (Mars_Native.md §6.4).
    pub render_scale: i32,
    pub antialiasing: i32,
    pub gpu: bool,
    /// Whether a multiple has been applied yet; until then even the values a fresh core holds are sent (Mars_Native.md §6.4.8).
    pub multiple_applied: bool,
    /// The last frame's phases on the host's thread, in nanoseconds, in `PHASES`' order (Mars_Native.md §6.6.2).
    pub profile: [i64; PHASES],
    /// The run-scope settings last applied, none before the first, so a value sent again unchanged does nothing (Mars_Native.md
    /// §6.4.8, §7.2).
    pub applied: Option<v1::Values>,
    /// The battery files a load replaced and the host has not written since, by file number: kept outside the state, so a state
    /// saved, loaded and saved again is the same bytes (Mars_Native.md §7.4).
    pub unsaved: [bool; v1::FILES],
    /// The shape and the two sizes last told to the host, so a frame that changes only the registers written is told its sizes
    /// without measuring them again (Mars_Native.md §7.2).
    pub told: Option<(crate::memory::bus::StateShape, i64, i64)>,
    /// The host keeps the Controller Paks apart from the game (`CREATE_SHARED_REMOVABLE`): a load leaves them as they stand
    /// (Mars_Native.md §7.7).
    pub paks_apart: bool,
    /// What the host has been told beside the frame: the IS-Viewer's bytes already logged, the device report last logged, and
    /// the frame rate machine info last gave (Mars_Native.md §9).
    pub is_viewer_logged: usize,
    pub gpu_logged: String,
    pub rate_told: (u64, u64),
}

/// `machine`, `machine/rdp-wait`, `present`, `present/rdp-wait`, `present/walk-join`: the phases `emusen_core_phases` reports.
pub const PHASES: usize = 5;

impl Drop for Core {
    /// A walk still out reads the device's pictures and the shadow, so it is joined before the machine goes.
    fn drop(&mut self) {
        self.scanout.join();
    }
}

impl Core {
    pub fn new(machine: Machine) -> Core {
        Core { machine, scanout: Scanout::default(), frame_serial: 0, skip_rendering: false, shown: false, render_scale: 1, antialiasing: 1, gpu: false, multiple_applied: false, profile: [0; PHASES], applied: None, unsaved: [false; v1::FILES], told: None, paks_apart: false, is_viewer_logged: 0, gpu_logged: "off".into(), rate_told: (0, 0) }
    }

    /// Every wait of this thread for the display processor's drain, in nanoseconds; zero while it runs on this thread.
    pub fn drain_waited(&self) -> i64 {
        self.machine.bus.dp.threads.as_deref().map_or(0, |t| t.counters.page_wait_nanos + t.counters.range_wait_nanos + t.counters.join_nanos)
    }

    /// The machine to the field's end, its time and its waits for the drain the frame's first two phases.
    pub fn advance(&mut self) {
        let (started, waited) = (std::time::Instant::now(), self.drain_waited());
        self.machine.run_frame();
        self.profile = [started.elapsed().as_nanos() as i64, (self.drain_waited() - waited).max(0), 0, 0, 0];
    }

    /// `present`, its time, its waits for the drain and for the walk still out the frame's last three phases.
    pub fn present_profiled(&mut self) {
        let (started, waited, joined) = (std::time::Instant::now(), self.drain_waited(), self.scanout.presenter_waits().1);
        self.present();
        self.profile[2] = started.elapsed().as_nanos() as i64;
        self.profile[3] = (self.drain_waited() - waited).max(0);
        self.profile[4] = (self.scanout.presenter_waits().1 - joined).max(0);
    }

    /// `Present`: the VI's scan of what the machine left, at once or deferred as the machine is set; the serial moves when the shown frame does.
    pub fn present(&mut self) {
        let presented = self.machine.present(&mut self.scanout);
        self.shown = presented.walked;
        self.frame_serial += presented.replaced as i64;
    }

    pub fn run_frame(&mut self) {
        self.machine.run_frame();
        if !self.skip_rendering {
            self.present();
        }
    }

    /// `MarsDebugSpaces.TryPhysical`: kernel mode's view of a 32-bit virtual address, the TLB consulted and never faulted.
    pub fn physical(&self, address: u32) -> Option<u32> {
        let wide = address as i32 as i64 as u64;
        match segments::decode(wide, Mode::Kernel, false) {
            Segment::Direct(physical) => Some(physical),
            Segment::Mapped => match self.machine.cpu.tlb.try_translate(wide, self.machine.cpu.cop0[ENTRY_HI], false) {
                TlbResult::Mapped(physical) => Some(physical),
                _ => None,
            },
            Segment::Illegal => None,
        }
    }

    /// `MarsDebugSpaces.Window`: which memory a physical address names and where, for memories only, since a register can answer a read by changing.
    fn window(&self, physical: u32) -> Option<(u32, usize)> {
        let bus = &self.machine.bus;
        if (physical as usize) < bus.rdram.len() {
            return Some((space::RDRAM, physical as usize));
        }
        if (map::SP_DMEM_BASE..map::SP_REGISTERS_BASE).contains(&physical) {
            let local = (physical - map::SP_DMEM_BASE) % (2 * map::SP_MEM_SIZE);
            return Some(if local < map::SP_MEM_SIZE { (space::DMEM, local as usize) } else { (space::IMEM, (local - map::SP_MEM_SIZE) as usize) });
        }
        if (map::PIF_RAM_BASE..map::PIF_RAM_BASE + map::PIF_RAM_SIZE).contains(&physical) {
            return Some((space::PIF_RAM, (physical - map::PIF_RAM_BASE) as usize));
        }
        if (map::CART_DOMAIN1_ADDRESS2..map::PIF_ROM_BASE).contains(&physical) {
            let offset = (physical - map::CART_DOMAIN1_ADDRESS2) as usize;
            if offset < self.memory(space::ROM).map_or(0, <[u8]>::len) {
                return Some((space::ROM, offset));
            }
        }
        None
    }

    /// A named memory's bytes, or none for a number that names no memory.
    pub fn memory(&self, space: u32) -> Option<&[u8]> {
        let bus = &self.machine.bus;
        match space {
            space::RDRAM => Some(&bus.rdram),
            space::DMEM => Some(&bus.sp_dmem[..]),
            space::IMEM => Some(&bus.sp_imem[..]),
            space::PIF_RAM => Some(&bus.pif_ram),
            space::ROM => Some(bus.cart.as_ref().map_or(&[][..], |rom| &rom.rom[..])),
            _ => None,
        }
    }

    fn memory_mut(&mut self, space: u32) -> Option<&mut [u8]> {
        let bus = &mut self.machine.bus;
        match space {
            space::RDRAM => Some(&mut bus.rdram),
            space::DMEM => Some(&mut bus.sp_dmem[..]),
            space::IMEM => Some(&mut bus.sp_imem[..]),
            space::PIF_RAM => Some(&mut bus.pif_ram),
            _ => None,
        }
    }

    /// Bytes from a memory into `out`, zero wherever the address names nothing, and nothing disturbed; the host reads between frames.
    pub fn read_memory(&self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        // C# waits at site 9 per byte; RDRAM's whole slice is formed below, so the drain is waited for whole (Mars_Native.md §5.6.4).
        if space == space::RDRAM || space == space::CPU {
            self.machine.bus.dp.wait_all();
        }
        if space == space::CPU {
            for (i, byte) in out.iter_mut().enumerate() {
                *byte = self
                    .physical(address.wrapping_add(i as u32))
                    .and_then(|physical| self.window(physical))
                    .and_then(|(space, offset)| self.memory(space).map(|bytes| bytes[offset]))
                    .unwrap_or(0);
            }
            return Ok(());
        }
        let bytes = self.memory(space).ok_or(STATUS_NO_SUCH_SPACE)?;
        out.fill(0);
        let start = address as usize;
        if start < bytes.len() {
            let n = out.len().min(bytes.len() - start);
            out[..n].copy_from_slice(&bytes[start..start + n]);
        }
        Ok(())
    }

    /// Bytes into a memory, those past its end dropped, as a store to memory not installed is; returns how many landed, and any lands as a write the idle loop sees.
    pub fn write_memory(&mut self, space: u32, address: u32, data: &[u8]) -> Result<usize, i32> {
        if space == space::RDRAM || space == space::CPU {
            self.machine.join_rdp();
        }
        let mut landed = 0;
        if space == space::CPU {
            for (i, &value) in data.iter().enumerate() {
                let Some((space, offset)) = self.physical(address.wrapping_add(i as u32)).and_then(|physical| self.window(physical)) else { continue };
                if let Some(bytes) = self.memory_mut(space) {
                    bytes[offset] = value;
                    landed += 1;
                }
            }
        } else {
            if space == space::ROM {
                return Err(STATUS_READ_ONLY);
            }
            let bytes = self.memory_mut(space).ok_or(STATUS_NO_SUCH_SPACE)?;
            let start = address as usize;
            if start < bytes.len() {
                landed = data.len().min(bytes.len() - start);
                bytes[start..start + landed].copy_from_slice(&data[..landed]);
            }
        }
        if landed > 0 {
            *self.machine.bus.written += 1;
        }
        Ok(landed)
    }

    /// The registry's ROM patches as the host resolved them, three words each: the ROM offset, the value, and the byte compared or
    /// `NO_COMPARE`; they replace the last list, and none clears it (Mars_Native.md §6.6.1).
    pub fn set_rom_patches(&mut self, words: &[u32]) {
        let entries = words
            .chunks_exact(3)
            .map(|w| RomPatch { address: w[0], value: w[1] as u8, compare: (w[2] != NO_COMPARE).then_some(w[2] as u8) })
            .collect();
        self.machine.bus.rom_patches = crate::Skip(RomPatches::new(entries).map(Arc::new));
    }
}

/// The third word of a ROM patch that compares nothing.
pub const NO_COMPARE: u32 = emusen_native::core::flags::NO_COMPARE;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::bus::RDRAM_SIZE;

    fn core() -> Core {
        Core::new(Machine::new(RDRAM_SIZE).unwrap())
    }

    #[test]
    fn a_read_past_a_memory_s_end_is_zero_and_a_write_there_is_dropped() {
        let mut c = core();
        c.machine.bus.rdram[RDRAM_SIZE - 1] = 0x5A;
        let mut out = [0xEEu8; 4];
        c.read_memory(space::RDRAM, (RDRAM_SIZE - 1) as u32, &mut out).unwrap();
        assert_eq!(out, [0x5A, 0, 0, 0]);
        assert_eq!(c.write_memory(space::RDRAM, (RDRAM_SIZE - 2) as u32, &[1, 2, 3]).unwrap(), 2);
        assert_eq!(&c.machine.bus.rdram[RDRAM_SIZE - 2..], &[1, 2]);
        assert_eq!(c.write_memory(space::PIF_RAM, 64, &[9]).unwrap(), 0);
    }

    #[test]
    fn a_write_that_lands_is_one_the_idle_loop_sees_and_one_that_does_not_is_not() {
        let mut c = core();
        let before = *c.machine.bus.written;
        c.write_memory(space::DMEM, 0x2000, &[1]).unwrap();
        assert_eq!(*c.machine.bus.written, before);
        c.write_memory(space::DMEM, 0xFFF, &[1]).unwrap();
        assert_eq!(*c.machine.bus.written, before + 1);
        assert_eq!(c.machine.bus.sp_dmem[0xFFF], 1);
    }

    #[test]
    fn the_rom_is_read_and_never_written_and_an_unknown_space_is_refused() {
        let mut c = core();
        assert_eq!(c.write_memory(space::ROM, 0, &[1]), Err(STATUS_READ_ONLY));
        assert_eq!(c.write_memory(9, 0, &[1]), Err(STATUS_NO_SUCH_SPACE));
        assert_eq!(c.read_memory(9, 0, &mut [0]), Err(STATUS_NO_SUCH_SPACE));
    }

    #[test]
    fn the_processor_s_view_reaches_rdram_through_both_direct_segments_and_imem_through_its_mirror() {
        let mut c = core();
        c.machine.bus.rdram[0x1234] = 0x11;
        c.machine.bus.sp_imem[0x10] = 0x22;
        let mut out = [0u8; 1];
        for address in [0x8000_1234u32, 0xA000_1234] {
            c.read_memory(space::CPU, address, &mut out).unwrap();
            assert_eq!(out[0], 0x11);
        }
        c.read_memory(space::CPU, 0xA400_1010 + 0x2000, &mut out).unwrap();
        assert_eq!(out[0], 0x22);
        c.write_memory(space::CPU, 0x8000_0010, &[0x33]).unwrap();
        assert_eq!(c.machine.bus.rdram[0x10], 0x33);
    }

    #[test]
    fn an_unmapped_address_in_the_processor_s_view_reads_zero_and_takes_no_write() {
        let mut c = core();
        let mut out = [0xEEu8; 2];
        c.read_memory(space::CPU, 0x0000_1000, &mut out).unwrap();
        assert_eq!(out, [0, 0]);
        assert_eq!(c.write_memory(space::CPU, 0x0000_1000, &[1, 2]).unwrap(), 0);
    }
}
