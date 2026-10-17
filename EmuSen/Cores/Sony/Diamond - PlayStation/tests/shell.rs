//! The open kernel's shell (Diamond_Shell.md §3, §4.2): the machine on the open kernel with no disc, cards built from
//! psx-spx's layout, presses scripted, and what the shell did read from its TTY lines and the cards' bytes.

use diamond::kernel::{self, Kernel};
use diamond::machine::{Config, Gpu, Machine};
use diamond::media::{Market, Media};

#[path = "support/card.rs"]
mod card;
#[path = "support/iso.rs"]
#[allow(dead_code)]
mod iso;

const UP: u32 = 1 << 4;
const DOWN: u32 = 1 << 6;
const L1: u32 = 1 << 10;
const R1: u32 = 1 << 11;
const TRIANGLE: u32 = 1 << 12;
const CIRCLE: u32 = 1 << 13;
const CROSS: u32 = 1 << 14;

/// The open kernel with an empty drive and the two cards (None for an empty slot), run until the shell has read both.
fn shell(one: Option<&[u8]>, two: Option<&[u8]>) -> Machine {
    let media = Media { kind: diamond::media::Kind::Empty, market: Some(Market::Americas) };
    let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.pad"), None], &[], kernel::open_kernel().map(|k| (k, Kernel::Open)));
    if let Some(c) = one {
        m.insert_card(0, Some(c));
    }
    if let Some(c) = two {
        m.insert_card(1, Some(c));
    }
    until(&mut m, "shell: card 2", 2000);
    m
}

fn tty(m: &Machine) -> String {
    String::from_utf8_lossy(&m.board.tty.text).into_owned()
}

/// Frames run until the TTY holds `needle` `n` times or more.
fn until_n(m: &mut Machine, needle: &str, n: usize, frames: usize) {
    for _ in 0..frames {
        if tty(m).matches(needle).count() >= n {
            return;
        }
        m.advance();
    }
    panic!("no {needle:?} in {frames} frames; the TTY:\n{}", tty(m));
}

fn until(m: &mut Machine, needle: &str, frames: usize) {
    until_n(m, needle, 1, frames)
}

/// A press: held four frames, released four.
fn press(m: &mut Machine, bits: u32) {
    m.board.sio0.set_buttons(0, bits);
    for _ in 0..4 {
        m.advance();
    }
    m.board.sio0.set_buttons(0, 0);
    for _ in 0..4 {
        m.advance();
    }
}

fn presses(m: &mut Machine, list: &[u32]) {
    for &b in list {
        press(m, b);
    }
}

fn card_of(m: &Machine, n: usize) -> Vec<u8> {
    m.card_in(n).unwrap().data.to_vec()
}

const AMBER: &str = "BASLUS-00001AMBER";
const BASALT: &str = "BASLUS-00002BASALT";

fn two_saves() -> Vec<u8> {
    let c = card::with_save(card::formatted(), AMBER, "Amber Saga".as_bytes(), 1, 1, 0xA1);
    card::with_save(c, BASALT, &[0x82, 0x61, 0x82, 0x81, 0x82, 0x93, 0x88, 0x9F], 3, 2, 0xB2)
}

/// §3.1: the drive and both cards read and reported, Home first; Memory Cards lists the saves, and a save copied to the other card
/// arrives whole under its name, its chain written as psx-spx's layout gives it, the source unchanged.
#[test]
fn a_save_is_copied_to_the_other_card_whole() {
    let one = two_saves();
    let mut m = shell(Some(&one), Some(&card::formatted()));
    let lines: Vec<String> = tty(&m).lines().map(|l| l.trim_end().to_owned()).filter(|l| l.starts_with("shell:")).collect();
    assert_eq!(lines[..3], ["shell: drive empty", "shell: card 1 ready", "shell: card 2 ready"], "{}", tty(&m));
    presses(&mut m, &[CROSS, DOWN, CROSS, CROSS]);
    until(&mut m, "shell: copy", 1500);
    assert!(tty(&m).contains(&format!("shell: copy 1 {BASALT} 2 ok")), "{}", tty(&m));
    let two = card_of(&m, 1);
    let first = card::entry(&two, BASALT).expect("the copy's directory frame");
    assert_eq!((two[first * 128], u32::from_le_bytes(two[first * 128 + 4..first * 128 + 8].try_into().unwrap())), (0x51, 0x4000));
    let (src, dst) = (card::chain(&one, card::entry(&one, BASALT).unwrap()), card::chain(&two, first));
    assert_eq!(dst.len(), 2);
    for (s, d) in src.iter().zip(&dst) {
        assert!(one[s * 8192..(s + 1) * 8192] == two[d * 8192..(d + 1) * 8192], "block {s} copied to {d}");
    }
    assert_eq!(card_of(&m, 0), one, "the source card unchanged");
}

