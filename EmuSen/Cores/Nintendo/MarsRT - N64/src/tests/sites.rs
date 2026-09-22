//! Each of C#'s wait sites met with a draw or a load still pending on a held drain: the access must wait for it and see what the list at once left. See Mars_Native.md §5.6.3.

use std::time::{Duration, Instant};

use super::threads::{FRAMEBUFFER, LIST, TEXTURE, WIDTH, at_once, hand_over, scene, state, threaded};
use crate::machine::Machine;
use crate::memory::bus_access::map::{AI_BASE, SI_BASE, SP_DMEM_BASE, SP_REGISTERS_BASE};
use crate::memory::si::TRANSFER_CYCLES;
use crate::vi::scan::{self, Scanout};

const FILL: u32 = 0x2408_0005;
const HELD: Duration = Duration::from_millis(300);

/// Rows 0 to 7 of the frame buffer filled with a word that is also `addiu t0, zero, 5`.
fn fill_list() -> Vec<u64> {
    vec![
        (0x2F << 56) | (3 << 52),
        (0x3F << 56) | (2 << 51) | (((WIDTH - 1) as u64) << 32) | FRAMEBUFFER as u64,
        (0x2D << 56) | (((WIDTH << 2) as u64) << 12) | (240 << 2),
        (0x37 << 56) | FILL as u64,
        (0x36 << 56) | ((((WIDTH - 1) << 2) as u64) << 44) | ((7u64 << 2) << 32),
        0x29 << 56,
    ]
}

/// Both machines set up alike, the list handed to each, the threaded one's drain held; the access, released from another thread after `HELD`.
fn after_a_held_list(setup: impl Fn(&mut Machine), list: &[u64], access: impl Fn(&mut Machine) -> Vec<u8>) {
    let (mut once, mut drain) = (at_once(), threaded());
    setup(&mut once);
    setup(&mut drain);
    hand_over(&mut once, list, LIST);
    drain.bus.dp.threads.as_deref().unwrap().hold();
    hand_over(&mut drain, list, LIST);

    let resume = drain.bus.dp.threads.as_deref().unwrap().resumer();
    let watchdog = std::thread::spawn(move || {
        std::thread::sleep(HELD);
        resume();
    });
    let started = Instant::now();
    let seen = access(&mut drain);
    let waited = started.elapsed();
    watchdog.join().unwrap();
    let want = access(&mut once);

    assert!(waited >= HELD - Duration::from_millis(50), "the access went ahead of the list after {waited:?}");
    assert!(seen == want, "the access saw what the list had not yet left");
    drain.join_rdp();
    assert!(state(&once) == state(&drain), "the machines part after the list");
}

/// The processor in kernel mode at a KSEG0 address.
fn at(m: &mut Machine, pc: u32) {
    m.cpu.pc = 0xFFFF_FFFF_0000_0000 | pc as u64;
    m.cpu.next_pc = m.cpu.pc.wrapping_add(4);
    m.cpu.cop0_written(&m.bus);
}

fn program(m: &mut Machine, words: &[u32]) {
    for (i, &w) in words.iter().enumerate() {
        m.bus.write32(0x1000 + i as u32 * 4, w);
    }
    at(m, 0x8000_1000);
}

#[test]
fn site_0_a_bus_read_waits_for_the_draw() {
    after_a_held_list(|_| {}, &fill_list(), |m| m.bus.read32(FRAMEBUFFER + 0x100).to_be_bytes().to_vec());
}

#[test]
fn site_1_a_bus_write_waits_for_the_load_that_reads_its_bytes() {
    after_a_held_list(|_| {}, &scene(0x0102_0304, true, 16, false), |m| {
        m.bus.write32(TEXTURE + 0x40, 0xDEAD_BEEF);
        vec![]
    });
}

#[test]
fn site_2_a_load_waits_for_the_draw() {
    let setup = |m: &mut Machine| program(m, &[0x3C09_8020, 0x8D28_0100]);
    after_a_held_list(setup, &fill_list(), |m| {
        m.run_steps(2);
        m.cpu.gpr[8].to_be_bytes().to_vec()
    });
}

#[test]
fn site_3_a_store_waits_for_the_load_that_reads_its_bytes() {
    let setup = |m: &mut Machine| {
        program(m, &[0x3C09_8030, 0xAD2A_0040]);
        m.cpu.gpr[10] = 0xDEAD_BEEF;
    };
    after_a_held_list(setup, &scene(0x0102_0304, true, 16, false), |m| {
        m.run_steps(2);
        vec![]
    });
}

#[test]
fn site_4_a_fetch_waits_for_the_draw() {
    let setup = |m: &mut Machine| {
        for i in 0..64 {
            m.bus.write32(FRAMEBUFFER + i * 4, 0x2408_0001);
        }
        at(m, 0x8000_0000 | FRAMEBUFFER);
    };
    after_a_held_list(setup, &fill_list(), |m| {
        m.run_steps(1);
        m.cpu.gpr[8].to_be_bytes().to_vec()
    });
}

