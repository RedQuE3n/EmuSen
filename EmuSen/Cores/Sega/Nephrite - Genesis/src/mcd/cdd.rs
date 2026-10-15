//! The CDD, the drive's own microcontroller, as the sub CPU meets it through `$FF8036`-`$FF804B`: once every 1/75 s
//! it sends ten nibbles of status and takes ten of command, and the gate array raises level 4. The Hardware Manual
//! blacks out this protocol (§3-6), so its rules are those of mcd-verificator's sources, krikzz's notes and the drive's
//! exchanges recorded in mcd-verificator's `cd-log/LL-mcd-tst/` (Nephrite_Native.md §51, D-33).

use emusen_disc::{Msf, Place};
use emusen_native::core::{Kept, SEEK_SPAN, seek};

/// The disc in the drive, its bytes the host's where the host lends them (`BORROW`).
pub(crate) type Disc = emusen_disc::Disc<Kept>;

/// Whether the drive names its places to the host (`MEDIA_SEEK`), which core ABI 1.6 lets the core claim.
pub(in crate::mcd) const NAMES_PLACES: bool = true;

/// The drive's number in machine info's `drives`.
pub(in crate::mcd) const DRIVE: u32 = 0;

/// Sub clocks a sector, 1/75 s: 500,000/3, made of two of 166,667 and one of 166,666.
pub(in crate::mcd) const SECTOR: [u64; 3] = [166_667, 166_667, 166_666];

/// Exchanges from power-on until the drive answers, busy till then and deaf to commands, and from the first report
/// asked for after it until the TOC is read (the LL logs: 61 to 79 busy exchanges after a host's first, the TOC 13 to
/// 35; chosen within those, D-33 and D-36).
pub(in crate::mcd) const SPIN_UP: u32 = 75;
pub(in crate::mcd) const TOC_READ: u32 = 20;
/// A seek's exchanges before the head reports a place, and the sectors it lands short of its target (the LL logs: 12,
/// and 14 to 15).
pub(in crate::mcd) const SEEK_BUSY: u32 = 12;
pub(in crate::mcd) const LAND_SHORT: u32 = 14;
/// A play reports playing, and a seek pauses, from three sectors short of the target; a pause loops over the eight
/// sectors up to there (krikzz's notes: "seek pos 250, loop range 240-247").
pub(in crate::mcd) const SETTLE: u32 = 3;
pub(in crate::mcd) const LOOP: u32 = 7;

/// The status's first nibble.
pub(in crate::mcd) mod status {
    pub(in crate::mcd) const STOPPED: u8 = 0x0;
    pub(in crate::mcd) const PLAYING: u8 = 0x1;
    pub(in crate::mcd) const SEEKING: u8 = 0x2;
    pub(in crate::mcd) const SCANNING: u8 = 0x3;
    pub(in crate::mcd) const PAUSED: u8 = 0x4;
    pub(in crate::mcd) const TRAY_OPEN: u8 = 0x5;
    pub(in crate::mcd) const CHECKSUM: u8 = 0x6;
    pub(in crate::mcd) const READY: u8 = 0x9;
    pub(in crate::mcd) const NO_DISC: u8 = 0xB;
    pub(in crate::mcd) const LEAD_OUT: u8 = 0xC;
}

/// The second nibble when the status carries no report: busy, or nothing to say.
const NONE: u8 = 0xF;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::mcd) enum Mode {
    /// The host clock has not yet risen: the drive waits.
    Idle,
    Spinning(u32),
    ReadingToc(u32),
    /// Stopped: `READY` after the TOC was read, `STOPPED` after a stop.
    Stopped(u8),
    /// A seek: its target, whether it plays on, the exchanges left before the head reports, then the coarse
    /// travel's exchanges left and total, from where.
    Seeking { target: u32, play: bool, busy: u32, left: u32, total: u32, from: u32 },
    /// Reading up to the target sector by sector; `status` is what it reports meanwhile.
    Approach { target: u32, play: bool, status: u8 },
    Playing,
    /// Looping over `low..=high`, an exchange with no report between laps.
    Paused { low: u32, high: u32, gap: bool },
    /// Fast forward or back.
    Scanning { forward: bool },
    TrayOpen,
}