/// §3.1: a copy refused with words: no card in the other slot, the same name there, too few free blocks.
#[test]
fn a_copy_is_refused_when_the_other_card_cannot_take_it() {
    let mut m = shell(Some(&two_saves()), None);
    presses(&mut m, &[CROSS, CROSS, CROSS]);
    until(&mut m, "shell: copy", 600);
    assert!(tty(&m).contains(&format!("shell: copy 1 {AMBER} 2 not-ready")), "{}", tty(&m));

    let mut m = shell(Some(&two_saves()), Some(&two_saves()));
    presses(&mut m, &[CROSS, CROSS, CROSS]);
    until(&mut m, "shell: copy", 600);
    assert!(tty(&m).contains(&format!("shell: copy 1 {AMBER} 2 same-name")), "{}", tty(&m));

    let mut full = card::formatted();
    for k in 0..7 {
        full = card::with_save(full, &format!("BASLUS-0001{k}FULL"), b"Full", 1, 2, k as u8);
    }
    full = card::with_save(full, "BASLUS-00019LAST", b"Last", 1, 1, 9);
    let mut m = shell(Some(&two_saves()), Some(&full));
    presses(&mut m, &[CROSS, DOWN, CROSS, CROSS]);
    until(&mut m, "shell: copy", 600);
    assert!(tty(&m).contains(&format!("shell: copy 1 {BASALT} 2 too-full")), "{}", tty(&m));
    assert_eq!(card_of(&m, 1), full, "nothing written");
}

/// §3.2: a delete asks first with the cursor on Cancel; Cancel deletes nothing; Delete marks the save's frames deleted
/// (A1h, A3h) and keeps its blocks, as the kernel's erase does.
#[test]
fn a_delete_asks_first_and_cancel_deletes_nothing() {
    let mut m = shell(Some(&two_saves()), None);
    presses(&mut m, &[CROSS, CROSS, DOWN, CROSS, CROSS]);
    for _ in 0..120 {
        m.advance();
    }
    assert!(!tty(&m).contains("shell: delete"), "Cancel chosen first: {}", tty(&m));
    assert_eq!(card_of(&m, 0), two_saves());
    presses(&mut m, &[DOWN, CROSS, DOWN, CROSS, UP, CROSS]);
    until(&mut m, "shell: delete", 600);
    assert!(tty(&m).contains(&format!("shell: delete 1 {BASALT} ok")), "{}", tty(&m));
    let c = card_of(&m, 0);
    let first = card::entry(&c, BASALT).unwrap();
    assert_eq!(c[first * 128], 0xA1);
    assert_eq!(c[card::chain(&c, first)[1] * 128], 0xA3);
}

/// §3.2: Triangle's Format asks first; formatted, the card holds no save and reads ready.
#[test]
fn a_format_asks_first_and_erases_the_card() {
    let mut m = shell(Some(&two_saves()), None);
    presses(&mut m, &[CROSS, TRIANGLE, CROSS, CROSS]);
    for _ in 0..60 {
        m.advance();
    }
    assert!(!tty(&m).contains("shell: format"), "{}", tty(&m));
    presses(&mut m, &[TRIANGLE, CROSS, UP, CROSS]);
    until(&mut m, "shell: format 1 ok", 1200);
    until_n(&mut m, "shell: card 1 ready", 2, 1200);
    let c = card_of(&m, 0);
    assert!((1..16).all(|b| c[b * 128] == 0xA0), "every block free");
    assert_eq!(&c[..2], b"MC");
}

/// §3.1: a title frame whose icon flag is not 11h-13h is listed as damaged and can be deleted all the same; a card
/// in the second slot is reached with R1, and Circle and Start go back.
#[test]
fn a_damaged_save_is_listed_and_deleted_and_the_second_slot_is_reached() {
    let damaged = card::with_save(card::formatted(), "BASLUS-00003BROKEN", b"Broken", 0, 1, 0x33);
    let mut m = shell(None, Some(&damaged));
    presses(&mut m, &[CROSS, R1, CIRCLE, CROSS, R1, CROSS, DOWN, CROSS, UP, CROSS]);
    until(&mut m, "shell: delete", 600);
    assert!(tty(&m).contains("shell: delete 2 BASLUS-00003BROKEN ok"), "{}", tty(&m));
    presses(&mut m, &[L1, CROSS]);
    for _ in 0..30 {
        m.advance();
    }
    assert_eq!(tty(&m).matches("shell: copy").count(), 0, "no save in Slot 1 to act on");
}

