//! The reset path to the shell's entry, and the state psx-spx's memory map and control blocks describe.

use blackmoon_r3000::Size;
use diamond_kernel_harness::image;
use diamond_kernel_harness::machine::{Machine, SHELL};

fn booted() -> Machine {
    Machine::boot(&image::committed())
}

fn jump_target(m: &Machine, at: u32, register: u32) -> u32 {
    let (hi, lo, jr, nop) = (m.rig.word(at), m.rig.word(at + 4), m.rig.word(at + 8), m.rig.word(at + 12));
    assert_eq!(hi >> 16, 0x3C00 | register, "LUI at {at:X}");
    assert_eq!(lo >> 16, 0x2400 | register << 5 | register, "ADDIU at {at:X}");
    assert_eq!(jr, register << 21 | 8, "JR at {at:X}");
    assert_eq!(nop, 0);
    (hi << 16).wrapping_add(lo as u16 as i16 as u32)
}

#[test]
fn the_memory_control_and_cache_are_set_to_the_documented_values_before_ram_is_used() {
    let m = booted();
    let io: Vec<(u32, u32)> =
        m.rig.writes.iter().filter(|w| (0x1F80_1000..0x1F80_1064).contains(&w.0)).map(|w| (w.0, w.2)).collect();
    assert_eq!(io[0], (0x1F80_1060, 0x0B88), "the RAM's size register first");
    for (port, value) in [
        (0x1F80_1000, 0x1F00_0000),
        (0x1F80_1004, 0x1F80_2000),
        (0x1F80_1008, 0x0013_243F),
        (0x1F80_100C, 0x0000_3022),
        (0x1F80_1010, 0x0013_243F),
        (0x1F80_1014, 0x2009_31E1),
        (0x1F80_1018, 0x0002_0843),
        (0x1F80_101C, 0x0007_0777),
        (0x1F80_1020, 0x0003_1125),
    ] {
        assert!(io.contains(&(port, value)), "{port:X} = {value:X}");
    }
    assert_eq!(m.rig.bcc, 0x0001_E988, "BCC as psx-spx marks it usually set");
    assert!(m.rig.isolated_writes >= 256, "the I-cache's 256 tags cleared");
}

#[test]
fn the_shell_is_entered_in_kernel_mode_with_bev_clear_and_interrupts_enabled_but_masked() {
    let m = booted();
    assert_eq!(m.cpu.regs.pc, SHELL);
    assert_eq!(m.cpu.regs.sr, 0x0000_0401);
    assert_eq!(m.cpu.regs.r[29], 0x801F_FF00, "SYSTEM.CNF's default STACK");
    assert_eq!((m.rig.i_mask, m.rig.i_stat), (0, 0));
    assert!(m.exceptions.is_empty());
}

#[test]
fn the_ram_map_of_the_first_100h_bytes() {
    let m = booted();
    assert_eq!(m.rig.word(0x8000_0060), 2, "RAM size in megabytes");
    assert_eq!(m.rig.word(0x8000_0064), 0);
    assert_eq!(m.rig.word(0x8000_0068), 0xFF);
    let handler = jump_target(&m, 0x8000_0080, 26);
    assert_eq!(m.rig.bytes(0, 16), m.rig.bytes(0x8000_0080, 16), "00000000h holds a copy of the exception vector");
    assert_ne!(m.rig.word(0) & 0xFFFF, 0, "the halfword at 0 is non-zero");
    for v in [0xA0, 0xB0, 0xC0] {
        let target = jump_target(&m, 0x8000_0000 | v, 8);
        assert_eq!(target >> 20, 0x9FC, "{v:X}h jumps into the kernel's ROM");
    }
    let mut m = m;
    let c0 = m.b(0x56, &[]);
    let named = m.rig.word(c0 + 4 * 6);
    assert_eq!(named >> 20, 0x9FC, "C(06h) names the handler's copy in the ROM, where a game's patch finds the ROM");
    assert!((0x8000_0500..0x8000_DF80).contains(&handler), "the vector jumps to its copy in the kernel's RAM");
    assert_eq!(m.rig.bytes(named, 0xD0), m.rig.bytes(handler, 0xD0), "the same handler (§25.3)");
    let b0 = m.b(0x57, &[]);
    assert_eq!(b0, c0 + 0x200, "C(80h) and up read B(00h) and up");
    assert!(c0 > 0x8000_0500 && c0 < 0x8000_DF80);
    assert_eq!(m.rig.word(0xBFC0_0104), 3, "the console type of a retail console");
    assert_eq!(m.rig.word(0xBFC0_0100), 0x2026_1011);
}

