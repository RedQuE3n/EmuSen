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

/// The disc an image and its companions make, by the PlayStation 2's rules (§25.2): CD-XA framing for 2,048-byte
/// sectors read raw, and an image of 2,048-byte sectors taken for a DVD, as the library's DVD titles are kept, unless
/// its sheet or its bytes say a CD. The host's bytes are kept, never copied (Sapphire_Plan.md §5.11).
pub fn open(image: Kept, companions: Vec<Kept>) -> Result<Disc, emusen_disc::Refusal> {
    emusen_disc::open_medium(image, companions, emusen_disc::IsoForm::Mode2Form1, emusen_disc::Lone::Detect, emusen_disc::Kind::Dvd)
}

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
    /// ReadKey's five key bytes, 20h to 24h (D-461).
    pub const KEY: u32 = 0x20;
    /// The sixth key byte, given by a ReadKey whose fourth parameter is 4Bh (D-461).
    pub const KEY_EXTRA: u32 = 0x34;
    /// The disc's authentication after ReadKey: 05h after one whose fourth parameter is 4Bh, 01h after any other, 00h
    /// before (D-461).
    pub const KEY_STATUS: u32 = 0x38;
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
    /// ReadKey, which the player's BIOS sends before it boots a disc, its result at register 38h [R the export
    /// `sceCdReadKey`, Israelson's ps2tek "CDVDMAN"; the number M PCSX2's drive and the BIOS's own traffic, D-461].
    pub const READ_KEY: u8 = 0x0C;
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
    /// The console's region parameters [M PCSX2, Sapphire_Disputes.md D-348].
    pub const REGION: u8 = 0x36;
    /// Two the player's BIOS sends at boot, whose purpose no document gives [M PCSX2, D-348].
    pub const BOOT_1A: u8 = 0x1A;
    pub const BOOT_22: u8 = 0x22;
    pub const BOOT_17: u8 = 0x17;
    pub const BOOT_1E: u8 = 0x1E;
    /// Those below 40h that PCSX2 answers with the one byte 00h, the player's BIOS sending 24h at boot and retrying it on
    /// 80h; no document gives their purpose [M PCSX2, D-348].
    pub const ACKNOWLEDGED: [u8; 12] = [0x0B, 0x13, 0x14, 0x16, 0x18, 0x1B, 0x1C, 0x24, 0x29, 0x31, 0x38, 0x3E];
}

/// S command 36h's fifteen bytes for each market, measured on PCSX2 2.8.2 with the gates' images (2026-10-25): a
/// zero, 08h, a zero, the market's letter twice, the language, two letters and five zeros; no document describes them.
pub const REGION_JAPAN: [u8; 15] = *b"\x00\x08\x00JJjpnJJ\x00\x00\x00\x00\x00";
pub const REGION_AMERICAS: [u8; 15] = *b"\x00\x08\x00AAengAU\x00\x00\x00\x00\x00";
pub const REGION_EUROPE: [u8; 15] = *b"\x00\x08\x00EEengEE\x00\x00\x00\x00\x00";

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
    /// Register 38h, ReadKey's result: every disc authenticated (Sapphire_Plan.md §5.6, D-461).
    key: u8,
    /// The disc's five key bytes, from its boot program's name, read at 20h-24h after ReadKey: derived from the disc,
    /// which the state names, so never written in it.
    key_bytes: [u8; 6],
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
    /// A reason was set in I_STAT since the bus last looked, which raises the IOP's IRQ 2 (D-481); taken at once, so
    /// never part of a state.
    raised: bool,
    pub rtc: Rtc,
    pub nvm: Nvm,
    /// The console's answer to S command 36h, set by the machine from its market; absent (80h) without one.
    pub region: Option<[u8; 15]>,
    /// The S and N commands received, counted by number: the drive's own report of what it was asked (S24).
    pub s_count: [u32; 256],
    pub n_count: [u32; 256],
    /// Each S command's last parameters and their count, for the same report.
    pub s_last: Box<[([u8; 16], usize); 256]>,
    /// Each N command's last parameters and their count, the same report for the N commands (S28).
    pub n_last: Box<[([u8; 16], usize); 256]>,
    /// When set, every register access in order: the offset, whether written, the value; the drive's own report (S28).
    pub trace: Option<Vec<(u32, bool, u8)>>,
}

/// Register 38h after a ReadKey whose fourth parameter is 4Bh, which also gives the sixth byte at 34h (D-461).
const KEY_FULL: u8 = 0x05;