#[test]
fn site_5_the_idle_test_waits_for_the_draw() {
    let setup = |m: &mut Machine| {
        m.bus.write32(FRAMEBUFFER, 0x1000_FFFF);
        m.bus.write32(FRAMEBUFFER + 4, 0);
        at(m, 0x8000_0000 | FRAMEBUFFER);
    };
    after_a_held_list(setup, &fill_list(), |m| {
        let cap = m.bus.cycles + 1_000_000;
        vec![m.cpu.try_idle(&mut m.bus, cap, true) as u8]
    });
}

#[test]
fn site_6_the_serial_interfaces_read_waits_for_the_draw() {
    after_a_held_list(|_| {}, &fill_list(), |m| {
        m.bus.write32(SI_BASE, FRAMEBUFFER + 0x100);
        m.bus.write32(SI_BASE + 0x10, 0);
        m.bus.pif_ram.to_vec()
    });
}

#[test]
fn site_6_the_serial_interfaces_landing_waits_for_the_draw_beneath_it() {
    let setup = |m: &mut Machine| {
        m.bus.write32(SI_BASE, FRAMEBUFFER + 0x100);
        m.bus.write32(SI_BASE + 0x04, 0);
    };
    after_a_held_list(setup, &fill_list(), |m| {
        m.bus.tick(TRANSFER_CYCLES + 1);
        vec![]
    });
}

#[test]
fn site_7_the_audio_interface_waits_for_the_draw() {
    let setup = |m: &mut Machine| {
        m.bus.write32(AI_BASE + 0x10, 1520);
        m.bus.write32(AI_BASE + 0x08, 1);
    };
    after_a_held_list(setup, &fill_list(), |m| {
        m.bus.write32(AI_BASE, FRAMEBUFFER + 0x100);
        m.bus.write32(AI_BASE + 0x04, 16);
        m.bus.tick(400_000);
        m.bus.ai.drain(64).iter().flat_map(|s| s.to_be_bytes()).collect()
    });
}

/// A 256-column sixteen-bit picture of the frame buffer's first rows.
fn picture(m: &mut Machine) {
    let r = &mut m.bus.vi.registers;
    r[0] = 2 | (3 << 8);
    r[1] = FRAMEBUFFER;
    r[2] = WIDTH;
    r[6] = 525;
    r[9] = (108 << 16) | (108 + 256);
    r[10] = (34 << 16) | (34 + 8 * 2);
    r[12] = 0x400;
    r[13] = 0x400;
}

#[test]
fn site_8_the_scan_at_once_waits_for_the_draw() {
    let setup = |m: &mut Machine| picture(m);
    after_a_held_list(setup, &fill_list(), |m| {
        let mut out = Scanout::default();
        scan::present_now(&mut m.bus, &mut out);
        out.frame
    });
}

#[test]
fn site_8_the_deferred_capture_waits_for_the_draw() {
    let setup = |m: &mut Machine| picture(m);
    after_a_held_list(setup, &fill_list(), |m| {
        let mut out = Scanout::default();
        scan::present_deferred(&mut m.bus, &mut out);
        out.join();
        out.frame
    });
}

#[test]
fn site_9_a_cheats_read_waits_for_the_draw_and_its_write_for_the_load() {
    after_a_held_list(|_| {}, &fill_list(), |m| vec![m.bus.cheat_read8(FRAMEBUFFER + 0x101)]);
    after_a_held_list(|_| {}, &scene(0x0102_0304, true, 16, false), |m| {
        m.bus.cheat_write8(TEXTURE + 0x41, 0x5A);
        vec![]
    });
}

#[test]
fn site_11_a_transfer_into_the_signal_processor_waits_for_the_draw() {
    after_a_held_list(|_| {}, &fill_list(), |m| {
        m.bus.write32(SP_REGISTERS_BASE, 0);
        m.bus.write32(SP_REGISTERS_BASE + 0x04, FRAMEBUFFER + 0x100);
        m.bus.write32(SP_REGISTERS_BASE + 0x08, 0x40 - 1);
        m.bus.sp_dmem[..0x40].to_vec()
    });
}

#[test]
fn site_11_a_transfer_out_of_the_signal_processor_waits_for_the_draw_beneath_it() {
    let setup = |m: &mut Machine| m.bus.write32(SP_DMEM_BASE, 0x1234_5678);
    after_a_held_list(setup, &fill_list(), |m| {
        m.bus.write32(SP_REGISTERS_BASE, 0);
        m.bus.write32(SP_REGISTERS_BASE + 0x04, FRAMEBUFFER + 0x100);
        m.bus.write32(SP_REGISTERS_BASE + 0x0C, 0x40 - 1);
        vec![]
    });
}
