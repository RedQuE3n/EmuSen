//! Stage S31's runner (Sapphire_Native.md §41): the debugger on the player's BIOS, digests only, and its cost.
//!
//! `cargo run --release --example s31 -- bios <bios> <americas|europe|japan> <frames>` starts three consoles on a BIOS
//! given by path: one run plainly, one through the debugger's frame with nothing armed, and one with every table armed
//! (calls, stores, loads, interrupts, the profile, every processor's coverage, a breakpoint on each processor that no
//! step meets, watches and data breakpoints over RAM and the scratchpad), each stop resumed at once. Every frame's
//! picture and sound and the state every 60 frames are compared with the plain console's. Only whether they are equal
//! and the times are printed: the tables are cleared unread, and no address, value, program counter or instruction is
//! printed or kept (Sapphire_Plan.md §1.4). The image is handed to the machines and never read here.
//!
//! `cargo run --release --example s31 -- cost <frames>` times the same three ways on a program of the project's form,
//! an EE loop of loads, stores and calls.

use std::time::Instant;

use emusen_native::core::Core;
use emusen_native::debug::{flag, range, run, stop};
use emusen_native::ffi::StateMachine;
use sapphire::debugger::{EE, IOP, VU0, VU1};
use sapphire::machine::{Machine, Market, RAM_ID};
use sapphire::media::Media;

const ORIGIN: u32 = 0x0010_0000;

fn console(market: Market) -> Machine {
    Machine::new(Media::read(&sapphire::media::program(&[0], ORIGIN)).unwrap(), market, [Some("ps2.dualshock2"), None])
}

fn state(m: &Machine) -> Vec<u8> {
    let mut v = vec![0; m.state_size()];
    let n = m.save_state(&mut v).expect("the state saves");
    v.truncate(n);
    v
}

fn arm_everything(m: &mut Machine) {
    let coverage = (0..4).fold(0, |f, n| f | 1 << (flag::COVERAGE + n));
    m.debug.hooks.configure(flag::CALLS | flag::WRITES | flag::READS | flag::INTERRUPTS | flag::PROFILING | coverage, i32::MIN, -1);
    for p in [EE, IOP] {
        m.debug_breakpoints(p, &[-2, -2]);
    }
    for p in [VU0, VU1] {
        m.debug_breakpoints(p, &[4, 4]);
    }
    m.debug.hooks.set_ranges(range::WATCH, &[RAM_ID, 0, 0x1FF_FFFF, 2, 0, 0x3FFF]);
    m.debug.hooks.set_ranges(range::BREAK, &[RAM_ID, 0x1FF_FFF0, 0x1FF_FFFF]);
    m.debug.hooks.set_ranges(range::READ_WATCH, &[RAM_ID, 0, 0x1FF_FFFF, 2, 0, 0x3FFF]);
    m.debug.hooks.set_ranges(range::READ_BREAK, &[2, 0x3FF0, 0x3FFF]);
}

/// A frame through the debugger, each stop resumed at once and every log and coverage map cleared unread.
fn debug_frame(m: &mut Machine) {
    let mut flags = 0;
    loop {
        let why = Core::debug_run_frame(m, flags, &mut 0).unwrap();
        let h = &mut m.debug.hooks;
        h.calls_log.clear();
        h.writes_log.clear();
        h.reads_log.clear();
        h.profile.clear();
        for c in h.coverage.iter_mut().flatten() {
            c.bits.fill(0);
        }
        if why == stop::FRAME {
            return;
        }
        flags = run::UNCHECKED;
    }
}

/// Runs the three consoles `frames` frames; prints the comparisons and the times.
fn compare(mut make: impl FnMut() -> Machine, frames: i64) {
    let (mut plain, mut unarmed, mut armed) = (make(), make(), make());
    arm_everything(&mut armed);
    let mut chunk = vec![0i16; 1 << 16];
    let mut sound = |m: &mut Machine| {
        let n = m.drain_audio(&mut chunk, usize::MAX);
        chunk[..2 * n].to_vec()
    };
    let (mut t_plain, mut t_unarmed, mut t_armed) = (0.0, 0.0, 0.0);
    let (mut pictures, mut heard, mut states, mut compared) = ([0; 2], [0; 2], [0; 2], 0);
    for frame in 1..=frames {
        let t = Instant::now();
        Core::advance(&mut plain, &mut 0).unwrap();
        t_plain += t.elapsed().as_secs_f64();
        let t = Instant::now();
        debug_frame(&mut unarmed);
        t_unarmed += t.elapsed().as_secs_f64();
        let t = Instant::now();
        debug_frame(&mut armed);
        t_armed += t.elapsed().as_secs_f64();
        let s = sound(&mut plain);
        for (k, m) in [&mut unarmed, &mut armed].into_iter().enumerate() {
            pictures[k] += usize::from(m.frame() != plain.frame() || m.frame_info() != plain.frame_info());
            heard[k] += usize::from(sound(m) != s);
        }
        if frame % 60 == 0 || frame == frames {
            let p = state(&plain);
            compared += 1;
            for (k, m) in [&unarmed, &armed].into_iter().enumerate() {
                states[k] += usize::from(state(m) != p);
            }
        }
    }
    let ms = |t: f64| 1e3 * t / frames as f64;
    for (k, name) in ["attached, nothing armed", "every table armed"].into_iter().enumerate() {
        println!("{name}: over {frames} frames pictures differing {}, sound differing {}, states differing {} of {compared}", pictures[k], heard[k], states[k]);
    }
    println!("per frame: plain {:.2} ms, attached and unarmed {:.2} ms ({:+.1}%), every table armed {:.2} ms ({:+.1}%)", ms(t_plain), ms(t_unarmed), 100.0 * (t_unarmed / t_plain - 1.0), ms(t_armed), 100.0 * (t_armed / t_plain - 1.0));
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a[1].as_str() {
        "bios" => {
            let image = std::fs::read(&a[2]).expect("the BIOS");
            let market = match a[3].as_str() {
                "europe" => Market::Europe,
                "japan" => Market::Japan,
                _ => Market::Americas,
            };
            compare(
                || {
                    let mut m = console(market);
                    m.insert_bios(&image);
                    m
                },
                a[4].parse().unwrap(),
            )
        }
        "cost" => {
            let source = "lui s0, 0x20\nlui s1, 0x7000\nloop: jal routine\nnop\nj loop\nnop
routine: lw t1, 0(s0)\naddiu t1, t1, 1\nsw t1, 0(s0)\nsb t1, 0x10(s1)\nlq t2, 0x20(s0)\nsq t2, 0x40(s0)\njr ra\nnop";
            let words = emusen_mips::assemble(source, ORIGIN, &emusen_mips::Dialect::R5900).unwrap();
            compare(
                || {
                    let mut m = console(Market::Americas);
                    let mut sys = sapphire::board::System::new(ORIGIN);
                    sys.board.map.load(ORIGIN, &words);
                    m.sys = Box::new(sys);
                    m.booted = true;
                    m
                },
                a[2].parse().unwrap(),
            )
        }
        other => panic!("no mode {other}"),
    }
}