/// What one exchange did: the sector read, if any, and whether the status went to the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::mcd) struct Tick {
    pub(in crate::mcd) sector: Option<u32>,
    pub(in crate::mcd) int4: bool,
}

pub struct Cdd {
    pub(in crate::mcd) disc: Option<Disc>,
    pub(in crate::mcd) mode: Mode,
    /// HOCK, the host clock: set, the drive and the host exchange status and command each sector.
    pub hock: bool,
    pub status: [u8; 10],
    pub command: [u8; 10],
    /// A command written and not yet taken; the sub clock DTS falls at.
    pub(in crate::mcd) command_new: bool,
    pub(in crate::mcd) dts_until: u64,
    /// The report the status carries (0 absolute time, 1 relative, 2 track, 3 the disc's length, 4 its tracks, 5 a
    /// track's start), and the track that 5 asks for.
    pub report: u8,
    pub(in crate::mcd) report_track: u8,
    pub head: u32,
    /// The status's flags nibble: bit 2 the place last reported is in a data track, bit 0 the drive is not playing.
    pub(in crate::mcd) data: bool,
    /// Exchanges since the drive spun up, for the lead-in's cycle of TOC items.
    pub(in crate::mcd) laps: u64,
    /// The sub clock of the next exchange and which of the three periods it ends.
    pub(in crate::mcd) next: u64,
    pub(in crate::mcd) phase: u8,
    /// Whether the TOC has been read since the disc spun up: the first report asked for reads it, and status 9 waits on it.
    pub(in crate::mcd) toc_read: bool,
    /// The file and byte last named to the host (`MEDIA_SEEK`); not state.
    pub(in crate::mcd) named: Option<Place>,
}

impl Cdd {
    /// A drive at power-on: with a disc it spins up at once, whether or not the host clocks it (D-36).
    pub(in crate::mcd) fn new(disc: Option<Disc>, now: u64) -> Cdd {
        let mode = if disc.is_some() { Mode::Spinning(SPIN_UP) } else { Mode::Idle };
        Cdd { disc, mode, hock: false, status: [0; 10], command: [0; 10], command_new: false, dts_until: 0, report: 0, report_track: 1, toc_read: false, head: 0, data: false, laps: 0, next: now + SECTOR[0], phase: 0, named: None }
    }

    fn checksum(n: &[u8]) -> u8 {
        !n[..9].iter().fold(0u8, |a, &b| a.wrapping_add(b & 15)) & 15
    }

    /// The host clock's write: its first rise starts the drive, which answers once its disc has spun up.
    pub(in crate::mcd) fn set_hock(&mut self, hock: bool) {
        if hock && self.mode == Mode::Idle {
            self.mode = if self.disc.is_some() { Mode::Spinning(SPIN_UP) } else { Mode::Stopped(status::NO_DISC) };
        }
        self.hock = hock;
    }

    /// The tray opened, the drive stopping; or closed, the disc's TOC read again, or no disc found.
    pub(in crate::mcd) fn set_tray(&mut self, open: bool) {
        if open {
            self.mode = Mode::TrayOpen;
        } else if self.mode == Mode::TrayOpen {
            self.mode = if self.disc.is_some() { Mode::Spinning(SPIN_UP) } else { Mode::Stopped(status::NO_DISC) };
        }
    }

    fn lead_out(&self) -> u32 {
        self.disc.as_ref().map_or(0, |d| d.lead_out())
    }

    fn data_at(&self, at: u32) -> bool {
        self.disc.as_ref().and_then(|d| d.track_at(at)).is_some_and(|t| !t.audio)
    }

    /// Tells the host where the head is going, unless that lies in the megabyte after the place last named
    /// (Nephrite_Native.md §53.2, §56).
    fn name(&mut self, at: u32) {
        if !NAMES_PLACES {
            return;
        }
        let Some(place) = self.disc.as_ref().and_then(|d| d.place(at)) else { return };
        if self.named.is_some_and(|n| n.file == place.file && place.offset >= n.offset && place.offset - n.offset < SEEK_SPAN) {
            return;
        }
        self.named = Some(place);
        seek(DRIVE, place.file.map(|f| f as u32), place.offset);
    }

