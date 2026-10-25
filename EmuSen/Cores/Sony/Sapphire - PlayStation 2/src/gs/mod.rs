//! The GS (GS User's Manual; ps2tek "GS"), stage S16: its general registers as the host interface writes them, its
//! privileged registers at 12000000h, CSR and IMR with the SIGNAL, FINISH and LABEL events, the interrupt it gives the
//! INTC, its local memory (`local`, in the page and block layout of every format, `psm`) and the three transfers
//! (`transfer`). The GS acts at once on each write: nothing of its timing is modelled yet. Drawing (S17): the vertex
//! queue and the drawing kick (`draw`), the primitives' coverage and interpolation (`raster`) and the pixel pipeline
//! (`pixel`), into which texturing (S18) and the other pixel operations (S19) slot; the PCRTC (S20) attaches at the
//! privileged registers and `hsync`/`vsync`. Sapphire_Native.md §24, §27.

pub mod draw;
pub mod local;
pub mod pixel;
pub mod psm;
pub mod raster;
pub mod transfer;

use std::collections::VecDeque;

use local::Local;
use transfer::{Direction, Transfer};

/// The general registers by address (§7.3).
pub mod reg {
    pub const PRIM: u8 = 0x00;
    pub const RGBAQ: u8 = 0x01;
    pub const ST: u8 = 0x02;
    pub const UV: u8 = 0x03;
    pub const XYZF2: u8 = 0x04;
    pub const XYZ2: u8 = 0x05;
    pub const TEX0_1: u8 = 0x06;
    pub const TEX0_2: u8 = 0x07;
    pub const CLAMP_1: u8 = 0x08;
    pub const CLAMP_2: u8 = 0x09;
    pub const FOG: u8 = 0x0A;
    pub const XYZF3: u8 = 0x0C;
    pub const XYZ3: u8 = 0x0D;
    pub const TEX1_1: u8 = 0x14;
    pub const TEX1_2: u8 = 0x15;
    pub const TEX2_1: u8 = 0x16;
    pub const TEX2_2: u8 = 0x17;
    pub const XYOFFSET_1: u8 = 0x18;
    pub const XYOFFSET_2: u8 = 0x19;
    pub const PRMODECONT: u8 = 0x1A;
    pub const PRMODE: u8 = 0x1B;
    pub const TEXCLUT: u8 = 0x1C;
    pub const SCANMSK: u8 = 0x22;
    pub const MIPTBP1_1: u8 = 0x34;
    pub const MIPTBP1_2: u8 = 0x35;
    pub const MIPTBP2_1: u8 = 0x36;
    pub const MIPTBP2_2: u8 = 0x37;
    pub const TEXA: u8 = 0x3B;
    pub const FOGCOL: u8 = 0x3D;
    pub const TEXFLUSH: u8 = 0x3F;
    pub const SCISSOR_1: u8 = 0x40;
    pub const SCISSOR_2: u8 = 0x41;
    pub const ALPHA_1: u8 = 0x42;
    pub const ALPHA_2: u8 = 0x43;
    pub const DIMX: u8 = 0x44;
    pub const DTHE: u8 = 0x45;
    pub const COLCLAMP: u8 = 0x46;
    pub const TEST_1: u8 = 0x47;
    pub const TEST_2: u8 = 0x48;
    pub const PABE: u8 = 0x49;
    pub const FBA_1: u8 = 0x4A;
    pub const FBA_2: u8 = 0x4B;
    pub const FRAME_1: u8 = 0x4C;
    pub const FRAME_2: u8 = 0x4D;
    pub const ZBUF_1: u8 = 0x4E;
    pub const ZBUF_2: u8 = 0x4F;
    pub const BITBLTBUF: u8 = 0x50;
    pub const TRXPOS: u8 = 0x51;
    pub const TRXREG: u8 = 0x52;
    pub const TRXDIR: u8 = 0x53;
    pub const HWREG: u8 = 0x54;
    pub const SIGNAL: u8 = 0x60;
    pub const FINISH: u8 = 0x61;
    pub const LABEL: u8 = 0x62;

    /// Whether §7.3 lists a register at `a`.
    pub const fn defined(a: u8) -> bool {
        matches!(a, 0x00..=0x0A | 0x0C | 0x0D | 0x14..=0x1C | 0x22 | 0x34..=0x37 | 0x3B | 0x3D | 0x3F | 0x40..=0x54 | 0x60..=0x62)
    }
}

