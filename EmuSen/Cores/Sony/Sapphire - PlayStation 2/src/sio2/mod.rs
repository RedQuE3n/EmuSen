//! SIO2 (stage S25, Sapphire_Native.md §30): the IOP's serial port to the controllers, the memory cards and the
//! multitap, at its registers 1F808200h-1F808280h (ps2tek "SIO2 Registers"): SEND3's queue of up to sixteen commands,
//! each a port and a length, SEND1 and SEND2 for each port, FIFOIN and FIFOOUT, CTRL, the three RECV words and the
//! interrupt flags; the IOP DMAC's channels 11 and 12 fill FIFOIN and empty FIFOOUT (`Ports`). A queue started by CTRL's
//! bit 0 sends each command to its port's device as the documents give it (`pad`, `card`, `tap`), byte for byte at the
//! port's rate, a busy card holding its command until it is ready, and raises the IOP's interrupt 17 when the last ends.
//!
//! The ports [A, Sapphire_Disputes.md D-220]: 0 and 1 the two controller connectors, 2 and 3 the two card slots, which
//! on the PlayStation 2 are not wired together (psx-spx "PS2 multitap"); a peripheral byte that no device on its port
//! answers reads HiZ.

pub mod card;
pub mod pad;
pub mod tap;

use std::collections::VecDeque;

use crate::clock::IOP;
use crate::iop::dma as iop_dma;
use card::MemoryCard;
use pad::DualShock2;

pub const BASE: u32 = 0x1F80_8200;
pub const SEND3: u32 = 0x1F80_8200;
pub const SEND12: u32 = 0x1F80_8240;
pub const FIFOIN: u32 = 0x1F80_8260;
pub const FIFOOUT: u32 = 0x1F80_8264;
pub const CTRL: u32 = 0x1F80_8268;
pub const RECV1: u32 = 0x1F80_826C;
pub const RECV2: u32 = 0x1F80_8270;
pub const RECV3: u32 = 0x1F80_8274;
pub const ISTAT: u32 = 0x1F80_8280;

/// The IOP's interrupt source (ps2tek "IOP Interrupts": IRQ17).
pub const SOURCE: u32 = 17;
/// CTRL's actions: bit 0 starts the queue, bits 2 and 3 reset SIO2 (ps2tek); neither is kept [C].
pub const CTRL_START: u32 = 1;
pub const CTRL_RESET: u32 = 0xC;
/// RECV1 after a queue whose peripherals all answered, and after one that met no device: for a controller's command,
/// with bit 13 set, and for any other, with bit 12 (ps2tek "SIO2_RECV1"; Sapphire_Disputes.md D-221).
pub const CONNECTED: u32 = 0x1100;
pub const NO_PAD: u32 = 0x2D100;
pub const NO_DEVICE: u32 = 0x1D100;
/// RECV2's one known value (ps2tek: "Always equal to 0xF?").
pub const RECV2_VALUE: u32 = 0xF;
/// The time one byte takes on a port, in IOP cycles: 250,000 bytes a second [C: no document gives SIO2's rates, nor
/// what SEND1 and SEND2 hold; Sapphire_Native.md §30.3].
pub const BYTE: u64 = 147;

/// The longest command a SEND3 entry carries, one past its length field: bits 18 to 26 [C: ps2tek gives 18-24, seven
/// bits, which cannot carry its own 132-byte card reads; Sapphire_Disputes.md D-230].
pub const LENGTH_MAX: usize = 512;

/// The peripheral bytes (ps2tek's list).
pub const PAD: u8 = 0x01;
pub const CARD: u8 = 0x81;

/// One slot of the console, a controller connector and a card slot, with a multitap in it or not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub tap: bool,
    /// The pads and cards: index 0 alone without a tap, the tap's four with one.
    pub pads: [Option<DualShock2>; tap::SLOTS],
    pub cards: [Option<MemoryCard>; tap::SLOTS],
    /// The tap's selections.
    pad_slot: Option<usize>,
    card_slot: Option<usize>,
}

