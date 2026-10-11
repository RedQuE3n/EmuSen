//! SIO0, the controller and memory card port (psx-spx "Serial Interfaces (SIO)", "Controller and Memory Card
//! Overview", "Signals"): its five registers, a byte shifted out and in at the baud rate, the receive FIFO, the
//! address byte that selects a slot's device, /ACK and the DSR interrupt on IRQ7. Each port holds a controller and a
//! card, or a multitap with four of each. Every rule and its source is Diamond_Pad.md.

use crate::card::Card;
use crate::multitap::Multitap;
use crate::pad::{Controller, HIGH_Z, Reply};

/// The control register's bits (psx-spx "CR").
pub mod cr {
    pub const TXEN: u16 = 1 << 0;
    pub const DTR: u16 = 1 << 1;
    pub const RXEN: u16 = 1 << 2;
    pub const ACKNOWLEDGE: u16 = 1 << 4;
    pub const RESET: u16 = 1 << 6;
    pub const DSRIEN: u16 = 1 << 12;
    pub const PORT: u16 = 1 << 13;
    /// What reads back: the write-only bits 4 and 6, and 14-15, read zero.
    pub const READABLE: u16 = 0x3FAF;
}

/// The status register's bits (psx-spx "SR").
pub mod sr {
    pub const TXRDY: u32 = 1 << 0;
    pub const RXRDY: u32 = 1 << 1;
    pub const TXIDLE: u32 = 1 << 2;
    pub const DSR: u32 = 1 << 7;
    pub const IRQ: u32 = 1 << 9;
}

/// The addresses a selected slot answers (psx-spx "Device addressing").
pub const CONTROLLER: u8 = 0x01;
pub const MEMORY_CARD: u8 = 0x81;

/// /ACK falls this long after a byte's last clock and stays low this long [C, Diamond_Pad.md §3].
pub const ACK_AFTER: u64 = 412;
pub const ACK_LOW: u64 = 100;

const NEVER: u64 = u64::MAX;

/// The receive FIFO's eight entries: a byte received fills every unused entry, and a read past the data takes those
/// copies, then zeros (psx-spx "DR Read Notes").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RxFifo {
    pub entries: [u8; 8],
    pub read: u8,
    pub count: u8,
}

impl RxFifo {
    pub fn push(&mut self, byte: u8) {
        if self.count < 8 {
            for i in self.count..8 {
                self.entries[((self.read + i) & 7) as usize] = byte;
            }
            self.count += 1;
        } else {
            self.entries[((self.read + 7) & 7) as usize] = byte;
        }
    }

    /// The oldest entry and the three after it, as DR reads.
    pub fn preview(&self) -> u32 {
        (0..4).fold(0, |v, i| v | (self.entries[((self.read + i) & 7) as usize] as u32) << (8 * i))
    }

    pub fn pop(&mut self, n: u8) {
        for _ in 0..n {
            self.entries[self.read as usize] = 0;
            self.read = (self.read + 1) & 7;
            self.count = self.count.saturating_sub(1);
        }
    }
}

/// Which device of the selected port answers: its controller or card, or a multitap's slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Controller,
    Card,
    TapController(u8),
    TapCard(u8),
}

/// Where the selected slot's transfer stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    /// Nothing selected.
    #[default]
    Idle,
    /// Selected, waiting for the address byte.
    Address,
    /// A device answering, at the byte after the address numbered here.
    Device(Target, u8),
    /// Nobody answering until the slot is deselected.
    Silent,
}

/// What one port's connectors hold.
#[derive(Clone, Debug, Default)]
pub struct Port {
    pub controller: Option<Controller>,
    pub card: Option<Card>,
    pub tap: Option<Multitap>,
}

