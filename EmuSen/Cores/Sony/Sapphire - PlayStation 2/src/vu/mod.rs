//! The vector units (Sapphire_Plan.md §7, S12 and S13): one `Vu` per unit with its registers, flags and memories, the
//! operations both units execute (`ops`, decoded by `emusen_mips::vu` alone and computed on `crate::float`; the EFU's
//! in `efu`), and `Vpu`, VU0 and VU1 with what the EE reaches through COP2: VU0's control registers, VU0's divider in
//! macro mode, and each unit's micro mode behind `micro::Micro`, which `pipe::Pipe` implements with the pipelines of the
//! manual's §3.4. What the units reach outside the VPU (PATH1, VIF's TOP and ITOP, the debug interrupts) is `Link`.
//! Written from the VU User's Manual; the record is Sapphire_Native.md §21 (S12) and §23 (S13).

pub mod efu;
pub mod micro;
pub mod ops;
pub mod pipe;

use micro::Micro;

/// VU0's memories are 4 KiB each, VU1's 16 KiB (VU User's Manual §1.1.5, §1.1.6).
pub const VU0_MEM: usize = 4 << 10;
pub const VU1_MEM: usize = 16 << 10;

/// The status flag's bits (§3.3.2): Z, S, U, O, I, D, then their sticky flags ZS to DS.
pub mod status {
    pub const Z: u32 = 1;
    pub const S: u32 = 1 << 1;
    pub const U: u32 = 1 << 2;
    pub const O: u32 = 1 << 3;
    pub const I: u32 = 1 << 4;
    pub const D: u32 = 1 << 5;
    pub const IS: u32 = 1 << 10;
    pub const DS: u32 = 1 << 11;
    /// The sticky flags lie six bits above the flags they collect.
    pub const STICKY: u32 = 6;
    /// What CTC2 may write: the sticky flags (§5.1.3: "values written to the lower 6 bits … are ignored").
    pub const WRITABLE: u32 = 0xFC0;
}

/// A result on its way to Q or P: the first cycle whose T stage reads it, the value, and the status flag's I and D it
/// writes as it lands (a micro-mode divide's; a macro divide writes them at once, and the EFU none).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Landing {
    pub at: u64,
    pub value: u32,
    pub flags: Option<u32>,
}

/// One vector unit's state: 32 vector registers (x in bits 31..0 of a field array's element 0), 16 integer registers,
/// ACC, I, Q, P, R, the three flag registers, TPC, and its data and micro memories; the divider's and the EFU's result in
/// flight, and the first cycle each may start another.
#[derive(Clone)]
pub struct Vu {
    vf: [[u32; 4]; 32],
    vi: [u16; 16],
    pub acc: [u32; 4],
    pub i: u32,
    pub q: u32,
    pub p: u32,
    /// The random number register, 23 bits (§3.1.6).
    pub r: u32,
    /// 12 bits (§3.3.2).
    pub status: u32,
    /// 16 bits: O, U, S and Z, each x to w from its high bit (§3.3.1).
    pub mac: u32,
    /// 24 bits: the current judgement and the three before it (§3.3.3).
    pub clip: u32,
    pub tpc: u16,
    pub data: Box<[u8]>,
    pub micro: Box<[u8]>,
    pub q_next: Option<Landing>,
    pub div_free: u64,
    pub p_next: Option<Landing>,
    /// A second EFU result, which can start a cycle before the first lands (§3.4.6).
    pub p_later: Option<Landing>,
    pub efu_free: u64,
}

/// VF00's fixed value: 0.0, 0.0, 0.0, 1.0 (§3.1.1).
const VF00: [u32; 4] = [0, 0, 0, 0x3F80_0000];

impl Vu {
    pub fn new(mem: usize) -> Vu {
        let mut vf = [[0; 4]; 32];
        vf[0] = VF00;
        Vu {
            vf,
            vi: [0; 16],
            acc: [0; 4],
            i: 0,
            q: 0,
            p: 0,
            r: 0,
            status: 0,
            mac: 0,
            clip: 0,
            tpc: 0,
            data: vec![0; mem].into_boxed_slice(),
            micro: vec![0; mem].into_boxed_slice(),
            q_next: None,
            div_free: 0,
            p_next: None,
            p_later: None,
            efu_free: 0,
        }
    }

