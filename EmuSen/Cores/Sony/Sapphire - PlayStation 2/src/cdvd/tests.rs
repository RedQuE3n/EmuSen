//! The drive driven at its registers as the IOP's CDVDMAN would (Sapphire_Native.md §25.6), on synthetic discs: a
//! single-layer DVD, a dual-layer one whose second volume places the layer break, and CDs of data and of data with
//! audio. Every expected value is the documents' or §25's record, computed here apart from the model.

use emusen_disc::cue::Sheet;
use emusen_disc::ecma130;
use emusen_disc::ecma267::{self, FRAME};
use emusen_disc::{IsoForm, Kind, Lone, open_medium};
use emusen_native::core::Kept;

use super::timing::{self, CD_SPEED, DVD_SPEED, FAST_SEEK, FULL_SEEK, IOP_HZ, SPIN_UP, block};
use super::*;

const LAYER0: u32 = 1024;
const TOTAL: u32 = 1100;

fn user(lba: u32) -> [u8; 2048] {
    std::array::from_fn(|k| (k as u32 ^ lba.wrapping_mul(0x9E37) ^ (lba >> 8)) as u8)
}

fn pvd(blocks: u32) -> [u8; 2048] {
    let mut d = [0u8; 2048];
    d[..6].copy_from_slice(b"\x01CD001");
    d[80..84].copy_from_slice(&blocks.to_le_bytes());
    d
}

/// A DVD image of `sectors`, its first volume `volume` blocks, a second descriptor at layer 1's sector 16 where asked.
fn dvd_image(sectors: u32, layer0: Option<u32>) -> Vec<u8> {
    let mut v: Vec<u8> = (0..sectors).flat_map(user).collect();
    v[16 * 2048..17 * 2048].copy_from_slice(&pvd(layer0.unwrap_or(sectors)));
    if let Some(l) = layer0 {
        v[(l + 16) as usize * 2048..(l + 17) as usize * 2048].copy_from_slice(&pvd(sectors - l));
    }
    v
}

fn dvd(sectors: u32, layer0: Option<u32>) -> Disc {
    open_medium(Kept::from(dvd_image(sectors, layer0)), Vec::new(), IsoForm::Mode2Form1, Lone::Detect, Kind::Dvd).unwrap()
}

const CD_DATA: u32 = 60;
const CD_AUDIO: u32 = 20;

fn cd_data(i: u32) -> [u8; 2352] {
    let mut s = [0u8; 2352];
    ecma130::mode2_form1(150 + i, [0, 0, ecma130::submode::DATA, 0], &user(i), &mut s);
    s
}

fn cd_audio(i: u32) -> [u8; 2352] {
    std::array::from_fn(|k| (k as u32 * 7 + i) as u8)
}

/// A CD of one data track, and with an audio track after it where asked.
fn cd(audio: bool) -> Disc {
    let data: Vec<u8> = (0..CD_DATA).flat_map(cd_data).collect();
    if !audio {
        return open_medium(Kept::from(data), Vec::new(), IsoForm::Mode2Form1, Lone::Detect, Kind::Dvd).unwrap();
    }
    let sheet = "FILE \"d.bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\nFILE \"a.bin\" BINARY\nTRACK 02 AUDIO\nINDEX 01 00:00:00\n";
    let a: Vec<u8> = (0..CD_AUDIO).flat_map(cd_audio).collect();
    Disc::Cd(emusen_disc::Disc::from_sheet(&Sheet::parse(sheet).unwrap(), vec![Kept::from(data), Kept::from(a)], IsoForm::Mode2Form1).unwrap())
}

/// An N command with its parameters, at `now`.
fn n(c: &mut Cdvd, cmd: u8, params: &[u8], now: u64) {
    params.iter().for_each(|&p| c.write8(reg::N_STATUS, p, now));
    c.write8(reg::N_COMMAND, cmd, now);
}

fn seek(c: &mut Cdvd, lba: u32, now: u64) {
    n(c, ncmd::SEEK, &lba.to_le_bytes(), now);
}

