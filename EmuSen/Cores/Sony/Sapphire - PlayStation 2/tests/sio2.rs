//! Stage S25's gate, part A (Sapphire_Native.md §30.8): the project's SIO2 programs, bare IOP programs driving the
//! DualShock 2, the memory card, SIO2's DMA channels and the multitap through SIO2's registers, run on
//! `blackmoon-r3000` with Sapphire's IOP bus inside the board and graded by the records the documents give.

mod s21;
mod s25;

use sapphire::board::Board;
use sapphire::clock::{self, IOP};
use sapphire::sio2::card::MemoryCard;
use sapphire::sio2::pad::{DualShock2, Input, bit};

/// Runs an IOP program in a board until its done word, `setup` given the board first; the board and the expected record.
fn run(p: s21::Program, setup: impl FnOnce(&mut Board)) -> (Board, Vec<Option<u32>>) {
    let name = p.name;
    let (source, expected) = p.finish();
    let words = s21::assemble(&source);
    let mut b = Board::new(clock::NTSC);
    b.iop.bus.load(s21::ORIGIN, &words);
    b.iop.bus.load(0x80, &s21::vector());
    setup(&mut b);
    b.iop.enter(s21::K1 | s21::ORIGIN);
    let mut spr = vec![0u8; 16 << 10];
    let limit = IOP * 2_000_000_000;
    while b.iop.bus.word(s21::DONE_AT) != s21::DONE {
        assert!(b.now < limit, "{name}: no done word by EE cycle {limit}, IOP pc {:08x}", b.iop.cpu.regs.pc);
        let t = b.now + IOP * 200_000;
        b.run_to(t, &mut spr);
    }
    (b, expected)
}

/// Runs and grades a program; the board, for further checks.
fn grade(p: s21::Program, setup: impl FnOnce(&mut Board)) -> Board {
    let name = p.name;
    let (b, expected) = run(p, setup);
    let mut wrong = Vec::new();
    for (i, want) in expected.iter().enumerate() {
        let got = b.iop.bus.word(s21::RECORD + 4 * i as u32);
        if let Some(w) = want
            && *w != got
        {
            wrong.push(format!("word {i}: {got:08x}, expected {w:08x}"));
        }
    }
    assert!(wrong.is_empty(), "{name}: {} of {} words differ, first {:?}", wrong.len(), expected.len(), &wrong[..wrong.len().min(8)]);
    b
}

/// Both card slots holding an erased card, as the referee's two cards are.
fn two_cards(b: &mut Board) {
    for s in &mut b.iop.bus.sio2.slots {
        s.cards[0] = Some(MemoryCard::erased());
    }
}

#[test]
fn the_dualshock_2_answers_identification_configuration_and_modes_as_documented() {
    let b = grade(s25::pad_program(), |_| {});
    let pad = b.iop.bus.sio2.slots[0].pads[0].as_ref().unwrap();
    assert!(!pad.analog && !pad.config && !pad.locked, "digital again, out of configuration mode, unlocked by the last 44h");
    assert_eq!(pad.rumble, [0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF]);
}

#[test]
fn the_pressure_bytes_follow_the_buttons_held_in_the_documented_order() {
    grade(s25::pressure_program(), |b| {
        let pad = b.iop.bus.sio2.slots[0].pads[0].as_mut().unwrap();
        pad.input = Input::from_digital(bit::RIGHT | bit::CROSS | bit::R2, [0x80; 4]);
    });
}

#[test]
fn the_card_probes_reads_writes_and_erases_pages_with_their_spare_bytes() {
    let b = grade(s25::card_program(), two_cards);
    let card = b.iop.bus.sio2.slots[0].cards[0].as_ref().unwrap();
    assert!(card.dirty);
    assert!(card.image.iter().all(|&x| x == 0xFF), "every page written was erased again");
    assert!(b.now >= 3 * clock::EE_HZ / 500, "three erases of 2 ms each");
}

#[test]
fn sio2s_dma_channels_carry_a_queue_in_and_its_replies_out() {
    grade(s25::dma_program(), two_cards);
}

#[test]
fn a_multitap_answers_its_frame_and_routes_the_slot_it_selects() {
    grade(s25::tap_program(), |b| {
        let s = &mut b.iop.bus.sio2.slots[0];
        s.tap = true;
        s.cards[0] = Some(MemoryCard::erased());
        s.pads[0] = Some(DualShock2::default());
    });
}

#[test]
fn a_command_nothing_acknowledges_ends_at_its_peripheral_byte() {
    let b = grade(s25::empty_program(), |b| {
        assert!(b.iop.bus.sio2.slots.iter().all(|s| s.cards.iter().all(Option::is_none)), "both card slots empty");
    });
    assert!(b.iop.bus.sio2.slots[1].pads[0].is_none(), "controller port 2 empty");
}