    /// Lands a quotient and an EFU result due by the T stage at cycle `t` (§3.4.5, §3.4.6: neither is hazard-checked,
    /// so a reader before then sees the old value).
    pub fn settle(&mut self, t: u64) {
        if let Some(l) = self.q_next
            && l.at <= t
        {
            self.q = l.value;
            if let Some(f) = l.flags {
                ops::apply_divide_flags(self, f);
            }
            self.q_next = None;
        }
        while let Some(l) = self.p_next
            && l.at <= t
        {
            self.p = l.value;
            self.p_next = self.p_later.take();
        }
    }

    /// Sends an EFU result to P at its landing, behind one still in flight.
    pub fn send_p(&mut self, l: Landing) {
        if self.p_next.is_some() { self.p_later = Some(l) } else { self.p_next = Some(l) }
    }

    /// The most recent EFU result in flight, which WAITP waits for.
    pub fn p_last(&self) -> Option<Landing> {
        self.p_later.or(self.p_next)
    }

    pub fn vf(&self, n: usize) -> [u32; 4] {
        self.vf[n]
    }

    /// Writes a vector register; VF00 keeps its constants (§3.1.1).
    pub fn set_vf(&mut self, n: usize, v: [u32; 4]) {
        if n != 0 {
            self.vf[n] = v;
        }
    }

    /// The register as 128 bits, x lowest (§5.1.1, CPR[2,n]).
    pub fn vf_quad(&self, n: usize) -> u128 {
        self.vf[n].iter().rev().fold(0u128, |a, &f| a << 32 | f as u128)
    }

    pub fn set_vf_quad(&mut self, n: usize, q: u128) {
        self.set_vf(n, [q as u32, (q >> 32) as u32, (q >> 64) as u32, (q >> 96) as u32]);
    }

    /// An integer register; only VI00 to VI15 exist, and a field naming 16 to 31 reads zero [A].
    pub fn vi(&self, n: usize) -> u16 {
        if n < 16 { self.vi[n] } else { 0 }
    }

    /// Writes an integer register; VI00 stays zero and a field naming 16 to 31 writes nothing (§3.1.2) [A].
    pub fn set_vi(&mut self, n: usize, v: u16) {
        if (1..16).contains(&n) {
            self.vi[n] = v;
        }
    }

    /// The control registers' initial values on a reset (§5.1.3): the three flag registers cleared.
    pub fn reset_flags(&mut self) {
        self.status = 0;
        self.mac = 0;
        self.clip = 0;
    }

    /// The qword at index `a` of the data memory, which mirrors past its end [A].
    pub fn mem_q(&self, a: usize) -> u128 {
        let at = (a * 16) % self.data.len();
        u128::from_le_bytes(self.data[at..at + 16].try_into().unwrap())
    }

    pub fn set_mem_q(&mut self, a: usize, q: u128) {
        let at = (a * 16) % self.data.len();
        self.data[at..at + 16].copy_from_slice(&q.to_le_bytes());
    }

    /// The register a VU0 data address from 4000h reaches (§5.5.3 to §5.5.5): VU1's registers by qword, 0 for a
    /// reserved one, as 128 bits with the unused bits zero.
    pub fn window_read(&self, k: usize) -> u128 {
        match k {
            0..32 => self.vf_quad(k),
            32..48 => self.vi[k - 32] as u128,
            48 => self.status as u128,
            49 => self.mac as u128,
            50 => self.clip as u128,
            52 => self.r as u128,
            53 => self.i as u128,
            54 => self.q as u128,
            55 => self.p as u128,
            58 => self.tpc as u128,
            _ => 0,
        }
    }

