//! The command set (psx-spx "CDROM Controller Command Summary" and the chapters after it): each command's parameters,
//! its first response, the second it defers and their delays (psx-spx "CDROM - Response Timings", Diamond_Disputes.md
//! D-24). Diamond_CDROM.md §4 is the table this follows.

use super::{Activity, Cdrom, Reply, Then, drive, mode, stat};
use crate::media::Market;
use emusen_disc::Msf;
use emusen_disc::msf::{MINUTE, bcd, from_bcd};

/// The first response's delay with the motor on, as ps1-tests' `cdrom/timing` measured Nop's on a console (its
/// average, 45,993 cycles), and Init's (75,086); with the motor stopped, psx-spx's table (D-24).
pub const FIRST_NORMAL: u64 = 46_000;
pub const FIRST_STOPPED: u64 = 0x0005_CF4;
pub const FIRST_INIT: u64 = 75_000;
/// Second responses: Init's once the head holds (the timing log's 476,300), GetID's (psx-spx), Pause's at either speed
/// (the timing log's 1,006,472-1,046,336, where psx-spx's PSone gives 2,167,580 and 1,097,107) and when already
/// paused, and Stop's at each speed and when stopped (psx-spx) [D-24].
pub const INIT_SECOND: u64 = 476_000;
pub const GETID_SECOND: u64 = 0x0004_A00;
pub const PAUSE_SECOND: u64 = 1_020_000;
pub const PAUSE_PAUSED: u64 = 0x0001_DF2;
pub const STOP_SINGLE: u64 = 0x0D38_ACA;
pub const STOP_DOUBLE: u64 = 0x18A_6076;
pub const STOP_STOPPED: u64 = 0x0001_D7B;
/// The next response after an acknowledge: the shortest command-to-response the table shows, one pass of the
/// controller's loop [C, D-24].
pub const AFTER_ACK: u64 = 0x1D00;
/// ReadTOC's second response, "about 1 second" (psx-spx), and Reset's silence, an eighth of a second.
pub const READ_TOC: u64 = 33_868_800;
pub const RESET_QUIET: u64 = 33_868_800 / 8;
/// GetQ's second response, the head's trip to the lead-in and back [C, D-25].
pub const GETQ_SECOND: u64 = 33_868_800 / 10;

/// The error bytes (psx-spx "Status code").
const BAD_VALUE: u8 = 0x10;
const BAD_COUNT: u8 = 0x20;
const BAD_COMMAND: u8 = 0x40;
const NOT_READY: u8 = 0x80;

fn bytes(b: &[u8]) -> Reply {
    let mut a = [0u8; 12];
    a[..b.len()].copy_from_slice(b);
    Reply::Bytes(a, b.len() as u8)
}

/// Commands that need a disc in the closed drive (psx-spx: 02h-09h, 0Bh-0Dh, 10h-16h, 1Ah, 1Bh, 1Dh).
fn needs_disc(c: u8) -> bool {
    matches!(c, 0x02..=0x09 | 0x0B..=0x0D | 0x10..=0x16 | 0x1A | 0x1B | 0x1D)
}

/// Commands that need the TOC read, failing while the drive spins up.
fn needs_toc(c: u8) -> bool {
    matches!(c, 0x10 | 0x11 | 0x13 | 0x14 | 0x1A | 0x1D)
}

/// Each command's parameter count, where it has a fixed one.
fn params_for(c: u8) -> Option<std::ops::RangeInclusive<usize>> {
    Some(match c {
        0x02 => 3..=3,
        0x03 => 0..=1,
        0x0D => 2..=2,
        0x0E => 1..=1,
        0x12 | 0x14 => 1..=1,
        0x19 => 1..=16,
        0x1C => 0..=16,
        0x1D => 2..=2,
        0x01 | 0x04..=0x0B | 0x0C | 0x0F..=0x11 | 0x13 | 0x15 | 0x16 | 0x1A | 0x1B | 0x1E => 0..=0,
        _ => return None,
    })
}