    fn seek_to(&mut self, target: u32, play: bool) {
        self.name(target);
        let from = self.head;
        self.mode = Mode::Seeking { target, play, busy: SEEK_BUSY, left: 0, total: 0, from };
    }

    /// The command taken at an exchange: its checksum checked, then acted on.
    fn take(&mut self) -> Option<u8> {
        if !std::mem::take(&mut self.command_new) {
            return None;
        }
        let c = self.command;
        if Cdd::checksum(&c) != c[9] & 15 {
            return Some(status::CHECKSUM);
        }
        let ready = !matches!(self.mode, Mode::Idle | Mode::Spinning(_) | Mode::ReadingToc(_) | Mode::TrayOpen) && self.disc.is_some();
        let msf = || Msf::from_bcd([c[2] << 4 | c[3] & 15, (c[4] & 15) << 4 | c[5] & 15, (c[6] & 15) << 4 | c[7] & 15]).map(Msf::sector);
        match c[0] & 15 {
            0x1 if ready => self.mode = Mode::Stopped(status::STOPPED),
            0x2 => {
                if !self.toc_read && self.mode == Mode::Stopped(status::STOPPED) {
                    self.mode = Mode::ReadingToc(TOC_READ);
                } else if self.mode == Mode::Stopped(status::STOPPED) {
                    // Stopped with its TOC read, a TOC request finds it ready again (krikzz's "9 only after toc cmd"; D-40).
                    self.mode = Mode::Stopped(status::READY);
                }
                self.report = c[3] & 15;
                if self.report == 5 {
                    self.report_track = (c[4] & 15) * 10 + (c[5] & 15);
                    if ready {
                        self.mode = Mode::Stopped(status::READY);
                    }
                }
            }
            // A play or a seek during the TOC's read is taken, the read cut short (mcd-verificator's CDC INIT, D-36).
            0x3 | 0x4 if ready || matches!(self.mode, Mode::ReadingToc(_)) => {
                if let Some(at) = msf() {
                    self.toc_read = true;
                    self.report = 0;
                    self.seek_to(at, c[0] & 15 == 3);
                }
            }
            0x6 if ready => {
                let high = self.head + 2;
                match self.mode {
                    Mode::Playing | Mode::Approach { .. } | Mode::Scanning { .. } => self.mode = Mode::Paused { low: high.saturating_sub(LOOP), high, gap: false },
                    // Stopped, a pause is a seek to where the head is, ending paused there (D-39).
                    Mode::Stopped(_) => self.seek_to(self.head, false),
                    _ => {}
                }
            }
            0x7 if ready => match self.mode {
                Mode::Paused { low, high, .. } => {
                    self.head = low;
                    self.mode = Mode::Approach { target: high + SETTLE - 1, play: true, status: status::PAUSED };
                }
                Mode::Stopped(_) => self.seek_to(emusen_disc::msf::PROGRAM_START, true),
                Mode::Scanning { .. } => self.mode = Mode::Playing,
                _ => {}
            },
            0x8 | 0x9 if ready => self.mode = Mode::Scanning { forward: c[0] & 15 == 8 },
            0xC => {
                if self.mode == Mode::TrayOpen {
                    self.mode = if self.disc.is_some() { Mode::Spinning(SPIN_UP) } else { Mode::Stopped(status::NO_DISC) };
                }
            }
            0xD => self.mode = Mode::TrayOpen,
            _ => {}
        }
        None
    }