    /// A write through the window: each register keeps its width; the MAC flags and TPC are read-only, and the status
    /// flag takes its sticky bits alone, as through CTC2 (§5.5.5's table).
    pub fn window_write(&mut self, k: usize, q: u128) {
        let w = q as u32;
        match k {
            0..32 => self.set_vf_quad(k, q),
            32..48 => self.set_vi(k - 32, w as u16),
            48 => self.status = self.status & !status::WRITABLE | w & status::WRITABLE,
            50 => self.clip = w & 0xFF_FFFF,
            52 => self.r = w & 0x7F_FFFF,
            53 => self.i = w,
            54 => self.q = w,
            55 => self.p = w,
            _ => {}
        }
    }
}

/// PATH1 as VU1's XGKICK reaches it: the packet's quadword address in VU1's data memory and the cycle of the kick, and
/// the cycle the transfer ends (§3.4.9). The GIF implements it at S15.
pub trait Path1 {
    fn kick(&mut self, data: &[u8], addr: u16, at: u64) -> u64;
}

/// PATH1 until the GIF exists: each kick recorded, every transfer taking `busy` cycles.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Path1StandIn {
    pub kicks: Vec<(u16, u64)>,
    pub busy: u64,
}

impl Path1 for Path1StandIn {
    fn kick(&mut self, _: &[u8], addr: u16, at: u64) -> u64 {
        self.kicks.push((addr, at));
        at + self.busy
    }
}

/// What the units reach outside the VPU: FBRST's D and T bit enables (§5.1.3), the interrupt a D or T bit raised (bit 0
/// VU0, bit 1 VU1, §7.3.1), VIF0's and VIF1's TOP and ITOP for XTOP and XITOP (S14 drives them), and PATH1.
pub struct Link {
    pub fbrst: u32,
    pub irq: u32,
    pub top: [u16; 2],
    pub itop: [u16; 2],
    pub path1: Box<dyn Path1>,
}

impl Default for Link {
    fn default() -> Self {
        Link { fbrst: 0, irq: 0, top: [0; 2], itop: [0; 2], path1: Box::new(Path1StandIn::default()) }
    }
}

/// VU0 and VU1, and their link to the rest of the machine.
pub struct Units {
    pub vu0: Vu,
    pub vu1: Vu,
    pub link: Link,
}

/// FBRST's bits (§5.1.3): force break, reset, D bit enable and T bit enable for VU0, the same eight bits up for VU1.
pub mod fbrst {
    pub const FB0: u32 = 1;
    pub const RS0: u32 = 1 << 1;
    pub const DE0: u32 = 1 << 2;
    pub const TE0: u32 = 1 << 3;
    pub const KEPT: u32 = 0x0C0C;
    pub const VU1: u32 = 8;
}

/// VPU-STAT's bits for VU0 (§5.1.3); VU1's are eight bits up, with VGW and EFU there.
pub mod vpu_stat {
    pub const VBS: u32 = 1;
    pub const VDS: u32 = 1 << 1;
    pub const VTS: u32 = 1 << 2;
    pub const VFS: u32 = 1 << 3;
    pub const VGW: u32 = 1 << 4;
    pub const DIV: u32 = 1 << 5;
    pub const EFU: u32 = 1 << 6;
    pub const VU1: u32 = 8;
}

/// VU0's control register numbers in CCR[2,16] to CCR[2,31] (§5.1.3).
pub mod ccr {
    pub const STATUS: usize = 16;
    pub const MAC: usize = 17;
    pub const CLIP: usize = 18;
    pub const R: usize = 20;
    pub const I: usize = 21;
    pub const Q: usize = 22;
    pub const TPC: usize = 26;
    pub const CMSAR0: usize = 27;
    pub const FBRST: usize = 28;
    pub const VPU_STAT: usize = 29;
    pub const CMSAR1: usize = 31;
}

/// What CCR[2,19] reads, the value ps2autotests' `cpu/vu0_macro/transfer` console text shows after its write
/// (Sapphire_Disputes.md D-45).
pub const CCR19: u32 = 0x2E30;

/// The vector processing units as the EE sees them: both units, VU0's control registers, and each unit's micro mode.
pub struct Vpu {
    pub units: Units,
    pub cmsar0: u16,
    pub cmsar1: u16,
    /// CCR[2,24], which keeps the bits written to it under 0C0Ch (D-45).
    pub ccr24: u32,
    /// The issue cycle of the last macro operation, which a VCALLMS the cycle after waits one cycle behind (§5.4.5).
    pub last_macro: Option<u64>,
    pub micro0: Box<dyn Micro>,
    pub micro1: Box<dyn Micro>,
}

