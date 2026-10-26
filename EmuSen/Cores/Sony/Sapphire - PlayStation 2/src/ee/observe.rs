//! What the debugger sees of the EE core without changing it (stage S31, Sapphire_Native.md §41): where a virtual
//! address lands with no exception raised and no event counted, the bytes a load or store at the PC would reach, the
//! address the next instruction runs from, and the bytes of a place as the core sees them (a valid data cache line
//! over memory). The interpreter's step is untouched; the debugger's frame asks these before and after each step.

use super::cache::{DCache, ICache};
use super::cop0;
use super::Place;
use super::tlb::Target;
use super::{Ee, status};

/// RDRAM's physical size, the 32 MiB the map holds (EE User's Manual §2.1).
const RDRAM: u32 = 32 << 20;
/// The boot ROM's physical window.
const ROM: std::ops::Range<u32> = 0x1FC0_0000..0x2000_0000;

/// A load's or a store's reach: its first virtual address and its byte count, and whether it stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reach {
    pub vaddr: u32,
    pub bytes: u32,
    pub store: bool,
}

/// A load's or store's bytes, whether it stores, and for the unaligned forms and LQ/SQ the unit's alignment and whether
/// the form reaches from the unit's start up to the address.
type Shape = (u32, bool, Option<(u32, bool)>);

/// The bytes each load and store names, by the decoder's mnemonic.
fn shape(name: &str) -> Option<Shape> {
    Some(match name {
        "lb" | "lbu" => (1, false, None),
        "sb" => (1, true, None),
        "lh" | "lhu" => (2, false, None),
        "sh" => (2, true, None),
        "lw" | "lwu" | "lwc1" => (4, false, None),
        "sw" | "swc1" => (4, true, None),
        "ld" => (8, false, None),
        "sd" => (8, true, None),
        "lqc2" => (16, false, None),
        "sqc2" => (16, true, None),
        "lq" => (16, false, Some((16, false))),
        "sq" => (16, true, Some((16, false))),
        "lwl" => (4, false, Some((4, true))),
        "lwr" => (4, false, Some((4, false))),
        "swl" => (4, true, Some((4, true))),
        "swr" => (4, true, Some((4, false))),
        "ldl" => (8, false, Some((8, true))),
        "ldr" => (8, false, Some((8, false))),
        "sdl" => (8, true, Some((8, true))),
        "sdr" => (8, true, Some((8, false))),
        _ => return None,
    })
}

impl Ee {
    /// The address the instruction after the one at the PC runs from: a taken branch's target while the PC is its
    /// delay slot.
    pub fn next_pc(&self) -> u32 {
        self.next_pc
    }

    /// Where `vaddr` lands for a data access in the current mode, as `translate` would place it, with nothing changed:
    /// none where the access would raise an exception.
    pub fn place(&self, vaddr: u32) -> Option<Place> {
        let allowed = self.kernel() || vaddr < 0x8000_0000 || (self.supervisor() && (0xC000_0000..0xE000_0000).contains(&vaddr));
        if !allowed {
            return None;
        }
        match vaddr >> 29 {
            4 => return Some(Place::Physical(vaddr - 0x8000_0000, super::Mode::Cached)),
            5 => return Some(Place::Physical(vaddr - 0xA000_0000, super::Mode::Uncached)),
            0..=3 if self.status() & status::ERL != 0 => return Some(Place::Physical(vaddr, super::Mode::Uncached)),
            _ => {}
        }
        let hit = self.tlb.lookup(vaddr, self.cop0[cop0::ENTRYHI] & 0xFF)?;
        hit.valid.then_some(match hit.target {
            Target::Scratchpad(o) => Place::Scratchpad(o),
            Target::Physical { paddr, .. } => Place::Physical(paddr, super::Mode::Cached),
        })
    }

    /// The bytes the load or store `word` would reach if it ran now from the PC; none for any other instruction.
    pub fn reach(&self, word: u32) -> Option<Reach> {
        let emusen_mips::r5900::Op::Cpu(name, emusen_mips::r5900::Form::Mem(..)) = emusen_mips::r5900::operation(word)? else { return None };
        let (bytes, store, unit) = shape(name)?;
        let a = (self.gpr[(word >> 21 & 31) as usize] as u32).wrapping_add(word as u16 as i16 as i32 as u32);
        Some(match unit {
            None => Reach { vaddr: a, bytes, store },
            Some((16, false)) => Reach { vaddr: a & !15, bytes, store },
            Some((n, true)) => Reach { vaddr: a & !(n - 1), bytes: (a & (n - 1)) + 1, store },
            Some((n, false)) => Reach { vaddr: a, bytes: n - (a & (n - 1)), store },
        })
    }