fn read_params(lba: u32, count: u32, size: u8) -> Vec<u8> {
    let mut p = lba.to_le_bytes().to_vec();
    p.extend(count.to_le_bytes());
    p.extend([0, 0, size]);
    p
}

/// An S command's answer, every byte until the status says none is left.
fn s(c: &mut Cdvd, cmd: u8, params: &[u8], now: u64) -> Vec<u8> {
    params.iter().for_each(|&p| c.write8(reg::S_STATUS, p, now));
    c.write8(reg::S_COMMAND, cmd, now);
    let mut out = Vec::new();
    while c.read8(reg::S_STATUS) & 0x40 == 0 {
        out.push(c.read8(reg::S_RESULT));
    }
    out
}

/// The time the drive's N command ends, run event by event; panics past `limit`.
fn finish(c: &mut Cdvd, limit: u64) -> u64 {
    loop {
        let t = c.next_event();
        assert!(t <= limit, "still busy at {limit}");
        c.run(t);
        if c.read8(reg::N_STATUS) & 0x80 == 0 {
            return t;
        }
    }
}

/// A read taken by DMA as each block is ready: the blocks and the time each was handed over.
fn read_all(c: &mut Cdvd, limit: u64) -> Vec<(u64, Vec<u8>)> {
    let mut out = Vec::new();
    while c.read8(reg::N_STATUS) & 0x80 != 0 {
        let t = c.next_event();
        assert!(t <= limit, "still busy at {limit}");
        c.run(t);
        let mut b = vec![0u8; c.dma_ready()];
        if !b.is_empty() {
            let k = c.dma_take(t, &mut b);
            assert_eq!(k, b.len());
            out.push((t, b));
        }
    }
    out
}

#[test]
fn at_power_on_the_registers_report_the_disc() {
    for (disc, kind) in [(Some(dvd(300, None)), 0x14), (Some(dvd(TOTAL, Some(LAYER0))), 0x14), (Some(cd(false)), 0x12), (Some(cd(true)), 0x13), (None, 0x00)] {
        let present = disc.is_some();
        let mut c = Cdvd::new(disc);
        assert_eq!(c.read8(reg::DISK_TYPE), kind);
        assert_eq!(c.read8(reg::DRIVE), if present { 0x0A } else { 0x00 }, "spinning and paused, as the referees report");
        assert_eq!((c.read8(reg::N_STATUS), c.read8(reg::S_STATUS), c.read8(reg::I_STAT)), (0x40, 0x40, 0));
        assert!(!c.irq());
    }
}

#[test]
fn the_s_commands_answer_as_the_documents_and_the_record_say() {
    let mut c = Cdvd::new(Some(dvd(300, None)));
    assert_eq!(s(&mut c, scmd::SUBCOMMAND, &[0], 0), VERSION, "D-124");
    assert_eq!(s(&mut c, scmd::SUBCOMMAND, &[1], 0), [ABSENT_SUBCOMMAND]);
    for absent in [0x02, 0x12, 0x36, 0x80, 0x84, 0x85, 0x8E, 0xFF] {
        assert_eq!(s(&mut c, absent, &[], 0), [ABSENT], "{absent:02X}h: absent, MagicGate's and SECRMAN's commands among them");
    }
    assert_eq!(s(&mut c, scmd::FORBID_DVD, &[], 0), [FORBID_DVD_ANSWER]);
    let when = [0, 0x30, 0x15, 0x08, 0, 0x29, 0x02, 0x24];
    assert_eq!(s(&mut c, scmd::WRITE_RTC, &when, 0), [0]);
    assert_eq!(s(&mut c, scmd::READ_RTC, &[], IOP_HZ * 90), [0, 0x00, 0x17, 0x08, 0, 0x29, 0x02, 0x24], "ninety seconds on");
    let block: Vec<u8> = (0x11..0x21).collect();
    assert_eq!(s(&mut c, scmd::OPEN_CONFIG, &[1, 2, 1], 0), [0]);
    assert_eq!(s(&mut c, scmd::WRITE_CONFIG, &block, 0), [0]);
    assert_eq!(s(&mut c, scmd::CLOSE_CONFIG, &[], 0), [0]);
    s(&mut c, scmd::OPEN_CONFIG, &[0, 2, 1], 0);
    assert_eq!(s(&mut c, scmd::READ_CONFIG, &[], 0), block);
    assert_eq!(s(&mut c, scmd::READ_CONFIG, &[], 0), [0; 16]);
    assert_eq!(c.read8(reg::S_COMMAND), scmd::READ_CONFIG);
}

