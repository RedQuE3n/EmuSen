//! The SIF (stage S22, Sapphire_Native.md §26), the interface between the EE and the IOP on the SBUS: the mailbox
//! registers each side reaches at its own address (ps2tek "SIF Registers"), and the FIFOs through which the EE's DMAC
//! channels 5 to 7 and the IOP's channels 9, 10 and 2 move SIF0 (IOP to EE), SIF1 (EE to IOP) and SIF2 (either way) as
//! one transfer (ps2tek "SIF DMA"; EE Overview §2.6). A FIFO holds 32 words, 128 bytes (EE User's Manual §1.2's table);
//! the EE moves quadwords of four words into and out of it, the IOP single words. The EE's side reaches it through
//! `EePorts`, the IOP's through `IopPorts`, each wrapping the stand-ins of the channels whose stages come later; a side
//! that changes a FIFO marks the other to be woken, since a channel held for want of words or room waits until then.
//! The SIF RPC protocol the BIOS's modules speak over these is S24's, above this unit.

use std::collections::VecDeque;

use crate::dmac;
use crate::iop::dma as iop_dma;

/// Each side's base: the EE's at 1000F200h, the IOP's at 1D000000h (ps2tek "SIF Registers").
pub const EE_BASE: u32 = 0x1000_F200;
pub const IOP_BASE: u32 = 0x1D00_0000;

/// The registers by offset from either base.
pub mod reg {
    /// Written by the EE only; read by both.
    pub const MSCOM: u32 = 0x00;
    /// Written by the IOP only; read by both.
    pub const SMCOM: u32 = 0x10;
    /// The EE's write sets bits, the IOP's clears the bits written as 1.
    pub const MSFLG: u32 = 0x20;
    /// The IOP's write sets bits, the EE's clears the bits written as 1.
    pub const SMFLG: u32 = 0x30;
    pub const CTRL: u32 = 0x40;
    pub const BD6: u32 = 0x60;
}

/// CTRL as each side reads it: bit 1 and bits 28-31 set on both, bit 8 on the EE's alone (ps2tek "SIF_CTRL";
/// Sapphire_Disputes.md D-141).
pub const CTRL_EE: u32 = 0xF000_0102;
pub const CTRL_IOP: u32 = 0xF000_0002;
/// The bit of the EE's write to CTRL that raises the IOP's interrupt 1, the one witness's reading (D-141).
pub const CTRL_IOP_INTERRUPT: u32 = 1 << 18;
/// The bit of the EE's write to CTRL that resets the IOP, the referee's behaviour (D-141), taken with the IOP's boot path
/// as decided 2026-10-25 by the tester (Sapphire_Disputes.md D-320).
pub const CTRL_IOP_RESET: u32 = 1 << 19;
/// The IOP's interrupt source the EE's CTRL write raises: ps2tek's IRQ1, "GPU (used in PSX mode)" (D-141).
pub const IOP_SOURCE: u32 = 1;

/// Words a FIFO holds: 128 bytes (EE User's Manual §1.2, "SIF ... 128-byte FIFO embedded"; D-140).
pub const FIFO_WORDS: usize = 32;

/// The three transfers, by FIFO.
pub const SIF0: usize = 0;
pub const SIF1: usize = 1;
pub const SIF2: usize = 2;

/// The register at each side's offset, if one is there: the EE's 1000F200h-1000F2FFh, the IOP's 1D000000h-1D0000FFh.
pub fn ee_register(paddr: u32) -> Option<u32> {
    (paddr & !0xFF == EE_BASE).then_some(paddr & 0xF0)
}

/// A FIFO of words, in the order they entered.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Fifo {
    words: VecDeque<u32>,
}

impl Fifo {
    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn room(&self) -> usize {
        FIFO_WORDS.saturating_sub(self.words.len())
    }

    pub fn push(&mut self, w: u32) {
        self.words.push_back(w);
    }

    /// The oldest word; a read of an empty FIFO gives 0, which a channel's request never allows.
    pub fn pop(&mut self) -> u32 {
        self.words.pop_front().unwrap_or(0)
    }

