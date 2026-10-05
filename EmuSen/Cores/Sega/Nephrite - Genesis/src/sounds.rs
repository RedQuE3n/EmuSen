//! The sound chips as the 68000's programs see them: the YM2612's busy flag and timers polled through `$A04000`,
//! the counts the loops leave in RAM checked against the rules of `ym2612.rs` (Nephrite_Native.md §18).

use crate::machine::Machine;
use crate::media::Media;
use crate::pictures::program;

/// RESET released and the Z80's bus requested and granted, then `lea $A04000,a2`.
const Z80_BUS: [u16; 16] = [0x33FC, 0x0100, 0x00A1, 0x1200, 0x33FC, 0x0100, 0x00A1, 0x1100, 0x0839, 0x0000, 0x00A1, 0x1100, 0x66F6, 0x45F9, 0x00A0, 0x4000];

/// `setup`, then a loop of `addq.w #1,d1; btst #bit,offset(a2); bne/beq` until the bit changes, the count stored
/// at `$FF0000`.
fn polls(setup: &[u16], offset: u16, bit: u16, until_set: bool) -> u16 {
    let mut code = Z80_BUS.to_vec();
    code.extend_from_slice(&[0x7200]);
    code.extend_from_slice(setup);
    // addq.w #1,d1 (4); btst #bit,d16(a2) (16); bne.s / beq.s back (10): 30 clocks a round.
    code.extend_from_slice(&[0x5241, 0x082A, bit, offset, if until_set { 0x67F6 } else { 0x66F6 }]);
    code.extend_from_slice(&[0x33C1, 0x00FF, 0x0000, 0x60FE]);
    let image = program(&[0x8004, 0x8104, 0x8F02], &[], &code);
    let mut m = Machine::new(&image, Media::read(&image));
    while m.frames < 3 {
        m.advance();
    }
    u16::from_be_bytes([m.genesis.hw.wram[0], m.genesis.hw.wram[1]])
}

/// The busy flag reads set for 192 of the 68000's clocks after a data write, about six rounds of 30 clocks, and only
/// at port 0 of the discrete chip; an address write leaves it clear.
#[test]
fn a_program_sees_the_busy_flag_for_192_clocks_at_port_0_only() {
    let write = [0x14BC, 0x002A, 0x157C, 0x0080, 0x0001];
    let at_0 = polls(&write, 0, 7, false);
    assert!((6..=8).contains(&at_0), "{at_0} polls at port 0");
    assert_eq!(polls(&write, 2, 7, false), 1, "port 2 shows no busy flag on the discrete chip");
    assert_eq!(polls(&[0x14BC, 0x002A], 0, 7, false), 1, "an address write sets no busy flag");
}

/// Timer A at `$3F0` counts sixteen samples, the first at the sample after it is loaded: it overflows 15 to 16
/// samples (2,160 to 2,304 of the 68000's clocks, 72 to 77 rounds of 30) after the load.
#[test]
fn a_program_sees_timer_a_overflow_after_its_period() {
    let start = [0x14BC, 0x0024, 0x157C, 0x00FC, 0x0001, 0x14BC, 0x0025, 0x157C, 0x0000, 0x0001, 0x14BC, 0x0027, 0x157C, 0x0005, 0x0001];
    let n = polls(&start, 0, 0, true);
    assert!((72..=77).contains(&n), "{n} polls until timer A's flag");
}
