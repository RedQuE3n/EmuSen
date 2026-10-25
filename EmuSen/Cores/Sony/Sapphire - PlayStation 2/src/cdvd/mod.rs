//! The CDVD drive controller at its register interface (Sapphire_Plan.md §5.6): the open replacement of the Mechacon's
//! firmware, modelled from ps2tek's "CDVD Drive" chapter with the referees' answers where it is silent. N commands run
//! in the drive's time and raise its interrupt; S commands answer at once. The disc is `emusen-disc`'s medium; the IOP's
//! bus and DMA channel 3 (S21) reach this through `read8`, `write8` and `dma_take`. Sapphire_Native.md §25.

pub mod nvm;
pub mod rtc;
pub mod timing;

use emusen_disc::ecma267::{self, FRAME, TrackPath};
use emusen_native::core::Kept;

use nvm::{BLOCK, Nvm};
use rtc::Rtc;
use timing::{CD_SPEED, DVD_SPEED, PROMPT, SPIN_UP, STOP, block};

/// A disc in the drive: a compact disc or a DVD over the host's kept bytes.
pub type Disc = emusen_disc::Medium<Kept>;

/// The registers' base on the IOP's bus, and their offsets from it [R ps2tek "CDVD I/O Ports"].
pub const BASE: u32 = 0x1F40_2000;
pub mod reg {
    pub const N_COMMAND: u32 = 0x04;
    /// The N status when read, a parameter when written.
    pub const N_STATUS: u32 = 0x05;
    pub const ERROR: u32 = 0x06;
    pub const BREAK: u32 = 0x07;
    pub const I_STAT: u32 = 0x08;
    pub const DRIVE: u32 = 0x0A;
    pub const STICKY: u32 = 0x0B;
    pub const DISK_TYPE: u32 = 0x0F;
    pub const S_COMMAND: u32 = 0x16;
    /// The S status when read, a parameter when written.
    pub const S_STATUS: u32 = 0x17;
    pub const S_RESULT: u32 = 0x18;
}

/// I_STAT's bits [R ps2tek].
pub mod istat {
    pub const DATA_READY: u8 = 1 << 0;
    pub const COMPLETE: u8 = 1 << 1;
    pub const POWER_OFF: u8 = 1 << 2;
    pub const EJECTED: u8 = 1 << 3;
}

/// The drive status's bits [R ps2tek].
pub mod drive {
    pub const TRAY_OPEN: u8 = 1 << 0;
    pub const SPINNING: u8 = 1 << 1;
    pub const READING: u8 = 1 << 2;
    pub const PAUSED: u8 = 1 << 3;
    pub const SEEKING: u8 = 1 << 4;
}

/// The disk type register's values [R ps2tek].
pub mod disk_type {
    pub const NONE: u8 = 0x00;
    pub const DETECTING: u8 = 0x01;
    pub const DETECTING_CD: u8 = 0x02;
    pub const DETECTING_DVD: u8 = 0x03;
    pub const DETECTING_DUAL: u8 = 0x04;
    pub const PS2_CD: u8 = 0x12;
    pub const PS2_CDDA: u8 = 0x13;
    pub const PS2_DVD: u8 = 0x14;
    pub const CDDA: u8 = 0xFD;
}

/// The N commands [R ps2tek "CDVD N Commands"; READ_CDDA's number M PCSX2, a CD-only read, D-126].
pub mod ncmd {
    pub const NOP: u8 = 0x00;
    pub const NOP_SYNC: u8 = 0x01;
    pub const STANDBY: u8 = 0x02;
    pub const STOP: u8 = 0x03;
    pub const PAUSE: u8 = 0x04;
    pub const SEEK: u8 = 0x05;
    pub const READ_CD: u8 = 0x06;
    pub const READ_CDDA: u8 = 0x07;
    pub const READ_DVD: u8 = 0x08;
    pub const GET_TOC: u8 = 0x09;
}