#[test]
fn the_tray_opens_closes_and_the_disc_is_detected_as_it_spins_up() {
    let mut c = Cdvd::new(Some(dvd(TOTAL, Some(LAYER0))));
    assert_eq!(s(&mut c, scmd::TRAY, &[0], 0), [0]);
    assert_eq!((c.read8(reg::DRIVE), c.read8(reg::DISK_TYPE), c.read8(reg::I_STAT)), (drive::TRAY_OPEN, 0, istat::EJECTED));
    assert_eq!(s(&mut c, scmd::TRAY, &[0], 0), [ABSENT], "an open tray does not open again");
    assert_eq!(s(&mut c, scmd::UPDATE_STICKY, &[], 0), [0]);
    assert_eq!(c.read8(reg::STICKY), drive::TRAY_OPEN);
    seek(&mut c, 10, 0);
    assert_eq!(finish(&mut c, PROMPT), PROMPT);
    assert_eq!(c.read8(reg::ERROR), NO_DISC);
    assert_eq!(s(&mut c, scmd::TRAY, &[1], 1000), [0]);
    assert_eq!((c.read8(reg::DRIVE), c.read8(reg::DISK_TYPE)), (0, disk_type::DETECTING));
    c.run(1000 + SPIN_UP / 2);
    assert_eq!(c.read8(reg::DISK_TYPE), disk_type::DETECTING_DUAL, "a dual-layer DVD while it spins up");
    c.run(1000 + SPIN_UP - 1);
    assert_eq!(c.read8(reg::DISK_TYPE), disk_type::DETECTING_DUAL);
    c.run(1000 + SPIN_UP);
    assert_eq!((c.read8(reg::DRIVE), c.read8(reg::DISK_TYPE)), (0x0A, 0x14));
    assert!(c.read8(reg::STICKY) & drive::TRAY_OPEN != 0, "the sticky status keeps the tray's opening");
    s(&mut c, scmd::UPDATE_STICKY, &[], 0);
    assert_eq!(c.read8(reg::STICKY), 0x0A);
    let mut c = Cdvd::new(Some(cd(false)));
    s(&mut c, scmd::TRAY, &[0], 0);
    assert!(c.insert(Some(dvd(300, None))));
    s(&mut c, scmd::TRAY, &[1], 0);
    c.run(SPIN_UP / 2);
    assert_eq!(c.read8(reg::DISK_TYPE), disk_type::DETECTING_DVD);
    assert!(!c.insert(None), "not with the tray closed");
}

#[test]
fn seeks_take_ps2teks_times_and_raise_the_interrupt_at_their_end() {
    let mut c = Cdvd::new(Some(dvd(1_000_000, None)));
    let b = block(2048, DVD_SPEED);
    let mut now = 0;
    for (to, expect) in [(0, PROMPT), (15, 15 * b), (31, FAST_SEEK), (31 + 14_763, FAST_SEEK), (31 + 14_763 + 14_764, FULL_SEEK), (0, FULL_SEEK)] {
        seek(&mut c, to, now);
        assert_eq!(c.read8(reg::N_STATUS), 0x80, "busy, not ready");
        assert_eq!(c.read8(reg::DRIVE), drive::SPINNING | drive::SEEKING);
        c.run(now + expect - 1);
        assert!(!c.irq() && c.read8(reg::N_STATUS) & 0x80 != 0, "seek to {to}: not before its time");
        let end = finish(&mut c, now + expect);
        assert_eq!(end - now, expect, "seek to {to}");
        assert_eq!((c.read8(reg::I_STAT), c.read8(reg::N_STATUS), c.read8(reg::DRIVE)), (istat::COMPLETE, 0x40, 0x0A));
        c.write8(reg::I_STAT, istat::COMPLETE, end);
        assert!(!c.irq());
        now = end + 100;
    }
    n(&mut c, ncmd::STOP, &[], now);
    let end = finish(&mut c, now + STOP);
    assert_eq!((end - now, c.read8(reg::DRIVE)), (STOP, 0));
    seek(&mut c, 100, end);
    assert_eq!(finish(&mut c, u64::MAX) - end, SPIN_UP + FAST_SEEK, "a stopped drive spins up first");
    let now = c.next_event().min(end + SPIN_UP + FAST_SEEK + 1);
    n(&mut c, ncmd::STANDBY, &[], now);
    assert_eq!(finish(&mut c, u64::MAX) - now, FAST_SEEK, "standby seeks to sector 0");
    n(&mut c, ncmd::PAUSE, &[], now);
    assert_eq!(finish(&mut c, u64::MAX) - now, PROMPT);
    let mut c = Cdvd::new(Some(cd(false)));
    let b = block(2048, CD_SPEED);
    seek(&mut c, 7, 0);
    assert_eq!(finish(&mut c, u64::MAX), 7 * b);
    seek(&mut c, 15, 10 * b);
    assert_eq!(finish(&mut c, u64::MAX) - 10 * b, FAST_SEEK, "a CD's contiguous bound is 8");
}