impl Default for Vpu {
    fn default() -> Self {
        Vpu::new()
    }
}

impl Vpu {
    pub fn new() -> Vpu {
        Vpu {
            units: Units { vu0: Vu::new(VU0_MEM), vu1: Vu::new(VU1_MEM), link: Link::default() },
            cmsar0: 0,
            cmsar1: 0,
            ccr24: 0,
            last_macro: None,
            micro0: Box::new(pipe::Pipe::new(0)),
            micro1: Box::new(pipe::Pipe::new(1)),
        }
    }

    /// Lands VU0's quotient for a macro instruction issued at cycle `t`, which reads Q at its T stage a cycle later
    /// (§5.4.1); a transfer reads in its issue cycle, so it passes one cycle less.
    pub fn settle_q(&mut self, t: u64) {
        self.units.vu0.settle(t + 1);
    }

    /// Runs VU0's microprogram to its end; the T stage of its last pair (the E bit's delay slot), none when no
    /// microprogram has run. A macro instruction issues 3 cycles after it, QMFC2.I and CFC2.I 4 (§5.4.5, §5.4.6).
    pub fn finish0(&mut self, t: u64) -> Option<u64> {
        self.micro0.finish(&mut self.units, t)
    }

    /// The cycle QMTC2.I and CTC2.I may issue in: once VU0 runs a pair with M set, or has ended (§5.4.6).
    pub fn release0(&mut self, t: u64) -> u64 {
        self.micro0.release(&mut self.units, t).unwrap_or(t)
    }

    /// Starts unit `unit`'s microprogram at doubleword `addr`, its first pair's T stage at cycle `first`: VCALLMS,
    /// VCALLMSR, CMSAR1 and VIF's MSCAL family (S14) all come here. False when the unit cannot start.
    pub fn start(&mut self, unit: usize, addr: u16, first: u64) -> bool {
        let m = if unit == 0 { &mut self.micro0 } else { &mut self.micro1 };
        m.start(&mut self.units, addr, first)
    }

    /// The interrupts a D or T bit raised since the last call, bit 0 VU0 and bit 1 VU1, for the INTC.
    pub fn take_interrupts(&mut self) -> u32 {
        std::mem::take(&mut self.units.link.irq)
    }

    /// Brings both units' micro mode to EE cycle `t`.
    pub fn run_to(&mut self, t: u64) {
        self.micro0.run_to(&mut self.units, t);
        self.micro1.run_to(&mut self.units, t);
    }

    /// VU1 running a microprogram: the COP2 condition signal BC2F and BC2T test (§5.5.1).
    pub fn vu1_busy(&mut self, t: u64) -> bool {
        self.micro1.run_to(&mut self.units, t);
        self.micro1.busy()
    }

    /// CCR[2,n] as CFC2 reads it at cycle `t` (§5.1.2, §5.1.3; D-45 for the reserved numbers).
    pub fn ccr(&mut self, n: usize, t: u64) -> u32 {
        self.micro0.run_to(&mut self.units, t);
        let vu = &self.units.vu0;
        match n {
            0..16 => vu.vi(n) as u32,
            ccr::STATUS => vu.status,
            ccr::MAC => vu.mac,
            ccr::CLIP => vu.clip,
            19 => CCR19,
            ccr::R => vu.r,
            ccr::I => vu.i,
            ccr::Q => {
                self.settle_q(t.saturating_sub(1));
                self.units.vu0.q
            }
            24 => self.ccr24,
            ccr::TPC => vu.tpc as u32,
            ccr::CMSAR0 => self.cmsar0 as u32,
            ccr::FBRST => self.units.link.fbrst,
            ccr::VPU_STAT => self.vpu_stat(t),
            ccr::CMSAR1 => self.cmsar1 as u32,
            _ => 0,
        }
    }

