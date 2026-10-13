//! The controller driven at its registers as a program would, on synthetic discs built here: each command's responses,
//! their order and delays, reads and their buffer, the filter, XA and CD-DA against their sources, the lid.

use super::*;
use emusen_disc::cue::Sheet;
use emusen_disc::ecma130::{self, submode};
use crate::medium::Disc;
use emusen_disc::IsoForm;
use emusen_native::core::Kept;

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
    Disc::from_sheet(&Sheet::parse(sheet).unwrap(), vec![Kept::from(d), Kept::from(a)], IsoForm::Mode2Form1).unwrap()
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

    /// Runs until the drive has spun up and holds, and clears the shell flag power-on leaves with a Getstat.
    pub fn settle(&mut self) {
        self.now += drive::SPIN_UP + 4 * SECTOR_SINGLE;
        self.c.run(self.now);
        let (_, b, _) = self.command(0x01, &[]);
        assert_eq!(b, vec![stat::SHELL_OPEN | stat::MOTOR], "power-on reports the shell as once opened");
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

/// Reads `n` sectors from `at` with mode `m`, taking each INT1's sector through BFRD and the data FIFO as DMA would.
fn read_sectors(h: &mut Host, at: u32, n: usize, m: u8) -> Vec<Vec<u8>> {
    h.command(0x0E, &[m]);
    let t = emusen_disc::Msf::of(at).bcd();
    h.command(0x02, &t);
    h.command(0x06, &[]);
    let mut out = Vec::new();
    while out.len() < n {
        let (int, b) = h.next(10_000_000).expect("an INT1");
        assert_eq!((int, b[0] & stat::READ), (1, stat::READ));
        h.write(3, 0x80);
        let len = if m & mode::WHOLE != 0 { 0x924 } else { 0x800 };
        let mut s = Vec::with_capacity(len);
        while s.len() < len {
            s.extend_from_slice(&h.c.data_word().to_le_bytes());
        }
        h.write(3, 0x00);
        out.push(s);
    }
    h.command(0x09, &[]);
    assert_eq!(h.next(10_000_000).map(|r| r.0), Some(2), "Pause's second response");
    out
}

#[test]
fn reads_return_every_sector_whole_or_as_user_data_at_either_speed() {
    let mut h = Host::new(Some(disc()));
    h.settle();
    let mut raw = [0u8; 2352];
    for (m, whole) in [(0x00, false), (mode::DOUBLE, false), (mode::WHOLE, true), (mode::WHOLE | mode::DOUBLE, true)] {
        let got = read_sectors(&mut h, 150 + 40, 20, m);
        for (i, s) in got.iter().enumerate() {
            h.c.disc.as_ref().unwrap().get().read(190 + i as u32, &mut raw);
            let want = if whole { &raw[12..2352] } else { &raw[24..2072] };
            assert_eq!(&s[..], want, "mode {m:02X}, sector {i}");
        }
    }
}

#[test]
fn sectors_come_at_75_and_150_a_second() {
    let mut h = Host::new(Some(disc()));
    h.settle();
    for (m, period) in [(0x00, SECTOR_SINGLE), (mode::DOUBLE, SECTOR_DOUBLE)] {
        h.command(0x0E, &[m]);
        h.command(0x02, &[0x00, 0x02, 0x40]);
        h.command(0x06, &[]);
        let mut times = Vec::new();
        for _ in 0..6 {
            let r = h.wait(10_000_000).unwrap();
            times.push(r.2);
            h.ack();
        }
        for w in times.windows(2).skip(1) {
            assert!((w[1] - w[0]).abs_diff(period) < 300, "{m:02X}: {}", w[1] - w[0]);
        }
        h.command(0x09, &[]);
        h.next(10_000_000);
    }
}

/// psx-spx's buffer test: an INT1 not handled for a while leaves the oldest sector and then the newest.
#[test]
fn a_late_handler_gets_the_oldest_sector_then_the_newest() {
    let mut h = Host::new(Some(disc()));
    h.settle();
    h.command(0x0E, &[0x00]);
    h.command(0x02, &[0x00, 0x02, 0x00]);
    h.command(0x06, &[]);
    let first = h.wait(10_000_000).unwrap();
    h.ack();
    assert_eq!(h.c.delivered.at, 150, "{first:?}");
    h.wait(10_000_000).unwrap();
    h.now += 5 * SECTOR_SINGLE;
    h.c.run(h.now);
    assert_eq!(h.c.delivered.at, 151, "the oldest is the one pending");
    h.ack();
    h.wait(10_000_000).unwrap();
    assert!(h.c.delivered.at >= 155, "then the newest: {}", h.c.delivered.at);
}

#[test]
fn the_toc_commands_answer_from_the_disc() {
    let mut h = Host::new(Some(disc()));
    h.settle();
    assert_eq!(h.command(0x13, &[]).1, vec![stat::MOTOR, 0x01, 0x02]);
    let lead_out = emusen_disc::Msf::of(150 + DATA + AUDIO);
    assert_eq!(h.command(0x14, &[0x00]).1, vec![stat::MOTOR, bcd(lead_out.m), bcd(lead_out.s)]);
    let t2 = emusen_disc::Msf::of(150 + DATA + 150);
    assert_eq!(h.command(0x14, &[0x02]).1, vec![stat::MOTOR, bcd(t2.m), bcd(t2.s)]);
    assert_eq!(h.command(0x14, &[0x03]).0, 5);
    assert_eq!(h.command(0x0F, &[]).1, vec![stat::MOTOR, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(h.command(0x19, &[0x20]).1, vec![0x97, 0x01, 0x10, 0xC2]);
    assert_eq!(h.command(0x19, &[0x22]).1, b"for U/C".to_vec());
    let (int, _, _) = h.command(0x1A, &[]);
    assert_eq!(int, 3);
    assert_eq!(h.next(1_000_000), Some((2, vec![stat::MOTOR, 0x00, 0x20, 0x00, b'S', b'C', b'E', b'A'])));
}

fn bcd(v: u8) -> u8 {
    emusen_disc::msf::bcd(v)
}

/// Play: the track's samples reach the output unchanged at unity volume, a report on every tenth sector, and INT4 at
/// the disc's end.
#[test]
fn cd_da_plays_its_source_through_the_matrix_with_reports() {
    let mut h = Host::new(Some(disc()));
    h.settle();
    h.command(0x0E, &[mode::REPORT]);
    h.command(0x03, &[0x02]);
    let mut reports = Vec::new();
    let mut frames = Vec::new();
    loop {
        let Some((int, b)) = h.next(20_000_000) else { break };
        h.c.take_audio(h.c.out.len as usize, Some(&mut frames));
        if int == 4 {
            break;
        }
        reports.push(b);
    }
    let start = 150 + DATA + 150;
    let want: Vec<[i16; 2]> = (150..AUDIO).flat_map(|i| (0..588u32).map(move |k| {
        let v = (i * 588 + k) as i16;
        [v, (-(v as i32)).clamp(-32768, 32767) as i16]
    })).collect();
    let skip = frames.iter().position(|f| *f == want[0]).expect("the track's first frame");
    assert_eq!(&frames[skip..skip + want.len()], &want[..], "the samples, in order and unchanged");
    assert!(reports.len() >= 10, "{}", reports.len());
    let abs: Vec<&Vec<u8>> = reports.iter().filter(|r| r[4] & 0x80 == 0).collect();
    assert!(abs.iter().all(|r| r[1] == 0x02 && r[5] % 0x20 == 0), "{abs:02X?}");
    assert!(reports.iter().any(|r| r[4] & 0x80 != 0), "relative reports, ss with bit 7");
    let _ = start;
    assert_eq!(h.c.activity, Activity::Stopped, "the end of the disc stops the drive");
}

/// An XA-ADPCM stream of 4-bit samples with filter 0, interleaved with a second channel and with data, so that the
/// decoder's output is the source exactly and the 44,100 Hz output the source through psx-spx's interpolation.
#[test]
fn xa_adpcm_plays_its_channel_and_delivers_the_data_between() {
    let source = |ch: usize, n: usize| -> i16 { ((((n * (3 + ch * 2) + ch) % 16) as i8) << 4 >> 4) as i16 * 0x1000 };
    let xa_sector = |at: u32, k: usize, channel: u8| -> [u8; 2352] {
        let mut user = [0u8; 2324];
        for portion in 0..18 {
            let p = &mut user[portion * 128..portion * 128 + 128];
            for j in 0..28 {
                for blk in 0..4 {
                    let n = (k * 18 + portion) * 112 + blk * 28 + j;
                    let l = source(0, n) >> 12 & 0xF;
                    let r = source(1, n) >> 12 & 0xF;
                    p[16 + j * 4 + blk] = (l | r << 4) as u8;
                }
            }
        }
        let mut s = [0u8; 2352];
        ecma130::mode2_form2(at, [1, channel, submode::AUDIO | submode::FORM2 | submode::REAL_TIME, 1], &user, &mut s);
        s
    };
    let mut bin = Vec::new();
    let mut k = 0;
    for i in 0..64u32 {
        let at = 150 + i;
        let s = match i % 4 {
            0 => { k += 1; xa_sector(at, k - 1, 0) }
            2 => xa_sector(at, 999, 1),
            _ => {
                let mut s = [0u8; 2352];
                ecma130::mode2_form1(at, [1, 0, submode::DATA, 0], &[i as u8; 2048], &mut s);
                s
            }
        };
        bin.extend_from_slice(&s);
    }
    let disc = Disc::from_image(Kept::from(bin), IsoForm::Mode2Form1).unwrap();
    let mut h = Host::new(Some(disc));
    h.settle();
    h.command(0x0D, &[1, 0]);
    h.command(0x0E, &[mode::XA | mode::FILTER | mode::DOUBLE]);
    h.command(0x02, &[0x00, 0x02, 0x00]);
    h.command(0x1B, &[]);
    let mut frames = Vec::new();
    let mut data = Vec::new();
    while data.len() < 30 {
        let Some((int, _)) = h.next(5_000_000) else { break };
        assert_eq!(int, 1);
        data.push(h.c.delivered.at);
        h.c.take_audio(h.c.out.len as usize, Some(&mut frames));
    }
    assert!(data.iter().all(|&at| at % 2 == 1), "only the data sectors come as INT1: {data:?}");
    let sectors = k.min(data.len() / 2);
    let decoded: Vec<[i16; 2]> = (0..sectors * 2016).map(|n| [source(0, n), source(1, n)]).collect();
    let mut ring = [[0i16; 32]; 2];
    let (mut p, mut six) = (0usize, 6);
    let mut want = Vec::new();
    for f in &decoded {
        ring[0][p & 31] = f[0];
        ring[1][p & 31] = f[1];
        p += 1;
        six -= 1;
        if six == 0 {
            six = 6;
            for t in &xa::ZIGZAG {
                let o = |c: usize| (1..=29).map(|i| ring[c][(p - i) & 31] as i32 * t[i - 1] >> 15).sum::<i32>().clamp(-32768, 32767) as i16;
                want.push([o(0), o(1)]);
            }
        }
    }
    assert!(frames.len() >= want.len().saturating_sub(2400), "{} frames, {} expected", frames.len(), want.len());
    let n = frames.len().min(want.len());
    assert_eq!(&frames[..n], &want[..n], "the stream as psx-spx's interpolation makes it of its source");
    assert!(n > 4000);
}

#[test]
fn the_lid_interrupts_and_the_shell_flag_clears_on_getstat_once_closed() {
    let mut h = Host::new(Some(disc()));
    h.settle();
    h.c.open_lid(h.now);
    assert_eq!(h.next(1_000_000), Some((5, vec![stat::ERROR, 0x08])));
    assert_eq!(h.command(0x01, &[]).1, vec![stat::SHELL_OPEN | stat::MOTOR], "still spinning down");
    h.now += drive::SPIN_DOWN;
    assert_eq!(h.command(0x01, &[]).1, vec![stat::SHELL_OPEN]);
    assert_eq!(h.command(0x1A, &[]).1, vec![stat::SHELL_OPEN | stat::ERROR, 0x80], "GetID with the door open");
    h.c.close_lid(h.now, None);
    assert_eq!(h.command(0x01, &[]).1, vec![stat::SHELL_OPEN], "the flag is reported once more");
    assert_eq!(h.command(0x01, &[]).1, vec![0x00], "then cleared, the motor spinning up");
    h.now += drive::SPIN_UP;
    assert_eq!(h.command(0x01, &[]).1, vec![stat::MOTOR]);
}

/// A state saved mid-read and loaded into a fresh controller over the same disc continues as the original does.
#[test]
fn a_state_mid_read_continues_the_same() {
    let mut a = Host::new(Some(disc()));
    a.settle();
    a.command(0x02, &[0x00, 0x03, 0x00]);
    a.command(0x06, &[]);
    a.next(10_000_000);
    a.now += SECTOR_SINGLE / 2;
    a.c.run(a.now);
    let mut w = emusen_native::StateWriter::counter();
    a.c.write_state(&mut w);
    let mut buf = vec![0u8; w.len()];
    let mut w = emusen_native::StateWriter::new(&mut buf);
    a.c.write_state(&mut w);
    let mut b = Host { c: Cdrom::new(None, Market::Americas), now: a.now };
    b.c.disc = a.c.disc.clone();
    b.c.read_state(&mut emusen_native::StateReader::new(&buf)).unwrap();
    for _ in 0..5 {
        let ra = a.wait(10_000_000);
        let rb = b.wait(10_000_000);
        assert_eq!(ra, rb);
        assert_eq!(a.c.delivered.at, b.c.delivered.at);
        a.ack();
        b.ack();
    }
}


/// BFRD written twice, as a game's libcd does after HCHPCTL is cleared: the second write leaves the FIFO where the
/// first put it, so that a transfer begun between them is not restarted (Diamond_CDROM.md §3).
#[test]
fn bfrd_set_again_does_not_restart_the_fifo() {
    let mut h = Host::new(Some(disc()));
    h.settle();
    h.command(0x0E, &[mode::WHOLE]);
    h.command(0x02, &[0x00, 0x02, 0x16]);
    h.command(0x06, &[]);
    h.next(10_000_000).expect("an INT1");
    h.write(3, 0x00);
    h.write(3, 0x80);
    let first = [h.c.data_byte(), h.c.data_byte(), h.c.data_byte(), h.c.data_byte()];
    assert_eq!(first, [0x00, 0x02, 0x16, 0x02], "the header first, in whole-sector mode");
    h.write(3, 0x80);
    assert_eq!(h.c.data_byte(), 1, "the subheader's file number, not the header again");
}