/// The privileged registers by their GS address GA[6:0] (EE User's Manual §2.2.3: the EE's address bit 12 and bits
/// 9..4), and their EE addresses.
pub mod privileged {
    pub const PMODE: u32 = 0x00;
    pub const SMODE1: u32 = 0x01;
    pub const SMODE2: u32 = 0x02;
    pub const SRFSH: u32 = 0x03;
    pub const SYNCH1: u32 = 0x04;
    pub const SYNCH2: u32 = 0x05;
    pub const SYNCV: u32 = 0x06;
    pub const DISPFB1: u32 = 0x07;
    pub const DISPLAY1: u32 = 0x08;
    pub const DISPFB2: u32 = 0x09;
    pub const DISPLAY2: u32 = 0x0A;
    pub const EXTBUF: u32 = 0x0B;
    pub const EXTDATA: u32 = 0x0C;
    pub const EXTWRITE: u32 = 0x0D;
    pub const BGCOLOR: u32 = 0x0E;
    pub const CSR: u32 = 0x40;
    pub const IMR: u32 = 0x41;
    pub const BUSDIR: u32 = 0x44;
    pub const SIGLBLID: u32 = 0x48;

    /// The EE's physical addresses the GS answers (§2.2.3): 64 KiB from 12000000h; above it to 13FFFFFFh, a bus error.
    pub const BASE: u32 = 0x1200_0000;
    pub const END: u32 = 0x1201_0000;
    pub const BUS_ERROR_END: u32 = 0x1400_0000;

    /// The GS address an EE address reaches; bits 10, 11 and 13 to 15 are not decoded, so their ranges are images.
    pub const fn ga(paddr: u32) -> u32 {
        (paddr >> 12 & 1) << 6 | (paddr >> 4 & 0x3F)
    }

    /// The EE address of a GS address.
    pub const fn address(ga: u32) -> u32 {
        BASE | (ga >> 6) << 12 | (ga & 0x3F) << 4
    }
}

/// CSR's fields (§7.2).
pub mod csr {
    pub const SIGNAL: u32 = 1;
    pub const FINISH: u32 = 1 << 1;
    pub const HSINT: u32 = 1 << 2;
    pub const VSINT: u32 = 1 << 3;
    pub const EDWINT: u32 = 1 << 4;
    /// The five events, each cleared by writing 1 and masked by IMR's bit eight places up.
    pub const EVENTS: u32 = 0x1F;
    pub const FLUSH: u32 = 1 << 8;
    pub const RESET: u32 = 1 << 9;
    /// FIFO's field: 01 empty, 00 neither empty nor almost full (§7.2).
    pub const FIFO_EMPTY: u32 = 1 << 14;
    /// REV and ID, which no document gives (Sapphire_Disputes.md D-101).
    pub const REV: u32 = 0x1B;
    pub const ID: u32 = 0x55;
}

/// IMR after a reset: "All initial values after reset are 1 (masked)", bits 13 and 14 as §7.2 asks them written.
pub const IMR_RESET: u64 = 0x7F00;

/// What the GIF (S15) drives: the general registers' writes from PACKED and REGLIST data and A+D, HWREG's from IMAGE
/// data, and the reads of a local-to-host transfer once the bus is reversed; it holds while the GS is stalled.
pub trait HostInterface {
    fn write(&mut self, reg: u8, data: u64);
    fn image(&mut self, data: u64) {
        self.write(reg::HWREG, data);
    }
    fn read(&mut self) -> Option<u64>;
    /// The GS takes no more data: a second SIGNAL waits on the first's flag (§6.2.1).
    fn stalled(&self) -> bool;
    /// BUSDIR's direction: the host interface carries local-to-host data (§4.2.2).
    fn reversed(&self) -> bool;
}

pub struct Gs {
    pub local: Local,
    regs: [u64; 0x63],
    /// The privileged registers as written, by GS address.
    privileged: [u64; 128],
    /// CSR's five event flags.
    events: u32,
    imr: u64,
    siglblid: u64,
    pub transfer: Transfer,
    /// The vertex queue (§3.2.3).
    pub queue: draw::Queue,
    /// A SIGNAL written while the flag of the one before was set, waiting with the writes after it.
    pending: Option<u64>,
    held: VecDeque<(u8, u64)>,
    line: bool,
    rose: bool,
}

