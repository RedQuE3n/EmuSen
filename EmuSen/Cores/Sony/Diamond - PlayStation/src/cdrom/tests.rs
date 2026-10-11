//! The controller driven at its registers as a program would, on synthetic discs built here: each command's responses,
//! their order and delays, reads and their buffer, the filter, XA and CD-DA against their sources, the lid.

use super::*;
use emusen_disc::cue::Sheet;
use emusen_disc::ecma130::{self, submode};
use emusen_disc::{Disc, IsoForm};

pub const DATA: u32 = 600;
pub const AUDIO: u32 = 300;

/// A disc of a mode 2 data track of `DATA` sectors whose user data names each sector, then an audio track whose
/// frame k of sector i is (i·588 + k, −(i·588 + k)) truncated to 16 bits.
pub fn disc() -> Disc {
    let mut d = Vec::new();
    for i in 0..DATA {
        let user: Vec<u8> = (0..2048u32).map(|k| (k + i) as u8).collect();
        let mut s = [0u8; 2352];
        ecma130::mode2_form1(150 + i, [1, 0, submode::DATA, 0], &user, &mut s);
        d.extend_from_slice(&s);
    }
    let mut a = Vec::new();
    for i in 0..AUDIO {
        for k in 0..588u32 {
            let v = (i * 588 + k) as i16;
            a.extend_from_slice(&v.to_le_bytes());
            a.extend_from_slice(&(-(v as i32)).clamp(-32768, 32767).to_le_bytes()[..2]);
        }
    }
    let sheet = "FILE \"d\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\nFILE \"a\" BINARY\nTRACK 02 AUDIO\nINDEX 00 00:00:00\nINDEX 01 00:02:00\n";
    Disc::from_sheet(&Sheet::parse(sheet).unwrap(), vec![d, a], IsoForm::Mode2Form1).unwrap()
}

/// A program's side of the registers: the clock it runs at.
pub struct Host {
    pub c: Cdrom,
    pub now: u64,
}

impl Host {
    pub fn new(d: Option<Disc>) -> Host {
        let mut h = Host { c: Cdrom::new(d, Market::Americas), now: 0 };
        h.write(0, 1);
        h.write(2, 0x1F);
        h.write(0, 0);
        h
    }

    pub fn write(&mut self, reg: u32, v: u8) {
        self.c.run(self.now);
        self.c.write8(reg, v, self.now);
        self.now += 20;
    }

    pub fn read(&mut self, reg: u32) -> u8 {
        self.c.run(self.now);
        let v = self.c.read8(reg);
        self.now += 20;
        v
    }

    /// Waits for an interrupt, up to `limit` cycles; its type, its bytes, and when it came.
    pub fn wait(&mut self, limit: u64) -> Option<(u8, Vec<u8>, u64)> {
        let end = self.now + limit;
        while self.now < end {
            let next = self.c.next_event().min(end).max(self.now + 1);
            self.now = next;
            self.c.run(self.now);
            if self.c.flags & 7 != 0 {
                let t = self.now;
                self.write(0, 1);
                let int = self.read(3) & 7;
                self.write(0, 0);
                let mut bytes = Vec::new();
                while self.read(0) & 0x20 != 0 {
                    bytes.push(self.read(1));
                }
                return Some((int, bytes, t));
            }
        }
        None
    }

    pub fn ack(&mut self) {
        self.write(0, 1);
        self.write(3, 0x1F);
        self.write(0, 0);
    }

    /// A command with its parameters, waiting for and acknowledging its first response.
    pub fn command(&mut self, cmd: u8, params: &[u8]) -> (u8, Vec<u8>, u64) {
        for &p in params {
            self.write(2, p);
        }
        let at = self.now;
        self.write(1, cmd);
        let mut r = self.wait(100_000_000).expect("a first response");
        self.ack();
        r.2 -= at;
        r
    }

    /// The next response after the first, acknowledged.
    pub fn next(&mut self, limit: u64) -> Option<(u8, Vec<u8>)> {
        let r = self.wait(limit)?;
        self.ack();
        Some((r.0, r.1))
    }

    /// Runs until the drive has spun up and holds.
    pub fn settle(&mut self) {
        self.now += drive::SPIN_UP + 4 * SECTOR_SINGLE;
        self.c.run(self.now);
    }
}

#[test]
fn getstat_after_power_on_shows_the_motor_and_its_first_response_comes_after_its_delay() {
    let mut h = Host::new(Some(disc()));
    h.settle();
    let (int, b, t) = h.command(0x01, &[]);
    assert_eq!((int, b), (3, vec![stat::MOTOR]));
    assert!((commands::FIRST_NORMAL..commands::FIRST_NORMAL + 200).contains(&t), "{t}");
    let (int, b, _) = h.command(0x00, &[]);
    assert_eq!((int, b), (5, vec![stat::MOTOR | stat::ERROR, 0x40]));
    let (int, b, _) = h.command(0x02, &[0x00, 0x02]);
    assert_eq!((int, b), (5, vec![stat::MOTOR | stat::ERROR, 0x20]), "Setloc needs three");
    let (int, b, _) = h.command(0x02, &[0x00, 0x60, 0x00]);
    assert_eq!((int, b), (5, vec![stat::MOTOR | stat::ERROR, 0x10]));
}

#[test]
fn getlocl_fails_until_a_seek_and_getlocp_reports_the_pause_before_track_one() {
    let mut h = Host::new(Some(disc()));
    h.settle();
    let (int, b, _) = h.command(0x10, &[]);
    assert_eq!((int, b[1]), (5, 0x80));
    let (int, q, _) = h.command(0x11, &[]);
    assert_eq!((int, q[0], q[1]), (3, 0x01, 0x00), "track 1's pause: {q:02X?}");
    assert!(q[5..] >= [0x00, 0x01, 0x67][..] && q[5..] <= [0x00, 0x02, 0x00][..], "{q:02X?}");
    h.command(0x02, &[0x00, 0x02, 0x16]);
    let (int, _, _) = h.command(0x15, &[]);
    assert_eq!(int, 3);
    assert_eq!(h.next(200_000_000), Some((2, vec![stat::MOTOR])));
    let (int, l, _) = h.command(0x10, &[]);
    assert_eq!(int, 3);
    assert!(l[..3] >= [0x00, 0x02, 0x08][..] && l[..3] <= [0x00, 0x02, 0x16][..] && l[3] == 2, "{l:02X?}");
    assert_eq!(&l[4..], &[1, 0, submode::DATA, 0]);
}

#[test]
fn debug_settle() {
    let mut h = Host::new(Some(disc()));
    h.settle();
    eprintln!("{:?} pos {} hover {} q {:02X?} drive_at {} now {}", h.c.activity, h.c.position, h.c.hover, h.c.q.0, h.c.drive_at, h.now);
    let r = h.command(0x10, &[]);
    eprintln!("{r:02X?} q {:02X?}", h.c.q.0);
    let r = h.command(0x11, &[]);
    eprintln!("{r:02X?} q {:02X?} results {:02X?} len {}", h.c.q.0, h.c.results, h.c.result_len);
}