    /// One exchange at its sub clock: the command taken, the head moved a sector, the status composed.
    pub(in crate::mcd) fn exchange(&mut self) -> Tick {
        let refused = self.take();
        self.laps += 1;
        let mut sector = None;
        let mut report = true;
        let s = match self.mode {
            Mode::Idle => {
                return Tick { sector: None, int4: false };
            }
            Mode::Spinning(n) => {
                if n <= 1 {
                    (self.mode, self.toc_read) = (Mode::Stopped(status::STOPPED), false);
                } else {
                    self.mode = Mode::Spinning(n - 1);
                }
                report = false;
                status::STOPPED
            }
            Mode::ReadingToc(n) => {
                if n <= 1 {
                    (self.mode, self.toc_read, self.laps) = (Mode::Stopped(status::READY), true, 0);
                } else {
                    self.mode = Mode::ReadingToc(n - 1);
                }
                report = false;
                status::STOPPED
            }
            Mode::Stopped(s) => {
                report = self.report >= 3 && s != status::NO_DISC;
                s
            }
            Mode::Seeking { target, play, busy, left, total, from } => {
                let land = target.saturating_sub(LAND_SHORT);
                if busy > 0 {
                    let m = if busy == 1 {
                        let n = ((from.abs_diff(land) as f64).sqrt() * 0.9).round() as u32;
                        if n == 0 {
                            self.head = land;
                            Mode::Approach { target, play, status: status::SEEKING }
                        } else {
                            Mode::Seeking { target, play, busy: 0, left: n, total: n, from }
                        }
                    } else {
                        Mode::Seeking { target, play, busy: busy - 1, left, total, from }
                    };
                    self.mode = m;
                    report = false;
                } else {
                    let done = total - left + 1;
                    self.head = (from as i64 + (land as i64 - from as i64) * done as i64 / total as i64) as u32;
                    self.mode = if left <= 1 { Mode::Approach { target, play, status: status::SEEKING } } else { Mode::Seeking { target, play, busy, left: left - 1, total, from } };
                }
                status::SEEKING
            }
            Mode::Approach { target, play, status: st } => {
                sector = Some(self.head);
                let s = if self.head + SETTLE >= target {
                    if play {
                        self.mode = Mode::Playing;
                        status::PLAYING
                    } else {
                        let high = self.head;
                        self.mode = Mode::Paused { low: high.saturating_sub(LOOP), high, gap: true };
                        status::PAUSED
                    }
                } else {
                    st
                };
                self.head += 1;
                s
            }
            Mode::Playing => {
                sector = Some(self.head);
                self.head += 1;
                if self.head > self.lead_out() { status::LEAD_OUT } else { status::PLAYING }
            }
            Mode::Paused { low, high, gap } => {
                if gap && self.head > high {
                    self.head = low;
                    self.mode = Mode::Paused { low, high, gap: false };
                    report = false;
                } else {
                    sector = Some(self.head);
                    self.head += 1;
                    if self.head > high {
                        self.mode = Mode::Paused { low, high, gap: true };
                    }
                }
                status::PAUSED
            }
            Mode::Scanning { forward } => {
                self.head = if forward { self.head + 12 } else { self.head.saturating_sub(12) };
                status::SCANNING
            }
            Mode::TrayOpen => {
                report = false;
                status::TRAY_OPEN
            }
        };
        if let Some(at) = sector {
            self.data = self.data_at(at);
            self.name(at);
        } else if matches!(self.mode, Mode::Seeking { busy: 0, .. }) {
            self.data = self.data_at(self.head);
        }
        let playing = matches!(self.mode, Mode::Playing) && s == status::PLAYING;
        let flags = (self.data as u8) << 2 | !playing as u8;
        let mut n = [0u8; 10];
        n[0] = refused.unwrap_or(s);
        n[1] = NONE;
        n[8] = flags;
        // Spun up and its TOC not yet read, the drive is stopped and not busy: its report nibble 0, its place zero, and
        // its flags 0 (D-36).
        let idle = refused.is_none() && self.mode == Mode::Stopped(status::STOPPED) && !self.toc_read && self.disc.is_some();
        if idle {
            (n[1], n[8]) = (0, 0);
        }
        if refused.is_some() {
            n[2..8].copy_from_slice(&self.status[2..8]);
        } else if report && let Some(r) = self.report_nibbles(s) {
            n[1] = self.report;
            n[2..8].copy_from_slice(&r);
            // A track's start carries the track's number, its last BCD digit, where the flags go (krikzz's notes; D-33).
            if self.report == 5 {
                n[8] = emusen_disc::msf::bcd(self.report_track) & 15;
            }
        }
        n[9] = Cdd::checksum(&n);
        self.status = n;
        Tick { sector, int4: self.hock }
    }

