//! The CD-ROM controller at its register interface (psx-spx "CDROM Drive"), the open replacement for the HC05's
//! program (Diamond_Plan.md §5.6): the decoder's host registers and FIFOs, the command protocol with its first and
//! second responses and their delays, and the drive beneath it: spin-up, seeks by distance, reads at 75 and 150
//! sectors a second, play, the sector buffer, XA-ADPCM and CD-DA to the SPU's input, subchannel Q, the lid.
//! Diamond_CDROM.md records each rule, its source and its disputes.

pub mod audio;
mod commands;
mod drive;
mod state;
pub mod xa;

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::medium::Disc;
use emusen_disc::{Place, SubQ};
use emusen_native::core::{SEEK_SPAN, seek};

use crate::media::Market;
use audio::Output;
use xa::Xa;

/// A sector's time at the standard speed, 33,868,800 × 2,352 / 4 / 44,100 cycles (psx-spx "INT1 Rate"), and double.
pub const SECTOR_SINGLE: u64 = 451_584;
pub const SECTOR_DOUBLE: u64 = SECTOR_SINGLE / 2;

/// The status byte (psx-spx "Status code").
pub mod stat {
    pub const ERROR: u8 = 1 << 0;
    pub const MOTOR: u8 = 1 << 1;
    pub const SEEK_ERROR: u8 = 1 << 2;
    pub const ID_ERROR: u8 = 1 << 3;
    pub const SHELL_OPEN: u8 = 1 << 4;
    pub const READ: u8 = 1 << 5;
    pub const SEEK: u8 = 1 << 6;
    pub const PLAY: u8 = 1 << 7;
}

/// Setmode's bits (psx-spx "Setmode").
pub mod mode {
    pub const CDDA: u8 = 1 << 0;
    pub const AUTO_PAUSE: u8 = 1 << 1;
    pub const REPORT: u8 = 1 << 2;
    pub const FILTER: u8 = 1 << 3;
    pub const IGNORE: u8 = 1 << 4;
    pub const WHOLE: u8 = 1 << 5;
    pub const XA: u8 = 1 << 6;
    pub const DOUBLE: u8 = 1 << 7;
}

/// What a deferred response carries: the status byte as it is when delivered, or fixed bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reply {
    Stat,
    Bytes([u8; 12], u8),
}

/// A response waiting for its time and for the interrupt before it to be acknowledged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pending {
    pub int: u8,
    pub reply: Reply,
    pub at: u64,
}

/// What the drive is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    /// The spindle is still (or spinning down, `motor` still set until it stops).
    Stopped,
    /// Spinning up, then reading the TOC, before `then`.
    SpinningUp,
    /// Holding near `hover`, the head stepping back over the last few sectors as a paused drive does.
    Idle,
    Seeking,
    Reading,
    Playing,
}

/// What follows a seek or a spin-up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Then {
    /// Hold, and send INT2 if a command waits for it (SeekL, SeekP, Init, MotorOn).
    Hold { complete: bool },
    Read,
    Play,
}

/// The sector a data delivery hands over: its raw bytes and whether the filter must still pass it.
#[derive(Clone)]
pub struct Buffered {
    pub raw: Box<[u8; 2352]>,
    pub at: u32,
    pub second_try: bool,
}