impl Default for Gs {
    fn default() -> Gs {
        Gs::new()
    }
}

/// The byte lanes an access of `bytes` at `paddr` covers in a 64-bit register, as a shift and a mask [A].
fn lanes(paddr: u32, bytes: usize) -> (u32, u64) {
    let shift = 8 * (paddr & 7);
    let mask = if bytes >= 8 { u64::MAX } else { ((1u64 << (8 * bytes)) - 1) << shift };
    (if bytes >= 8 { 0 } else { shift }, mask)
}

impl Gs {
    pub fn new() -> Gs {
        let mut privileged = [0; 128];
        privileged[privileged::IMR as usize] = IMR_RESET;
        Gs {
            local: Local::default(),
            regs: [0; 0x63],
            privileged,
            events: 0,
            imr: IMR_RESET,
            siglblid: 0,
            transfer: Transfer::default(),
            queue: draw::Queue::default(),
            pending: None,
            held: VecDeque::new(),
            line: false,
            rose: false,
        }
    }

    /// A general register as last written.
    pub fn register(&self, reg: u8) -> u64 {
        self.regs.get(reg as usize).copied().unwrap_or(0)
    }

    pub fn siglblid(&self) -> u64 {
        self.siglblid
    }

    /// A privileged register as last written, by GS address: the PCRTC's settings (S20) are read here.
    pub fn privileged_register(&self, ga: u32) -> u64 {
        self.privileged[(ga & 0x7F) as usize]
    }

    /// CSR as the EE reads it (§7.2): the events, FIFO's status, REV and ID; FLUSH, RESET, NFIELD and FIELD read 0
    /// until the PCRTC (S20) drives the fields.
    pub fn csr(&self) -> u64 {
        let fifo = if self.held.is_empty() && self.pending.is_none() { csr::FIFO_EMPTY } else { 0 };
        u64::from(self.events | fifo | csr::REV << 16 | csr::ID << 24)
    }

    /// The interrupt to the INTC: a level, any event whose IMR bit is clear (§6.1; the Restriction's "GS Interrupt
    /// Handler" note: a second event while the first is unhandled gives no new edge).
    pub fn line(&self) -> bool {
        self.events & !(self.imr >> 8) as u32 & csr::EVENTS != 0
    }

    /// Whether the line has risen since the last call; the INTC latches its source at that edge (EEUM §3.1).
    pub fn take_rise(&mut self) -> bool {
        std::mem::take(&mut self.rose)
    }

    fn update_line(&mut self) {
        let l = self.line();
        if l && !self.line {
            self.rose = true;
        }
        self.line = l;
    }

    fn signal(&mut self, data: u64) {
        let (id, mask) = (data as u32, (data >> 32) as u32);
        let sig = self.siglblid as u32;
        self.siglblid = (self.siglblid & !0xFFFF_FFFF) | u64::from((sig & !mask) | (id & mask));
        self.events |= csr::SIGNAL;
    }

    fn apply(&mut self, r: u8, data: u64) {
        match r {
            reg::SIGNAL if self.events & csr::SIGNAL != 0 => self.pending = Some(data),
            reg::SIGNAL => self.signal(data),
            reg::FINISH => self.events |= csr::FINISH,
            reg::LABEL => {
                let (id, mask) = (u64::from(data as u32) << 32, data & 0xFFFF_FFFF_0000_0000);
                self.siglblid = (self.siglblid & !mask) | (id & mask);
            }
            reg::TRXDIR => {
                self.regs[r as usize] = data;
                let (b, p, s) = (self.regs[reg::BITBLTBUF as usize], self.regs[reg::TRXPOS as usize], self.regs[reg::TRXREG as usize]);
                self.transfer.start(data as u32 & 3, b, p, s, &mut self.local);
            }
            reg::HWREG => self.transfer.hwreg(data, &mut self.local),
            reg::PRIM => {
                self.regs[r as usize] = data;
                self.queue.reset();
            }
            reg::XYZ2 | reg::XYZF2 | reg::XYZ3 | reg::XYZF3 => {
                self.regs[r as usize] = data;
                self.vertex_kick(r, data);
            }
            _ if reg::defined(r) => self.regs[r as usize] = data,
            _ => {}
        }
    }

