//! The CDD, the drive's own microcontroller, as the sub CPU meets it through `$FF8036`-`$FF804B`: once every 1/75 s
//! it sends ten nibbles of status and takes ten of command, and the gate array raises level 4. The Hardware Manual
//! blacks out this protocol (§3-6), so its rules are those of mcd-verificator's sources, krikzz's notes and the drive's
//! exchanges recorded in mcd-verificator's `cd-log/LL-mcd-tst/` (Nephrite_Native.md §51, D-33).

use emusen_disc::{Msf, Place};
use emusen_native::core::{Kept, SEEK_SPAN, seek};

/// The disc in the drive, its bytes the host's where the host lends them (`BORROW`).
pub type Disc = emusen_disc::Disc<Kept>;

/// The drive's number in machine info's `drives`.
pub const DRIVE: u32 = 0;

/// Sub clocks a sector, 1/75 s: 500,000/3, made of two of 166,667 and one of 166,666.
pub const SECTOR: [u64; 3] = [166_667, 166_667, 166_666];

/// Exchanges from the host clock's first rise until the disc has spun up, and then until its TOC is read
/// (the LL logs: 53 to 150 between them; chosen within that).
pub const SPIN_UP: u32 = 60;
pub const TOC_READ: u32 = 20;
/// A seek's exchanges before the head reports a place, and the sectors it lands short of its target (the LL logs: 12,
/// and 14 to 15).
pub const SEEK_BUSY: u32 = 12;
pub const LAND_SHORT: u32 = 14;
/// A play reports playing, and a seek pauses, from three sectors short of the target; a pause loops over the eight
/// sectors up to there (krikzz's notes: "seek pos 250, loop range 240-247").
pub const SETTLE: u32 = 3;
pub const LOOP: u32 = 7;

/// The status's first nibble.
pub mod status {
    pub const STOPPED: u8 = 0x0;
    pub const PLAYING: u8 = 0x1;
    pub const SEEKING: u8 = 0x2;
    pub const SCANNING: u8 = 0x3;
    pub const PAUSED: u8 = 0x4;
    pub const TRAY_OPEN: u8 = 0x5;
    pub const CHECKSUM: u8 = 0x6;
    pub const READY: u8 = 0x9;
    pub const NO_DISC: u8 = 0xB;
    pub const LEAD_OUT: u8 = 0xC;
}

/// The second nibble when the status carries no report: busy, or nothing to say.
const NONE: u8 = 0xF;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
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
pub struct Tick {
    pub sector: Option<u32>,
    pub int4: bool,
}

pub struct Cdd {
    pub disc: Option<Disc>,
    pub mode: Mode,
    /// HOCK, the host clock: set, the drive and the host exchange status and command each sector.
    pub hock: bool,
    pub status: [u8; 10],
    pub command: [u8; 10],
    /// A command written and not yet taken; the sub clock DTS falls at.
    pub command_new: bool,
    pub dts_until: u64,
    /// The report the status carries (0 absolute time, 1 relative, 2 track, 3 the disc's length, 4 its tracks, 5 a
    /// track's start), and the track that 5 asks for.
    pub report: u8,
    pub report_track: u8,
    pub head: u32,
    /// The status's flags nibble: bit 2 the place last reported is in a data track, bit 0 the drive is not playing.
    pub data: bool,
    /// Exchanges since the drive spun up, for the lead-in's cycle of TOC items.
    pub laps: u64,
    /// The sub clock of the next exchange and which of the three periods it ends.
    pub next: u64,
    pub phase: u8,
    /// The file and byte last named to the host (`MEDIA_SEEK`); not state.
    pub named: Option<Place>,
}

impl Cdd {
    pub fn new(disc: Option<Disc>, now: u64) -> Cdd {
        Cdd { disc, mode: Mode::Idle, hock: false, status: [0; 10], command: [0; 10], command_new: false, dts_until: 0, report: 0, report_track: 1, head: 0, data: false, laps: 0, next: now + SECTOR[0], phase: 0, named: None }
    }

    fn checksum(n: &[u8]) -> u8 {
        !n[..9].iter().fold(0u8, |a, &b| a.wrapping_add(b & 15)) & 15
    }

    /// The host clock's write: its first rise starts the spindle.
    pub fn set_hock(&mut self, hock: bool) {
        if hock && self.mode == Mode::Idle {
            self.mode = if self.disc.is_some() { Mode::Spinning(SPIN_UP) } else { Mode::Stopped(status::NO_DISC) };
        }
        self.hock = hock;
    }

    fn lead_out(&self) -> u32 {
        self.disc.as_ref().map_or(0, |d| d.lead_out())
    }