#[derive(Clone)]
pub struct Cdrom {
    // --- the decoder's host interface ---
    pub index: u8,
    pub mask: u8,
    /// HINTSTS: the HC05's interrupt type in bits 0-2, BFEMPT and BFWRDY above.
    pub flags: u8,
    pub params: [u8; 16],
    pub param_len: u8,
    pub results: [u8; 16],
    pub result_len: u8,
    pub result_pos: u8,
    /// RSLRRDY: bytes of the last response not yet read.
    pub result_ready: bool,
    /// The data FIFO, loaded by BFRD from the sector the last INT1 handed over.
    pub data: Box<[u8; 2352]>,
    pub data_len: u16,
    pub data_pos: u16,
    pub hchpctl: u8,
    // --- the controller ---
    /// A command written and not yet executed, the cycle it may execute, and whether BUSYSTS shows it.
    pub command: Option<u8>,
    pub command_at: u64,
    /// Deferred responses in the order made (second responses, INT4, INT5 errors), and an INT1 sector ready.
    pub queue: [Option<Pending>; 4],
    pub mode: u8,
    /// The sector size in effect, kept through a Setmode with the Ignore bit (psx-spx).
    pub whole: bool,
    pub filter: (u8, u8),
    pub setloc: u32,
    pub setloc_pending: bool,
    pub shell_flag: bool,
    pub seek_error: bool,
    pub id_error: bool,
    /// The cycle until which a Reset leaves commands unanswered.
    pub reset_until: u64,
    /// A command of 58h-5Fh has stopped the program until the decoder is reset (psx-spx).
    pub crashed: bool,
    // --- the drive ---
    pub activity: Activity,
    pub then: Then,
    pub motor: bool,
    pub double: bool,
    /// The head's sector, the seek's target, the sector a pause holds near, and the drive's next event.
    pub position: u32,
    pub target: u32,
    pub hover: u32,
    /// Where a Read with no new Setloc begins: the sector a seek reached, or the one received last (psx-spx "Setloc,
    /// Read, Pause").
    pub resume: u32,
    pub drive_at: u64,
    pub motor_stops_at: u64,
    pub lid_open: bool,
    /// A seek waiting for the spin-up to end, and whether it is SeekL's, which fails on an audio track.
    pub pending_seek: bool,
    pub seek_data: bool,
    // --- the sector buffer, its deliveries and the positions they report ---
    pub delivered: Buffered,
    pub waiting: Option<Buffered>,
    pub int1_at: u64,
    pub header: [u8; 8],
    pub header_valid: bool,
    pub q: SubQ,
    pub play_track: u8,
    // --- audio ---
    pub xa: Xa,
    pub out: Output,
    /// Scratch for the sector the head reads and for a decoded XA sector, so that no sector allocates.
    pub scratch: Box<[u8; 2352]>,
    pub xa_frames: Vec<[i16; 2]>,
    /// The console's market: its region string, its firmware date, the SCEx letter its discs are licensed with.
    pub market: Market,
    pub disc: Option<DiscSlot>,
    /// The next event's cycle, as `due` computes it.
    pub next: u64,
    /// A development tool's trace of responses and commands to stderr; never set while a BIOS runs its own code.
    pub trace: bool,
}

/// The console's one drive, as machine info's `drives`, MEDIA's exports and the drive's events number it.
pub const DRIVE: u32 = 0;

/// The disc in the drive, shared by a board and its copies (a state load works on a copy) rather than duplicated.
#[derive(Clone)]
pub struct DiscSlot(Arc<Mutex<InDrive>>);

/// A disc while it is in the drive: the disc, and the place of it last named to the host, which is no state.
pub struct InDrive {
    disc: Disc,
    named: Option<Place>,
}

impl Deref for InDrive {
    type Target = Disc;

    fn deref(&self) -> &Disc {
        &self.disc
    }
}

impl DerefMut for InDrive {
    fn deref_mut(&mut self) -> &mut Disc {
        &mut self.disc
    }
}

impl InDrive {
    /// Tells the host that the head is coming to sector `at`, by the file and byte that hold it, unless that byte lies
    /// in the megabyte after the place last named (`EMUSEN_EVENT_MEDIA_SEEK`; Diamond_CDROM.md §5). A hint for a host
    /// that reads ahead of the drive: nothing here depends on it, and a sector the image does not hold names nothing.
    pub fn name(&mut self, at: u32) {
        self.name_unless(at, |named, place| place >= named && place - named < SEEK_SPAN);
    }

    /// The same for the sector a held head passes over, which lies a few sectors to either side of a place named for
    /// it: said only where nothing near was named, as after a state loaded with the head elsewhere.
    pub fn name_held(&mut self, at: u32) {
        self.name_unless(at, |named, place| named.abs_diff(place) < SEEK_SPAN);
    }

    fn name_unless(&mut self, at: u32, known: impl Fn(u64, u64) -> bool) {
        let Some(place) = self.disc.place(at) else { return };
        if self.named.is_some_and(|n| n.file == place.file && known(n.offset, place.offset)) {
            return;
        }
        self.named = Some(place);
        seek(DRIVE, place.file.map(|f| f as u32), place.offset);
    }
}

impl DiscSlot {
    pub fn new(d: Disc) -> DiscSlot {
        DiscSlot(Arc::new(Mutex::new(InDrive { disc: d, named: None })))
    }

    pub fn get(&self) -> MutexGuard<'_, InDrive> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// The firmware date and version 19h,20h answers (psx-spx's table): the PU-18 boards of SCPH-5500 and SCPH-5501/5502.
pub fn firmware(market: Market) -> [u8; 4] {
    if market == Market::Japan { [0x96, 0x09, 0x12, 0xC2] } else { [0x97, 0x01, 0x10, 0xC2] }
}