#[test]
fn a_dual_layer_seek_is_timed_across_the_radius() {
    let mut c = Cdvd::new(Some(dvd(TOTAL, Some(LAYER0))));
    let b = block(2048, DVD_SPEED);
    seek(&mut c, LAYER0 - 1, 0);
    let t = finish(&mut c, u64::MAX);
    seek(&mut c, LAYER0 + 3, t);
    assert_eq!(finish(&mut c, u64::MAX) - t, FAST_SEEK, "over to layer 1, three sectors in: a change of layer");
    let t = c.next_event().min(t + FAST_SEEK);
    seek(&mut c, LAYER0 + 13, t);
    assert_eq!(finish(&mut c, u64::MAX) - t, 10 * b, "ten sectors along layer 1, which lie at a smaller radius");
    assert_eq!(timing::radius(LAYER0 + 13, Some(LAYER0)), (1, LAYER0 - 14));
}

#[test]
fn a_dvd_reads_sector_for_sector_with_the_layer_break_placed() {
    for (sectors, layer0) in [(300, None), (TOTAL, Some(LAYER0))] {
        let image = dvd_image(sectors, layer0);
        let mut c = Cdvd::new(Some(dvd(sectors, layer0)));
        n(&mut c, ncmd::READ_DVD, &read_params(0, sectors, 0), 0);
        let blocks = read_all(&mut c, u64::MAX);
        assert_eq!(blocks.len(), sectors as usize);
        let b = block(FRAME, DVD_SPEED);
        for (lba, (t, f)) in blocks.iter().enumerate() {
            let lba = lba as u32;
            let layer = layer0.is_some_and(|l| lba >= l) as u8;
            let number = 0x03_0000 + lba - if layer == 1 { LAYER0 } else { 0 };
            assert_eq!(f.len(), FRAME);
            assert_eq!(f[0], layer | if layer0.is_some() { 0x20 } else { 0 }, "sector {lba}: the volume number and a dual layer disk's b29 (D-121, D-122)");
            assert_eq!(&f[1..4], &number.to_be_bytes()[1..], "sector {lba}: numbered from the volume's start (D-121)");
            assert!(ecma267::holds(f.as_slice().try_into().unwrap()), "sector {lba}: IED and EDC");
            assert_eq!(&f[12..2060], &image[lba as usize * 2048..(lba as usize + 1) * 2048], "sector {lba}");
            assert_eq!(*t, (lba as u64 + 1) * b, "sector {lba} one block after the last");
        }
        assert_eq!(c.read8(reg::I_STAT), istat::DATA_READY | istat::COMPLETE, "ps2tek: a read raises both");
        assert_eq!(c.read8(reg::DRIVE), 0x0A);
    }
}

