//! The drive beneath the controller: the spindle, seeks timed by distance, the head stepping back while paused (psx-spx
//! "SeekL", "SeekP"), the sector each period brings at 75 or 150 a second, and what becomes of it: an INT1 delivery,
//! XA-ADPCM to the decoder, CD-DA to the output, a play report, the end of a track (Diamond_CDROM.md §5).

use super::commands::{LEAD_OUT_READABLE, OUTER_LIMIT};
use super::xa::Coding;
use super::{Activity, Buffered, Cdrom, DiscSlot, Reply, SECTOR_DOUBLE, SECTOR_SINGLE, Then, mode, stat};
use emusen_disc::ecma130::submode;
use emusen_disc::Sector;
use emusen_disc::msf::MINUTE;

/// Where an idle drive holds after spin-up or Init: just before 00:02:00, as the console's getloc log finds it.
pub const PARKED: u32 = 150;
/// Spinning up and reading the TOC, a seek's fixed part and its part over the whole disc, fitted to the timing log's
/// first sector after a seek back of 100 sectors [C, D-25].
pub const SPIN_UP: u64 = 33_868_800 * 3 / 4;
pub const SEEK_BASE: u64 = 400_000;
pub const SEEK_SPAN: u64 = 33_868_800 * 300 / 1000;
/// A seek ahead by fewer sectors than this follows the spiral instead of jumping [C, D-25].
pub const READ_AHEAD: u32 = 32;
/// The spindle's stop after the lid opens [C, D-25].
pub const SPIN_DOWN: u64 = 33_868_800 / 2;

impl Cdrom {
    pub(super) fn period(&self) -> u64 {
        if self.mode & mode::DOUBLE != 0 { SECTOR_DOUBLE } else { SECTOR_SINGLE }
    }

    /// A seek's time from `from` to `to`: following the spiral a sector a period when just ahead, else a jump of a
    /// fixed time and one growing with the square root of the distance.
    pub(super) fn seek_time(&self, from: u32, to: u32) -> u64 {
        let period = self.period();
        if to >= from && to - from < READ_AHEAD {
            return period * (to - from).max(1) as u64;
        }
        let d = from.abs_diff(to) as f64 / (74.0 * MINUTE as f64);
        SEEK_BASE + (SEEK_SPAN as f64 * d.sqrt()) as u64
    }

    /// The motor starts; the TOC is read; then `then`.
    pub(super) fn spin_up(&mut self, now: u64, then: Then) {
        self.motor_stops_at = u64::MAX;
        self.activity = Activity::SpinningUp;
        self.motor = true;
        self.then = then;
        self.pending_seek = false;
        self.drive_at = now + SPIN_UP;
    }

    /// Starts a seek to `target`, spinning up first if the motor is off.
    pub(super) fn start_seek(&mut self, now: u64, target: u32, then: Then) {
        self.seek_error = false;
        self.waiting = None;
        self.int1_at = u64::MAX;
        self.target = target;
        if !self.motor || self.activity == Activity::SpinningUp {
            let wait = self.activity == Activity::SpinningUp;
            if !wait {
                self.spin_up(now, then);
            }
            self.then = then;
            self.pending_seek = true;
            return;
        }
        self.double = self.mode & mode::DOUBLE != 0;
        self.activity = Activity::Seeking;
        self.then = then;
        self.drive_at = now + self.seek_time(self.position, target);
    }

    /// Reading or playing ends; the head holds where it is, and a sector waiting for its INT1 is dropped (psx-spx's
    /// "Sector Buffer VS Pause" tests: no INT1 follows Pause's first response).
    pub(super) fn end_activity(&mut self) {
        self.waiting = None;
        self.int1_at = u64::MAX;
        if matches!(self.activity, Activity::Reading | Activity::Playing | Activity::Seeking) {
            self.hover = self.position;
            self.activity = Activity::Idle;
        }
        self.pending_seek = false;
    }

    /// The spindle stops at `at`; until then the motor bit stays.
    pub(super) fn stop_motor(&mut self, at: u64) {
        self.activity = Activity::Stopped;
        self.drive_at = u64::MAX;
        self.motor_stops_at = at;
    }

    /// The last sector a seek can find: the lead-out's readable stretch past the program [C, D-26].
    pub(super) fn readable_end(&self) -> u32 {
        self.disc.as_ref().map_or(0, |d| (d.get().lead_out() + LEAD_OUT_READABLE).max(OUTER_LIMIT))
    }