/// ReadKey's six bytes (20h-24h, then 34h) for a boot program named in the PlayStation 2's form, four letters, an
/// underscore, three digits, a full stop and two (`SCUS_971.24`): bit fields of the letters' codes and of the five
/// digits' number, measured on PCSX2 over eleven names (D-461); for a name of any other form zeros and 04h, as measured.
pub fn read_key(program: &str) -> [u8; 6] {
    let cut = program.rfind('\\').max(program.rfind(':')).map_or(0, |i| i + 1);
    let name = program[cut..].split(';').next().unwrap_or("").as_bytes();
    let form = name.len() == 11 && name[..4].iter().all(u8::is_ascii_alphabetic) && name[4] == b'_' && name[8] == b'.';
    let digits: Vec<u8> = name.get(5..8).into_iter().chain(name.get(9..11)).flatten().copied().collect();
    if !form || !digits.iter().all(u8::is_ascii_digit) {
        return [0, 0, 0, 0, 0, 0x04];
    }
    let l = &name[..4];
    let n = digits.iter().fold(0u32, |v, d| v * 10 + u32::from(d - b'0'));
    [
        (l[3] & 1) << 7 | (n >> 10) as u8 & 0x7F,
        (l[2] & 3) << 6 | l[3] >> 1,
        (l[1] & 7) << 5 | (l[2] >> 2) & 0x1F,
        (l[0] & 0xF) << 4 | (l[1] >> 3) & 0xF,
        ((n & 0x1F) as u8) << 3 | (l[0] >> 4) & 7,
        ((n >> 5 & 0x1F) as u8) << 3 | 0x04,
    ]
}

