//! Events, threads, the heaps, programs and the configuration; the timer, GPU and pad entry points; and the card
//! calls with no card and the absent character sets (Diamond_OpenKernel.md §4 and §5, Diamond_OpenKernel_Phase2.md).

use diamond_kernel_harness::image;
use diamond_kernel_harness::machine::{Machine, STACK};
use diamond_kernel_harness::psexe::Exe;

fn booted() -> Machine {
    Machine::boot(&image::committed())
}

#[test]
fn events_open_disabled_and_follow_their_modes() {
    let mut m = booted();
    let tcb_events = m.rig.word(0x8000_0120);
    let e = m.b(0x08, &[0xF300_0001, 0x20, 0x2000, 0]);
    assert_eq!(e, 0xF100_0000);
    let evcb = tcb_events;
    assert_eq!(
        (m.rig.word(evcb), m.rig.word(evcb + 4), m.rig.word(evcb + 8), m.rig.word(evcb + 12)),
        (0xF300_0001, 0x1000, 0x20, 0x2000)
    );
    assert_eq!(m.b(0x0A, &[e]), 0, "WaitEvent on a disabled event");
    m.b(0x07, &[0xF300_0001, 0x20]);
    assert_eq!(m.b(0x0B, &[e]), 0, "a disabled event is not delivered");
    assert_eq!(m.b(0x0C, &[e]), 1);
    m.b(0x07, &[0xF300_0001, 0x40]);
    assert_eq!(m.b(0x0B, &[e]), 0, "spec must match too");
    m.b(0x07, &[0xF300_0001, 0x20]);
    assert_eq!(m.rig.word(evcb + 4), 0x4000);
    m.b(0x20, &[0xF300_0001, 0x20]);
    assert_eq!(m.rig.word(evcb + 4), 0x2000, "UnDeliverEvent: busy again");
    m.b(0x07, &[0xF300_0001, 0x20]);
    assert_eq!(m.b(0x0A, &[e]), 1, "WaitEvent on a ready event");
    assert_eq!(m.rig.word(evcb + 4), 0x2000);
    assert_eq!(m.b(0x0D, &[e]), 1);
    assert_eq!(m.rig.word(evcb + 4), 0x1000);
    assert_eq!(m.b(0x09, &[e]), 1);
    assert_eq!(m.rig.word(evcb + 4), 0);
    assert_eq!(m.b(0x09, &[0xF100_0333]), 1, "CloseEvent returns 1 for any handle");
    // A callback event: its function runs on delivery and it never becomes ready.
    m.code(0x8015_0000, "lui t0, 0x8017\nlw t1, 0(t0)\nnop\naddiu t1, t1, 1\njr ra\nsw t1, 0(t0)");
    let c = m.b(0x08, &[0xF300_0002, 4, 0x1000, 0x8015_0000]);
    m.b(0x0C, &[c]);
    m.b(0x07, &[0xF300_0002, 4]);
    m.b(0x07, &[0xF300_0002, 4]);
    assert_eq!(m.rig.word(0x8017_0000), 2);
    assert_eq!(m.b(0x0B, &[c]), 0);
    assert_eq!(m.c(0x04, &[]), 1, "get_free_EvCB_slot: the first free");
    for _ in 1..16 {
        m.b(0x08, &[0xF300_0003, 1, 0x2000, 0]);
    }
    assert_eq!(m.b(0x08, &[0xF300_0003, 1, 0x2000, 0]), u32::MAX, "sixteen EvCBs");
}

#[test]
fn wait_event_returns_once_an_interrupt_delivers() {
    let mut m = booted();
    let e = m.b(0x08, &[0xF200_0000, 2, 0x2000, 0]);
    m.b(0x0C, &[e]);
    m.b(0x04, &[0]);
    m.rig.put(0x8017_0000, e);
    let exe = Exe::program(
        "addiu sp, sp, -32\nlui t0, 0x8017\nlw a0, 0(t0)\naddiu t2, zero, 0xB0\njalr t2\naddiu t1, zero, 0x0A\nor s0, v0, zero\ndone: j done\nnop",
        0x8001_0000,
    );
    exe.sideload(&mut m);
    for _ in 0..2000 {
        m.step();
    }
    assert_eq!(m.cpu.regs.pc >> 20, 0x9FC, "waiting inside the kernel");
    m.rig.raise(4);
    m.run_to(0x8001_0000 + 4 * 8, 100_000).unwrap();
    assert_eq!(m.cpu.regs.r[16], 1);
    assert_eq!(m.exceptions, [0]);
}