    pub(super) fn drive_event(&mut self, t: u64) {
        match self.activity {
            Activity::SpinningUp => {
                self.activity = Activity::Idle;
                self.hover = PARKED;
                self.position = PARKED - 8;
                if self.pending_seek {
                    self.pending_seek = false;
                    self.activity = Activity::Seeking;
                    self.drive_at = t + self.seek_time(self.position, self.target);
                    return;
                }
                if self.then == (Then::Hold { complete: true }) {
                    self.defer(2, Reply::Stat, t);
                }
                self.drive_at = t + self.period();
            }
            Activity::Seeking => self.seek_done(t),
            Activity::Idle => {
                let (lo, hi) = if self.double { (self.hover.saturating_sub(5), self.hover + 2) } else { (self.hover.saturating_sub(8), self.hover) };
                self.position = if self.position >= hi || self.position < lo { lo } else { self.position + 1 };
                self.pass_over(self.position);
                self.drive_at = t + self.period();
            }
            Activity::Reading => {
                self.read_sector(t);
                self.position += 1;
                self.drive_at = t + self.period();
            }
            Activity::Playing => {
                self.play_sector(t);
                if self.activity == Activity::Playing {
                    self.position += 1;
                }
                self.drive_at = t + self.period();
            }
            Activity::Stopped => {}
        }
    }

    fn seek_done(&mut self, t: u64) {
        let audio = self.disc.as_ref().is_some_and(|d| d.get().track_at(self.target).is_some_and(|tr| tr.audio));
        let complete = matches!(self.then, Then::Hold { complete: true });
        if self.target >= self.readable_end() || (self.seek_data && audio && complete) {
            self.seek_error = true;
            self.activity = Activity::Stopped;
            self.motor = false;
            self.seek_data = false;
            let s = self.stat();
            if complete || self.then != (Then::Hold { complete: false }) {
                let mut b = [0u8; 12];
                b[..2].copy_from_slice(&[s, 0x04]);
                self.defer(5, Reply::Bytes(b, 2), t);
            }
            return;
        }
        let validates = self.seek_data && !audio;
        self.seek_data = false;
        self.position = self.target;
        self.hover = self.target;
        self.resume = self.target;
        match self.then {
            Then::Hold { complete } => {
                self.activity = Activity::Idle;
                self.header_valid |= validates;
                self.pass_over(self.position);
                if complete {
                    self.defer(2, Reply::Stat, t);
                }
                self.drive_at = t + self.period();
            }
            Then::Read => {
                self.activity = Activity::Reading;
                self.double = self.mode & mode::DOUBLE != 0;
                self.drive_at = t + self.period();
            }
            Then::Play => {
                self.activity = Activity::Playing;
                self.play_track = self.disc.as_ref().map_or(0, |d| d.get().track_at(self.target).map_or(0, |tr| tr.number));
                self.drive_at = t + self.period();
            }
        }
    }

    /// The head passes `at` while holding: its Q, and its header once a data header has been read.
    fn pass_over(&mut self, at: u32) {
        let Some(slot) = self.disc.clone() else { return };
        let mut d = slot.get();
        let q = d.subq(at);
        if q.valid() {
            self.q = q;
        }
        if self.header_valid {
            let kind = d.read(at, &mut self.scratch);
            if !kind.audio() {
                self.header.copy_from_slice(&self.scratch[12..20]);
            }
        }
    }

    /// A sector arrives while reading: XA-ADPCM to the decoder if it qualifies, else an INT1 delivery (psx-spx
    /// "Data/ADPCM Sector Filtering/Delivery").
    fn read_sector(&mut self, t: u64) {
        let at = self.position;
        self.resume = at;
        let Some(slot) = self.disc.clone() else { return };
        let mut d = slot.get();
        let q = d.subq(at);
        if q.valid() {
            self.q = q;
        }
        let kind = d.read(at, &mut self.scratch);
        if let Sector::Data { mode: m, .. } = kind {
            self.header.copy_from_slice(&self.scratch[12..20]);
            self.header_valid = true;
            let sm = self.scratch[18];
            let realtime_audio = m == 2 && sm & (submode::AUDIO | submode::REAL_TIME) == submode::AUDIO | submode::REAL_TIME;
            let matches = (self.scratch[16], self.scratch[17]) == self.filter;
            if realtime_audio && self.mode & mode::XA != 0 && (self.mode & mode::FILTER == 0 || matches) {
                self.play_xa();
                return;
            }
            if realtime_audio && self.mode & mode::FILTER != 0 {
                return;
            }
        } else if self.mode & mode::CDDA == 0 {
            return;
        }
        if self.flags & 7 == 0 && self.waiting.is_none() {
            self.delivered.raw.copy_from_slice(&self.scratch[..]);
            self.delivered.at = at;
            let s = self.stat() | if self.mode & mode::IGNORE != 0 { stat::ID_ERROR } else { 0 };
            self.respond(1, &[s]);
        } else {
            let w = self.waiting.get_or_insert_with(|| Buffered { raw: Box::new([0; 2352]), at: 0, second_try: true });
            w.raw.copy_from_slice(&self.scratch[..]);
            w.at = at;
            w.second_try = true;
            self.int1_at = self.int1_at.min(t);
        }
    }