/// Sector `lba`'s 2,048 bytes of user data, empty past the disc's end.
fn user_sector(disc: &mut Disc, lba: u64) -> Vec<u8> {
    let Ok(lba) = u32::try_from(lba) else { return Vec::new() };
    match disc {
        Disc::Dvd(d) if lba < d.sectors() => {
            let mut u = [0u8; 2048];
            d.read_user(lba, &mut u);
            u.to_vec()
        }
        Disc::Cd(d) => {
            let mut raw = [0u8; 2352];
            d.read(lba + emusen_disc::msf::PROGRAM_START, &mut raw);
            let data = if raw[15] == 1 { 16 } else { 24 };
            raw[data..data + 2048].to_vec()
        }
        _ => Vec::new(),
    }
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
            key: 0,
            key_bytes: [0; 6],
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
            raised: false,
            rtc: Rtc::default(),
            nvm: Nvm::default(),
            region: None,
            s_count: [0; 256],
            n_count: [0; 256],
            s_last: Box::new([([0; 16], 0); 256]),
            n_last: Box::new([([0; 16], 0); 256]),
            trace: None,
        };
        if let Some(d) = disc {
            c.in_at_power_on(d);
        }
        c
    }

    /// A disc that was in the drive when the console was switched on: spun up and paused at sector 0, the tray never
    /// having moved, so no eject is reported (Sapphire_Native.md §39.2).
    pub fn in_at_power_on(&mut self, d: Disc) {
        self.load(d);
        self.disk_type = kind_of(self.disc.as_mut().unwrap());
        self.spinning = true;
        self.set_drive(drive::SPINNING | drive::PAUSED);
    }

    /// A disc is numbered, as the drive reports a DVD's sectors, from (030000) on each layer (D-121).
    fn load(&mut self, mut d: Disc) {
        if let Disc::Dvd(v) = &mut d {
            v.set_track_path(TrackPath::Parallel);
        }
        self.key_bytes = read_key(&crate::media::boot2_by(&mut |n| user_sector(&mut d, n)).unwrap_or_default());
        self.disc = Some(d);
    }

    pub fn disc(&self) -> Option<&Disc> {
        self.disc.as_ref()
    }

    pub fn tray_open(&self) -> bool {
        self.tray_open
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

    /// A reason set in I_STAT, which raises IRQ 2 whatever reasons are still pending (D-481).
    fn reason(&mut self, bit: u8) {
        self.istat |= bit;
        self.raised = true;
    }

    /// Whether a reason was set since the last call.
    pub fn take_raised(&mut self) -> bool {
        std::mem::take(&mut self.raised)
    }

    pub fn read8(&mut self, offset: u32) -> u8 {
        let v = self.register(offset);
        if let Some(t) = self.trace.as_mut().filter(|t| t.len() < 1 << 20) {
            t.push((offset, false, v));
        }
        v
    }

    fn register(&mut self, offset: u32) -> u8 {
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
            reg::KEY_STATUS => self.key,
            o if (reg::KEY..reg::KEY + 5).contains(&o) && self.key != 0 => self.key_bytes[(o - reg::KEY) as usize],
            reg::KEY_EXTRA if self.key == KEY_FULL => self.key_bytes[5],
            _ => 0,
        }
    }

    pub fn write8(&mut self, offset: u32, v: u8, now: u64) {
        if let Some(t) = self.trace.as_mut().filter(|t| t.len() < 1 << 20) {
            t.push((offset, true, v));
        }
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
        self.n_count[cmd as usize] += 1;
        let given = self.n_params.len;
        let p = self.n_params.take();
        self.n_last[cmd as usize] = (p, given);
        let word = |at: usize| u32::from_le_bytes([p[at], p[at + 1], p[at + 2], p[at + 3]]);
        self.n_command = cmd;
        self.error = 0;
        self.busy = true;
        let dvd = matches!(self.disc, Some(Disc::Dvd(_)));
        let speed = if dvd { DVD_SPEED } else { CD_SPEED };
        let disc_command = matches!(cmd, ncmd::STANDBY | ncmd::SEEK | ncmd::READ_CD | ncmd::READ_CDDA | ncmd::READ_DVD | ncmd::GET_TOC | ncmd::READ_KEY);
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
            ncmd::READ_KEY => {
                self.key = if p[3] == 0x4B { KEY_FULL } else { 1 };
                self.pending = Some((now + PROMPT, Event::Complete(self.drive)));
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
        self.reason(istat::COMPLETE);
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
            if let Some(t) = self.detect.filter(|&t| self.pending.is_none_or(|(p, _)| t <= p)) {
                self.detected(t);
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
        self.reason(istat::COMPLETE);
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
                    self.reason(istat::DATA_READY);
                    self.complete(drive::SPINNING | drive::PAUSED);
                }
            }
        }
        n
    }

    fn scmd(&mut self, cmd: u8, now: u64) {
        self.s_count[cmd as usize] += 1;
        let given = self.s_params.len;
        let p = self.s_params.take();
        self.s_last[cmd as usize] = (p, given);
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
            c if scmd::ACKNOWLEDGED.contains(&c) => &[0],
            scmd::BOOT_1A => &[1],
            scmd::BOOT_22 => &[0; 10],
            scmd::BOOT_17 => &[0; 9],
            scmd::BOOT_1E => &[0x00, 0x14, 0x00, 0x00, 0x00],
            scmd::REGION => match &self.region {
                Some(r) => {
                    block[..15].copy_from_slice(r);
                    &block[..15]
                }
                None => &[ABSENT],
            },
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
        self.key = 0;
        if open {
            if self.busy {
                self.brk();
            }
            self.reason(istat::EJECTED);
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

/// The registers' window on the IOP's bus, byte registers from `BASE` [R ps2tek "CDVD I/O Ports"].
pub const END: u32 = BASE + 0x40;

/// The IOP DMAC's peripherals with the drive on channel 3, which takes the buffered sector four bytes a word, the
/// others going to `others` (Sapphire_Native.md §35.3).
pub struct Ports<'a, P: crate::iop::dma::Peripherals> {
    pub others: &'a mut P,
    pub cdvd: &'a mut Cdvd,
    /// The IOP cycle of the DMAC's step.
    pub now: u64,
}

impl<P: crate::iop::dma::Peripherals> crate::iop::dma::Peripherals for Ports<'_, P> {
    fn request(&mut self, ch: usize, want: u32, to_memory: bool) -> u32 {
        match ch {
            crate::iop::dma::CDVD if to_memory => want.min((self.cdvd.dma_ready() / 4) as u32),
            crate::iop::dma::CDVD => 0,
            _ => self.others.request(ch, want, to_memory),
        }
    }

    fn put(&mut self, ch: usize, w: u32) {
        if ch != crate::iop::dma::CDVD {
            self.others.put(ch, w);
        }
    }

    fn get(&mut self, ch: usize) -> u32 {
        if ch != crate::iop::dma::CDVD {
            return self.others.get(ch);
        }
        let mut b = [0u8; 4];
        self.cdvd.dma_take(self.now, &mut b);
        u32::from_le_bytes(b)
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

/// The sector buffer's room in the state: the largest block a read cuts, CD-DA's 2,352 bytes.
const BUFFER_SLOT: usize = 2352;

fn form_code(f: Form) -> (u8, u32) {
    match f {
        Form::Cd(n) => (0, n as u32),
        Form::CdDa => (1, 0),
        Form::Dvd => (2, 0),
        Form::Toc => (3, 0),
    }
}

fn form_of(code: u8, n: u32) -> Form {
    match code {
        1 => Form::CdDa,
        2 => Form::Dvd,
        3 => Form::Toc,
        _ => Form::Cd((n as usize).min(BUFFER_SLOT)),
    }
}

/// The drive's block of `SPH2` (Sapphire_Native.md §38.2): the tray, the N and S commands with their FIFOs, the status
/// registers, the head's position and the spindle, the command ending and the disc's detection, the read in progress,
/// the sector buffer, the clock and the EEPROM's session. The disc is named by the state's header, never held in it;
/// the region answer is the console's market; the command counts are the drive's reports.
impl emusen_native::State for Cdvd {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        let fifos = [&self.n_params, &self.s_params, &self.s_result];
        w.bytes("FifoBytes", [fifos[0].bytes, fifos[1].bytes, fifos[2].bytes].as_flattened());
        w.u32s("FifoAt", fifos.map(|f| [f.len as u32, f.pos as u32]).as_flattened());
        w.bytes("Registers", &[self.n_command, self.error, self.istat, self.drive, self.sticky, self.disk_type, self.s_command, self.key]);
        w.bools("Flags", &[self.tray_open, self.busy, self.spinning]);
        w.u32("Position", self.position);
        let (at, event) = self.pending.unwrap_or((0, Event::Sector));
        w.bool("Pending", self.pending.is_some());
        w.u64("PendingAt", at);
        w.u8("PendingEvent", match event {
            Event::Complete(s) => s,
            Event::Sector => 0xFF,
        });
        w.bool("PendingSector", matches!(event, Event::Sector));
        crate::state::opt_u64(w, "Detect", self.detect);
        let r = self.read.unwrap_or(Read { lba: 0, left: 0, form: Form::Dvd, block: 0, waiting: false });
        let (form, size) = form_code(r.form);
        w.bools("Read", &[self.read.is_some(), r.waiting]);
        w.u32s("ReadWords", &[r.lba, r.left, size]);
        w.u8("ReadForm", form);
        w.u64("ReadBlock", r.block);
        let mut buffer = [0u8; BUFFER_SLOT];
        buffer[..self.buffer.len().min(BUFFER_SLOT)].copy_from_slice(&self.buffer[..self.buffer.len().min(BUFFER_SLOT)]);
        w.u32s("BufferAt", &[self.buffer.len() as u32, self.taken as u32]);
        w.bytes("Buffer", &buffer);
        w.u64s("Clock", &self.rtc.saved());
        let (flags, session) = self.nvm.session_saved();
        w.bools("Eeprom", &flags);
        w.u32s("EepromSession", &session);
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        let mut bytes = [0u8; 48];
        r.bytes(&mut bytes)?;
        let mut at = [0u32; 6];
        r.u32s(&mut at)?;
        for (k, f) in [&mut self.n_params, &mut self.s_params, &mut self.s_result].into_iter().enumerate() {
            f.bytes.copy_from_slice(&bytes[16 * k..16 * k + 16]);
            f.len = (at[2 * k] as usize).min(16);
            f.pos = (at[2 * k + 1] as usize).min(f.len);
        }
        let mut g = [0u8; 8];
        r.bytes(&mut g)?;
        [self.n_command, self.error, self.istat, self.drive, self.sticky, self.disk_type, self.s_command, self.key] = g;
        let mut f = [false; 3];
        r.bools(&mut f)?;
        [self.tray_open, self.busy, self.spinning] = f;
        self.position = r.u32()?;
        let pending = r.bool()?;
        let at = r.u64()?;
        let status = r.u8()?;
        let sector = r.bool()?;
        self.pending = pending.then_some((at, if sector { Event::Sector } else { Event::Complete(status) }));
        self.detect = crate::state::read_opt_u64(r)?;
        let mut rf = [false; 2];
        r.bools(&mut rf)?;
        let mut rw = [0u32; 3];
        r.u32s(&mut rw)?;
        let form = form_of(r.u8()?, rw[2]);
        let block = r.u64()?;
        self.read = rf[0].then_some(Read { lba: rw[0], left: rw[1], form, block, waiting: rf[1] });
        let mut b = [0u32; 2];
        r.u32s(&mut b)?;
        let mut buffer = [0u8; BUFFER_SLOT];
        r.bytes(&mut buffer)?;
        self.buffer.clear();
        self.buffer.extend_from_slice(&buffer[..(b[0] as usize).min(BUFFER_SLOT)]);
        self.taken = (b[1] as usize).min(self.buffer.len());
        let mut c = [0u64; 2];
        r.u64s(&mut c)?;
        self.rtc = rtc::Rtc::restored(c);
        let mut e = [false; 2];
        r.bools(&mut e)?;
        let mut s = [0u32; 2];
        r.u32s(&mut s)?;
        self.nvm.restore_session(e, s);
        Ok(())
    }
}

impl Cdvd {
    /// Whether the sector buffer is within its slot in the state.
    pub fn fits(&self) -> bool {
        self.buffer.len() <= BUFFER_SLOT
    }
}

#[cfg(test)]
mod tests;