#[test]
fn threads_open_change_and_return_with_1() {
    let mut m = booted();
    // The thread: marks 'T', then changes back to the boot thread.
    m.code(
        0x8015_0000,
        "lui t0, 0x8017\naddiu t1, zero, 0x54\nsb t1, 1(t0)\nlui a0, 0xFF00\naddiu t2, zero, 0xB0\njalr t2\naddiu t1, zero, 0x10\nloop: j loop\nnop",
    );
    let h = m.b(0x0E, &[0x8015_0000, 0x801E_0000, 0x4242]);
    assert_eq!(h, 0xFF00_0001);
    assert_eq!(m.c(0x05, &[]), 2, "get_free_TCB_slot");
    let tcb1 = m.rig.word(0x8000_0110) + 0xC0;
    assert_eq!((m.rig.word(tcb1), m.rig.word(tcb1 + 4)), (0x4000, 0x1000));
    assert_eq!(
        (
            m.rig.word(tcb1 + 0x88),
            m.rig.word(tcb1 + 8 + 4 * 29),
            m.rig.word(tcb1 + 8 + 4 * 30),
            m.rig.word(tcb1 + 8 + 4 * 28)
        ),
        (0x8015_0000, 0x801E_0000, 0x801E_0000, 0x4242)
    );
    m.run_program(
        "addiu sp, sp, -32\nlui a0, 0xFF00\nori a0, a0, 1\naddiu t2, zero, 0xB0\njalr t2\naddiu t1, zero, 0x10\n\
         lui t0, 0x8017\naddiu t1, zero, 0x4D\nsb t1, 2(t0)\nor s0, v0, zero\ndone: j done\nnop",
        100_000,
    );
    assert_eq!(m.rig.bytes(0x8017_0001, 2), b"TM");
    assert_eq!(m.cpu.regs.r[16], 1, "ChangeTh returns 1 to the thread changed back to");
    assert_eq!(m.rig.word(m.rig.word(0x8000_0108)), m.rig.word(0x8000_0110), "TCB 0 current again");
    assert_eq!(m.b(0x0F, &[h]), 1);
    assert_eq!(m.rig.word(tcb1), 0x1000);
}

#[test]
fn the_heap_needs_init_heap_and_marks_freed_blocks_at_buf_minus_4() {
    let mut m = booted();
    assert_eq!(m.a(0x33, &[16]), 0, "no heap before InitHeap");
    m.a(0x39, &[0x8010_0000, 0x1000]);
    let p = m.a(0x33, &[10]);
    assert!(p >= 0x8010_0004 && p % 4 == 0);
    assert_eq!(m.rig.word(p - 4) & 1, 0);
    let q = m.a(0x33, &[8]);
    assert!(q >= p + 12);
    m.a(0x34, &[p]);
    assert_eq!(m.rig.word(p - 4) & 1, 1, "free sets [buf-4] bit 0");
    assert_eq!(m.a(0x33, &[4]), p, "the freed block reused");
    assert_eq!(m.a(0x33, &[0x2000]), 0, "too large");
    m.rig.put_bytes(0x8010_0800, &[0xAA; 32]);
    let z = m.a(0x37, &[4, 3]);
    assert_eq!(m.rig.bytes(z, 12), [0; 12], "calloc zeroes");
    m.rig.put_bytes(q, b"12345678");
    let r = m.a(0x38, &[q, 16]);
    assert_eq!(m.rig.bytes(r, 8), b"12345678");
    assert_eq!(m.rig.word(q - 4) & 1, 1, "realloc freed the old block");
    let k = m.b(0x00, &[0x40]);
    assert!((0xA000_E000..0xA001_0000).contains(&k), "kernel memory: {k:X}");
}

#[test]
fn qsort_bsearch_and_lsearch_with_a_callback() {
    let mut m = booted();
    m.a(0x39, &[0x8012_0000, 0x1000]);
    m.code(0x8015_0000, "lw t0, 0(a0)\nlw t1, 0(a1)\nnop\njr ra\nsubu v0, t0, t1");
    let base = 0x8010_0000;
    for (i, v) in [5u32, 3, 9, 1, 7].iter().enumerate() {
        m.rig.put(base + 4 * i as u32, *v);
    }
    m.a(0x31, &[base, 5, 4, 0x8015_0000]);
    let sorted: Vec<u32> = (0..5).map(|i| m.rig.word(base + 4 * i)).collect();
    assert_eq!(sorted, [1, 3, 5, 7, 9]);
    m.rig.put(0x8010_0100, 7);
    assert_eq!(m.a(0x36, &[0x8010_0100, base, 5, 4, 0x8015_0000]), base + 12);
    assert_eq!(m.a(0x35, &[0x8010_0100, base, 5, 4, 0x8015_0000]), base + 12);
    m.rig.put(0x8010_0100, 4);
    assert_eq!(m.a(0x36, &[0x8010_0100, base, 5, 4, 0x8015_0000]), 0);
}