impl Default for Slot {
    /// Nothing plugged in; a tap put in selects its slot A for both until told otherwise [C].
    fn default() -> Slot {
        Slot { tap: false, pads: Default::default(), cards: Default::default(), pad_slot: Some(0), card_slot: Some(0) }
    }
}

impl Slot {
    fn pad(&mut self) -> Option<&mut DualShock2> {
        let i = if self.tap { self.pad_slot? } else { 0 };
        self.pads[i].as_mut()
    }

    fn card(&mut self) -> Option<&mut MemoryCard> {
        let i = if self.tap { self.card_slot? } else { 0 };
        self.cards[i].as_mut()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sio2 {
    pub send3: [u32; 16],
    pub send12: [u32; 8],
    pub ctrl: u32,
    pub recv1: u32,
    pub istat: u32,
    fifo_in: VecDeque<u8>,
    fifo_out: VecDeque<u8>,
    /// A queue's replies until it ends, and the EE cycle it ends; RECV1 then.
    pending: VecDeque<u8>,
    pending_recv1: u32,
    done_at: Option<u64>,
    pub slots: [Slot; 2],
    /// A queue has ended since the IOP's DMAC last looked: it is woken for SIO2out.
    wake: bool,
}

impl Default for Sio2 {
    fn default() -> Sio2 {
        let mut s = Sio2 {
            send3: [0; 16],
            send12: [0; 8],
            ctrl: 0,
            recv1: 0,
            istat: 0,
            fifo_in: VecDeque::new(),
            fifo_out: VecDeque::new(),
            pending: VecDeque::new(),
            pending_recv1: 0,
            done_at: None,
            slots: Default::default(),
            wake: false,
        };
        s.slots[0].pads[0] = Some(DualShock2::default());
        s
    }
}

/// Whether `paddr` is one of SIO2's registers.
pub fn is_register(paddr: u32) -> bool {
    (BASE..=ISTAT).contains(&paddr)
}

impl Sio2 {
    /// The EE cycle at which the running queue ends.
    pub fn next_event(&self) -> u64 {
        self.done_at.unwrap_or(u64::MAX)
    }

    pub fn take_wake(&mut self) -> bool {
        std::mem::take(&mut self.wake)
    }

    pub fn read(&mut self, paddr: u32) -> u32 {
        match paddr {
            SEND3..SEND12 => self.send3[((paddr - SEND3) / 4) as usize],
            SEND12..FIFOIN => self.send12[((paddr - SEND12) / 4) as usize],
            FIFOOUT => u32::from(self.fifo_out.pop_front().unwrap_or(card::HIZ)),
            CTRL => self.ctrl,
            RECV1 => self.recv1,
            RECV2 => RECV2_VALUE,
            ISTAT => self.istat,
            _ => 0,
        }
    }

    /// A register's write at EE cycle `t`.
    pub fn write(&mut self, paddr: u32, v: u32, t: u64) {
        match paddr {
            SEND3..SEND12 => self.send3[((paddr - SEND3) / 4) as usize] = v,
            SEND12..FIFOIN => self.send12[((paddr - SEND12) / 4) as usize] = v,
            FIFOIN => self.fifo_in.push_back(v as u8),
            CTRL => {
                if v & CTRL_RESET != 0 {
                    self.fifo_in.clear();
                    self.fifo_out.clear();
                }
                self.ctrl = v & !(CTRL_START | CTRL_RESET);
                if v & CTRL_START != 0 && self.done_at.is_none() {
                    self.start(t);
                }
            }
            ISTAT => self.istat &= !v,
            _ => {}
        }
    }

    /// Runs the queue at EE cycle `t`: each entry until a zero, its bytes taken from FIFOIN (00h for any missing [C]),
    /// answered by its port's device; the replies are held until the queue's end.
    fn start(&mut self, t: u64) {
        let mut at = t;
        let mut recv1 = CONNECTED;
        let (mut cmd, mut reply) = ([0u8; LENGTH_MAX], [0u8; LENGTH_MAX]);
        for &entry in &self.send3 {
            if entry == 0 {
                break;
            }
            let port = (entry & 3) as usize;
            let len = (entry >> 18) as usize & (LENGTH_MAX - 1);
            for b in &mut cmd[..len] {
                *b = self.fifo_in.pop_front().unwrap_or(0);
            }
            let (cmd, reply) = (&cmd[..len], &mut reply[..len]);
            reply.fill(card::HIZ);
            let slot = &mut self.slots[port & 1];
            let peripheral = cmd.first().copied().unwrap_or(0);
            if port >= 2
                && peripheral == CARD
                && let Some(c) = slot.card()
            {
                at = at.max(c.busy_until);
            }
            at += len as u64 * BYTE * IOP;
            let answered = match (port >= 2, peripheral) {
                (_, tap::PERIPHERAL) if slot.tap => {
                    let (ok, select) = tap::exchange(cmd, reply);
                    match select {
                        tap::Select::Pads(s) => slot.pad_slot = s,
                        tap::Select::Cards(s) => slot.card_slot = s,
                        tap::Select::None => {}
                    }
                    ok
                }
                (false, PAD) => slot.pad().map(|p| p.exchange(cmd, reply)).is_some(),
                (true, CARD) => slot.card().is_some_and(|c| c.exchange(cmd, reply, at)),
                _ => false,
            };
            if !answered {
                reply.fill(card::HIZ);
                recv1 = if peripheral == PAD { NO_PAD } else { NO_DEVICE };
            }
            self.pending.extend(reply.iter().copied());
        }
        self.pending_recv1 = recv1;
        self.done_at = Some(at.max(t + IOP));
    }

    /// Ends the running queue if it is due by EE cycle `t`: its replies in FIFOOUT, RECV1 set, the flag; whether the
    /// IOP's interrupt is raised.
    pub fn complete(&mut self, t: u64) -> bool {
        match self.done_at {
            Some(d) if d <= t => {
                self.done_at = None;
                self.fifo_out.extend(self.pending.drain(..));
                self.recv1 = self.pending_recv1;
                self.istat |= 1;
                self.wake = true;
                true
            }
            _ => false,
        }
    }

    /// The pad in controller port `port` (0 or 1), or slot `slot` of a tap there.
    pub fn pad_mut(&mut self, port: usize, slot: usize) -> Option<&mut DualShock2> {
        self.slots.get_mut(port)?.pads.get_mut(slot)?.as_mut()
    }

    /// The card in card slot `port` (0 or 1), or slot `slot` of a tap there.
    pub fn card_mut(&mut self, port: usize, slot: usize) -> Option<&mut MemoryCard> {
        self.slots.get_mut(port)?.cards.get_mut(slot)?.as_mut()
    }
}

/// The IOP DMAC's peripherals with SIO2's two channels answered from its FIFOs: channel 11 takes words into FIFOIN, a
/// word's bytes from its lowest, and channel 12 gives FIFOOUT's bytes four to a word, the others going to `others`.
pub struct Ports<'a, P: iop_dma::Peripherals> {
    pub others: &'a mut P,
    pub sio2: &'a mut Sio2,
}

impl<P: iop_dma::Peripherals> iop_dma::Peripherals for Ports<'_, P> {
    fn request(&mut self, ch: usize, want: u32, to_memory: bool) -> u32 {
        match ch {
            iop_dma::SIO2_IN => want,
            iop_dma::SIO2_OUT => want.min((self.sio2.fifo_out.len() / 4) as u32),
            _ => self.others.request(ch, want, to_memory),
        }
    }