    fn data_at(&self, at: u32) -> bool {
        self.disc.as_ref().and_then(|d| d.track_at(at)).is_some_and(|t| !t.audio)
    }

    /// Tells the host where the head is going, unless that lies in the megabyte after the place last named.
    fn name(&mut self, at: u32) {
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
                self.report = c[3] & 15;
                if self.report == 5 {
                    self.report_track = (c[4] & 15) * 10 + (c[5] & 15);
                    if ready {
                        self.mode = Mode::Stopped(status::READY);
                    }
                }
            }
            0x3 | 0x4 if ready => {
                if let Some(at) = msf() {
                    self.report = 0;
                    self.seek_to(at, c[0] & 15 == 3);
                }
            }
            0x6 if ready => {
                let high = self.head + 2;
                self.mode = match self.mode {
                    Mode::Playing | Mode::Approach { .. } | Mode::Scanning { .. } => Mode::Paused { low: high.saturating_sub(LOOP), high, gap: false },
                    m => m,
                };
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
    pub fn exchange(&mut self) -> Tick {
        let refused = self.take();
        self.laps += 1;
        let mut sector = None;
        let mut report = true;
        let s = match self.mode {
            Mode::Idle => {
                return Tick { sector: None, int4: false };
            }
            Mode::Spinning(n) => {
                self.mode = if n <= 1 { Mode::ReadingToc(TOC_READ) } else { Mode::Spinning(n - 1) };
                report = false;
                status::STOPPED
            }
            Mode::ReadingToc(n) => {
                if n <= 1 {
                    self.mode = Mode::Stopped(status::READY);
                    self.laps = 0;
                } else {
                    self.mode = Mode::ReadingToc(n - 1);
                }
                report = false;
                status::READY
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
        if refused.is_some() {
            n[2..8].copy_from_slice(&self.status[2..8]);
        } else if report && let Some(r) = self.report_nibbles(s) {
            n[1] = self.report;
            n[2..8].copy_from_slice(&r);
        }
        n[8] = flags;
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
    pub fn read(&self, off: u32, now: u64) -> u16 {
        match off {
            0x00 => (self.data as u16) << 8 | (self.hock as u16) << 2 | (now < self.dts_until) as u16,
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
    pub fn write(&mut self, off: u32, v: u16, mask: u16, now: u64) {
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
pub const DTS: u64 = 2_500;

#[cfg(test)]
mod tests {
    use super::*;

    /// A disc of one mode 1 data track of `data` sectors and an audio track of `audio`, kept as a host's copy.
    pub fn disc(data: u32, audio: u32) -> Disc {
        let user: Vec<u8> = (0..data * 2048).map(|i| (i / 2048) as u8).collect();
        let pcm = vec![0u8; audio as usize * 2352];
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

    #[test]
    fn the_drive_spins_up_reads_its_toc_and_answers() {
        let mut c = Cdd::new(Some(disc(300, 75)), 0);
        c.set_hock(true);
        c.exchange();
        assert_eq!(line(&c), "0F.000000.1F", "spinning");
        send(&mut c, [2, 0, 0, 4, 0, 0, 0, 0, 0]);
        for _ in 0..SPIN_UP + TOC_READ {
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
    }

    #[test]
    fn a_play_seeks_lands_short_and_reads_on() {
        let mut c = Cdd::new(Some(disc(3000, 75)), 0);
        c.set_hock(true);
        for _ in 0..SPIN_UP + TOC_READ + 1 {
            c.exchange();
        }
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
        for _ in 0..SPIN_UP + TOC_READ + 1 {
            c.exchange();
        }
        send(&mut c, [4, 0, 0, 0, 0, 2, 5, 0, 0]);
        let reads: Vec<Option<u32>> = (0..120).map(|_| c.exchange().sector).collect();
        let target = Msf::new(0, 2, 50).sector();
        let max = reads.iter().flatten().max().unwrap();
        assert_eq!(*max, target - SETTLE, "never past three short");
        let tail: Vec<u32> = reads[100..].iter().flatten().copied().collect();
        assert!(tail.iter().all(|&a| (target - SETTLE - LOOP..=target - SETTLE).contains(&a)), "looping: {tail:?}");
    }

    #[test]
    fn a_bad_checksum_is_answered_and_ignored() {
        let mut c = Cdd::new(Some(disc(300, 75)), 0);
        c.set_hock(true);
        for _ in 0..SPIN_UP + TOC_READ + 1 {
            c.exchange();
        }
        for k in 0..5 {
            c.write(0x0C + 2 * k, if k == 0 { 0x0300 } else { 0 }, 0xFFFF, 0);
        }
        c.exchange();
        assert_eq!(c.status[0], status::CHECKSUM);
        c.exchange();
        assert_eq!(c.status[0], status::READY, "the command was not taken");
    }
}
