//! Each of C#'s wait sites met with a draw or a load still pending on a held drain: the access must wait for it and see what the list at once left. See Mars_Native.md §5.6.3.

use std::sync::atomic::Ordering::Relaxed;
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
    fill_rows(0, 7)
}

/// The rows from `top` to `bottom` filled so.
fn fill_rows(top: u64, bottom: u64) -> Vec<u64> {
    vec![
        (0x2F << 56) | (3 << 52),
        (0x3F << 56) | (2 << 51) | (((WIDTH - 1) as u64) << 32) | FRAMEBUFFER as u64,
        (0x2D << 56) | (((WIDTH << 2) as u64) << 12) | (240 << 2),
        (0x37 << 56) | FILL as u64,
        (0x36 << 56) | ((((WIDTH - 1) << 2) as u64) << 44) | ((bottom << 2) << 32) | (top << 2),
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

/// Both machines set up alike and the list handed to each, the threaded one's drain held; the access, which must not wait for the list,
/// must see what the list at once leaves, and the machines must agree once the drain is released. `page` must be marked, or the case was not reached.
fn ahead_of_a_held_list(setup: impl Fn(&mut Machine), list: &[u64], page: usize, access: impl Fn(&mut Machine) -> Vec<u8>) {
    let (mut once, mut drain) = (at_once(), threaded());
    setup(&mut once);
    setup(&mut drain);
    hand_over(&mut once, list, LIST);
    drain.bus.dp.threads.as_deref().unwrap().hold();
    hand_over(&mut drain, list, LIST);
    let marks = &drain.bus.dp.marks;
    assert!(marks.marks[page >> 12].load(Relaxed) != 0, "the access's page is not marked, so the case was not reached");

    let resume = drain.bus.dp.threads.as_deref().unwrap().resumer();
    let watchdog = std::thread::spawn(move || {
        std::thread::sleep(HELD);
        resume();
    });
    let started = Instant::now();
    let seen = access(&mut drain);
    let waited = started.elapsed();
    let want = access(&mut once);
    watchdog.join().unwrap();

    assert!(waited < HELD - Duration::from_millis(50), "the access waited {waited:?} for a list that does not reach its bytes");
    assert!(seen == want, "the access saw what the list at once does not leave");
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

/// A sixteen-bit tile of 32 by 16 texels loaded from `image`, which need not be aligned.
fn load_from(image: u32) -> Vec<u64> {
    vec![
        (0x3D << 56) | (2 << 51) | (63 << 32) | image as u64,
        (0x35 << 56) | (2 << 51) | (16 << 41),
        (0x34 << 56) | ((31u64 << 2) << 12) | (15 << 2),
        0x29 << 56,
    ]
}

/// A load's first window is read from its first texel aligned down to eight bytes, so a write to those bytes waits for it (Mars_Native.md §6.14).
#[test]
fn site_1_a_bus_write_to_the_bytes_below_an_unaligned_loads_first_texel_waits_for_the_load() {
    after_a_held_list(|_| {}, &load_from(TEXTURE + 0x104), |m| {
        m.bus.write32(TEXTURE + 0x100, 0xDEAD_BEEF);
        vec![]
    });
}

/// A write below the eight bytes a load's first window reads meets its page and not its bytes, so it goes ahead of the load (Mars_Native.md §6.14).
#[test]
fn site_1_a_bus_write_below_a_loads_first_window_goes_ahead_of_it() {
    ahead_of_a_held_list(|_| {}, &load_from(TEXTURE + 0x104), (TEXTURE + 0xFC) as usize, |m| {
        m.bus.write32(TEXTURE + 0xFC, 0xDEAD_BEEF);
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

/// `picture`'s last line is 7, so the walk's window reads line 10, and a draw there alone still holds the capture (Mars_Native.md §6.14).
#[test]
fn site_8_the_capture_waits_for_a_draw_on_the_last_line_the_walk_reads() {
    let setup = |m: &mut Machine| picture(m);
    for deferred in [false, true] {
        after_a_held_list(setup, &fill_rows(10, 10), |m| {
            let mut out = Scanout::default();
            if deferred {
                scan::present_deferred(&mut m.bus, &mut out);
                out.join();
            } else {
                scan::present_now(&mut m.bus, &mut out);
            }
            out.frame
        });
    }
}

/// A draw below every line the walk reads meets the capture's pages and not its bytes, so the capture goes ahead of it (Mars_Native.md §6.14).
#[test]
fn site_8_the_capture_goes_ahead_of_a_draw_below_the_lines_the_walk_reads() {
    for deferred in [false, true] {
        ahead_of_a_held_list(picture, &fill_rows(12, 20), (FRAMEBUFFER + 11 * WIDTH * 2 - 1) as usize, |m| {
            let mut out = Scanout::default();
            if deferred {
                scan::present_deferred(&mut m.bus, &mut out);
                out.join();
            } else {
                scan::present_now(&mut m.bus, &mut out);
            }
            out.frame
        });
    }
}

/// `picture` at two on the device, or none without one; the scan is the device's and its capture the walk's bytes at one (Mars_Native.md §6.15).
fn on_the_device(m: &mut Machine) -> bool {
    picture(m);
    m.set_scale(2);
    m.set_gpu(true);
    m.bus.dp.multiple.can_scan_out()
}

/// Two deferred scans with nothing drawn between, the second a repeat and not walked, and the picture the first showed.
fn scanned_twice(m: &mut Machine) -> Vec<u8> {
    let mut out = Scanout::default();
    scan::present_deferred(&mut m.bus, &mut out);
    scan::present_deferred(&mut m.bus, &mut out);
    out.join();
    let mut seen = out.frame;
    seen.extend_from_slice(&out.repeated_scans.to_le_bytes());
    seen
}

/// On the device at a multiple the capture still waits for a draw on the last line the walk at one reads, so that the repeat of the next
/// scan is decided over what the list at once leaves (Mars_Native.md §6.15).
#[test]
fn site_8_on_the_device_the_capture_waits_for_a_draw_on_the_last_line_the_walk_reads() {
    if crate::rdp::gpu::GpuDevice::device_names().is_empty() {
        return;
    }
    // Something drawn at the multiple already, or the scan joins the drain to learn whether anything has (Mars_Native.md §6.4.4).
    let setup = |m: &mut Machine| {
        assert!(on_the_device(m));
        hand_over(m, &fill_list(), LIST);
        m.join_rdp();
        assert!(m.bus.dp.scaled_drawn());
    };
    after_a_held_list(setup, &fill_rows(10, 10), scanned_twice);
}

/// On the device at a multiple the capture goes ahead of a draw below the lines the walk at one reads, where C#'s reach and coarse wait,
/// kept on the processor at a multiple, would wait for it; the device's scan is ordered after it by the drain's leader (Mars_Native.md §6.15).
#[test]
fn site_8_on_the_device_the_capture_goes_ahead_of_a_draw_below_the_lines_the_walk_reads() {
    if crate::rdp::gpu::GpuDevice::device_names().is_empty() {
        return;
    }
    let list = fill_rows(12, 20);
    let (mut once, mut drain) = (at_once(), threaded());
    assert!(on_the_device(&mut once) && on_the_device(&mut drain));
    // Something drawn at the multiple already, or the scan joins the drain to learn whether anything has (Mars_Native.md §6.4.4).
    for m in [&mut once, &mut drain] {
        hand_over(m, &fill_list(), LIST);
        m.join_rdp();
        assert!(m.bus.dp.scaled_drawn());
    }
    hand_over(&mut once, &list, LIST);
    drain.bus.dp.threads.as_deref().unwrap().hold();
    hand_over(&mut drain, &list, LIST);
    assert!(drain.bus.dp.marks.marks[((FRAMEBUFFER + 11 * WIDTH * 2 - 1) >> 12) as usize].load(Relaxed) != 0, "the capture's page is not marked, so the case was not reached");

    let resume = drain.bus.dp.threads.as_deref().unwrap().resumer();
    let watchdog = std::thread::spawn(move || {
        std::thread::sleep(HELD);
        resume();
    });
    let (mut seen, mut want) = (Scanout::default(), Scanout::default());
    let started = Instant::now();
    scan::present_deferred(&mut drain.bus, &mut seen);
    let waited = started.elapsed();
    scan::present_deferred(&mut once.bus, &mut want);
    seen.join();
    want.join();
    watchdog.join().unwrap();

    assert!(waited < HELD - Duration::from_millis(50), "the capture waited {waited:?} for a list that does not reach the lines the walk reads");
    assert!(seen.frame == want.frame, "the device's picture is not the one the list at once leaves");
    drain.join_rdp();
    assert!(state(&once) == state(&drain), "the machines part after the list");
}

#[test]
fn site_9_a_cheats_read_waits_for_the_draw_and_its_write_for_the_load() {
    after_a_held_list(|_| {}, &fill_list(), |m| vec![m.bus.cheat_read8(FRAMEBUFFER + 0x101)]);
    after_a_held_list(|_| {}, &scene(0x0102_0304, true, 16, false), |m| {
        m.bus.cheat_write8(TEXTURE + 0x41, 0x5A);
        vec![]
    });
}

/// The host's read and write through `Core`, which cheats and the debugger use, wait for the drain whole (Mars_Native.md §5.6.4).
#[test]
fn site_9_the_hosts_read_and_write_wait_for_the_drain() {
    use crate::ffi::{Core, space};
    for write in [false, true] {
        let (mut once, mut drain) = (Core::new(at_once()), Core::new(threaded()));
        hand_over(&mut once.machine, &fill_list(), LIST);
        drain.machine.bus.dp.threads.as_deref().unwrap().hold();
        hand_over(&mut drain.machine, &fill_list(), LIST);

        let resume = drain.machine.bus.dp.threads.as_deref().unwrap().resumer();
        let watchdog = std::thread::spawn(move || {
            std::thread::sleep(HELD);
            resume();
        });
        let at = FRAMEBUFFER + WIDTH * 2 * 4;
        let access = |c: &mut Core| {
            let mut seen = [0u8; 16];
            if write {
                c.write_memory(space::RDRAM, at, &[0xAB; 16]).unwrap();
            }
            c.read_memory(space::RDRAM, at, &mut seen).unwrap();
            seen
        };
        let started = Instant::now();
        let seen = access(&mut drain);
        let waited = started.elapsed();
        watchdog.join().unwrap();

        assert!(waited >= HELD - Duration::from_millis(50), "the host went ahead of the list after {waited:?}");
        assert!(seen == access(&mut once), "the host saw what the list had not yet left");
        drain.machine.join_rdp();
        assert!(state(&once.machine) == state(&drain.machine), "the machines part after the list");
    }
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