#[test]
fn the_table_of_tables_and_the_control_blocks() {
    let m = booted();
    let entry = |o: u32| (m.rig.word(0x8000_0100 + o), m.rig.word(0x8000_0104 + o));
    let (excb, excb_size) = entry(0x00);
    let (pcb, pcb_size) = entry(0x08);
    let (tcb, tcb_size) = entry(0x10);
    let (evcb, evcb_size) = entry(0x20);
    let (fcb, fcb_size) = entry(0x40);
    let (dcb, dcb_size) = entry(0x50);
    assert_eq!(
        (excb_size, pcb_size, tcb_size, evcb_size, fcb_size, dcb_size),
        (0x20, 4, 4 * 0xC0, 0x10 * 0x1C, 0x10 * 0x2C, 0x0A * 0x50)
    );
    for a in [excb, pcb, tcb, evcb] {
        assert!((0xA000_E000..0xA001_0000).contains(&a), "{a:X} in the kernel's control-block memory");
    }
    for a in [fcb, dcb] {
        assert!((0x8000_0500..0x8000_DF80).contains(&a), "{a:X} in the kernel's variables");
    }
    for o in [0x18, 0x28, 0x30, 0x38, 0x48] {
        assert_eq!(entry(o), (0, 0), "{o:X}h unused");
    }
    assert_eq!(m.rig.word(pcb), tcb, "the PCB names TCB 0, the boot thread");
    let status: Vec<u32> = (0..4).map(|i| m.rig.word(tcb + 0xC0 * i)).collect();
    assert_eq!(status, [0x4000, 0x1000, 0x1000, 0x1000]);
    assert!((0..16).all(|i| m.rig.word(evcb + 0x1C * i + 4) == 0), "every EvCB free");
    let chain = |p: u32| {
        let mut n = 0;
        let mut e = m.rig.word(excb + 8 * p);
        while e != 0 {
            assert_ne!(m.rig.word(e + 8), 0);
            e = m.rig.word(e);
            n += 1;
        }
        n
    };
    assert_eq!(
        [chain(0), chain(1), chain(2), chain(3)],
        [1, 4, 0, 1],
        "syscall; vblank and the timers; none; the default handler"
    );
    assert_eq!(m.rig.string(m.rig.word(dcb)), "tty");
    assert_eq!((m.rig.word(dcb + 4), m.rig.word(dcb + 8)), (3, 1), "the DUART TTY's flags and sector size");
    assert_eq!(m.rig.string(m.rig.word(dcb + 0x0C)), "CONSOLE");
    assert_eq!((m.rig.word(fcb), m.rig.word(fcb + 0x2C)), (1, 2), "std_in and std_out open");
    assert_eq!((m.rig.word(fcb + 0x1C), m.rig.word(fcb + 0x28 + 0x2C)), (dcb, 1));
}

#[test]
fn every_table_entry_points_into_the_kernel_and_the_shared_stubs_are_shared() {
    let mut m = booted();
    let c0 = m.b(0x56, &[]);
    let b0 = m.b(0x57, &[]);
    let table = |m: &Machine, t: char, n: u32| match t {
        'A' => m.rig.word(0x8000_0200 + 4 * n),
        'B' => m.rig.word(b0 + 4 * n),
        _ => m.rig.word(c0 + 4 * n),
    };
    let calls = diamond_kernel_harness::calls::parse(&diamond_kernel_harness::calls::source());
    assert_eq!(calls.len(), 0xC0 + 0x100 + 0x80);
    let mut shared = std::collections::HashMap::new();
    for c in &calls {
        let f = table(&m, c.table, c.number);
        assert_eq!(f >> 20, 0x9FC, "{}({:02X}h) {:08X}", c.table, c.number, f);
        if c.status == 'N' {
            let first = *shared.entry(c.name.clone()).or_insert(f);
            assert_eq!(first, f, "{}({:02X}h) {}", c.table, c.number, c.name);
        }
    }
    assert_eq!(
        m.rig.writes.iter().filter(|w| w.1 == Size::Byte && w.0 == 0x1F80_2023).count(),
        0,
        "the boot prints nothing"
    );
}