    /// An XA-ADPCM sector through the decoder and the interpolator to the output.
    fn play_xa(&mut self) {
        let coding = Coding::of(self.scratch[19]);
        let mut frames = std::mem::take(&mut self.xa_frames);
        frames.clear();
        let (mut l, mut r) = (0usize, 0usize);
        frames.resize(coding.samples(), [0, 0]);
        self.xa.decode(&self.scratch[24..24 + 2304], coding, |ch, s| {
            if coding.stereo {
                let i = if ch == 0 { &mut l } else { &mut r };
                frames[*i][ch] = s;
                *i += 1;
            } else {
                frames[l] = [s, s];
                l += 1;
            }
        });
        let repeat = if coding.half_rate { 2 } else { 1 };
        for f in &frames {
            for _ in 0..repeat {
                let out = &mut self.out;
                self.xa.resample(*f, |o| out.push(o, true));
            }
        }
        self.xa_frames = frames;
    }

    /// A sector while playing: its samples to the output, a report on every tenth sector when asked, and the end of
    /// the track or the disc (psx-spx "Play", "Report", "AutoPause").
    fn play_sector(&mut self, t: u64) {
        let at = self.position;
        let Some(slot) = self.disc.clone() else { return };
        let mut d = slot.get();
        let lead_out = d.lead_out();
        let q = d.subq(at);
        if q.valid() {
            self.q = q;
        }
        let track = d.track_at(at).map_or(0xAA, |tr| tr.number);
        if at >= lead_out {
            self.activity = Activity::Idle;
            self.hover = at;
            let s = self.stat();
            self.respond_or_defer(4, s, t);
            self.stop_motor(t + super::commands::STOP_SINGLE);
            return;
        }
        if track != self.play_track && self.mode & mode::AUTO_PAUSE != 0 {
            self.activity = Activity::Idle;
            self.hover = at.saturating_sub(1);
            let s = self.stat();
            self.respond_or_defer(4, s, t);
            return;
        }
        self.play_track = track;
        let kind = d.read(at, &mut self.scratch);
        let mut peak = 0u16;
        for c in self.scratch.chunks_exact(4) {
            let (l, r) = if kind.audio() { (i16::from_le_bytes([c[0], c[1]]), i16::from_le_bytes([c[2], c[3]])) } else { (0, 0) };
            peak = peak.max(l.unsigned_abs().min(0x7FFF));
            self.out.push([l, r], false);
        }
        let f = at % 75;
        if self.mode & mode::REPORT != 0 && f % 10 == 0 && self.flags & 7 == 0 {
            let s = self.stat();
            let g = self.q.getloc();
            let report = if (f / 10) % 2 == 0 { [s, g[0], g[1], g[5], g[6], g[7], peak as u8, (peak >> 8) as u8] } else { [s, g[0], g[1], g[2], g[3] | 0x80, g[4], peak as u8, (peak >> 8) as u8] };
            self.respond(1, &report);
        }
    }

    fn respond_or_defer(&mut self, int: u8, s: u8, t: u64) {
        if self.flags & 7 == 0 {
            self.respond(int, &[s]);
        } else {
            let mut b = [0u8; 12];
            b[0] = s;
            self.defer(int, Reply::Bytes(b, 1), t);
        }
    }

    // --- the lid ---

    /// The lid opens: the drive stops, INT5 with error 08h whatever was pending (psx-spx "Status code").
    pub fn open_lid(&mut self, now: u64) {
        if self.lid_open {
            return;
        }
        self.run(now);
        self.lid_open = true;
        self.shell_flag = true;
        self.seek_error = false;
        self.id_error = false;
        self.waiting = None;
        self.pending_seek = false;
        if self.motor {
            self.stop_motor(now + SPIN_DOWN);
        } else {
            self.activity = Activity::Stopped;
            self.drive_at = u64::MAX;
        }
        let mut b = [0u8; 12];
        b[..2].copy_from_slice(&[stat::ERROR, 0x08]);
        self.queue = [None; 4];
        self.defer(5, Reply::Bytes(b, 2), now);
        self.refresh();
    }

    /// The lid closes on `disc` (the same one or another); the drive spins up and reads the TOC.
    pub fn close_lid(&mut self, now: u64, disc: Option<emusen_disc::Disc>) {
        if !self.lid_open {
            return;
        }
        self.run(now);
        self.lid_open = false;
        if let Some(d) = disc {
            self.disc = Some(DiscSlot::new(d));
        }
        if self.disc.is_some() {
            self.spin_up(now, Then::Hold { complete: false });
        }
        self.refresh();
    }
}