impl Cdrom {
    pub fn new(disc: Option<Disc>, market: Market) -> Cdrom {
        Cdrom::powered(disc.map(DiscSlot::new), false, market)
    }

    /// The drive as its controller comes out of reset, holding the disc in `slot` with its lid as it is: closed on a
    /// disc, it spins up (Diamond_Native.md §27).
    pub fn powered(slot: Option<DiscSlot>, lid_open: bool, market: Market) -> Cdrom {
        let empty = Buffered { raw: Box::new([0; 2352]), at: 0, second_try: false };
        let mut c = Cdrom {
            index: 0,
            mask: 0,
            flags: 0,
            params: [0; 16],
            param_len: 0,
            results: [0; 16],
            result_len: 0,
            result_pos: 0,
            result_ready: false,
            data: Box::new([0; 2352]),
            data_len: 0,
            data_pos: 0,
            hchpctl: 0,
            command: None,
            command_at: 0,
            queue: [None; 4],
            mode: 0,
            whole: false,
            filter: (0, 0),
            setloc: 0,
            setloc_pending: false,
            shell_flag: true,
            seek_error: false,
            id_error: false,
            reset_until: 0,
            crashed: false,
            activity: Activity::Stopped,
            then: Then::Hold { complete: false },
            motor: false,
            double: false,
            position: 0,
            target: 0,
            hover: drive::PARKED,
            resume: drive::PARKED,
            drive_at: u64::MAX,
            motor_stops_at: u64::MAX,
            lid_open,
            pending_seek: false,
            seek_data: false,
            delivered: empty,
            waiting: None,
            int1_at: u64::MAX,
            header: [0; 8],
            header_valid: false,
            q: SubQ::default(),
            play_track: 0,
            xa: Xa::default(),
            out: Output::default(),
            scratch: Box::new([0; 2352]),
            xa_frames: Vec::with_capacity(4032),
            market,
            disc: slot,
            next: u64::MAX,
            trace: false,
        };
        if c.disc.is_some() && !lid_open {
            c.spin_up(0, Then::Hold { complete: false });
        }
        c.refresh();
        c
    }

    /// Whether the controller asks for IRQ2: HINTSTS and HINTMSK meet.
    pub fn line(&self) -> bool {
        self.flags & self.mask & 0x1F != 0
    }

    /// The status byte.
    pub fn stat(&self) -> u8 {
        let mut s = 0;
        match self.activity {
            Activity::Reading => s |= stat::READ,
            Activity::Seeking => s |= stat::SEEK,
            Activity::Playing => s |= stat::PLAY,
            _ => {}
        }
        if self.motor && self.activity != Activity::SpinningUp {
            s |= stat::MOTOR;
        }
        if self.shell_flag {
            s |= stat::SHELL_OPEN;
        }
        if self.seek_error {
            s |= stat::SEEK_ERROR;
        }
        if self.id_error {
            s |= stat::ID_ERROR;
        }
        s
    }

    // --- registers ---

    /// HSTS (psx-spx): the bank, ADPBUSY, the FIFOs' states, BUSYSTS.
    pub fn status(&self) -> u8 {
        let mut s = self.index;
        if self.out.from_xa && self.out.len > 0 {
            s |= 1 << 2;
        }
        if self.param_len == 0 {
            s |= 1 << 3;
        }
        if self.param_len < 16 {
            s |= 1 << 4;
        }
        if self.result_ready {
            s |= 1 << 5;
        }
        if self.data_pos < self.data_len {
            s |= 1 << 6;
        }
        if self.command.is_some() {
            s |= 1 << 7;
        }
        s
    }

    pub fn read8(&mut self, offset: u32) -> u8 {
        if self.trace && offset & 3 != 1 {
            eprintln!("cd read {}.{} hsts {:02X} pos {} len {}", offset & 3, self.index, self.status(), self.result_pos, self.result_len);
        }
        match (offset & 3, self.index) {
            (0, _) => self.status(),
            (1, _) => {
                let v = self.results[self.result_pos as usize];
                if self.trace {
                    eprintln!("cd result read {} -> {v:02X} ready {}", self.result_pos, self.result_ready);
                }
                self.result_pos = (self.result_pos + 1) & 15;
                if self.result_pos == self.result_len & 15 {
                    self.result_ready = false;
                }
                v
            }
            (2, _) => self.data_byte(),
            (3, 0 | 2) => 0xE0 | self.mask,
            _ => 0xE0 | self.flags,
        }
    }