    /// The writes held behind a stalled SIGNAL, taken in order until another stalls.
    fn drain(&mut self) {
        while self.pending.is_none() {
            let Some((r, d)) = self.held.pop_front() else { break };
            self.apply(r, d);
        }
    }

    /// CSR's reset (§7.2, "GS System Reset"): the events, the stalled SIGNAL and the held writes go, a transfer ends
    /// and IMR masks everything (§7.2's IMR); the registers and the memory stay [A].
    fn reset(&mut self) {
        self.events = 0;
        self.pending = None;
        self.held.clear();
        self.transfer.end();
        self.imr = IMR_RESET;
        self.privileged[privileged::IMR as usize] = IMR_RESET;
    }

    fn write_csr(&mut self, action: u32) {
        if action & csr::RESET != 0 {
            self.reset();
        }
        if action & csr::FLUSH != 0 {
            self.held.clear();
        }
        self.events &= !(action & csr::EVENTS);
        if action & csr::SIGNAL != 0
            && let Some(d) = self.pending.take()
        {
            self.signal(d);
            self.drain();
        }
    }

    /// An EE read at `paddr` of `bytes` (EE User's Manual §2.2.3, "must be accessed using LD/SD"); a narrower access
    /// reads its lanes [A]. SIGLBLID is readable (§7.2); every other address, the write-only registers and the
    /// reserved ones, reads CSR (Sapphire_Disputes.md D-100).
    pub fn read_privileged(&mut self, paddr: u32, bytes: usize) -> u64 {
        let ga = privileged::ga(paddr);
        let v = if ga == privileged::SIGLBLID { self.siglblid } else { self.csr() };
        let (shift, mask) = lanes(paddr, bytes);
        (v & mask) >> shift
    }

    /// An EE write; a narrower one writes its lanes alone [A], so CSR acts only on the bits it carries.
    pub fn write_privileged(&mut self, paddr: u32, bytes: usize, value: u64) {
        let ga = privileged::ga(paddr);
        let (shift, mask) = lanes(paddr, bytes);
        let v = (value << shift) & mask;
        let slot = &mut self.privileged[ga as usize];
        *slot = (*slot & !mask) | v;
        match ga {
            privileged::CSR => self.write_csr(v as u32),
            privileged::IMR => self.imr = self.privileged[ga as usize],
            privileged::SIGLBLID => self.siglblid = (self.siglblid & !mask) | v,
            _ => {}
        }
        self.update_line();
    }

    /// H-sync, from the blanking stand-in until the PCRTC (S20): HSINT.
    pub fn hsync(&mut self) {
        self.events |= csr::HSINT;
        self.update_line();
    }

    /// V-sync, likewise: VSINT.
    pub fn vsync(&mut self) {
        self.events |= csr::VSINT;
        self.update_line();
    }

    /// Whether an H-sync would change anything: only while HSINT is clear.
    pub fn wants_hsync(&self) -> bool {
        self.events & csr::HSINT == 0
    }
}

impl HostInterface for Gs {
    /// A general register written (§7.1): while the bus is reversed the general port cannot be reached and the write
    /// is lost [A]; while a SIGNAL is stalled it is held behind it.
    fn write(&mut self, reg: u8, data: u64) {
        if self.reversed() {
            return;
        }
        if self.pending.is_some() {
            self.held.push_back((reg, data));
        } else {
            self.apply(reg, data);
        }
        self.update_line();
    }

    /// The next 64 bits of a local-to-host transfer, only while BUSDIR reverses the bus.
    fn read(&mut self) -> Option<u64> {
        if !self.reversed() {
            return None;
        }
        self.transfer.read(&self.local)
    }

    fn stalled(&self) -> bool {
        self.pending.is_some()
    }

    fn reversed(&self) -> bool {
        self.privileged[privileged::BUSDIR as usize] & 1 != 0
    }
}

impl Gs {
    /// Whether a local-to-host transfer has data left for the host.
    pub fn reading(&self) -> bool {
        self.transfer.active == Some(Direction::LocalToHost)
    }
}