    /// The byte at a place as the core sees it: the data cache's valid line over RDRAM, the scratchpad, the boot ROM;
    /// none for a register or a place no memory answers, which the debugger never reads.
    pub fn peek_byte(&self, rdram: &[u8], rom: &[u8], place: Place) -> Option<u8> {
        match place {
            Place::Scratchpad(o) => self.spram.get(o as usize & 0x3FFF).copied(),
            Place::Physical(p, _) if p < RDRAM => {
                let set = DCache::set(p);
                Some(match self.dcache.hit(set, p) {
                    Some(w) => self.dcache.data[set][w][(p & 63) as usize],
                    None => rdram.get(p as usize).copied()?,
                })
            }
            Place::Physical(p, _) if ROM.contains(&p) => Some(rom.get((p - ROM.start) as usize).copied().unwrap_or(0)),
            Place::Physical(..) => None,
        }
    }

    /// The word a fetch at `vaddr` would read, from the instruction cache's valid line or memory, with nothing changed;
    /// none where the fetch would fault or no memory answers.
    pub fn peek_code(&self, rdram: &[u8], rom: &[u8], vaddr: u32) -> Option<u32> {
        if vaddr & 3 != 0 {
            return None;
        }
        let place = self.place(vaddr)?;
        if let Place::Physical(p, _) = place
            && p < RDRAM
            && let Some(w) = self.icache.hit(ICache::set(vaddr), p)
        {
            return Some(self.icache.data[ICache::set(vaddr)][w][(p >> 2) as usize & 15]);
        }
        let mut b = [0u8; 4];
        for (i, v) in b.iter_mut().enumerate() {
            *v = self.peek_byte(rdram, rom, match place {
                Place::Scratchpad(o) => Place::Scratchpad(o + i as u32),
                Place::Physical(p, m) => Place::Physical(p + i as u32, m),
            })?;
        }
        Some(u32::from_le_bytes(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(text: &str) -> u32 {
        emusen_mips::assemble_line(text, 0, &emusen_mips::Dialect::R5900).unwrap()
    }

    /// Each form's bytes as the interpreter's own reach them: the unaligned forms within their unit, LQ and SQ at the
    /// quadword below, a load of a register into itself named by the base before it runs.
    #[test]
    fn each_load_and_store_reaches_the_bytes_the_interpreter_does() {
        let mut e = Ee::new(0x0010_0000);
        e.gpr[8] = 0x0020_0005;
        let r = |t: &str| e.reach(word(t)).map(|r| (r.vaddr, r.bytes, r.store));
        assert_eq!(r("lw t0, 3(t0)"), Some((0x0020_0008, 4, false)));
        assert_eq!(r("sb t1, -1(t0)"), Some((0x0020_0004, 1, true)));
        assert_eq!(r("lwl t1, 0(t0)"), Some((0x0020_0004, 2, false)), "the unit's first two bytes");
        assert_eq!(r("lwr t1, 1(t0)"), Some((0x0020_0006, 2, false)), "the unit's last two");
        assert_eq!(r("sdl t1, 2(t0)"), Some((0x0020_0000, 8, true)));
        assert_eq!(r("sdr t1, 2(t0)"), Some((0x0020_0007, 1, true)));
        assert_eq!(r("lq t1, 0x1a(t0)"), Some((0x0020_0010, 16, false)));
        assert_eq!(r("sqc2 vf1, 0xb(t0)"), Some((0x0020_0010, 16, true)));
        assert_eq!(r("addiu t0, t0, 4"), None);
        assert_eq!(r("cache 0x7, 0(t0)"), None, "CACHE names a line, not bytes it reads");
    }

    /// The place of an address answers as the access would, with no exception and no TLB miss counted.
    #[test]
    fn a_place_is_found_without_an_exception() {
        let e = Ee::new(0x0010_0000);
        assert_eq!(e.place(0x8001_0000), Some(Place::Physical(0x1_0000, super::super::Mode::Cached)));
        assert_eq!(e.place(0x0010_0000).map(|p| matches!(p, Place::Physical(0x10_0000, _))), Some(true));
        assert_eq!(e.place(0x7000_0010), Some(Place::Scratchpad(0x10)));
        assert_eq!(e.place(0x4000_0000), None, "no entry maps it");
        let mut user = Ee::new(0);
        user.cop0[cop0::STATUS] = 0x10;
        assert_eq!(user.place(0x8000_0000), None, "kseg0 from user mode");
        assert_eq!(e.exceptions, [0; 16]);
    }
}