    /// RDDATA: the next byte, or past the sector's end the byte psx-spx says repeats.
    pub fn data_byte(&mut self) -> u8 {
        if self.data_len == 0 {
            return 0;
        }
        if self.data_pos < self.data_len {
            let v = self.data[self.data_pos as usize];
            self.data_pos += 1;
            return v;
        }
        let at = if self.data_len == 0x800 { 0x800 - 8 } else { self.data_len as usize - 4 };
        self.data[at]
    }

    /// DRQSTS: the data FIFO holds bytes, which DMA channel 3 waits for.
    pub fn data_ready(&self) -> bool {
        self.data_pos < self.data_len
    }

    /// A DMA word: four RDDATA bytes, as the bus splits them.
    pub fn data_word(&mut self) -> u32 {
        u32::from_le_bytes([self.data_byte(), self.data_byte(), self.data_byte(), self.data_byte()])
    }

    pub fn write8(&mut self, offset: u32, value: u8, now: u64) {
        self.write_register(offset, value, now);
        self.refresh();
    }

    fn write_register(&mut self, offset: u32, value: u8, now: u64) {
        if self.trace {
            eprintln!("cd write {offset}.{} {value:02X} at {now}", self.index);
        }
        match (offset & 3, self.index) {
            (0, _) => self.index = value & 3,
            (1, 0) => self.write_command(value, now),
            (2, 0) => {
                if self.param_len < 16 {
                    self.params[self.param_len as usize] = value;
                    self.param_len += 1;
                }
            }
            (3, 0) => self.write_hchpctl(value),
            (1, 1) => {}
            (2, 1) => self.mask = value & 0x1F,
            (3, 1) => self.acknowledge(value, now),
            (1, 2) => {}
            (2, 2) => self.out.written[0] = value,
            (3, 2) => self.out.written[1] = value,
            (1, 3) => self.out.written[2] = value,
            (2, 3) => self.out.written[3] = value,
            (3, 3) => {
                self.out.xa_muted = value & 1 != 0;
                if value & 1 << 5 != 0 {
                    self.out.apply_volume();
                }
            }
            _ => {}
        }
    }

    /// HCHPCTL: BFRD set loads the data FIFO with the sector the last INT1 handed over, from its start; set again while
    /// the FIFO still holds bytes it changes nothing; cleared, it empties the FIFO (Diamond_CDROM.md §3).
    fn write_hchpctl(&mut self, value: u8) {
        let rising = value & 0x80 != 0 && self.hchpctl & 0x80 == 0;
        self.hchpctl = value;
        if value & 0x80 == 0 {
            self.data_len = 0;
            self.data_pos = 0;
            return;
        }
        if !rising && self.data_pos < self.data_len {
            return;
        }
        let raw = &self.delivered.raw;
        let (from, len) = if self.whole { (12, 0x924) } else if raw[15] == 1 { (16, 0x800) } else { (24, 0x800) };
        self.data[..len].copy_from_slice(&raw[from..from + len]);
        self.data_len = len as u16;
        self.data_pos = 0;
    }

    /// HCLRCTL: acknowledging the interrupt flags lets the next response or the waiting command through, the result
    /// FIFO kept until a new response replaces it (Diamond_CDROM.md §3); CLRPRM empties the parameter FIFO, CHPRST resets the decoder.
    fn acknowledge(&mut self, value: u8, now: u64) {
        self.flags &= !(value & 0x1F);
        if value & 7 != 0 && self.flags & 7 == 0 {
            let next = now + commands::AFTER_ACK;
            self.int1_at = self.int1_at.max(next);
            for p in self.queue.iter_mut().flatten() {
                p.at = p.at.max(next);
            }
            if self.command.is_some() {
                self.command_at = self.command_at.max(next);
            }
        }
        if value & 0x40 != 0 {
            self.param_len = 0;
        }
        if value & 0x80 != 0 {
            self.param_len = 0;
            self.result_len = 0;
            self.result_pos = 0;
            self.data_len = 0;
            self.data_pos = 0;
            self.result_ready = false;
            self.flags = 0;
            self.crashed = false;
        }
    }

    /// Sets the result FIFO and the interrupt type.
    fn respond(&mut self, int: u8, bytes: &[u8]) {
        if self.trace {
            eprintln!("cd INT{int} {bytes:02X?} pos {} {:?}", self.position, self.activity);
        }
        let n = bytes.len().min(16);
        self.results[..n].copy_from_slice(&bytes[..n]);
        self.results[n..].fill(0);
        self.result_len = n as u8;
        self.result_pos = 0;
        self.result_ready = n > 0;
        self.flags = self.flags & !7 | int;
    }