    /// The six nibbles of the report asked for, or None where there is none to give now.
    fn report_nibbles(&self, s: u8) -> Option<[u8; 6]> {
        let d = self.disc.as_ref()?;
        let six = |b: [u8; 3]| [b[0] >> 4, b[0] & 15, b[1] >> 4, b[1] & 15, b[2] >> 4, b[2] & 15];
        let stopped = matches!(s, status::STOPPED | status::READY);
        match self.report {
            0..=2 if stopped => None,
            0 => Some(six([d.subq(self.head).0[7], d.subq(self.head).0[8], d.subq(self.head).0[9]])),
            1 => {
                let q = d.subq(self.head).0;
                Some(six([q[3], q[4], q[5]]))
            }
            2 => {
                let q = d.subq(self.head).0;
                let data = q[0] >> 4 & 4 != 0;
                Some([q[1] >> 4, q[1] & 15, 0, (q[2] != 0) as u8, if data { 4 } else { 0 }, 0])
            }
            3 => Some(six(Msf::of(d.lead_out()).bcd())),
            4 => {
                let toc = d.toc();
                let (f, l) = (emusen_disc::msf::bcd(toc.first), emusen_disc::msf::bcd(toc.last));
                Some([f >> 4, f & 15, l >> 4, l & 15, 0, 0])
            }
            5 => {
                // The lead-in repeats each track's item three times in turn, with the three of A0h-A2h: the
                // answer comes as the head passes the track's (the LL logs' `toc-commands`).
                let toc = d.toc();
                let items = toc.tracks.len() as u64 + 3;
                let k = (self.laps / 3 % items) as usize;
                let &(n, control, start) = toc.tracks.get(k)?;
                if n != self.report_track {
                    return None;
                }
                let mut r = six(Msf::of(start).bcd());
                if control & 4 != 0 {
                    r[4] |= 8;
                }
                Some(r)
            }
            _ => None,
        }
    }

    // ---- the registers

    /// `$FF8036`-`$FF804B` as the sub CPU reads them, by the word at `off` from `$FF8036`.
    pub(in crate::mcd) fn read(&self, off: u32, now: u64) -> u16 {
        match off {
            // D/M: 0 while music plays, 1 for data, a stop or a pause (§3-6).
            0x00 => (!(self.mode == Mode::Playing && !self.data) as u16) << 8 | (self.hock as u16) << 2 | (now < self.dts_until) as u16,
            0x02..=0x0A => {
                let i = (off as usize - 2) & !1;
                (self.status[i] as u16) << 8 | self.status[i + 1] as u16
            }
            _ => {
                let i = (off as usize - 0x0C) & !1;
                (self.command[i] as u16) << 8 | self.command[i + 1] as u16
            }
        }
    }

    /// A write: HOCK at `$FF8037`, the command's nibbles from `$FF8042`; the last, `$FF804B`, sends it.
    pub(in crate::mcd) fn write(&mut self, off: u32, v: u16, mask: u16, now: u64) {
        match off {
            0x00 => {
                if mask & 0x00FF != 0 {
                    self.set_hock(v & 4 != 0);
                }
            }
            0x0C..=0x14 => {
                let i = (off as usize - 0x0C) & !1;
                if mask & 0xFF00 != 0 {
                    self.command[i] = (v >> 8) as u8 & 15;
                }
                if mask & 0x00FF != 0 {
                    self.command[i + 1] = v as u8 & 15;
                    if i + 1 == 9 {
                        self.command_new = true;
                        self.dts_until = now + DTS;
                    }
                }
            }
            _ => {}
        }
    }
}

