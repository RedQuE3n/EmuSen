//! The memory card's files through the harness's card (Diamond_OpenKernel_Phase2.md §14, §25.2): read and write in
//! asynchronous mode accepted with 0, as Spyro the Dragon's loops require, their data carried by the job, and a file
//! written in one mode read back in the other.

use diamond_kernel_harness::image;
use diamond_kernel_harness::machine::Machine;
use diamond_kernel_harness::sio0::Card;

const NAME: u32 = 0x8010_0000;
const SRC: u32 = 0x8011_0000;
const DST: u32 = 0x8012_0000;
const ASYNC: u32 = 0x8000;

/// A blank card in slot 1, the card calls initialised, the card formatted and a one-block file made on it.
fn with_a_file() -> Machine {
    let mut m = Machine::boot(&image::committed());
    m.rig.sio0.card = Some(Card::blank());
    m.b(0x4A, &[0]);
    m.a(0x55, &[]);
    let device = m.text(NAME + 0x100, "bu00:");
    assert_eq!(m.b(0x41, &[device]), 1, "format");
    m.text(NAME, "bu00:BASLUS-00000ASYNC");
    let fd = m.b(0x32, &[NAME, 1 << 16 | 0x200 | 3]);
    assert!(fd < 16, "created: {fd:X}");
    assert_eq!(m.b(0x36, &[fd]), fd);
    let pattern: Vec<u8> = (0..0x2000u32).map(|i| (i * 7 + 3) as u8).collect();
    m.rig.put_bytes(SRC, &pattern);
    m
}

fn src(m: &Machine, offset: u32, n: usize) -> Vec<u8> {
    m.rig.bytes(SRC + offset, n)
}

#[test]
fn an_asynchronous_write_is_accepted_with_0_and_reads_back_synchronously() {
    let mut m = with_a_file();
    let fd = m.b(0x32, &[NAME, ASYNC | 3]);
    assert!(fd < 16, "opened asynchronously: {fd:X}");
    let event = m.b(0x08, &[fd, 4, 0x2000, 0]);
    m.b(0x0C, &[event]);
    assert_eq!(m.b(0x35, &[fd, SRC, 0x400]), 0, "write: accepted");
    assert_eq!(m.b(0x33, &[fd, 0, 1]), 0x400, "the position past the bytes accepted");
    assert_eq!(m.b(0x35, &[fd, SRC + 0x400, 0x400]), 0, "a second write, after the first job");
    assert_eq!(m.b(0x36, &[fd]), fd, "close, after the job");
    assert_eq!(m.b(0x0B, &[event]), 1, "the handle's event, spec 4");
    let fd = m.b(0x32, &[NAME, 3]);
    assert_eq!(m.b(0x34, &[fd, DST, 0x800]), 0x800, "read synchronously");
    assert_eq!(m.rig.bytes(DST, 0x800), src(&m, 0, 0x800));
    m.b(0x36, &[fd]);
}

#[test]
fn a_synchronous_write_reads_back_asynchronously() {
    let mut m = with_a_file();
    let fd = m.b(0x32, &[NAME, 3]);
    assert_eq!(m.b(0x35, &[fd, SRC, 0x800]), 0x800, "written synchronously");
    m.b(0x36, &[fd]);
    let fd = m.b(0x32, &[NAME, ASYNC | 3]);
    assert_eq!(m.b(0x33, &[fd, 0x400, 0]), 0x400);
    assert_eq!(m.b(0x34, &[fd, DST, 0x400]), 0, "read: accepted");
    assert_eq!(m.b(0x36, &[fd]), fd);
    assert_eq!(m.rig.bytes(DST, 0x400), src(&m, 0x400, 0x400));
    let card = m.rig.sio0.card.as_ref().unwrap();
    for f in 0..16 {
        let frame = card.frame(f);
        assert_eq!(frame[..127].iter().fold(0, |a, b| a ^ b), frame[127], "frame {f}'s checksum");
    }
    assert_eq!(&card.frame(1)[..4], [0x51, 0, 0, 0], "the file's first block in use");
}

#[test]
fn the_asynchronous_calls_refuse_what_the_synchronous_ones_refuse() {
    let mut m = with_a_file();
    let fd = m.b(0x32, &[NAME, ASYNC | 3]);
    assert_eq!(m.b(0x34, &[fd, DST, 0x7F]), u32::MAX, "a length not a whole frame");
    assert_eq!(m.b(0x33, &[fd, 0x2000, 0]), 0x2000);
    assert_eq!(m.b(0x34, &[fd, DST, 0x80]), u32::MAX, "at the file's end");
    assert_eq!(m.b(0x36, &[fd]), fd);
}