    pub fn words(&self) -> impl Iterator<Item = &u32> {
        self.words.iter()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sif {
    pub mscom: u32,
    pub smcom: u32,
    pub msflg: u32,
    pub smflg: u32,
    pub bd6: u32,
    /// The IOP's reset line as the EE's CTRL bit 19 last set it (D-320).
    pub reset_line: bool,
    /// SIF0's, SIF1's and SIF2's FIFOs.
    pub fifo: [Fifo; 3],
    /// A FIFO changed by the IOP's side, so a held EE channel should look again; and the reverse.
    wake_ee: bool,
    wake_iop: bool,
}

impl Sif {
    pub fn ee_read(&self, offset: u32) -> u32 {
        match offset {
            reg::MSCOM => self.mscom,
            reg::SMCOM => self.smcom,
            reg::MSFLG => self.msflg,
            reg::SMFLG => self.smflg,
            reg::CTRL => CTRL_EE,
            reg::BD6 => self.bd6,
            _ => 0,
        }
    }

    /// The EE's write; whether it raises the IOP's interrupt (D-141).
    pub fn ee_write(&mut self, offset: u32, v: u32) -> bool {
        match offset {
            reg::MSCOM => self.mscom = v,
            reg::MSFLG => self.msflg |= v,
            reg::SMFLG => self.smflg &= !v,
            reg::CTRL => return v & CTRL_IOP_INTERRUPT != 0,
            reg::BD6 => self.bd6 = v,
            _ => {}
        }
        false
    }

    pub fn iop_read(&self, offset: u32) -> u32 {
        match offset {
            reg::CTRL => CTRL_IOP,
            _ => self.ee_read(offset),
        }
    }

    pub fn iop_write(&mut self, offset: u32, v: u32) {
        match offset {
            reg::SMCOM => self.smcom = v,
            reg::MSFLG => self.msflg &= !v,
            reg::SMFLG => self.smflg |= v,
            reg::BD6 => self.bd6 = v,
            _ => {}
        }
    }

    /// Whether the IOP's side changed a FIFO since the last call.
    pub fn take_wake_ee(&mut self) -> bool {
        std::mem::take(&mut self.wake_ee)
    }

    /// Whether the EE's side changed a FIFO since the last call.
    pub fn take_wake_iop(&mut self) -> bool {
        std::mem::take(&mut self.wake_iop)
    }

    fn ee_take(&mut self, f: usize, n: usize) -> u128 {
        self.wake_iop = true;
        (0..n).fold(0u128, |q, i| q | u128::from(self.fifo[f].pop()) << (32 * i))
    }

    fn ee_give(&mut self, f: usize, q: u128) {
        self.wake_iop = true;
        for i in 0..4 {
            self.fifo[f].push((q >> (32 * i)) as u32);
        }
    }
}

/// The EE DMAC's peripherals: SIF0, SIF1 and SIF2 through the SIF, the other channels through `others`. SIF2's
/// direction is the EE channel's CHCR.DIR, `sif2_from_memory`.
pub struct EePorts<'a, P: dmac::Peripherals> {
    pub others: &'a mut P,
    pub sif: &'a mut Sif,
    pub sif2_from_memory: bool,
}

impl<P: dmac::Peripherals> EePorts<'_, P> {
    fn fifo(ch: usize) -> Option<usize> {
        match ch {
            dmac::SIF0 => Some(SIF0),
            dmac::SIF1 => Some(SIF1),
            dmac::SIF2 => Some(SIF2),
            _ => None,
        }
    }
}

impl<P: dmac::Peripherals> dmac::Peripherals for EePorts<'_, P> {
    /// Quadwords: SIF0's whole quadwords waiting, SIF1's room for whole ones, SIF2's by its direction.
    fn request(&mut self, ch: usize, want: usize) -> usize {
        let Some(f) = Self::fifo(ch) else { return self.others.request(ch, want) };
        let from_memory = f == SIF1 || (f == SIF2 && self.sif2_from_memory);
        let q = if from_memory { self.sif.fifo[f].room() / 4 } else { self.sif.fifo[f].len() / 4 };
        want.min(q)
    }

    fn put(&mut self, ch: usize, q: u128, tag: bool) {
        match Self::fifo(ch) {
            Some(f) => self.sif.ee_give(f, q),
            None => self.others.put(ch, q, tag),
        }
    }

    fn get(&mut self, ch: usize) -> u128 {
        match Self::fifo(ch) {
            Some(f) => self.sif.ee_take(f, 4),
            None => self.others.get(ch),
        }
    }

    /// SIF0's destination tag is the two words the IOP sends before a packet's data (ps2tek "IOP DMA"; D-142).
    fn tag_ready(&mut self, ch: usize) -> bool {
        match ch {
            dmac::SIF0 => self.sif.fifo[SIF0].len() >= 2,
            _ => self.others.tag_ready(ch),
        }
    }

    fn get_tag(&mut self, ch: usize) -> u128 {
        match ch {
            dmac::SIF0 => self.sif.ee_take(SIF0, 2),
            _ => self.others.get_tag(ch),
        }
    }
}