impl Cdrom {
    /// COMMAND: BUSYSTS rises at once; the command runs after its first response's delay, once no interrupt is
    /// pending (psx-spx "First Response").
    pub(super) fn write_command(&mut self, value: u8, now: u64) {
        if self.crashed {
            return;
        }
        if self.trace {
            eprintln!("cd command {value:02X} {:02X?}", &self.params[..self.param_len as usize]);
        }
        let delay = match value {
            0x0A | 0x1E => FIRST_INIT,
            _ if self.motor => FIRST_NORMAL,
            _ => FIRST_STOPPED,
        };
        self.command = Some(value);
        self.command_at = now + delay;
    }

    fn error(&mut self, code: u8) {
        let s = self.stat() | stat::ERROR;
        self.respond(5, &[s, code]);
    }

    fn ack(&mut self) {
        let s = self.stat();
        self.respond(3, &[s]);
    }

    /// Runs the waiting command at `now`: its first response, and what it defers.
    pub(super) fn execute(&mut self, now: u64) {
        let Some(c) = self.command.take() else { return };
        let n = self.param_len as usize;
        let p = self.params;
        let keeps_params = matches!(c, 0x1F | 0x50..=0x57);
        if !keeps_params {
            self.param_len = 0;
        }
        if matches!(c, 0x58..=0x5F) {
            self.crashed = true;
            return;
        }
        if let Some(range) = params_for(c)
            && !range.contains(&n)
        {
            return self.error(BAD_COUNT);
        }
        if needs_disc(c) && (self.lid_open || self.disc.is_none()) {
            return self.error(NOT_READY);
        }
        if needs_toc(c) && self.activity == Activity::SpinningUp {
            return self.error(NOT_READY);
        }
        match c {
            0x01 => {
                self.ack();
                if !self.lid_open {
                    self.shell_flag = false;
                }
            }
            0x02 => match Msf::from_bcd([p[0], p[1], p[2]]) {
                Some(m) => {
                    self.setloc = m.sector();
                    self.setloc_pending = true;
                    self.ack();
                }
                None => self.error(BAD_VALUE),
            },
            0x03 => {
                let track = if n == 1 { from_bcd(p[0]).unwrap_or(0) } else { 0 };
                let toc = self.disc.as_ref().map(|d| d.get().toc());
                let target = match toc {
                    Some(t) if track >= 1 && t.tracks.iter().any(|x| x.0 == track) => t.tracks.iter().find(|x| x.0 == track).map_or(0, |x| x.2),
                    Some(t) if track >= 1 => self.track_start(self.position).unwrap_or(t.tracks[0].2),
                    _ if self.setloc_pending => self.setloc,
                    _ => self.position,
                };
                self.setloc_pending = false;
                self.ack();
                if self.activity != Activity::Playing || track >= 1 || target != self.position {
                    self.start_seek(now, target, Then::Play);
                }
            }
            0x04 | 0x05 => {
                if self.activity != Activity::Playing {
                    return self.error(NOT_READY);
                }
                self.ack();
                let step = 75 * 2;
                self.position = if c == 0x04 { self.position + step } else { self.position.saturating_sub(step).max(self.first_start()) };
            }
            0x06 | 0x1B => {
                self.ack();
                if self.setloc_pending {
                    self.setloc_pending = false;
                    self.start_seek(now, self.setloc, Then::Read);
                } else if self.activity != Activity::Reading {
                    let at = if self.header_valid { self.delivered.at } else { self.position };
                    self.start_seek(now, at, Then::Read);
                }
            }
            0x07 => {
                if self.motor {
                    return self.error(BAD_COUNT);
                }
                self.ack();
                self.spin_up(now, Then::Hold { complete: true });
            }
            0x08 => {
                let was_still = !self.motor;
                self.end_activity();
                self.ack();
                let delay = if was_still { STOP_STOPPED } else if self.double { STOP_DOUBLE } else { STOP_SINGLE };
                self.stop_motor(now + delay);
                self.defer(2, Reply::Stat, now + delay);
            }
            0x09 => {
                let busy = matches!(self.activity, Activity::Reading | Activity::Playing | Activity::Seeking);
                self.ack();
                let delay = if busy { PAUSE_SECOND } else { PAUSE_PAUSED };
                self.end_activity();
                self.defer(2, Reply::Stat, now + delay);
            }
            0x0A => {
                self.mode = mode::WHOLE;
                self.whole = true;
                self.end_activity();
                self.waiting = None;
                self.queue = [None; 4];
                self.ack();
                if self.disc.is_some() && !self.lid_open {
                    self.start_seek(now, drive::PARKED, Then::Hold { complete: false });
                    let ready = if self.activity == Activity::Seeking { self.drive_at } else { self.drive_at + drive::SPIN_UP };
                    self.defer(2, Reply::Stat, ready.max(now + INIT_SECOND));
                } else {
                    self.defer(2, Reply::Stat, now + GETID_SECOND);
                }
            }
            0x0B => {
                self.out.muted = true;
                self.ack();
            }
            0x0C => {
                self.out.muted = false;
                self.ack();
            }
            0x0D => {
                self.filter = (p[0], p[1]);
                self.ack();
            }
            0x0E => {
                self.mode = p[0];
                if p[0] & mode::IGNORE == 0 {
                    self.whole = p[0] & mode::WHOLE != 0;
                }
                self.id_error = p[0] & mode::IGNORE != 0;
                self.ack();
            }
            0x0F => {
                let s = self.stat();
                self.respond(3, &[s, self.mode, 0, self.filter.0, self.filter.1]);
            }
            0x10 => {
                let audio = self.disc.as_ref().is_some_and(|d| d.get().track_at(self.position).is_some_and(|t| t.audio));
                if self.seek_error {
                    let s = self.stat();
                    self.respond(5, &[s, NOT_READY]);
                } else if !self.header_valid || self.activity == Activity::Seeking || audio {
                    self.error(NOT_READY);
                } else {
                    let h = self.header;
                    self.respond(3, &h);
                }
            }
            0x11 => {
                if self.seek_error {
                    let s = self.stat();
                    self.respond(5, &[s, NOT_READY]);
                } else {
                    let q = self.q.getloc();
                    self.respond(3, &q);
                }
            }
            0x12 => match p[0] {
                0 => self.error(BAD_VALUE),
                1 => {
                    if matches!(self.activity, Activity::Reading | Activity::Playing) {
                        return self.error(NOT_READY);
                    }
                    self.ack();
                    self.start_seek(now, drive::PARKED, Then::Hold { complete: true });
                }
                _ => {
                    self.ack();
                    let s = self.stat() | stat::SEEK_ERROR;
                    self.defer(5, bytes(&[s, 0x40]), now + GETID_SECOND);
                    self.defer(5, bytes(&[s, 0x40]), now + 2 * GETID_SECOND);
                }
            },
            0x13 => {
                let t = self.disc.as_ref().map(|d| d.get().toc());
                let (first, last) = t.map_or((1, 1), |t| (t.first, t.last));
                let s = self.stat();
                self.respond(3, &[s, bcd(first), bcd(last)]);
            }
            0x14 => {
                let Some(toc) = self.disc.as_ref().map(|d| d.get().toc()) else { return self.error(NOT_READY) };
                let at = match from_bcd(p[0]) {
                    Some(0) => Some(toc.lead_out),
                    Some(t) => toc.tracks.iter().find(|x| x.0 == t).map(|x| x.2),
                    None => None,
                };
                match at {
                    Some(at) => {
                        let m = Msf::of(at);
                        let s = self.stat();
                        self.respond(3, &[s, bcd(m.m), bcd(m.s)]);
                    }
                    None => self.error(BAD_VALUE),
                }
            }
            0x15 | 0x16 => {
                self.ack();
                let target = if self.setloc_pending { self.setloc } else { self.position };
                self.setloc_pending = false;
                self.seek_data = c == 0x15;
                self.start_seek(now, target, Then::Hold { complete: true });
            }
            0x19 => self.test(now, &p[..n]),
            0x1A => {
                self.ack();
                let reply = self.identify();
                self.defer(reply.0, reply.1, now + GETID_SECOND);
            }
            0x1C => {
                self.ack();
                self.reset_until = now + RESET_QUIET;
                self.end_activity();
                self.mode = 0;
                self.waiting = None;
                if self.disc.is_some() && !self.lid_open {
                    self.spin_up(now, Then::Hold { complete: false });
                }
            }
            0x1D => {
                let item = self.disc.as_ref().and_then(|d| if p[0] & 0xF == 1 { d.get().lead_in(p[1]) } else { None });
                self.ack();
                match item {
                    Some(q) => {
                        let mut b = [0u8; 11];
                        b[..10].copy_from_slice(&q.0[..10]);
                        self.defer(2, bytes(&b), now + GETQ_SECOND);
                    }
                    None => {
                        let s = self.stat() | stat::ERROR;
                        self.defer(5, bytes(&[s, NOT_READY]), now + 6 * READ_TOC);
                    }
                }
            }
            0x1E => {
                self.ack();
                self.defer(2, Reply::Stat, now + READ_TOC);
            }
            _ => self.error(BAD_COMMAND),
        }
    }