    fn put(&mut self, ch: usize, w: u32) {
        if ch == iop_dma::SIO2_IN {
            self.sio2.fifo_in.extend(w.to_le_bytes());
        } else {
            self.others.put(ch, w);
        }
    }

    fn get(&mut self, ch: usize) -> u32 {
        if ch == iop_dma::SIO2_OUT {
            let f = &mut self.sio2.fifo_out;
            u32::from_le_bytes([0; 4].map(|_| f.pop_front().unwrap_or(card::HIZ)))
        } else {
            self.others.get(ch)
        }
    }
}

/// Bytes each SIO2 FIFO holds in the state: a whole SEND3 queue of replies, sixteen transfers of 128 bytes, with room.
const FIFO_SLOTS: usize = 2048;

fn write_bytes(w: &mut emusen_native::StateWriter, name: &str, q: &VecDeque<u8>) {
    let (a, b) = q.as_slices();
    let mut slots = [0u8; FIFO_SLOTS];
    let n = a.len().min(FIFO_SLOTS);
    slots[..n].copy_from_slice(&a[..n]);
    let m = b.len().min(FIFO_SLOTS - n);
    slots[n..n + m].copy_from_slice(&b[..m]);
    w.group(name, |w| {
        w.u32("Length", q.len() as u32);
        w.bytes("Bytes", &slots);
    });
}