/// The IOP DMAC's peripherals: SIF0 (9), SIF1 (10) and SIF2 (2) through the SIF, the other channels through `others`.
pub struct IopPorts<'a, P: iop_dma::Peripherals> {
    pub others: &'a mut P,
    pub sif: &'a mut Sif,
}

impl<P: iop_dma::Peripherals> IopPorts<'_, P> {
    fn fifo(ch: usize) -> Option<usize> {
        match ch {
            iop_dma::SIF0 => Some(SIF0),
            iop_dma::SIF1 => Some(SIF1),
            iop_dma::SIF2 => Some(SIF2),
            _ => None,
        }
    }
}

impl<P: iop_dma::Peripherals> iop_dma::Peripherals for IopPorts<'_, P> {
    /// Words: SIF0's room, SIF1's words waiting, SIF2's by the direction asked.
    fn request(&mut self, ch: usize, want: u32, to_memory: bool) -> u32 {
        let Some(f) = Self::fifo(ch) else { return self.others.request(ch, want, to_memory) };
        let gives = f == SIF1 || (f == SIF2 && to_memory);
        let n = if gives { self.sif.fifo[f].len() } else { self.sif.fifo[f].room() };
        want.min(n as u32)
    }

    fn put(&mut self, ch: usize, w: u32) {
        match Self::fifo(ch) {
            Some(f) => {
                self.sif.wake_ee = true;
                self.sif.fifo[f].push(w);
            }
            None => self.others.put(ch, w),
        }
    }

    fn get(&mut self, ch: usize) -> u32 {
        match Self::fifo(ch) {
            Some(f) => {
                self.sif.wake_ee = true;
                self.sif.fifo[f].pop()
            }
            None => self.others.get(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flags_set_on_one_side_and_clear_on_the_other() {
        let mut s = Sif::default();
        s.ee_write(reg::MSFLG, 0x2_0000);
        s.ee_write(reg::MSFLG, 0x1_0000);
        assert_eq!(s.iop_read(reg::MSFLG), 0x3_0000);
        s.iop_write(reg::MSFLG, 0x2_0000);
        assert_eq!(s.ee_read(reg::MSFLG), 0x1_0000);
        s.iop_write(reg::SMFLG, 0x4_0000);
        s.ee_write(reg::SMFLG, 0x4_0000);
        assert_eq!(s.iop_read(reg::SMFLG), 0);
        s.iop_write(reg::MSCOM, 5);
        s.ee_write(reg::SMCOM, 6);
        assert_eq!((s.ee_read(reg::MSCOM), s.iop_read(reg::SMCOM)), (0, 0), "each COM is written by one side only");
        assert!(s.ee_write(reg::CTRL, CTRL_IOP_INTERRUPT) && !s.ee_write(reg::CTRL, 0x100));
        assert_eq!((s.ee_read(reg::CTRL), s.iop_read(reg::CTRL)), (CTRL_EE, CTRL_IOP));
    }

    #[test]
    fn a_quadword_is_four_words_low_first_and_a_full_fifo_takes_no_more() {
        let mut s = Sif::default();
        let mut stand = crate::board::StandIns::default();
        let mut ee = EePorts { others: &mut stand, sif: &mut s, sif2_from_memory: false };
        use dmac::Peripherals;
        assert_eq!(ee.request(dmac::SIF1, 100), 8, "32 words of room are eight quadwords");
        ee.put(dmac::SIF1, 0x4444_4444_3333_3333_2222_2222_1111_1111, false);
        assert_eq!(ee.request(dmac::SIF1, 100), 7);
        assert_eq!(ee.request(dmac::SIF0, 1), 0, "SIF0 is empty");
        assert!(s.take_wake_iop() && !s.take_wake_iop());
        assert_eq!(s.fifo[SIF1].words().copied().collect::<Vec<_>>(), [0x1111_1111, 0x2222_2222, 0x3333_3333, 0x4444_4444]);
    }
}