    /// GetID's second response (psx-spx "GetID"): a data disc licensed for the console's own region (Diamond_Plan.md
    /// §5.6), an audio disc refused, an empty drive.
    fn identify(&mut self) -> (u8, Reply) {
        let Some(toc) = self.disc.as_ref().map(|d| d.get().toc()) else { return (5, bytes(&[0x08, 0x40, 0, 0, 0, 0, 0, 0])) };
        let audio = toc.tracks.first().is_some_and(|t| t.1 & 4 == 0);
        if audio {
            self.id_error = true;
            let s = self.stat();
            return (5, bytes(&[s, 0x90, 0, 0, 0, 0, 0, 0]));
        }
        let letter = match self.market {
            Market::Japan => b'I',
            Market::Americas => b'A',
            Market::Europe => b'E',
        };
        let s = self.stat();
        (2, bytes(&[s, 0x00, toc.disc_type, 0x00, b'S', b'C', b'E', letter]))
    }

    /// Command 19h's sub-functions (psx-spx "Test Commands"): the version, switches, region and chip strings, the
    /// SCEx counters, and the drive mechanics' motor commands; the HC05's and the decoder's internals answer zero.
    fn test(&mut self, now: u64, p: &[u8]) {
        let s = self.stat();
        match p[0] {
            0x20 => self.respond(3, &firmware_of(self.market)),
            0x21 => {
                let pos0 = self.position < 150;
                self.respond(3, &[pos0 as u8 | (self.lid_open as u8) << 1]);
            }
            0x22 => {
                let text: &[u8] = match self.market {
                    Market::Japan => b"for Japan",
                    Market::Americas => b"for U/C",
                    Market::Europe => b"for Europe",
                };
                self.respond(3, text);
            }
            0x23 => self.respond(3, b"CXD2940Q/CXD1817Q/CXD2545Q/CXD1782BR"),
            0x24 => self.respond(3, b"CXD2940Q/CXD1817Q/CXD2545Q/CXD2510Q"),
            0x25 => self.respond(3, b"CXD2940Q/CXD1817Q/CXD1815Q/CXD1199BQ"),
            0x04 => self.respond(3, &[s]),
            0x05 => {
                let licensed = self.disc.as_ref().is_some_and(|d| d.get().toc().tracks.first().is_some_and(|t| t.1 & 4 != 0));
                self.respond(3, &[licensed as u8, licensed as u8]);
            }
            0x03 => {
                self.end_activity();
                self.stop_motor(now);
                self.respond(3, &[s]);
            }
            0x00..=0x02 | 0x10..=0x1A => {
                if !self.motor && self.disc.is_some() {
                    self.spin_up(now, Then::Hold { complete: false });
                }
                self.respond(3, &[s]);
            }
            0x50 => self.respond(3, &[s]),
            0x51 => self.respond(3, &[s, 0, 0]),
            0x60 | 0x71 => self.respond(3, &[0]),
            0x72 | 0x74 | 0x76 => self.respond(3, &[s]),
            0x73 => self.respond(3, &[0; 4]),
            0x75 => self.respond(3, &[0; 4]),
            _ => self.error(BAD_VALUE),
        }
    }

    fn first_start(&self) -> u32 {
        self.disc.as_ref().map_or(150, |d| d.get().tracks()[0].start)
    }

    fn track_start(&self, at: u32) -> Option<u32> {
        self.disc.as_ref().and_then(|d| d.get().track_at(at).map(|t| t.start))
    }
}

/// 19h,20h's four bytes.
fn firmware_of(m: Market) -> [u8; 4] {
    super::firmware(m)
}

/// How far a seek can reach: a fixed outer limit, 74:15:00, or 1:30 past the lead-out's start where a disc's program
/// runs later, fitted to the console's getloc log [C, D-26].
pub const OUTER_LIMIT: u32 = 74 * MINUTE + 15 * 75;
pub const LEAD_OUT_READABLE: u32 = 90 * 75;