    /// Queues a deferred response. A second response (INT2, or INT5 in its place) takes the one place the controller
    /// keeps for it, replacing one not yet delivered (psx-spx "Responses": one flag for INT2); the oldest of the others
    /// is dropped if the four places are full.
    fn defer(&mut self, int: u8, reply: Reply, at: u64) {
        if matches!(int, 2 | 5) {
            for q in self.queue.iter_mut() {
                if q.is_some_and(|p| matches!(p.int, 2 | 5)) {
                    *q = None;
                }
            }
            self.queue.sort_by_key(|q| q.is_none());
        }
        let slot = self.queue.iter().position(Option::is_none).unwrap_or_else(|| {
            self.queue.rotate_left(1);
            3
        });
        self.queue[slot] = Some(Pending { int, reply, at });
    }

    /// The earliest cycle anything here is due, kept current by every change so that the bus's scheduling reads one
    /// field.
    pub fn next_event(&self) -> u64 {
        self.next
    }

    pub(crate) fn refresh(&mut self) {
        self.next = self.due();
    }

    fn due(&self) -> u64 {
        let mut t = self.drive_at.min(self.motor_stops_at);
        if self.flags & 7 == 0 {
            if self.command.is_some() {
                t = t.min(self.command_at.max(self.reset_until));
            }
            for p in self.queue.iter().flatten() {
                t = t.min(p.at);
            }
            if self.waiting.is_some() {
                t = t.min(self.int1_at);
            }
        }
        t
    }

    /// Runs everything due by `now`, in time order; returns whether IRQ2 rose.
    pub fn run(&mut self, now: u64) -> bool {
        let was = self.line();
        let mut rose = false;
        loop {
            let at = self.due();
            if at > now {
                break;
            }
            self.step(at);
            if self.line() && !was {
                rose = true;
            }
        }
        self.refresh();
        rose || (self.line() && !was)
    }

    /// The one event due at `at`.
    fn step(&mut self, at: u64) {
        if self.motor_stops_at <= at {
            self.motor_stops_at = u64::MAX;
            self.motor = false;
            return;
        }
        if self.flags & 7 == 0 {
            let first = self.queue.iter().enumerate().filter_map(|(i, p)| p.map(|p| (p.at, i))).min();
            let cmd = self.command.map(|_| self.command_at.max(self.reset_until));
            let int1 = self.waiting.as_ref().map(|_| self.int1_at);
            let best = [first.map(|f| f.0), cmd, int1].into_iter().flatten().min();
            if best.is_some_and(|b| b <= at && b < self.drive_at) {
                let b = best.unwrap_or(at);
                if first.is_some_and(|f| f.0 == b) {
                    let i = first.map_or(0, |f| f.1);
                    let p = self.queue[i].take().unwrap_or(Pending { int: 0, reply: Reply::Stat, at: b });
                    self.queue[i..].rotate_left(1);
                    match p.reply {
                        Reply::Stat => {
                            let s = self.stat();
                            self.respond(p.int, &[s]);
                        }
                        Reply::Bytes(bytes, n) => self.respond(p.int, &bytes[..n as usize]),
                    }
                } else if int1 == Some(b) {
                    self.deliver_waiting();
                } else {
                    self.execute(b);
                }
                return;
            }
        }
        if self.drive_at <= at {
            let t = self.drive_at;
            self.drive_at = u64::MAX;
            self.drive_event(t);
        }
    }

    /// The INT1 of the sector waiting in the buffer, unless the filter, on its second try, now refuses it.
    fn deliver_waiting(&mut self) {
        let Some(b) = self.waiting.take() else { return };
        self.int1_at = u64::MAX;
        if b.second_try && self.mode & mode::FILTER != 0 && (b.raw[16], b.raw[17]) != self.filter {
            return;
        }
        self.delivered = b;
        let s = self.stat();
        self.respond(1, &[s]);
    }

    /// The SPU's CD input for `frames` samples: the frames are taken in order (and dropped until the SPU exists).
    pub fn take_audio(&mut self, frames: usize, mut tap: Option<&mut Vec<[i16; 2]>>) {
        for _ in 0..frames {
            let f = self.out.take();
            if let Some(t) = tap.as_deref_mut() {
                t.push(f);
            }
        }
    }
}

#[cfg(test)]
mod tests;