#[test]
fn a_read_waits_for_its_buffer_to_be_taken() {
    let mut c = Cdvd::new(Some(dvd(300, None)));
    let b = block(FRAME, DVD_SPEED);
    n(&mut c, ncmd::READ_DVD, &read_params(100, 3, 0), 0);
    assert_eq!(c.next_event(), FAST_SEEK + b, "a seek from sector 0 to 100, then a block");
    c.run(FAST_SEEK + b);
    assert_eq!((c.dma_ready(), c.read8(reg::DRIVE)), (FRAME, drive::SPINNING | drive::READING));
    c.run(FAST_SEEK + 10 * b);
    assert_eq!(c.next_event(), u64::MAX, "the second sector waits on the buffer");
    let mut f = [0u8; FRAME];
    let late = FAST_SEEK + 10 * b;
    assert_eq!(c.dma_take(late, &mut f[..1000]), 1000);
    assert_eq!(c.dma_ready(), FRAME - 1000);
    c.dma_take(late, &mut f[1000..]);
    assert_eq!(&f[1..4], &[0x03, 0x00, 100]);
    assert_eq!(c.dma_ready(), FRAME, "the waiting sector is handed over at once");
    c.dma_take(late, &mut f);
    assert_eq!(&f[1..4], &[0x03, 0x00, 101]);
    assert_eq!(c.next_event(), late + b);
    c.run(late + b);
    assert!(c.read8(reg::N_STATUS) & 0x80 != 0, "busy until the last sector is taken");
    c.dma_take(late + b, &mut f);
    assert_eq!((c.read8(reg::N_STATUS), c.read8(reg::I_STAT)), (0x40, 0x03));
}

#[test]
fn a_cd_reads_its_block_sizes_and_its_audio() {
    let mut c = Cdvd::new(Some(cd(true)));
    let mut now = 0;
    for (size, from, len) in [(0u8, 24, 2048), (1, 24, 2328), (2, 12, 2340)] {
        let lba = 5 + size as u32;
        n(&mut c, ncmd::READ_CD, &read_params(lba, 2, size), now);
        let blocks = read_all(&mut c, u64::MAX);
        now = blocks.last().unwrap().0;
        for (k, (_, b)) in blocks.iter().enumerate() {
            assert_eq!(b.as_slice(), &cd_data(lba + k as u32)[from..from + len], "block size {size}, sector {}", lba + k as u32);
        }
        c.write8(reg::I_STAT, 0xFF, 0);
    }
    let t = 1 << 40;
    n(&mut c, ncmd::READ_CDDA, &read_params(CD_DATA + 3, 2, 0), t);
    let blocks = read_all(&mut c, u64::MAX);
    assert_eq!(blocks[0].1, cd_audio(3), "the audio track's raw sectors");
    assert_eq!(blocks[1].0 - blocks[0].0, block(2352, CD_SPEED));
    n(&mut c, ncmd::READ_DVD, &read_params(0, 1, 0), t);
    finish(&mut c, u64::MAX);
    assert_eq!((c.read8(reg::ERROR), c.read8(reg::N_STATUS) & 1), (WRONG_MEDIUM, 1), "a DVD read of a CD (D-126)");
    let mut c = Cdvd::new(Some(dvd(300, None)));
    n(&mut c, ncmd::READ_CDDA, &read_params(0, 1, 0), 0);
    finish(&mut c, u64::MAX);
    assert_eq!(c.read8(reg::ERROR), WRONG_MEDIUM, "audio from a DVD");
    n(&mut c, ncmd::READ_CD, &read_params(20, 1, 0), 0);
    assert_eq!(read_all(&mut c, u64::MAX)[0].1, user(20), "a CD read of a DVD gives its Main Data");
    assert_eq!(c.read8(reg::ERROR), 0);
}

