//! What no program can show a referee, checked on the units directly: the card's busy times, the reply of a command to a
//! busy card held, the motors, and the pressure rule.

use super::card::{self, MemoryCard, timing};
use super::pad::{DualShock2, Input, bit};
use super::*;

fn queue(s: &mut Sio2, port: u32, bytes: &[u8], t: u64) {
    s.write(CTRL, CTRL_RESET, t);
    s.write(SEND3, port | (bytes.len() as u32) << 18, t);
    s.write(SEND3 + 4, 0, t);
    for &b in bytes {
        s.write(FIFOIN, u32::from(b), t);
    }
    s.write(CTRL, CTRL_START, t);
}

fn finish(s: &mut Sio2, n: usize) -> Vec<u8> {
    let t = s.next_event();
    assert!(s.complete(t), "the queue ends at its time");
    (0..n).map(|_| s.read(FIFOOUT) as u8).collect()
}

#[test]
fn a_queue_ends_after_its_bytes_and_a_busy_card_holds_the_next_command() {
    let mut s = Sio2::default();
    s.slots[0].cards[0] = Some(MemoryCard::erased());
    queue(&mut s, 2, &[0x81, 0x11, 0, 0], 1000);
    assert_eq!(s.next_event(), 1000 + 4 * BYTE * IOP, "four bytes at the port's rate");
    assert_eq!(finish(&mut s, 4), [0xFF, 0xFF, card::ACK, card::TERMINATOR]);
    assert_eq!(s.recv1, CONNECTED);
    // An erase, then a probe sent at once: the probe waits out the erase's 2 ms.
    let t = 10_000;
    queue(&mut s, 2, &[0x81, 0x21, 0x20, 0, 0, 0, 0, 0, 0], t);
    finish(&mut s, 9);
    queue(&mut s, 2, &[0x81, 0x82, 0, 0], t);
    let erase_end = s.next_event();
    finish(&mut s, 4);
    // 2 ms of the EE's 294.912 MHz.
    assert_eq!(s.slots[0].cards[0].as_ref().unwrap().busy_until, erase_end + 589_824);
    queue(&mut s, 2, &[0x81, 0x11, 0, 0], erase_end);
    assert_eq!(s.next_event(), erase_end + 589_824 + 4 * BYTE * IOP, "held until the erase ends");
    assert_eq!((timing::PROGRAM, timing::LOAD), (73_728, 7_372), "250 µs and 25 µs");
}

#[test]
fn a_write_clears_bits_and_an_erase_sets_its_whole_block() {
    let mut c = MemoryCard::erased();
    let mut reply = [0u8; 134];
    let send = |c: &mut MemoryCard, bytes: &[u8], reply: &mut [u8]| c.exchange(bytes, &mut reply[..bytes.len()], 0);
    for v in [0x0Fu8, 0xF0] {
        send(&mut c, &[0x81, 0x22, 17, 0, 0, 0, 0, 0, 0], &mut reply);
        let mut w = vec![0x81, 0x42, 128];
        w.resize(134, v);
        for _ in 0..4 {
            send(&mut c, &w, &mut reply);
        }
        let mut w = vec![0x81, 0x42, 16];
        w.resize(134, v);
        send(&mut c, &w, &mut reply);
        send(&mut c, &[0x81, 0x81, 0, 0], &mut reply);
    }
    assert!(c.image[17 * 528..18 * 528].iter().all(|&b| b == 0), "0Fh then F0h without an erase leaves 00h");
    send(&mut c, &[0x81, 0x21, 20, 0, 0, 0, 0, 0, 0], &mut reply);
    send(&mut c, &[0x81, 0x82, 0, 0], &mut reply);
    assert!(c.image[16 * 528..32 * 528].iter().all(|&b| b == 0xFF), "page 20's block, pages 16 to 31, erased");
}

#[test]
fn the_authentication_commands_get_no_answer() {
    let mut s = Sio2::default();
    s.slots[0].cards[0] = Some(MemoryCard::erased());
    for c in [0xF0, 0xF3, 0xF7] {
        queue(&mut s, 2, &[0x81, c, 0, 0, 0], 0);
        assert_eq!(finish(&mut s, 5), [0xFF; 5]);
        assert_eq!(s.recv1, NO_DEVICE, "Sapphire_Plan.md §11 Q7(b)");
    }
}

#[test]
fn rumble_drives_both_motors_through_4dh_and_the_old_method_before_it() {
    let mut p = DualShock2::default();
    let mut r = [0u8; 9];
    p.exchange(&[0x01, 0x42, 0, 0x40, 0x01], &mut r[..5]);
    assert_eq!(p.small, 0xFF, "the one-motor method: 40h-7Fh and an odd byte");
    for cmd in [[0x01, 0x43, 0, 1, 0, 0, 0, 0, 0], [0x01, 0x4D, 0, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF], [0x01, 0x43, 0, 0, 0, 0, 0, 0, 0]] {
        p.exchange(&cmd, &mut r);
    }
    p.exchange(&[0x01, 0x42, 0, 0x01, 0xC0], &mut r[..5]);
    assert_eq!((p.small, p.large), (0xFF, 0xC0));
    p.exchange(&[0x01, 0x42, 0, 0x00, 0x00], &mut r[..5]);
    assert_eq!((p.small, p.large), (0, 0));
}

#[test]
fn pressure_reads_ffh_for_a_pressed_button_and_00h_for_a_released_one() {
    let i = Input::from_digital(bit::CROSS | bit::RIGHT | bit::R2, [0x80; 4]);
    assert_eq!(i.pressure, [0xFF, 0, 0, 0, 0, 0, 0xFF, 0, 0, 0, 0, 0xFF]);
}