fn read_bytes(r: &mut emusen_native::StateReader, q: &mut VecDeque<u8>) -> Result<(), emusen_native::Truncated> {
    let len = r.u32()?;
    let mut slots = [0u8; FIFO_SLOTS];
    r.bytes(&mut slots)?;
    q.clear();
    q.extend(&slots[..(len as usize).min(FIFO_SLOTS)]);
    Ok(())
}

/// SIO2's block of `SPH2` (Sapphire_Native.md §38.2): the registers, the three FIFOs, the transfer's end, and in each
/// port the multitap, each pad's block and each card's bus state, a slot with none written as an empty one. Which
/// pads and cards are present is the machine's settings and files, kept across a load.
impl emusen_native::State for Sio2 {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        w.u32s("Send3", &self.send3);
        w.u32s("Send12", &self.send12);
        w.u32s("Registers", &[self.ctrl, self.recv1, self.istat, self.pending_recv1]);
        write_bytes(w, "FifoIn", &self.fifo_in);
        write_bytes(w, "FifoOut", &self.fifo_out);
        write_bytes(w, "Pending", &self.pending);
        crate::state::opt_u64(w, "DoneAt", self.done_at);
        w.bool("Wake", self.wake);
        for (name, s) in ["Port1", "Port2"].into_iter().zip(&self.slots) {
            w.group(name, |w| {
                w.bool("Tap", s.tap);
                crate::state::opt_u32(w, "PadSlot", s.pad_slot.map(|n| n as u32));
                crate::state::opt_u32(w, "CardSlot", s.card_slot.map(|n| n as u32));
                for (name, p) in ["Pad1", "Pad2", "Pad3", "Pad4"].into_iter().zip(&s.pads) {
                    w.group(name, |w| p.clone().unwrap_or_default().write_state(w));
                }
                for (name, c) in ["Card1", "Card2", "Card3", "Card4"].into_iter().zip(&s.cards) {
                    w.group(name, |w| match c {
                        Some(c) => c.write_bus(w),
                        None => MemoryCard::write_absent(w),
                    });
                }
            });
        }
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        r.u32s(&mut self.send3)?;
        r.u32s(&mut self.send12)?;
        let mut g = [0u32; 4];
        r.u32s(&mut g)?;
        [self.ctrl, self.recv1, self.istat, self.pending_recv1] = g;
        read_bytes(r, &mut self.fifo_in)?;
        read_bytes(r, &mut self.fifo_out)?;
        read_bytes(r, &mut self.pending)?;
        self.done_at = crate::state::read_opt_u64(r)?;
        self.wake = r.bool()?;
        for s in &mut self.slots {
            r.bool()?;
            s.pad_slot = crate::state::read_opt_u32(r)?.map(|n| (n as usize).min(tap::SLOTS - 1));
            s.card_slot = crate::state::read_opt_u32(r)?.map(|n| (n as usize).min(tap::SLOTS - 1));
            for p in &mut s.pads {
                match p {
                    Some(p) => p.read_state(r)?,
                    None => DualShock2::default().read_state(r)?,
                }
            }
            for c in &mut s.cards {
                match c {
                    Some(c) => c.read_bus(r)?,
                    None => MemoryCard::skip_bus(r)?,
                }
            }
        }
        Ok(())
    }
}

impl Sio2 {
    /// Whether the FIFOs are within their slots in the state.
    pub fn fits(&self) -> bool {
        [&self.fifo_in, &self.fifo_out, &self.pending].iter().all(|q| q.len() <= FIFO_SLOTS)
    }
}

#[cfg(test)]
mod tests;