#[test]
fn break_ends_a_read_and_get_toc_reports_the_disc() {
    let mut c = Cdvd::new(Some(dvd(300, None)));
    n(&mut c, ncmd::READ_DVD, &read_params(10, 50, 0), 0);
    c.run(FAST_SEEK);
    c.write8(reg::BREAK, 0, FAST_SEEK);
    assert_eq!((c.read8(reg::N_STATUS), c.read8(reg::I_STAT), c.next_event()), (0x40, istat::COMPLETE, u64::MAX));
    for (disc, structure, end, l0) in [(dvd(300, None), 0x01, 0x03_0000 + 299, 0), (dvd(TOTAL, Some(LAYER0)), 0x31, 0xFC_FC00 + (TOTAL - LAYER0 - 1), 0x03_0000 + LAYER0 - 1)] {
        let mut c = Cdvd::new(Some(disc));
        n(&mut c, ncmd::GET_TOC, &[], 0);
        let t = read_all(&mut c, u64::MAX);
        let f = &t[0].1;
        assert_eq!(t[0].0, block(FRAME, DVD_SPEED));
        assert_eq!((f[0], f[2], &f[4..8]), (0x01, structure, &[0, 3, 0, 0][..]), "ECMA-267 §26.5.1: version, structure, the Data Zone's start");
        assert_eq!(u32::from_be_bytes([0, f[9], f[10], f[11]]), end, "the Data Zone's last sector");
        assert_eq!(u32::from_be_bytes([0, f[13], f[14], f[15]]), l0, "layer 0's last, on OTP");
    }
    let mut c = Cdvd::new(Some(cd(true)));
    n(&mut c, ncmd::GET_TOC, &[], 0);
    let f = read_all(&mut c, u64::MAX).remove(0).1;
    assert_eq!((f[2], f[7], f[12], f[17], f[22]), (0xA0, 0x01, 0xA1, 0x02, 0xA2), "A0h's first track, A1h's last, A2h");
    assert_eq!((f[32], f[42]), (0x01, 0x02), "each track's pointer");
}

#[test]
fn an_image_opens_as_the_playstation_2s_disc_kinds() {
    let kind = |image: Vec<u8>, companions: Vec<Vec<u8>>| super::open(Kept::from(image), companions.into_iter().map(Kept::from).collect()).map(|m| m.kind());
    assert_eq!(kind(dvd_image(300, None), vec![]), Ok(Kind::Dvd), "an .iso is a DVD");
    assert_eq!(kind((0..CD_DATA).flat_map(cd_data).collect(), vec![]), Ok(Kind::Cd), "a raw track is a CD");
    let sheet = b"FILE \"d.bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\n".to_vec();
    assert_eq!(kind(sheet, vec![(0..CD_DATA).flat_map(cd_data).collect()]), Ok(Kind::Cd), "a sheet is a CD");
    let mut d = super::open(Kept::from(dvd_image(TOTAL, Some(LAYER0))), vec![]).unwrap();
    assert_eq!(d.dvd().map(|v| v.layer_break()), Some(Some(LAYER0)));
    assert!(kind(vec![0xA5; 5000], vec![]).is_err());
}

/// The S commands the player's BIOS sends at boot answer as PCSX2 does (D-348): 36h the console's region parameters,
/// absent without a market; 1Ah with the BIOS's parameters 01h; 22h ten zeros; 24h and the others PCSX2 acknowledges
/// 00h; a MagicGate command still absent (§5.6).
#[test]
fn the_boot_s_commands_answer_as_measured() {
    let mut c = Cdvd::new(None);
    assert_eq!(s(&mut c, 0x36, &[], 0), [ABSENT]);
    c.region = Some(REGION_EUROPE);
    assert_eq!(s(&mut c, 0x36, &[], 0), b"\x00\x08\x00EEengEE\x00\x00\x00\x00\x00");
    assert_eq!(s(&mut c, 0x1A, &[0x02, 0x1E, 0x41, 0x43], 0), [1]);
    assert_eq!(s(&mut c, 0x22, &[], 0), [0; 10]);
    assert_eq!(s(&mut c, 0x17, &[0x08], 0), [0; 9]);
    assert_eq!(s(&mut c, 0x1E, &[], 0), [0x00, 0x14, 0x00, 0x00, 0x00]);
    for cmd in scmd::ACKNOWLEDGED {
        assert_eq!(s(&mut c, cmd, &[1], 0), [0], "{cmd:02x}");
    }
    assert_eq!(s(&mut c, 0x84, &[], 0), [ABSENT]);
    assert_eq!((c.s_count[0x24], c.s_last[0x1A]), (1, ([0x02, 0x1E, 0x41, 0x43, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 4)));
}