/// §3.2's repeat: Down held moves once on the press and again after 24 frames and every 5 after, so that held 40
/// frames it reaches the fifth save, which is the one deleted.
#[test]
fn a_held_direction_repeats() {
    let mut c = card::formatted();
    for k in 0..6 {
        c = card::with_save(c, &format!("BASLUS-0002{k}SAVE"), format!("Save {k}").as_bytes(), 1, 1, k as u8);
    }
    let mut m = shell(Some(&c), None);
    press(&mut m, CROSS);
    m.board.sio0.set_buttons(0, DOWN);
    for _ in 0..40 {
        m.advance();
    }
    m.board.sio0.set_buttons(0, 0);
    for _ in 0..4 {
        m.advance();
    }
    presses(&mut m, &[CROSS, DOWN, CROSS, UP, CROSS]);
    until(&mut m, "shell: delete", 600);
    assert!(tty(&m).contains("shell: delete 1 BASLUS-00024SAVE ok"), "{}", tty(&m));
}

/// §3.1: each save's icon is in VRAM, its CLUT row at (960, 256 + k) and its first frame's texels at (896, 16k).
#[test]
fn the_icons_are_held_in_vram() {
    let one = two_saves();
    let mut m = shell(Some(&one), None);
    press(&mut m, CROSS);
    let at = |m: &Machine, x: usize, y: usize| u16::from_le_bytes([m.board.gpu.vram.bytes[2 * (y * 1024 + x)], m.board.gpu.vram.bytes[2 * (y * 1024 + x) + 1]]);
    let t = card::chain(&one, card::entry(&one, BASALT).unwrap())[0] * 8192;
    for i in 0..16 {
        assert_eq!(at(&m, 960 + i, 257), u16::from_le_bytes([one[t + 0x60 + 2 * i], one[t + 0x61 + 2 * i]]), "CLUT entry {i}");
    }
    for i in 0..4 {
        assert_eq!(at(&m, 896 + 8 + i, 16), u16::from_le_bytes([one[t + 3 * 128 + 2 * i], one[t + 3 * 128 + 1 + 2 * i]]), "frame 3's first row");
    }
}

/// §1.3's hand-over: a disc put in ends the shell, which says so, and the kernel boots it.
#[test]
fn a_disc_put_in_ends_the_shell_and_is_booted() {
    let mut m = shell(Some(&card::formatted()), None);
    let words = emusen_mips::assemble("lui $t0, 0x800F\nori $t0, $t0, 0xFFF0\nlui $t1, 0xD1A0\nori $t1, $t1, 0xD00E\nsw $t1, 0($t0)\nloop:\nj loop\nnop\n", 0x8001_0000, &blackmoon_r3000::dialect(None)).unwrap();
    let exe = diamond::exe::build(&words);
    let disc = iso::disc(&[iso::Entry::File("SYSTEM.CNF", b"BOOT = cdrom:\\PROG.EXE;1\r\n"), iso::Entry::File("PROG.EXE", &exe)]);
    m.set_lid(true, None);
    for _ in 0..30 {
        m.advance();
    }
    m.set_lid(false, Some(disc));
    until(&mut m, "shell: boot", 1200);
    for _ in 0..600 {
        m.advance();
        if m.board.ram_word(0xFFFF0) == 0xD1A0_D00E {
            return;
        }
    }
    panic!("the disc was not booted; the TTY:\n{}", tty(&m));
}

/// Not a test of behaviour: the screens written as PPM pictures under the scratch folder named by DIAMOND_SHELL_LOOK.
#[test]
fn look() {
    let Some(dir) = std::env::var_os("DIAMOND_SHELL_LOOK") else { return };
    let save = |m: &Machine, name: &str| {
        let mut out = format!("P6 320 240 255\n").into_bytes();
        for y in 0..240 {
            for x in 0..320 {
                let p = u16::from_le_bytes([m.board.gpu.vram.bytes[2 * (y * 1024 + x)], m.board.gpu.vram.bytes[2 * (y * 1024 + x) + 1]]);
                for c in [p & 31, p >> 5 & 31, p >> 10 & 31] {
                    out.push((c << 3 | c >> 2) as u8);
                }
            }
        }
        std::fs::write(std::path::Path::new(&dir).join(name), out).unwrap();
    };
    let mut c = two_saves();
    for k in 0..7 {
        c = card::with_save(c, &format!("BASLUS-0002{k}SAVE"), format!("Save number {k} of a long title here").as_bytes(), (k % 3 + 1) as u8, 1, k as u8);
    }
    c = card::with_save(c, "BASLUS-00003BROKEN", b"Broken", 0, 1, 0x33);
    let mut m = shell(Some(&c), Some(&card::formatted()));
    save(&m, "home.ppm");
    press(&mut m, CROSS);
    save(&m, "cards.ppm");
    press(&mut m, CROSS);
    save(&m, "actions.ppm");
    presses(&mut m, &[DOWN, CROSS]);
    save(&m, "confirm.ppm");
    presses(&mut m, &[CIRCLE, R1]);
    save(&m, "slot2.ppm");
}
