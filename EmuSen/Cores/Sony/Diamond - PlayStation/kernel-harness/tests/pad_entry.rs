//! The stretch behind B(5Bh)'s table entry, which Mega Man Legends' pad library stores words into and calls
//! (Diamond_OpenKernel_Phase2.md §27): the entry still ChangeClearPAD, every call into the stretch returning with
//! every register as it was, the stores landing in the ROM, and the stretch's form.

use diamond_kernel_harness::image;
use diamond_kernel_harness::machine::{Machine, SENTINEL, STACK};

/// The entry's own code, the stretch's end and the distance between its returns, in bytes from the entry (§27.2).
const CODE: u32 = 0x10;
const END: u32 = 0xA00;
const SPACING: u32 = 0x100;
/// The offsets the game calls, and the runs of words it stores at (§27.1).
const CALLED: [u32; 3] = [0x7A0, 0x884, 0x894];
const STORED: [(u32, u32); 4] = [(0x3D8, 4), (0x4E0, 4), (0x594, 11), (0x62C, 9)];
const JR_RA: u32 = 0x03E0_0008;

/// The booted kernel and the address its B table gives for B(5Bh), found as the game finds it.
fn entry() -> (Machine, u32) {
    let mut m = Machine::boot(&image::committed());
    let table = m.b(0x57, &[]);
    let e = m.rig.word(table + 4 * 0x5B);
    (m, e)
}

/// A call at `address` with every register set to a value of its own; the steps it took to come back.
fn call_at(m: &mut Machine, address: u32) -> usize {
    let r = &mut m.cpu.regs;
    for i in 1..32 {
        r.r[i] = 0x5A00_0000 + 0x0101 * i as u32;
    }
    (r.r[29], r.r[31], r.hi, r.lo) = (STACK, SENTINEL, 0x4849_4849, 0x4C4F_4C4F);
    r.pc = address;
    r.branch = Default::default();
    r.load = Default::default();
    let (writes, exceptions, ram) = (m.rig.writes.len(), m.exceptions.len(), m.rig.ram.clone());
    let steps = m.run_to(SENTINEL, 10_000).unwrap_or_else(|e| panic!("a call at {address:08X}: {e}"));
    let r = &m.cpu.regs;
    for i in 1..32 {
        let was = match i {
            29 => STACK,
            31 => SENTINEL,
            _ => 0x5A00_0000 + 0x0101 * i as u32,
        };
        assert_eq!(r.r[i], was, "a call at {address:08X}: R{i}");
    }
    assert_eq!((r.hi, r.lo), (0x4849_4849, 0x4C4F_4C4F), "a call at {address:08X}: HI and LO");
    assert_eq!((m.rig.writes.len(), m.exceptions.len()), (writes, exceptions), "a call at {address:08X}: stores, exceptions");
    assert!(m.rig.ram == ram, "a call at {address:08X}: RAM");
    steps
}

#[test]
fn b_5bh_s_table_entry_is_still_change_clear_pad() {
    let (mut m, e) = entry();
    assert_eq!(e >> 20, 0x9FC, "the entry names the ROM: {e:08X}");
    assert_eq!(m.b(0x5B, &[0]), 1, "the flag's default, returned as the old value");
    assert_eq!(m.b(0x5B, &[1]), 0);
    // The entry called at the address the table gives, as a program that copied it would.
    let r = &mut m.cpu.regs;
    (r.r[4], r.r[29], r.r[31], r.pc) = (0, STACK, SENTINEL, e);
    r.branch = Default::default();
    r.load = Default::default();
    m.run_to(SENTINEL, 100).expect("ChangeClearPAD returns");
    assert_eq!(m.cpu.regs.r[2], 1);
    assert_eq!(m.b(0x5B, &[1]), 0);
}

#[test]
fn a_call_at_an_offset_the_game_calls_returns_at_once_with_every_register_unchanged() {
    let (mut m, e) = entry();
    for offset in CALLED {
        let steps = call_at(&mut m, e + offset);
        assert!(steps <= (SPACING / 4) as usize, "a call at {offset:X}h past the entry took {steps} steps");
    }
    assert_eq!(m.b(0x5B, &[0]), 1, "ChangeClearPAD's flag was not touched");
}

#[test]
fn a_call_at_any_word_of_the_stretch_returns_within_one_spacing() {
    let (mut m, e) = entry();
    let mut longest = 0;
    for offset in (CODE..END).step_by(4) {
        longest = longest.max(call_at(&mut m, e + offset));
    }
    assert_eq!(longest, (SPACING / 4) as usize + 1, "the longest slide: from a return's delay slot through the next spacing");
}

#[test]
fn the_game_s_stores_land_in_the_rom_and_change_nothing() {
    let (mut m, e) = entry();
    assert!(STORED.iter().all(|(at, n)| CODE <= *at && at + 4 * n <= END), "every store lies in the stretch");
    assert!(CALLED.iter().all(|at| CODE <= *at && at + 8 <= END), "every call lies in the stretch");
    let mut source = format!("lui v0, 0x{:x}\nori v0, v0, 0x{:x}\n", e >> 16, e & 0xFFFF);
    let mut stores = 0;
    for (at, n) in STORED {
        for k in 0..n {
            source += &format!("sw ra, 0x{:x}(v0)\n", at + 4 * k);
            stores += 1;
        }
    }
    source += "done: j done\nnop\n";
    let before = m.rig.bios.clone();
    let writes = m.rig.writes.len();
    m.run_program(&source, 1000);
    let made = &m.rig.writes[writes..];
    assert_eq!(made.len(), stores);
    assert!(made.iter().all(|w| (0x1FC0_0000..0x1FC8_0000).contains(&w.0)), "every store's address is the ROM's");
    assert!(m.rig.bios == before, "the ROM is as it was");
    for offset in CALLED {
        call_at(&mut m, e + offset);
    }
    assert_eq!(m.b(0x5B, &[0]), 1);
}

#[test]
fn the_stretch_is_zero_fill_with_a_return_at_the_end_of_each_spacing() {
    let (m, e) = entry();
    assert_eq!(e % 16, 0, "the entry starts a cache line, so that a return and its delay slot share one");
    for offset in (CODE..END).step_by(4) {
        let word = m.rig.word(e + offset);
        let returns = offset % SPACING == SPACING - 8;
        assert_eq!(word, if returns { JR_RA } else { 0 }, "the word {offset:X}h past the entry");
    }
    assert_eq!((m.rig.word(e + END), m.rig.word(e + END + 4)), (JR_RA, 0), "one more return after the last");
}