/// The S commands [R ps2tek "CDVD S Commands"; TRAY's number, its parameter and WRITE_CONFIG's M PCSX2, §25.3].
pub mod scmd {
    pub const SUBCOMMAND: u8 = 0x03;
    pub const MECHACON_VERSION: u8 = 0x00;
    pub const UPDATE_STICKY: u8 = 0x05;
    pub const TRAY: u8 = 0x06;
    pub const READ_RTC: u8 = 0x08;
    pub const WRITE_RTC: u8 = 0x09;
    pub const FORBID_DVD: u8 = 0x15;
    pub const OPEN_CONFIG: u8 = 0x40;
    pub const READ_CONFIG: u8 = 0x41;
    pub const WRITE_CONFIG: u8 = 0x42;
    pub const CLOSE_CONFIG: u8 = 0x43;
}

/// MechaconVersion's four bytes, chosen by D-124 from the referees' answer, not from ps2tek's quotation.
pub const VERSION: [u8; 4] = [0x03, 0x06, 0x02, 0x00];
/// The one byte an S command the drive does not implement answers, and an unknown subcommand (D-125).
pub const ABSENT: u8 = 0x80;
pub const ABSENT_SUBCOMMAND: u8 = 0x81;
/// ForbidDVD's answer (D-125).
pub const FORBID_DVD_ANSWER: u8 = 0x05;
/// The error register after a read of the wrong medium (D-126), and with no disc ready [C].
pub const WRONG_MEDIUM: u8 = 0x14;
pub const NO_DISC: u8 = 0x01;

/// How a read's blocks are cut from a sector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Form {
    /// ReadCd's block sizes [R ps2tek: "1=2328 bytes, 2=2340 bytes, all others=2048 bytes"].
    Cd(usize),
    CdDa,
    Dvd,
    Toc,
}

