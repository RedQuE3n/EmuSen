//! The GS (GS User's Manual; ps2tek "GS"), stage S16: its general registers as the host interface writes them, its
//! privileged registers at 12000000h, CSR and IMR with the SIGNAL, FINISH and LABEL events, the interrupt it gives the
//! INTC, its local memory (`local`, in the page and block layout of every format, `psm`) and the three transfers
//! (`transfer`). The GS acts at once on each write: nothing of its timing is modelled yet. Drawing (S17): the vertex
//! queue and the drawing kick (`draw`), the primitives' coverage and interpolation (`raster`) and the pixel pipeline
//! (`pixel`), into which texturing and fog (S18: `texture`, the CLUT's buffer, `exact`) and the other pixel operations
//! (S19) slot; the PCRTC (S20) attaches at the privileged registers and `hsync`/`vsync`. The drawing may run on worker
//! threads (`workers`, S43), exact at every count. Sapphire_Native.md §24, §27, §31, §50.

pub mod draw;
pub mod exact;
pub mod local;
pub mod pixel;
pub mod psm;
pub mod raster;
pub mod texture;
pub mod transfer;
pub mod workers;

use std::collections::VecDeque;

use local::Local;
use psm::Psm;
use transfer::{Direction, Transfer};
use workers::{Cause, Mem, Workers};
use transfer::Dry;

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
    /// The field shown: 0 even, 1 odd (§7.2).
    pub const FIELD: u32 = 1 << 13;
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
    /// The drawing's workers, when the GS runs any (S43); dropped before the memory they draw into.
    workers: Option<Box<Workers>>,
    local: Local,
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
    /// The CLUT's temporary buffer and CBP0, CBP1 (§3.4.7).
    pub clut: texture::Clut,
    /// A SIGNAL written while the flag of the one before was set, waiting with the writes after it.
    pending: Option<u64>,
    held: VecDeque<(u8, u64)>,
    line: bool,
    rose: bool,
    /// CSR's FIELD (bit 13), set by the PCRTC at each V-sync (Sapphire_Native.md §29.3).
    field: u32,
    /// When set, every general register written, in order, with HWREG's writes counted: the GS's own report of what it
    /// was given (Sapphire_Native.md §39).
    pub trace: Option<(Vec<(u8, u64)>, u64)>,
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
        let mut gs = Gs {
            workers: None,
            local: Local::default(),
            regs: [0; 0x63],
            privileged,
            events: 0,
            imr: IMR_RESET,
            siglblid: 0,
            transfer: Transfer::default(),
            queue: draw::Queue::default(),
            clut: texture::Clut::default(),
            pending: None,
            held: VecDeque::new(),
            line: false,
            rose: false,
            field: 0,
            trace: None,
        };
        if let Some(n) = std::env::var("SAPPHIRE_GS_WORKERS").ok().as_deref().and_then(workers::parse) {
            gs.set_workers(n);
        }
        gs
    }

    /// The drawing's workers: 1 draws on the GS's own thread, the reference; 2 to `workers::MAX` on that many threads.
    /// The memory is the same either way at every point the machine can observe (Sapphire_Native.md §50).
    pub fn set_workers(&mut self, n: usize) {
        if let Some(w) = self.workers.take() {
            w.join();
        }
        if n >= 2 {
            self.workers = Some(Box::new(Workers::new(n.min(workers::MAX), &self.local, &self.clut)));
        }
    }

    /// The workers running: 1 when the GS draws on its own thread.
    pub fn worker_count(&self) -> usize {
        self.workers.as_ref().map_or(1, |w| w.count())
    }

    /// The workers' counts, when there are workers.
    pub fn worker_counts(&self) -> Option<workers::Counts> {
        self.workers.as_ref().map(|w| w.counts())
    }

    /// A handle holding the workers before their next primitive, for the tests; none on one thread.
    pub fn hold(&self) -> Option<workers::Hold> {
        self.workers.as_ref().map(|w| w.hold())
    }

    /// Primitives published that some worker has not yet drawn; 0 on one thread.
    pub fn pending(&self) -> u64 {
        self.workers.as_ref().map_or(0, |w| w.pending())
    }

    /// Every primitive drawn: the memory is then the GS's thread's until the next drawing kick.
    pub fn join(&self) {
        if let Some(w) = &self.workers {
            w.join();
        }
    }

    /// The local memory, every primitive drawn first.
    pub fn local(&self) -> &Local {
        self.join();
        &self.local
    }

    /// Word `i` of the local memory written, every primitive drawn first; false past its end.
    pub fn set_word(&mut self, i: usize, v: u32) -> bool {
        self.join();
        if i >= local::WORDS {
            return false;
        }
        self.local.set_word(i, v);
        true
    }

    /// A pixel written into local memory from outside the GS (a test or a tool), every primitive drawn first.
    pub fn write_pixel(&mut self, psm: Psm, bp: u32, bw: u32, x: u32, y: u32, v: u32) {
        self.join();
        self.local.write(psm, bp, bw, x, y, v);
    }

    /// Before the GS's thread reads the pixels from (`x0`, `y0`) to (`x1`, `y1`) of a buffer: the workers that write
    /// their pages are waited for.
    pub fn settle(&self, psm: Psm, bp: u32, bw: u32, xs: (u32, u32), ys: (u32, u32)) {
        if let Some(w) = &self.workers {
            w.settle_reads(&local::units(psm, bp, bw, xs, ys));
        }
    }

    /// A pixel read by the GS's thread from a page `settle` has waited for; checked in debug builds.
    #[inline]
    pub fn peek(&self, psm: Psm, bp: u32, bw: u32, x: u32, y: u32) -> u32 {
        #[cfg(debug_assertions)]
        if let Some(w) = &self.workers {
            assert!(w.settled(local::address(psm, bp, bw, x, y).word, local::lanes(psm)), "a read of a page a worker may still write");
        }
        self.local.read(psm, bp, bw, x, y)
    }

    /// The memory as the GS's own thread reaches it, each access waiting for the workers touching its page.
    fn mem(&mut self, cause: Cause) -> (Mem<'_>, &mut Transfer, &mut texture::Clut) {
        (Mem { local: &mut self.local, workers: self.workers.as_deref(), cause }, &mut self.transfer, &mut self.clut)
    }

    fn load_clut(&mut self, tex0: u64) {
        let texclut = self.regs[reg::TEXCLUT as usize];
        let (mut mem, _, clut) = self.mem(Cause::ClutLoad);
        if clut.load(tex0, texclut, &mut mem)
            && let Some(w) = self.workers.as_deref()
        {
            w.clut_changed();
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

    /// CSR as the EE reads it (§7.2): the events, FIELD, FIFO's status, REV and ID; FLUSH, RESET and NFIELD read 0
    /// (Sapphire_Disputes.md D-204).
    pub fn csr(&self) -> u64 {
        let fifo = if self.held.is_empty() && self.pending.is_none() { csr::FIFO_EMPTY } else { 0 };
        u64::from(self.events | self.field | fifo | csr::REV << 16 | csr::ID << 24)
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
                if let Some(w) = &self.workers {
                    w.flush();
                }
                let (b, p, s) = (self.regs[reg::BITBLTBUF as usize], self.regs[reg::TRXPOS as usize], self.regs[reg::TRXREG as usize]);
                let (mut mem, transfer, _) = self.mem(Cause::Copy);
                transfer.start(data as u32 & 3, b, p, s, &mut mem);
            }
            reg::HWREG => match self.workers.as_deref() {
                // A worker writes the pixels, in the ring's order; the GS's thread follows the transfer's place.
                Some(w) if self.transfer.active == Some(Direction::HostToLocal) => {
                    w.upload(&self.transfer, data);
                    self.transfer.hwreg(data, &mut Dry);
                }
                Some(_) => {}
                None => self.transfer.hwreg(data, &mut self.local),
            },
            reg::PRIM => {
                self.regs[r as usize] = data;
                self.queue.reset();
            }
            reg::TEX0_1 | reg::TEX0_2 => {
                self.regs[r as usize] = data;
                let c = r - reg::TEX0_1;
                if self.regs[(reg::TEX1_1 + c) as usize] >> 9 & 1 != 0 {
                    self.regs[(reg::MIPTBP1_1 + c) as usize] = texture::auto_miptbp1(data);
                }
                self.load_clut(data);
            }
            reg::TEX2_1 | reg::TEX2_2 => {
                self.regs[r as usize] = data;
                let t = (reg::TEX0_1 + r - reg::TEX2_1) as usize;
                self.regs[t] = (self.regs[t] & !texture::TEX2_BITS) | (data & texture::TEX2_BITS);
                self.load_clut(self.regs[t]);
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
        if let Some(w) = &self.workers {
            w.flush();
        }
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

    /// H-sync, from the PCRTC: HSINT.
    pub fn hsync(&mut self) {
        self.events |= csr::HSINT;
        self.update_line();
    }

    /// V-sync, from the PCRTC: VSINT, and FIELD for the field now shown.
    pub fn vsync(&mut self, odd: bool) {
        self.field = if odd { csr::FIELD } else { 0 };
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
        if let Some((t, images)) = self.trace.as_mut() {
            if reg == reg::HWREG {
                *images += 1;
            }
            if t.len() < 1 << 22 {
                t.push((reg, data));
            }
        }
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
        let (mut mem, transfer, _) = self.mem(Cause::Download);
        transfer.read(&mut mem)
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

/// General register writes held behind a stalled SIGNAL in the state.
const HELD_SLOTS: usize = 64;

/// The GS's block of `SPH2` (Sapphire_Native.md §38.2): the general and privileged registers, CSR's events and FIELD,
/// IMR, SIGLBLID, the transfer, the vertex queue, the CLUT buffer, a SIGNAL waiting with the writes behind it, the
/// interrupt line, and the 4 MiB.
impl emusen_native::State for Gs {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        self.join();
        w.u64s("Registers", &self.regs);
        w.u64s("Privileged", &self.privileged);
        w.u32s("Events", &[self.events, self.field]);
        w.u64s("ImrSiglblid", &[self.imr, self.siglblid]);
        w.structure("Transfer", &self.transfer);
        let q = &self.queue;
        w.u32("QueueLength", q.n as u32);
        w.i32s("QueueXy", q.v.map(|v| [v.x, v.y]).as_flattened());
        w.u32s("QueueZ", &q.v.map(|v| v.z));
        w.u64s("QueueRgbaqStUv", q.v.map(|v| [v.rgbaq, v.st, v.uv]).as_flattened());
        w.bytes("QueueF", &q.v.map(|v| v.f));
        w.u16s("ClutEntries", &self.clut.entries);
        w.u32s("ClutCbp", &self.clut.cbp);
        crate::state::opt_u64(w, "Pending", self.pending);
        let h = crate::state::slots::<_, HELD_SLOTS>(&self.held);
        w.u32("HeldLength", self.held.len() as u32);
        w.bytes("HeldRegisters", &h.map(|e| e.0));
        w.u64s("HeldData", &h.map(|e| e.1));
        w.bools("Line", &[self.line, self.rose]);
        w.u32s("Local", self.local.words());
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        self.join();
        if let Some(w) = self.workers.as_deref() {
            w.clut_changed();
        }
        r.u64s(&mut self.regs)?;
        r.u64s(&mut self.privileged)?;
        let mut e = [0u32; 2];
        r.u32s(&mut e)?;
        [self.events, self.field] = e;
        let mut i = [0u64; 2];
        r.u64s(&mut i)?;
        [self.imr, self.siglblid] = i;
        self.transfer.read_state(r)?;
        let n = r.u32()?;
        let mut xy = [0i32; 6];
        let mut z = [0u32; 3];
        let mut c = [0u64; 9];
        let mut f = [0u8; 3];
        r.i32s(&mut xy)?;
        r.u32s(&mut z)?;
        r.u64s(&mut c)?;
        r.bytes(&mut f)?;
        self.queue.n = (n as usize).min(3);
        self.queue.v = std::array::from_fn(|k| draw::Vertex { x: xy[2 * k], y: xy[2 * k + 1], z: z[k], rgbaq: c[3 * k], st: c[3 * k + 1], uv: c[3 * k + 2], f: f[k] });
        r.u16s(&mut self.clut.entries)?;
        r.u32s(&mut self.clut.cbp)?;
        self.pending = crate::state::read_opt_u64(r)?;
        let len = r.u32()?;
        let mut regs = [0u8; HELD_SLOTS];
        let mut data = [0u64; HELD_SLOTS];
        r.bytes(&mut regs)?;
        r.u64s(&mut data)?;
        crate::state::refill::<_, HELD_SLOTS>(&mut self.held, len, &std::array::from_fn(|k| (regs[k], data[k])));
        let mut l = [false; 2];
        r.bools(&mut l)?;
        [self.line, self.rose] = l;
        r.u32s(self.local.words_mut())
    }
}

impl Gs {
    /// Whether the writes held behind a SIGNAL are within their slots in the state.
    pub fn fits(&self) -> bool {
        self.held.len() <= HELD_SLOTS
    }
}