    /// VPU-STAT read by a transfer at cycle `t`: each unit's micro-mode state, its divider busy, VU1's EFU busy.
    pub fn vpu_stat(&mut self, t: u64) -> u32 {
        self.run_to(t);
        let busy = |free: u64, bit: u32| if free > t + 1 { bit } else { 0 };
        let (v0, v1) = (&self.units.vu0, &self.units.vu1);
        let efu = v1.p_last().map_or(0, |l| busy(l.at, vpu_stat::EFU));
        let unit0 = self.micro0.stat() | busy(v0.div_free, vpu_stat::DIV);
        let unit1 = self.micro1.stat() | busy(v1.div_free, vpu_stat::DIV) | efu;
        unit0 | unit1 << vpu_stat::VU1
    }

    /// CTC2's write of CCR[2,n] at cycle `t`; false where it would start micro mode that is not built (S13). A write of
    /// CMSAR1 while VU1 runs is ignored (§5.1.3).
    pub fn set_ccr(&mut self, n: usize, v: u32, t: u64) -> bool {
        let vu = &mut self.units.vu0;
        match n {
            0..16 => vu.set_vi(n, v as u16),
            ccr::STATUS => vu.status = vu.status & !status::WRITABLE | v & status::WRITABLE,
            ccr::CLIP => vu.clip = v & 0xFF_FFFF,
            ccr::R => vu.r = v & 0x7F_FFFF,
            ccr::I => vu.i = v,
            ccr::Q => {
                self.settle_q(t.saturating_sub(1));
                self.units.vu0.q = v;
            }
            24 => self.ccr24 = v & 0x0C0C,
            ccr::CMSAR0 => self.cmsar0 = v as u16,
            ccr::FBRST => self.write_fbrst(v, t),
            ccr::CMSAR1 if !self.vu1_busy(t) => {
                self.cmsar1 = v as u16;
                return self.micro1.start(&mut self.units, v as u16 >> 3, t + pipe::START);
            }
            _ => {}
        }
        true
    }

    /// FBRST (§5.1.3, §1.3): a reset initialises the unit's control registers and its enables, a force break stops a
    /// running microprogram; the enables are kept.
    fn write_fbrst(&mut self, v: u32, t: u64) {
        self.run_to(t);
        self.units.link.fbrst = v & fbrst::KEPT;
        if v & fbrst::FB0 != 0 {
            self.micro0.force_break(&mut self.units, t);
        }
        if v & fbrst::FB0 << fbrst::VU1 != 0 {
            self.micro1.force_break(&mut self.units, t);
        }
        if v & fbrst::RS0 != 0 {
            self.micro0.reset();
            self.units.vu0.reset_flags();
            self.units.link.fbrst &= !(0xC);
            self.units.vu0.q_next = None;
        }
        if v & fbrst::RS0 << fbrst::VU1 != 0 {
            self.micro1.reset();
            self.units.vu1.reset_flags();
            self.units.link.fbrst &= !(0xC << fbrst::VU1);
            self.units.vu1.q_next = None;
            self.units.vu1.p_next = None;
            self.units.vu1.p_later = None;
        }
    }

    /// The EE's view of the four memories at 11000000h to 1100FFFFh (EE User's Manual §2.1's map): VU0's micro and
    /// data memories each mirrored over 16 KiB, VU1's.
    pub fn memory(&mut self, paddr: u32) -> (&mut [u8], usize) {
        let off = (paddr & 0x3FFF) as usize;
        match paddr >> 14 & 3 {
            0 => (&mut self.units.vu0.micro[..], off % VU0_MEM),
            1 => (&mut self.units.vu0.data[..], off % VU0_MEM),
            2 => (&mut self.units.vu1.micro[..], off),
            _ => (&mut self.units.vu1.data[..], off),
        }
    }

    pub fn read(&mut self, paddr: u32, bytes: usize) -> u128 {
        let (m, at) = self.memory(paddr);
        let mut v = [0u8; 16];
        v[..bytes].copy_from_slice(&m[at..at + bytes]);
        u128::from_le_bytes(v)
    }

    pub fn write(&mut self, paddr: u32, bytes: usize, value: u128) {
        let (m, at) = self.memory(paddr);
        m[at..at + bytes].copy_from_slice(&value.to_le_bytes()[..bytes]);
    }
}