#[test]
fn exec_runs_a_loaded_program_with_its_stack_and_gp_and_returns_1() {
    let mut m = booted();
    let header = 0x8011_0000;
    let child = 0x8012_0000;
    m.code(child, "lui t0, 0x8017\nsw sp, 0(t0)\nsw gp, 4(t0)\nsw a0, 8(t0)\nsw a1, 12(t0)\njr ra\nnop");
    m.rig.put_bytes(0x8013_0000, &[0xEE; 16]);
    for (o, v) in [(0x00, child), (0x04, 0x1234), (0x18, 0x8013_0000), (0x1C, 16), (0x20, 0x801E_0000), (0x24, 0x100)] {
        m.rig.put(header + o, v);
    }
    m.cpu.regs.r[28] = 0x5555;
    assert_eq!(m.a(0x43, &[header, 7, 8]), 1);
    let seen: Vec<u32> = (0..4).map(|i| m.rig.word(0x8017_0000 + 4 * i)).collect();
    assert_eq!(seen, [0x801E_0100, 0x1234, 7, 8]);
    assert_eq!(m.rig.bytes(0x8013_0000, 16), [0; 16], "the memfill region zeroed");
    assert_eq!((m.cpu.regs.r[29], m.cpu.regs.r[28]), (STACK, 0x5555), "the caller's SP and GP back");
    assert_eq!(
        m.rig.word(header + 0x28),
        diamond_kernel_harness::machine::SENTINEL,
        "the caller's RA in headerbuf[28h]"
    );
}

#[test]
fn load_test_load_and_load_exec_fail_without_a_device_that_holds_files() {
    let mut m = booted();
    m.text(0x8010_0000, "pcdrv:\\PSX.EXE;1");
    assert_eq!(m.a(0x41, &[0x8010_0000, 0x8011_0000]), 0);
    assert_eq!(m.a(0x42, &[0x8010_0000, 0x8011_0000]), 0);
}

#[test]
fn flush_cache_clears_every_tag_and_restores_bcc_and_sr() {
    let mut m = booted();
    let before = m.rig.isolated_writes;
    m.cpu.regs.sr = 0x4000_0401;
    m.a(0x44, &[]);
    assert_eq!(m.rig.isolated_writes - before, 256);
    assert_eq!(m.rig.bcc, 0x0001_E988);
    assert_eq!(m.cpu.regs.sr, 0x4000_0401);
    let bcc: Vec<u32> = m.rig.writes.iter().filter(|w| w.0 == 0xFFFE_0130).map(|w| w.2).rev().take(2).collect();
    assert_eq!(bcc, [0x0001_E988, 0x804], "tag mode, then restored");
}

#[test]
fn configuration_system_information_and_set_mem() {
    let mut m = booted();
    m.a(0x9D, &[0x8010_0000, 0x8010_0004, 0x8010_0008]);
    let conf: Vec<u32> = (0..3).map(|i| m.rig.word(0x8010_0000 + 4 * i)).collect();
    assert_eq!(conf, [0x10, 4, 0x801F_FF00]);
    m.a(0x9C, &[8, 2, 0x801F_0000]);
    m.a(0x9D, &[0x8010_0000, 0x8010_0004, 0x8010_0008]);
    let conf: Vec<u32> = (0..3).map(|i| m.rig.word(0x8010_0000 + 4 * i)).collect();
    assert_eq!(conf, [8, 2, 0x801F_0000]);
    assert_eq!((m.rig.word(0x8000_0114), m.rig.word(0x8000_0124)), (2 * 0xC0, 8 * 0x1C));
    assert_eq!(m.a(0xB4, &[0]), 0x2026_1011);
    assert_eq!(m.a(0xB4, &[1]), 3);
    let v = m.a(0xB4, &[2]);
    assert_eq!(m.rig.string(v), "Diamond open kernel, phase 2");
    assert_eq!(m.rig.string(0xBFC0_0108), "EmuSen");
    assert_eq!(m.a(0xB4, &[5]), 2048);
    assert_eq!((m.a(0xB4, &[7]), m.a(0xB4, &[9]), m.a(0xB4, &[0x10])), (0x400, 0x200, 0));
    m.a(0x9F, &[2]);
    assert_eq!(m.rig.word(0x8000_0060), 2);
    assert_eq!(m.rig.writes.last().map(|w| (w.0, w.2)), Some((0x1F80_1060, 0x888)), "a single 2 MiB bank");
    m.a(0x9F, &[8]);
    assert_eq!((m.rig.word(0x8000_0060), m.rig.writes.last().unwrap().2), (8, 0xB88));
}