impl Form {
    fn size(self) -> usize {
        match self {
            Form::Cd(n) => n,
            Form::CdDa => 2352,
            Form::Dvd | Form::Toc => FRAME,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Read {
    lba: u32,
    /// Sectors not yet handed to the buffer.
    left: u32,
    form: Form,
    block: u64,
    /// The next sector is read and waits for the buffer to be taken.
    waiting: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Event {
    /// The N command ends, leaving the drive status given.
    Complete(u8),
    /// The read's next sector is off the disc.
    Sector,
}

/// A small fixed queue of bytes, the parameter and result FIFOs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Fifo {
    bytes: [u8; 16],
    len: usize,
    pos: usize,
}

impl Fifo {
    fn push(&mut self, v: u8) {
        if self.len < 16 {
            self.bytes[self.len] = v;
            self.len += 1;
        }
    }

    fn pop(&mut self) -> Option<u8> {
        (self.pos < self.len).then(|| {
            self.pos += 1;
            self.bytes[self.pos - 1]
        })
    }

    /// The parameters pushed, zeros past them, and the queue emptied.
    fn take(&mut self) -> [u8; 16] {
        std::mem::take(self).bytes
    }

    fn set(&mut self, bytes: &[u8]) {
        *self = Fifo::default();
        bytes.iter().for_each(|&b| self.push(b));
    }
}

pub struct Cdvd {
    disc: Option<Disc>,
    tray_open: bool,
    n_command: u8,
    n_params: Fifo,
    error: u8,
    istat: u8,
    drive: u8,
    sticky: u8,
    disk_type: u8,
    s_command: u8,
    s_params: Fifo,
    s_result: Fifo,
    busy: bool,
    position: u32,
    spinning: bool,
    pending: Option<(u64, Event)>,
    /// The disc's detection after the tray closes: when its next stage comes.
    detect: Option<u64>,
    read: Option<Read>,
    buffer: Vec<u8>,
    taken: usize,
    pub rtc: Rtc,
    pub nvm: Nvm,
}

/// The disk type a disc settles at: a CD with audio tracks beside its data is the CDDA kind, one of audio alone music.
fn kind_of(disc: &mut Disc) -> u8 {
    match disc {
        Disc::Dvd(_) => disk_type::PS2_DVD,
        Disc::Cd(d) => match (d.tracks().iter().any(|t| t.audio), d.tracks().iter().all(|t| t.audio)) {
            (_, true) => disk_type::CDDA,
            (true, false) => disk_type::PS2_CDDA,
            (false, false) => disk_type::PS2_CD,
        },
    }
}

fn detecting(disc: &Disc) -> u8 {
    match disc {
        Disc::Cd(_) => disk_type::DETECTING_CD,
        Disc::Dvd(d) if d.layers() == 2 => disk_type::DETECTING_DUAL,
        Disc::Dvd(_) => disk_type::DETECTING_DVD,
    }
}

impl Cdvd {
    /// A drive at power-on, its tray closed: a disc in it spun up and paused at sector 0, as both referees report
    /// the drive status 0Ah with one inserted (§25.3).
    pub fn new(disc: Option<Disc>) -> Cdvd {
        let mut c = Cdvd {
            disc: None,
            tray_open: false,
            n_command: 0,
            n_params: Fifo::default(),
            error: 0,
            istat: 0,
            drive: 0,
            sticky: 0,
            disk_type: disk_type::NONE,
            s_command: 0,
            s_params: Fifo::default(),
            s_result: Fifo::default(),
            busy: false,
            position: 0,
            spinning: false,
            pending: None,
            detect: None,
            read: None,
            buffer: Vec::with_capacity(2352),
            taken: 0,
            rtc: Rtc::default(),
            nvm: Nvm::default(),
        };
        if let Some(d) = disc {
            c.load(d);
            c.disk_type = kind_of(c.disc.as_mut().unwrap());
            c.spinning = true;
            c.set_drive(drive::SPINNING | drive::PAUSED);
        }
        c
    }

    /// A disc is numbered, as the drive reports a DVD's sectors, from (030000) on each layer (D-121).
    fn load(&mut self, mut d: Disc) {
        if let Disc::Dvd(v) = &mut d {
            v.set_track_path(TrackPath::Parallel);
        }
        self.disc = Some(d);
    }

    pub fn disc(&self) -> Option<&Disc> {
        self.disc.as_ref()
    }

    /// The host's disc change: only with the tray open, as a player's hand would.
    pub fn insert(&mut self, disc: Option<Disc>) -> bool {
        if !self.tray_open {
            return false;
        }
        self.disc = None;
        if let Some(d) = disc {
            self.load(d);
        }
        true
    }

    fn set_drive(&mut self, v: u8) {
        self.drive = v;
        self.sticky |= v;
    }

    /// The interrupt line to the IOP's INTC (IRQ2): any I_STAT bit set.
    pub fn irq(&self) -> bool {
        self.istat != 0
    }

    pub fn read8(&mut self, offset: u32) -> u8 {
        match offset {
            reg::N_COMMAND => self.n_command,
            reg::N_STATUS => (if self.busy { 0x80 } else { 0x40 }) | (self.error != 0) as u8,
            reg::ERROR => self.error,
            reg::I_STAT => self.istat,
            reg::DRIVE => self.drive,
            reg::STICKY => self.sticky,
            reg::DISK_TYPE => self.disk_type,
            reg::S_COMMAND => self.s_command,
            reg::S_STATUS => if self.s_result.pos < self.s_result.len { 0x00 } else { 0x40 },
            reg::S_RESULT => self.s_result.pop().unwrap_or(0),
            _ => 0,
        }
    }

    pub fn write8(&mut self, offset: u32, v: u8, now: u64) {
        match offset {
            reg::N_COMMAND if !self.busy => self.start(v, now),
            reg::N_STATUS => self.n_params.push(v),
            reg::BREAK => self.brk(),
            reg::I_STAT => self.istat &= !v,
            reg::S_COMMAND => self.scmd(v, now),
            reg::S_STATUS => self.s_params.push(v),
            _ => {}
        }
    }

    /// Whether a disc is in, the tray closed and its detection done.
    fn ready(&self) -> bool {
        self.disc.is_some() && !self.tray_open && self.detect.is_none()
    }

    fn fail(&mut self, error: u8, now: u64) {
        self.error = error;
        self.pending = Some((now + PROMPT, Event::Complete(self.drive)));
    }

    /// The seek's end from where the drive is to `to`, spinning a stopped drive up first.
    fn seek_end(&mut self, to: u32, now: u64, block: u64) -> u64 {
        let (dvd, brk) = match &self.disc {
            Some(Disc::Dvd(d)) => (true, d.layer_break()),
            _ => (false, None),
        };
        let spin = if self.spinning { 0 } else { SPIN_UP };
        self.spinning = true;
        now + spin + timing::seek(self.position, to, dvd, brk, block)
    }

    fn start(&mut self, cmd: u8, now: u64) {
        let p = self.n_params.take();
        let word = |at: usize| u32::from_le_bytes([p[at], p[at + 1], p[at + 2], p[at + 3]]);
        self.n_command = cmd;
        self.error = 0;
        self.busy = true;
        let dvd = matches!(self.disc, Some(Disc::Dvd(_)));
        let speed = if dvd { DVD_SPEED } else { CD_SPEED };
        let disc_command = matches!(cmd, ncmd::STANDBY | ncmd::SEEK | ncmd::READ_CD | ncmd::READ_CDDA | ncmd::READ_DVD | ncmd::GET_TOC);
        if disc_command && !self.ready() {
            return self.fail(NO_DISC, now);
        }
        match cmd {
            ncmd::STANDBY | ncmd::SEEK => {
                let to = if cmd == ncmd::SEEK { word(0) } else { 0 };
                let end = self.seek_end(to, now, block(2048, speed));
                self.position = to;
                self.set_drive(drive::SPINNING | drive::SEEKING);
                self.pending = Some((end.max(now + PROMPT), Event::Complete(drive::SPINNING | drive::PAUSED)));
            }
            ncmd::STOP => {
                self.position = 0;
                self.pending = Some((now + STOP, Event::Complete(0)));
            }
            ncmd::PAUSE => {
                let after = if self.spinning { drive::SPINNING | drive::PAUSED } else { self.drive };
                self.pending = Some((now + PROMPT, Event::Complete(after)));
            }
            ncmd::READ_CD | ncmd::READ_CDDA | ncmd::READ_DVD | ncmd::GET_TOC => {
                let form = match cmd {
                    ncmd::READ_CD => Form::Cd(match p[10] {
                        1 => 2328,
                        2 => 2340,
                        _ => 2048,
                    }),
                    ncmd::READ_CDDA => Form::CdDa,
                    ncmd::READ_DVD => Form::Dvd,
                    _ => Form::Toc,
                };
                if (form == Form::Dvd && !dvd) || (form == Form::CdDa && dvd) {
                    return self.fail(WRONG_MEDIUM, now);
                }
                let b = block(form.size(), speed);
                let (lba, count, at) = if form == Form::Toc { (0, 1, now) } else { (word(0), word(4), self.seek_end(word(0), now, b)) };
                self.buffer.clear();
                self.taken = 0;
                if count == 0 {
                    self.position = lba;
                    self.pending = Some((at.max(now + PROMPT), Event::Complete(drive::SPINNING | drive::PAUSED)));
                    return;
                }
                self.read = Some(Read { lba, left: count, form, block: b, waiting: false });
                self.set_drive(drive::SPINNING | if form == Form::Toc { drive::READING } else { drive::SEEKING });
                self.pending = Some((at + b, Event::Sector));
            }
            _ => self.pending = Some((now + PROMPT, Event::Complete(self.drive))),
        }
    }

    fn brk(&mut self) {
        if !self.busy {
            return;
        }
        self.pending = None;
        self.read = None;
        self.buffer.clear();
        self.busy = false;
        self.istat |= istat::COMPLETE;
        let after = if self.spinning { drive::SPINNING | drive::PAUSED } else { self.drive };
        self.set_drive(after);
    }

    pub fn next_event(&self) -> u64 {
        let p = self.pending.map_or(u64::MAX, |(t, _)| t);
        p.min(self.detect.unwrap_or(u64::MAX))
    }

    /// Brings the drive to `now`, taking every event due by then.
    pub fn run(&mut self, now: u64) {
        while self.next_event() <= now {
            if self.detect.is_some_and(|t| t <= now) {
                self.detected(self.detect.unwrap());
                continue;
            }
            let (at, event) = self.pending.take().unwrap();
            match event {
                Event::Complete(after) => self.complete(after),
                Event::Sector => self.sector(at),
            }
        }
    }

    fn complete(&mut self, after: u8) {
        self.busy = false;
        self.istat |= istat::COMPLETE;
        if after & drive::SPINNING == 0 && after & drive::TRAY_OPEN == 0 {
            self.spinning = false;
        }
        self.set_drive(after);
    }

    fn sector(&mut self, at: u64) {
        let Some(mut r) = self.read else { return };
        if self.taken < self.buffer.len() {
            r.waiting = true;
            self.read = Some(r);
            return;
        }
        self.deliver(&mut r, at);
    }

    /// Hands the read's next sector to the buffer at `at` and schedules the one after.
    fn deliver(&mut self, r: &mut Read, at: u64) {
        self.fill(r.form, r.lba);
        r.lba += 1;
        r.left -= 1;
        r.waiting = false;
        self.set_drive(drive::SPINNING | drive::READING);
        if r.left > 0 {
            self.pending = Some((at + r.block, Event::Sector));
        }
        self.position = r.lba;
        self.read = Some(*r);
    }

    /// The buffer's bytes for sector `lba` in `form`.
    fn fill(&mut self, form: Form, lba: u32) {
        self.buffer.clear();
        self.taken = 0;
        let Some(disc) = self.disc.as_mut() else { return };
        match (form, disc) {
            (Form::Toc, d) => self.buffer.extend_from_slice(&toc(d)),
            (Form::Dvd, Disc::Dvd(d)) => {
                let mut f = [0u8; FRAME];
                d.read(lba, &mut f);
                self.buffer.extend_from_slice(&f);
            }
            (Form::Cd(n), Disc::Dvd(d)) => {
                let mut u = [0u8; 2048];
                d.read_user(lba, &mut u);
                self.buffer.extend_from_slice(&u);
                self.buffer.resize(n, 0);
            }
            (form, Disc::Cd(d)) => {
                let mut raw = [0u8; 2352];
                d.read(lba + emusen_disc::msf::PROGRAM_START, &mut raw);
                let data = if raw[15] == 1 { 16 } else { 24 };
                let bytes = match form {
                    Form::Cd(2328) => &raw[24..],
                    Form::Cd(2340) => &raw[12..],
                    Form::Cd(_) => &raw[data..data + 2048],
                    _ => &raw[..],
                };
                self.buffer.extend_from_slice(bytes);
            }
            (_, Disc::Dvd(_)) => {}
        }
    }

    /// Bytes waiting for DMA channel 3.
    pub fn dma_ready(&self) -> usize {
        self.buffer.len() - self.taken
    }

    /// DMA channel 3 takes up to `out.len()` bytes of the buffered sector at `now`; when the buffer empties the next
    /// sector already read is handed over, and the read ends when its last sector has been taken.
    pub fn dma_take(&mut self, now: u64, out: &mut [u8]) -> usize {
        let n = out.len().min(self.dma_ready());
        out[..n].copy_from_slice(&self.buffer[self.taken..self.taken + n]);
        self.taken += n;
        if n > 0 && self.taken == self.buffer.len() {
            self.buffer.clear();
            self.taken = 0;
            if let Some(mut r) = self.read {
                if r.waiting {
                    self.deliver(&mut r, now);
                } else if r.left == 0 {
                    self.read = None;
                    self.pending = None;
                    self.istat |= istat::DATA_READY;
                    self.complete(drive::SPINNING | drive::PAUSED);
                }
            }
        }
        n
    }

    fn scmd(&mut self, cmd: u8, now: u64) {
        let p = self.s_params.take();
        self.s_command = cmd;
        let mut block = [0u8; BLOCK];
        let answer: &[u8] = match cmd {
            scmd::SUBCOMMAND if p[0] == scmd::MECHACON_VERSION => &VERSION,
            scmd::SUBCOMMAND => &[ABSENT_SUBCOMMAND],
            scmd::UPDATE_STICKY => {
                self.sticky = self.drive;
                &[0]
            }
            scmd::TRAY => match (p[0], self.tray_open) {
                (0, false) => {
                    self.tray(true, now);
                    &[0]
                }
                (1, true) => {
                    self.tray(false, now);
                    &[0]
                }
                _ => &[ABSENT],
            },
            scmd::READ_RTC => {
                block[..8].copy_from_slice(&self.rtc.read(now));
                &block[..8]
            }
            scmd::WRITE_RTC => {
                self.rtc.write(p[..8].try_into().unwrap(), now);
                &[0]
            }
            scmd::FORBID_DVD => &[FORBID_DVD_ANSWER],
            scmd::OPEN_CONFIG => {
                self.nvm.open([p[0], p[1], p[2]]);
                &[0]
            }
            scmd::READ_CONFIG => {
                block = self.nvm.read();
                &block
            }
            scmd::WRITE_CONFIG => {
                self.nvm.write(p.as_slice().try_into().unwrap());
                &[0]
            }
            scmd::CLOSE_CONFIG => {
                self.nvm.close();
                &[0]
            }
            _ => &[ABSENT],
        };
        let answer = answer.to_vec();
        self.s_result.set(&answer);
    }

    /// The tray opened or closed, by an S command or the player. Opening ends any N command and stops the disc;
    /// closing over a disc detects it while it spins up: DETECTING for the first half, the kind's detecting value for
    /// the second, then its type (§25.4).
    pub fn tray(&mut self, open: bool, now: u64) {
        if open == self.tray_open {
            return;
        }
        self.tray_open = open;
        self.spinning = false;
        self.detect = None;
        if open {
            if self.busy {
                self.brk();
            }
            self.istat |= istat::EJECTED;
            self.disk_type = disk_type::NONE;
            self.set_drive(drive::TRAY_OPEN);
            return;
        }
        self.set_drive(0);
        self.position = 0;
        if self.disc.is_some() {
            self.disk_type = disk_type::DETECTING;
            self.detect = Some(now + SPIN_UP / 2);
        }
    }

    fn detected(&mut self, at: u64) {
        let Some(disc) = self.disc.as_mut() else {
            self.detect = None;
            return;
        };
        if self.disk_type == disk_type::DETECTING {
            self.disk_type = detecting(disc);
            self.detect = Some(at + SPIN_UP / 2);
        } else {
            self.disk_type = kind_of(disc);
            self.detect = None;
            self.spinning = true;
            self.set_drive(drive::SPINNING | drive::PAUSED);
        }
    }
}

/// GetToc's 2,064-byte block: a DVD's physical format information (ECMA-267 §26.5.1) in its first 2,048 bytes; a CD's
/// lead-in items A0h, A1h, A2h and each track's, ten bytes of Q each from byte 0 [C, §25.4].
fn toc(disc: &mut Disc) -> [u8; FRAME] {
    let mut b = [0u8; FRAME];
    match disc {
        Disc::Dvd(d) => {
            let (dual, last) = (d.layers() == 2, d.sectors() - 1);
            b[0] = 0x01;
            b[1] = 0x02;
            b[2] = 0x01 | if dual { 0x10 | 0x20 } else { 0 };
            b[3] = if dual { 0x10 } else { 0 };
            b[5..8].copy_from_slice(&ecma267::DATA_ZONE.to_be_bytes()[1..]);
            let end = ecma267::physical(last, d.layer_break(), TrackPath::Opposite).1;
            b[9..12].copy_from_slice(&end.to_be_bytes()[1..]);
            if let Some(brk) = d.layer_break() {
                b[13..16].copy_from_slice(&(ecma267::DATA_ZONE + brk - 1).to_be_bytes()[1..]);
            }
        }
        Disc::Cd(d) => {
            let pointers: Vec<u8> = [0xA0, 0xA1, 0xA2].into_iter().chain(d.tracks().iter().map(|t| emusen_disc::subq::pointer(t.number))).collect();
            for (i, p) in pointers.into_iter().enumerate().take(FRAME / 10) {
                if let Some(q) = d.lead_in(p) {
                    b[i * 10..i * 10 + 10].copy_from_slice(&q.0[..10]);
                }
            }
        }
    }
    b
}

#[cfg(test)]
mod tests;