impl Port {
    /// The device the address byte names, if it is present and answers.
    fn address(&mut self, byte: u8) -> Option<Target> {
        match (&mut self.tap, byte) {
            (Some(t), 0x01) => t.select_a().then_some(Target::TapController(0)),
            (Some(t), 0x02..=0x04) => t.pads[byte as usize - 1].is_some().then_some(Target::TapController(byte - 1)),
            (Some(t), 0x81..=0x84) => t.cards[byte as usize - 0x81].is_some().then_some(Target::TapCard(byte - 0x81)),
            (Some(_), _) => None,
            (None, CONTROLLER) => self.controller.is_some().then_some(Target::Controller),
            (None, MEMORY_CARD) => self.card.is_some().then_some(Target::Card),
            (None, _) => None,
        }
    }

    fn exchange(&mut self, target: Target, step: u8, byte: u8) -> (Reply, bool) {
        let none = (HIGH_Z, false);
        match (target, &mut self.tap) {
            (Target::Controller, _) => self.controller.as_mut().map_or(none, |c| c.exchange(step, byte)),
            (Target::Card, _) => self.card.as_mut().map_or(none, |c| c.exchange(step, byte)),
            (Target::TapController(0), Some(t)) => t.exchange_a(step, byte),
            (Target::TapController(n), Some(t)) => t.pads[n as usize].as_mut().map_or(none, |c| c.exchange(step, byte)),
            (Target::TapCard(n), Some(t)) => t.cards[n as usize].as_mut().map_or(none, |c| c.exchange(step, byte)),
            _ => none,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Sio0 {
    pub mode: u16,
    pub control: u16,
    pub baud: u16,
    /// When the baud rate timer was last reloaded by a write.
    pub baud_base: u64,
    pub fifo: RxFifo,
    /// The byte being shifted, the cycle its first clock fell, and the byte waiting behind it.
    pub shifting: Option<u8>,
    pub started: u64,
    pub pending: Option<u8>,
    /// TXEN as it stood when DR was last written (psx-spx "DR": either it or the current TXEN starts a transfer).
    pub latched_txen: bool,
    /// The moments due: the last bit sampled, the last clock's end, /ACK falling and rising.
    pub sampled_at: u64,
    pub ends_at: u64,
    pub ack_at: u64,
    pub ack_ends: u64,
    pub dsr: bool,
    pub irq: bool,
    pub phase: Phase,
    pub ports: [Port; 2],
}

impl Sio0 {
    pub fn new(ports: [Port; 2]) -> Sio0 {
        Sio0 { sampled_at: NEVER, ends_at: NEVER, ack_at: NEVER, ack_ends: NEVER, ports, ..Sio0::default() }
    }

    /// One bit's time in CPU cycles (psx-spx "BR": the reload times the mode's factor, made even).
    pub fn bit_cycles(&self) -> u64 {
        let factor = [1, 1, 16, 64][(self.mode & 3) as usize];
        ((self.baud as u64 * factor) & !1).max(1)
    }

    pub fn next_event(&self) -> u64 {
        self.sampled_at.min(self.ends_at).min(self.ack_at).min(self.ack_ends)
    }

    /// Runs what is due by `now`, in order; true when the interrupt rose.
    pub fn run(&mut self, now: u64) -> bool {
        let mut rose = false;
        loop {
            let at = self.next_event();
            if at > now {
                return rose;
            }
            if at == self.sampled_at {
                self.sampled_at = NEVER;
                self.sample();
            } else if at == self.ends_at {
                self.ends_at = NEVER;
                self.shifting = None;
                self.try_start(at);
            } else if at == self.ack_at {
                self.ack_at = NEVER;
                self.ack_ends = at + ACK_LOW;
                self.dsr = true;
                rose |= self.raise();
            } else {
                self.ack_ends = NEVER;
                self.dsr = false;
            }
        }
    }

    /// The DSR interrupt's condition met: SR.9 rises if it was clear.
    fn raise(&mut self) -> bool {
        let rose = self.dsr && self.control & cr::DSRIEN != 0 && !self.irq;
        self.irq |= rose;
        rose
    }

    fn selected(&self) -> bool {
        self.control & cr::DTR != 0
    }

    fn port(&self) -> usize {
        (self.control & cr::PORT != 0) as usize
    }

    /// The last bit sampled: the selected slot answers the byte, and the answer is received.
    fn sample(&mut self) {
        let byte = self.shifting.unwrap_or(0xFF);
        let reply = self.answer(byte);
        if self.selected() || self.control & cr::RXEN != 0 {
            self.fifo.push(reply.byte);
            self.control &= !cr::RXEN;
        }
        if let Some(extra) = reply.ack {
            self.ack_at = self.ends_at + ACK_AFTER + extra as u64;
        }
    }

    fn answer(&mut self, byte: u8) -> Reply {
        if !self.selected() {
            return HIGH_Z;
        }
        let p = self.port();
        match self.phase {
            Phase::Address => match self.ports[p].address(byte) {
                Some(t) => {
                    self.phase = Phase::Device(t, 1);
                    Reply::more(HIGH_Z.byte)
                }
                None => {
                    self.phase = Phase::Silent;
                    HIGH_Z
                }
            },
            Phase::Device(t, step) => {
                let (reply, more) = self.ports[p].exchange(t, step, byte);
                self.phase = if more { Phase::Device(t, step.saturating_add(1)) } else { Phase::Silent };
                reply
            }
            Phase::Idle | Phase::Silent => HIGH_Z,
        }
    }

    /// Starts the waiting byte at `at` if the transmitter is free and enabled.
    fn try_start(&mut self, at: u64) {
        if self.shifting.is_some() || self.control & cr::TXEN == 0 && !self.latched_txen {
            return;
        }
        if let Some(byte) = self.pending.take() {
            let bit = self.bit_cycles();
            self.shifting = Some(byte);
            self.started = at;
            self.sampled_at = at + 7 * bit + bit / 2;
            self.ends_at = at + 8 * bit;
        }
    }

    pub fn status(&self, now: u64) -> u32 {
        let idle = self.shifting.is_none() && self.pending.is_none();
        let period = (self.bit_cycles() / 2).max(1);
        let timer = (period - (now - self.baud_base.min(now)) % period) as u32 & 0x1F_FFFF;
let bit = |on: bool, b: u32| if on { b } else { 0 };
        bit(self.pending.is_none(), sr::TXRDY) | bit(self.fifo.count > 0, sr::RXRDY) | bit(idle, sr::TXIDLE) | bit(self.dsr, sr::DSR) | bit(self.irq, sr::IRQ) | timer << 11
    }

    /// A read of `bytes` at `offset` from 1F801040h, brought to `now` by the caller.
    pub fn read(&mut self, offset: u32, bytes: u32, now: u64) -> u32 {
        let o = offset & 0xF;
        let word = match o & !3 {
            0x0 => {
                let v = self.fifo.preview();
                self.fifo.pop(if bytes == 4 { 4 } else { 1 });
                v
            }
            0x4 => self.status(now),
            0x8 => self.mode as u32 | (self.control as u32) << 16,
            _ => (self.baud as u32) << 16,
        };
        (word >> (8 * (o & 3))) & (u32::MAX >> (32 - 8 * bytes.min(4)))
    }

    /// A write at `offset` from 1F801040h at `now`; true when the interrupt rose.
    pub fn write(&mut self, offset: u32, bytes: u32, value: u32, now: u64) -> bool {
        match offset & 0xF {
            0x0..=0x3 => {
                self.pending = Some(value as u8);
                self.latched_txen = self.control & cr::TXEN != 0;
                self.try_start(now);
                false
            }
            0x8 => {
                self.mode = value as u16 & 0x13F;
                false
            }
            0xA => self.set_control(value as u16, now),
            0xC if bytes == 4 => self.set_baud((value >> 16) as u16, now),
            0xE => self.set_baud(value as u16, now),
            _ => false,
        }
    }

    fn set_baud(&mut self, value: u16, now: u64) -> bool {
        self.baud = value;
        self.baud_base = now;
        false
    }

    fn set_control(&mut self, value: u16, now: u64) -> bool {
        if value & cr::RESET != 0 {
            let ports = std::mem::take(&mut self.ports);
            *self = Sio0 { baud: self.baud, baud_base: self.baud_base, ..Sio0::new(ports) };
            return false;
        }
        let was = (self.selected(), self.port());
        self.control = value & cr::READABLE;
        if (self.selected(), self.port()) != was {
            self.phase = if self.selected() { Phase::Address } else { Phase::Idle };
        }
        if value & cr::ACKNOWLEDGE != 0 {
            self.irq &= self.dsr && self.control & cr::DSRIEN != 0;
        }
        self.try_start(now);
        self.raise()
    }

    /// The controller the host's port `n` names: 0 and 1 the two ports' (or their multitaps' slot A), 2-4 port 1's
    /// multitap slots B-D, 5-7 port 2's.
    pub fn controller(&mut self, n: usize) -> Option<&mut Controller> {
        let (port, slot) = match n {
            0 | 1 => (n, 0),
            2..=4 => (0, n - 1),
            5..=7 => (1, n - 4),
            _ => return None,
        };
        let p = &mut self.ports[port];
        match &mut p.tap {
            Some(t) => t.pads[slot].as_mut(),
            None if slot == 0 => p.controller.as_mut(),
            None => None,
        }
    }

    /// The card in slot `n`: 0 and 1 the two ports' (or their multitaps' slot A), then the multitaps' as above.
    pub fn card(&mut self, n: usize) -> Option<&mut Card> {
        let (port, slot) = match n {
            0 | 1 => (n, 0),
            2..=4 => (0, n - 1),
            5..=7 => (1, n - 4),
            _ => return None,
        };
        let p = &mut self.ports[port];
        match &mut p.tap {
            Some(t) => t.cards[slot].as_mut(),
            None if slot == 0 => p.card.as_mut(),
            None => None,
        }
    }

    pub fn set_buttons(&mut self, n: usize, mask: u32) {
        if let Some(c) = self.controller(n) {
            c.set_buttons(mask);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pad::DigitalPad;

    const BIT: u64 = 0x88;

    fn port() -> Sio0 {
        let pad = Port { controller: Some(Controller::Digital(DigitalPad { pressed: 1 << 3 })), ..Port::default() };
        let mut s = Sio0::new([pad, Port::default()]);
        s.write(0xE, 2, 0x88, 0);
        s.write(0x8, 2, 0x0D, 0);
        s
    }

    /// Sends each byte, acknowledges the last interrupt 100 cycles later and waits for the next, as psx-spx's
    /// "Emulation Note" describes the kernel's driver; the replies and when each interrupt rose.
    fn transfer(s: &mut Sio0, control: u16, bytes: &[u8]) -> (Vec<u8>, Vec<u64>) {
        let mut now = 1000;
        s.write(0xA, 2, control as u32, now);
        let (mut replies, mut irqs) = (Vec::new(), Vec::new());
        for &b in bytes {
            let sent = now;
            s.write(0x0, 1, b as u32, now);
            now += 100;
            s.run(now);
            s.write(0xA, 2, (control | cr::ACKNOWLEDGE) as u32, now);
            while now < sent + 8 * BIT + ACK_AFTER + ACK_LOW {
                now += 1;
                if s.run(now) {
                    irqs.push(now - sent);
                    break;
                }
            }
            s.run(sent + 8 * BIT);
            replies.push(s.read(0, 1, now) as u8);
        }
        (replies, irqs)
    }

    #[test]
    fn a_pad_read_answers_with_an_ack_after_every_byte_but_the_last() {
        let mut s = port();
        let (replies, irqs) = transfer(&mut s, cr::TXEN | cr::DTR | cr::DSRIEN, &[0x01, 0x42, 0, 0, 0]);
        assert_eq!(replies, [0xFF, 0x41, 0x5A, 0xF7, 0xFF]);
        assert_eq!(irqs, [8 * BIT + ACK_AFTER; 4], "1,500 cycles from the byte's first clock at the kernel's rate");
        assert_eq!(s.status(0) & (sr::IRQ | sr::RXRDY), 0);
    }

    #[test]
    fn port_2_and_a_card_slot_answer_as_empty() {
        for (control, address) in [(cr::TXEN | cr::DTR | cr::DSRIEN | cr::PORT, CONTROLLER), (cr::TXEN | cr::DTR | cr::DSRIEN, MEMORY_CARD)] {
            let mut s = port();
            let (replies, irqs) = transfer(&mut s, control, &[address, 0x42, 0]);
            assert_eq!((replies, irqs), (vec![0xFF; 3], vec![]));
        }
    }

    #[test]
    fn the_status_follows_a_byte_through_the_transmitter_and_the_fifo() {
        let mut s = port();
        s.write(0xA, 2, (cr::TXEN | cr::DTR) as u32, 0);
        s.write(0, 1, 0x01, 100);
        assert_eq!(s.status(100) & 7, sr::TXRDY, "started: another byte may be written, not idle");
        s.write(0, 1, 0x42, 101);
        assert_eq!(s.status(101) & 7, 0, "one waiting");
        s.run(100 + 7 * BIT + BIT / 2 - 1);
        assert_eq!(s.status(0) & sr::RXRDY, 0, "not before the eighth bit is sampled");
        s.run(100 + 7 * BIT + BIT / 2);
        assert_eq!(s.status(0) & 7, sr::RXRDY);
        s.run(100 + 16 * BIT);
        assert_eq!(s.status(0) & 7, sr::TXRDY | sr::RXRDY | sr::TXIDLE, "the second followed the first at once");
        assert_eq!(s.read(0, 2, 0) & 0xFFFF, 0x41FF, "a halfword previews the second entry");
        assert_eq!(s.read(0, 1, 0), 0x41, "and takes only one");
        assert_eq!((s.read(0, 1, 0), s.read(0, 1, 0), s.status(0) & sr::RXRDY), (0x41, 0x41, 0), "then the copies of the last");
    }

    #[test]
    fn the_fifo_keeps_copies_of_its_last_byte_then_zeros() {
        let mut f = RxFifo::default();
        for b in 1..=5 {
            f.push(b);
        }
        let read: Vec<u8> = (0..9).map(|_| {
            let v = f.preview() as u8;
            f.pop(1);
            v
        }).collect();
        assert_eq!(read, [1, 2, 3, 4, 5, 5, 5, 5, 0]);
    }

    #[test]
    fn no_transfer_starts_without_txen_and_the_interrupt_waits_for_dsrien() {
        let mut s = port();
        s.write(0xA, 2, cr::DTR as u32, 0);
        s.write(0, 1, 0x01, 10);
        assert_eq!(s.next_event(), NEVER);
        assert!(!s.write(0xA, 2, (cr::DTR | cr::TXEN) as u32, 20));
        assert_eq!(s.started, 20, "TXEN starts the waiting byte");
        assert!(!s.run(20 + 8 * BIT + ACK_AFTER), "/ACK fell with DSRIEN clear");
        assert_eq!(s.status(0) & (sr::DSR | sr::IRQ), sr::DSR);
        assert!(s.write(0xA, 2, (cr::DTR | cr::TXEN | cr::DSRIEN) as u32, 20 + 8 * BIT + ACK_AFTER + 1), "DSRIEN while /ACK is low");
        s.run(20 + 8 * BIT + ACK_AFTER + ACK_LOW);
        assert_eq!(s.status(0) & (sr::DSR | sr::IRQ), sr::IRQ, "SR.9 stays until acknowledged");
        s.write(0xA, 2, (cr::RESET) as u32, 0);
        assert_eq!((s.control, s.mode, s.baud, s.status(0) & 0x3FF), (0, 0, 0x88, sr::TXRDY | sr::TXIDLE));
    }
}