#[test]
fn timers_gpu_and_pads_through_their_documented_ports() {
    let mut m = booted();
    assert_eq!(m.b(0x02, &[1, 0x1234, 0x1010]), 1);
    let timer: Vec<(u32, u32)> = m.rig.writes.iter().rev().take(3).map(|w| (w.0, w.2)).collect();
    assert_eq!(timer, [(0x1F80_1114, 0x159), (0x1F80_1118, 0x1234), (0x1F80_1114, 0)]);
    assert_eq!(m.b(0x02, &[3, 0, 0]), 0);
    assert_eq!(m.b(0x03, &[3]), 0);
    assert_eq!(m.b(0x04, &[3]), 0, "vblank: enabled, but 0 returned");
    assert_eq!(m.rig.i_mask, 1);
    assert_eq!(m.b(0x04, &[2]), 1);
    assert_eq!(m.rig.i_mask, 1 | 1 << 6);
    assert_eq!(m.b(0x05, &[3]), 1);
    assert_eq!(m.b(0x06, &[0]), 1);
    assert_eq!(m.a(0x4D, &[]), 0x1C00_0000);
    m.cpu.regs.r[2] = 0x7777;
    m.a(0x48, &[0x0300_0001]);
    assert_eq!(m.cpu.regs.r[2], 0x7777, "SendGP1Command leaves v0");
    assert_eq!(m.rig.writes.last().map(|w| (w.0, w.2)), Some((0x1F80_1814, 0x0300_0001)));
    assert_eq!(m.a(0x49, &[0xE100_0000]), 0);
    assert_eq!(m.rig.writes.last().map(|w| (w.0, w.2)), Some((0x1F80_1810, 0xE100_0000)));
    assert_eq!(m.a(0x4C, &[]), 0x1F80_1814);
    // Pads: zero-filled buffers, then each vblank reads SIO0, where this bus answers no /ACK, so both read FFh.
    m.rig.put_bytes(0x8010_0000, &[0x55; 0x44]);
    m.b(0x12, &[0x8010_0000, 0x22, 0x8010_0022, 0x22]);
    assert_eq!(m.rig.bytes(0x8010_0000, 0x44), [0; 0x44]);
    m.b(0x13, &[]);
    assert_eq!(m.rig.word(m.rig.word(0x8000_0100) + 16) & 0xFFFF_FFFF != 0, true, "PadCardIrq at priority 2");
    m.rig.i_mask = 1;
    let exe = Exe::program("loop: j loop\nnop", 0x8001_0000);
    exe.sideload(&mut m);
    m.rig.raise(0);
    for _ in 0..100_000 {
        m.step();
    }
    assert_eq!((m.rig.bytes(0x8010_0000, 1), m.rig.bytes(0x8010_0022, 1)), (vec![0xFF], vec![0xFF]));
    assert_eq!(m.rig.i_stat, 0, "acknowledged by the pad handler");
    assert_eq!(m.b(0x5B, &[0]), 1, "ChangeClearPAD's old flag");
    m.b(0x14, &[]);
    assert_eq!(m.rig.word(m.rig.word(0x8000_0100) + 16), 0);
    assert_eq!(m.b(0x15, &[5, 0x8010_0100, 0, 0]), 0);
    assert_eq!(m.b(0x15, &[0x2000_0000, 0x8010_0100, 0, 0]), 2);
    assert_eq!(m.b(0x16, &[]), 0xFFFF_FFFF, "no pad answered");
    assert_eq!(m.rig.word(0x8010_0100), 0xFFFF_FFFF);
}

#[test]
fn the_card_calls_with_no_card_and_the_absent_character_sets() {
    let mut m = booted();
    assert_eq!(m.b(0x4F, &[0, 0, 0x8010_1000]), 1, "_card_read: queued");
    assert_eq!(m.b(0x5D, &[0]), 0x11, "_card_wait carries it out: failed, no card");
    assert_eq!(m.b(0x5C, &[0]), 0x11, "_card_status");
    assert_eq!(m.b(0x4F, &[0, 0x401, 0x8010_1000]), 0, "a sector past 400h");
    assert_eq!(m.b(0x51, &[0x8140]), u32::MAX, "Krom2RawAdd: no character sets");
    m.c(0x1A, &[1]);
    assert_eq!(m.c(0x1D, &[]), 1);
    assert_eq!(m.a(0x57, &[]), 0, "return 0");
}