/// DTS stays set while the ten nibbles go to the drive: about 200 µs (argued; the manual's 240 µs abort bound).
pub(in crate::mcd) const DTS: u64 = 2_500;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A disc of one mode 1 data track of `data` sectors and an audio track of `audio`, kept as a host's copy.
    pub(in crate::mcd) fn disc(data: u32, audio: u32) -> Disc {
        let user: Vec<u8> = (0..data * 2048).map(|i| (i / 2048) as u8).collect();
        // Audio sector i's frame k is (i, -k).
        let pcm: Vec<u8> = (0..audio as usize * 588).flat_map(|n| [((n / 588) as i16).to_le_bytes(), (-((n % 588) as i16)).to_le_bytes()].concat()).collect();
        let sheet = emusen_disc::cue::Sheet::parse("FILE \"d.iso\" BINARY\nTRACK 01 MODE1/2048\nINDEX 01 00:00:00\nFILE \"a.bin\" BINARY\nTRACK 02 AUDIO\nINDEX 01 00:00:00\n").unwrap();
        Disc::from_sheet(&sheet, vec![Kept::from(user), Kept::from(pcm)], emusen_disc::IsoForm::Mode1).unwrap()
    }

    fn send(c: &mut Cdd, nibbles: [u8; 9]) {
        let mut n = [0u8; 10];
        n[..9].copy_from_slice(&nibbles);
        n[9] = Cdd::checksum(&n);
        for k in 0..5 {
            c.write(0x0C + 2 * k as u32, (n[2 * k] as u16) << 8 | n[2 * k + 1] as u16, 0xFFFF, 0);
        }
    }

    fn line(c: &Cdd) -> String {
        let s = c.status;
        format!("{:X}{:X}.{:X}{:X}{:X}{:X}{:X}{:X}.{:X}{:X}", s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7], s[8], s[9])
    }

    #[test]
    fn the_checksum_is_the_nibbles_sum_inverted() {
        // `20.040000.09`, the logs' TOC request, and `94.013500.18`, its answer.
        assert_eq!(Cdd::checksum(&[2, 0, 0, 4, 0, 0, 0, 0, 0, 0]), 9);
        assert_eq!(Cdd::checksum(&[9, 4, 0, 1, 3, 5, 0, 0, 1, 0]), 8);
    }

    /// The drive spun up and its TOC read, as a host's first report asks for it.
    fn ready(c: &mut Cdd) {
        for _ in 0..SPIN_UP {
            c.exchange();
        }
        send(c, [2, 0, 0, 4, 0, 0, 0, 0, 0]);
        for _ in 0..TOC_READ + 2 {
            c.exchange();
        }
    }

    #[test]
    fn the_drive_spins_up_reads_its_toc_and_answers() {
        let mut c = Cdd::new(Some(disc(300, 75)), 0);
        c.set_hock(true);
        c.exchange();
        assert_eq!(line(&c), "0F.000000.1F", "spinning, busy");
        for _ in 0..SPIN_UP {
            c.exchange();
        }
        assert_eq!(line(&c), "00.000000.0F", "spun up, stopped, not busy");
        send(&mut c, [2, 0, 0, 4, 0, 0, 0, 0, 0]);
        c.exchange();
        assert_eq!(&line(&c)[..2], "0F", "reading the TOC");
        for _ in 0..TOC_READ {
            c.exchange();
        }
        assert_eq!(line(&c), "94.010200.1E", "tracks 1 to 2, answered once the TOC is read");
        send(&mut c, [2, 0, 0, 3, 0, 0, 0, 0, 0]);
        c.exchange();
        // The lead-out: 150 + 300 + 75 = 525 = 00:07:00.
        assert_eq!(&line(&c)[..9], "93.000700");
        send(&mut c, [2, 0, 0, 0, 0, 0, 0, 0, 0]);
        c.exchange();
        assert_eq!(&line(&c)[..2], "9F", "no position while stopped");
        // Track 2's start, when the lead-in's cycle passes it: its number where the flags were (krikzz's notes).
        send(&mut c, [2, 0, 0, 5, 0, 2, 0, 0, 0]);
        let answer = (0..40).find_map(|_| {
            c.exchange();
            (c.status[1] == 5).then(|| line(&c))
        });
        assert_eq!(answer.as_deref().map(|l| &l[..2]), Some("95"));
        assert_eq!(answer.unwrap().as_bytes()[10], b'2');
    }

    #[test]
    fn the_drive_spins_up_from_power_on_and_idles_with_its_flags_clear() {
        // Unclocked, the drive spins up all the same; a host's first clock then finds it idle (D-36).
        let mut c = Cdd::new(Some(disc(300, 75)), 0);
        for _ in 0..SPIN_UP {
            assert!(!c.exchange().int4);
        }
        c.set_hock(true);
        assert!(c.exchange().int4);
        assert_eq!(line(&c), "00.000000.0F");
        let mut empty = Cdd::new(None, 0);
        assert_eq!(empty.mode, Mode::Idle);
        empty.set_hock(true);
        empty.exchange();
        assert_eq!(empty.status[0], status::NO_DISC);
    }

    #[test]
    fn a_play_during_the_tocs_read_is_taken() {
        let mut c = Cdd::new(Some(disc(3000, 75)), 0);
        c.set_hock(true);
        for _ in 0..SPIN_UP {
            c.exchange();
        }
        send(&mut c, [2, 0, 0, 4, 0, 0, 0, 0, 0]);
        c.exchange();
        assert!(matches!(c.mode, Mode::ReadingToc(_)));
        send(&mut c, [3, 0, 0, 0, 0, 2, 0, 0, 0]);
        c.exchange();
        assert!(matches!(c.mode, Mode::Seeking { play: true, .. }), "{:?}", c.mode);
    }

    #[test]
    fn a_play_seeks_lands_short_and_reads_on() {
        let mut c = Cdd::new(Some(disc(3000, 75)), 0);
        c.set_hock(true);
        ready(&mut c);
        // Play from 00:20:00 = sector 1500.
        send(&mut c, [3, 0, 0, 0, 2, 0, 0, 0, 0]);
        let mut seen = Vec::new();
        for _ in 0..200 {
            let t = c.exchange();
            seen.push((c.status[0], t.sector));
        }
        assert!(seen[..SEEK_BUSY as usize].iter().all(|&(s, at)| s == status::SEEKING && at.is_none()), "busy first");
        let first_read = seen.iter().find_map(|&(_, at)| at).unwrap();
        assert_eq!(first_read, 1500 - LAND_SHORT);
        let playing = seen.iter().position(|&(s, _)| s == status::PLAYING).unwrap();
        assert_eq!(seen[playing].1, Some(1500 - SETTLE));
        assert!(seen[playing..].windows(2).all(|w| w[1].1 == w[0].1.map(|a| a + 1)), "sector by sector");
    }

    #[test]
    fn a_seek_pauses_and_loops_short_of_its_target() {
        let mut c = Cdd::new(Some(disc(3000, 75)), 0);
        c.set_hock(true);
        ready(&mut c);
        send(&mut c, [4, 0, 0, 0, 0, 2, 5, 0, 0]);
        let reads: Vec<Option<u32>> = (0..120).map(|_| c.exchange().sector).collect();
        let target = Msf::new(0, 2, 50).sector();
        let max = reads.iter().flatten().max().unwrap();
        assert_eq!(*max, target - SETTLE, "never past three short");
        let tail: Vec<u32> = reads[100..].iter().flatten().copied().collect();
        assert!(tail.iter().all(|&a| (target - SETTLE - LOOP..=target - SETTLE).contains(&a)), "looping: {tail:?}");
    }

    #[test]
    fn a_pause_when_stopped_seeks_to_the_head_and_pauses_there() {
        let mut c = Cdd::new(Some(disc(3000, 75)), 0);
        c.set_hock(true);
        ready(&mut c);
        c.head = 600;
        send(&mut c, [6, 0, 0, 0, 0, 0, 0, 0, 0]);
        let statuses: Vec<u8> = (0..60).map(|_| {
            c.exchange();
            c.status[0]
        }).collect();
        assert_eq!(statuses[0], status::SEEKING);
        assert_eq!(*statuses.last().unwrap(), status::PAUSED, "{statuses:?}");
    }

    #[test]
    fn a_toc_request_after_a_stop_finds_the_drive_ready() {
        let mut c = Cdd::new(Some(disc(300, 75)), 0);
        c.set_hock(true);
        ready(&mut c);
        send(&mut c, [1, 0, 0, 0, 0, 0, 0, 0, 0]);
        c.exchange();
        assert_eq!(c.status[0], status::STOPPED);
        send(&mut c, [2, 0, 0, 4, 0, 0, 0, 0, 0]);
        c.exchange();
        assert_eq!(&line(&c)[..9], "94.010200");
    }

    #[test]
    fn a_bad_checksum_is_answered_and_ignored() {
        let mut c = Cdd::new(Some(disc(300, 75)), 0);
        c.set_hock(true);
        ready(&mut c);
        for k in 0..5 {
            c.write(0x0C + 2 * k, if k == 0 { 0x0300 } else { 0 }, 0xFFFF, 0);
        }
        c.exchange();
        assert_eq!(c.status[0], status::CHECKSUM);
        c.exchange();
        assert_eq!(c.status[0], status::READY, "the command was not taken");
    }
}
