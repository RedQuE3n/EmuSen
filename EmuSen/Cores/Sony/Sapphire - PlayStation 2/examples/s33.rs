//! Stage S33's runner (Sapphire_Native.md §47): idle skipping on against off on whole machines.
//!
//! The workloads are S32's and S36's (`examples/bench.rs`, `examples/s36.rs`): `boot`, the console started empty and the
//! player's BIOS booting to OSDSYS; `elf <program>`, an EE program entered by the fast boot; `disc <image>`, a disc in
//! the drive and the program its SYSTEM.CNF names entered by the fast boot. A BIOS is handed to the machine by path and
//! never read here; nothing printed holds an address, an instruction or memory: counts, cycles and equality only.
//!
//! - `s33 equal <workload> <bios> <market> [image] <frames> <interpreter|recompiler>`: a machine with idle skipping off
//!   and one with it on, on the tier named, frame by frame: the picture and the sound after every frame and the state
//!   every 60 and at the end, compared; prints the counts of differing frames and what was skipped.
//! - `s33 time <workload> <bios> <market> [image] <frames> <interpreter|recompiler> <on|off> [from]`: one machine, each
//!   frame's `advance` timed; prints the mean and median milliseconds a frame from frame `from`, the frames a second, the
//!   load average around, and the share of the EE's cycles skipped.
//! - `s33 verify <workload> <bios> <market> [image] <frames>`: the recompiled machine with idle skipping on and the
//!   interpreter stepped beside it after every block and through every skipped turn (`Verify::Blocks`, quiet).
//! - `s33 state <workload> <bios> <market> [image] <frames> <more>`: a machine with idle skipping on runs `frames`, its
//!   state is loaded into a second, and both run `more` frames beside a third with idle skipping off that ran all along,
//!   compared after each frame.

use std::time::Instant;

use emusen_native::core::{Core, Kept};
use emusen_native::ffi::StateMachine;
use sapphire::blocks::{Blocks, Verify};
use sapphire::machine::{Machine, Market};
use sapphire::media::{Media, Sectors};

fn market(s: &str) -> Market {
    match s {
        "europe" => Market::Europe,
        "japan" => Market::Japan,
        _ => Market::Americas,
    }
}

/// A machine for the workload on `tier`, idle skipping `idle`, its program waiting for the fast boot where it has one.
fn machine(workload: &str, bios: &str, market_name: &str, image: Option<&str>, tier: &str, idle: bool) -> Machine {
    let ports = [Some("ps2.dualshock2"), None];
    let bios = std::fs::read(bios).expect("the BIOS");
    let market = market(market_name);
    let mut m = match workload {
        "boot" => {
            let mut m = Machine::new(Media::Empty, market, ports);
            m.insert_bios(&bios);
            m
        }
        "elf" | "disc" => {
            let path = image.expect("the workload's image");
            let image = std::fs::read(path).expect("the image");
            match (workload, Media::read(&image).expect("an EE program or a disc")) {
                ("elf", media @ Media::Elf(_)) => {
                    let mut m = Machine::new(media, market, ports);
                    m.insert_bios(&bios);
                    m.program = Some(image);
                    m
                }
                ("disc", Media::Disc(d)) => {
                    let name = d.program.rsplit(['\\', ':']).next().unwrap_or("").split(';').next().unwrap_or("").to_string();
                    let program = Sectors::new(&image).root_file(&name).unwrap_or_else(|| panic!("the disc's root holds no {name}"));
                    let disc = sapphire::cdvd::open(Kept::from(image), Vec::new()).expect("the disc opens");
                    let mut m = Machine::new(Media::Empty, market, ports);
                    m.insert_bios(&bios);
                    m.set_tray(true);
                    m.insert_disc(Some(disc));
                    m.set_tray(false);
                    m.program = Some(program);
                    m
                }
                (w, _) => panic!("{path} is not the image `{w}` takes"),
            }
        }
        other => panic!("no workload {other}"),
    };
    m.sys.board.iop.bus.sio2.slots[0].cards[0] = None;
    match tier {
        "recompiler" => m.blocks = Some(Box::new(Blocks::from_env())),
        "interpreter" => {}
        other => panic!("no tier {other}"),
    }
    m.idle.on = idle;
    m
}

fn state(m: &Machine) -> Vec<u8> {
    let mut v = vec![0; m.state_size()];
    let n = m.save_state(&mut v).expect("the state saves");
    v.truncate(n);
    v
}

fn sound(m: &mut Machine, buf: &mut [i16]) -> Vec<i16> {
    let n = m.drain_audio(buf, usize::MAX);
    buf[..n].to_vec()
}

fn load() -> String {
    std::fs::read_to_string("/proc/loadavg").map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" ")).unwrap_or_default()
}

/// What idle skipping did, against the EE's cycles over the same frames.
fn skipped(m: &Machine, cycles: u64) -> String {
    let k = &m.idle.counts;
    format!(
        "skips {}, EE cycles skipped {} of {} ({:.1} %), steps skipped {}, IOP cycles skipped {} in {} skips, proofs {} of {} attempts, woken {}, refused {:?}",
        k.ee_skips,
        k.ee_cycles,
        cycles,
        100.0 * k.ee_cycles as f64 / cycles.max(1) as f64,
        k.ee_steps,
        k.iop_cycles,
        k.iop_skips,
        k.proofs,
        k.attempts,
        k.woken,
        k.refused
    )
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let mode = a[0].as_str();
    let workload = a[1].as_str();
    let (bios, market_name) = (a[2].as_str(), a[3].as_str());
    let with_image = matches!(workload, "elf" | "disc");
    let image = with_image.then(|| a[4].as_str());
    let rest = &a[if with_image { 5 } else { 4 }..];
    let frames: usize = rest[0].parse().expect("a count of frames");
    let mut buf = vec![0i16; 1 << 16];
    let quiet = workload != "cost";
    match mode {
        "equal" => {
            let tier = rest[1].as_str();
            let mut plain = machine(workload, bios, market_name, image, tier, false);
            let mut fast = machine(workload, bios, market_name, image, tier, true);
            let (mut pictures, mut sounds, mut states, mut checked) = (0, 0, 0, 0);
            let started = Instant::now();
            for f in 0..frames {
                plain.advance();
                fast.advance();
                pictures += usize::from(plain.sys.board.pcrtc.picture.rgba != fast.sys.board.pcrtc.picture.rgba);
                sounds += usize::from(sound(&mut plain, &mut buf) != sound(&mut fast, &mut buf));
                if (f + 1) % 60 == 0 || f + 1 == frames {
                    checked += 1;
                    let differs = state(&plain) != state(&fast);
                    states += usize::from(differs);
                    if differs && !quiet {
                        eprintln!("frame {}: {:?}", f + 1, sapphire::blocks::compare(&plain.sys, &fast.sys, false));
                    }
                }
                if (f + 1) % 100 == 0 {
                    eprintln!("frame {} at {:.0} s: pictures differing {pictures}, sound {sounds}, states {states} of {checked}", f + 1, started.elapsed().as_secs_f64());
                }
            }
            println!("{workload} {tier}: over {frames} frames pictures differing {pictures}, sound differing {sounds}, states differing {states} of {checked}");
            println!("stopped: off {:?}, on {:?}; entered {:?} and {:?}", plain.stop, fast.stop, plain.entered, fast.entered);
            println!("{}", skipped(&fast, fast.sys.board.now));
        }
        "time" => {
            let tier = rest[1].as_str();
            let on = rest[2] == "on";
            let from: usize = rest.get(3).map_or(0, |f| f.parse().expect("a frame"));
            let mut m = machine(workload, bios, market_name, image, tier, on);
            let before = load();
            let mut times = Vec::with_capacity(frames);
            let (mut at, mut counts) = (0, m.idle.counts);
            for f in 0..frames {
                if f == from {
                    (at, counts) = (m.sys.board.now, m.idle.counts);
                }
                let t = Instant::now();
                m.advance();
                times.push(t.elapsed().as_secs_f64() * 1e3);
                let _ = sound(&mut m, &mut buf);
            }
            let window = &times[from.min(times.len())..];
            let mean = window.iter().sum::<f64>() / window.len().max(1) as f64;
            let mut sorted = window.to_vec();
            sorted.sort_by(|x, y| x.partial_cmp(y).unwrap());
            println!(
                "{workload} {tier} idle {}: frames {from}-{frames}, mean {mean:.3} ms, median {:.3} ms, fps {:.3}; load {before} before, {} after",
                rest[2],
                sorted.get(sorted.len() / 2).copied().unwrap_or(0.0),
                1e3 / mean,
                load()
            );
            let k = &m.idle.counts;
            let cycles = m.sys.board.now - at;
            println!(
                "window: EE cycles skipped {} of {cycles} ({:.1} %), skips {}, IOP cycles skipped {}, proofs {} of {} attempts, woken {}",
                k.ee_cycles - counts.ee_cycles,
                100.0 * (k.ee_cycles - counts.ee_cycles) as f64 / cycles.max(1) as f64,
                k.ee_skips - counts.ee_skips,
                k.iop_cycles - counts.iop_cycles,
                k.proofs - counts.proofs,
                k.attempts - counts.attempts,
                k.woken - counts.woken
            );
            println!("whole run: {}", skipped(&m, m.sys.board.now));
        }
        "verify" => {
            let mut fast = machine(workload, bios, market_name, image, "recompiler", true);
            while fast.waiting() && fast.frames < frames as i64 {
                fast.advance();
            }
            let mut twin = machine(workload, bios, market_name, image, "interpreter", false);
            twin.load_state(&state(&fast)).expect("the twin loads the state");
            let mut reference = std::mem::replace(&mut twin.sys, Box::new(sapphire::board::System::new(0)));
            (reference.ee.exceptions, reference.ee.exceptions2) = (fast.sys.ee.exceptions, fast.sys.ee.exceptions2);
            reference.board.iop.tty.device = fast.sys.board.iop.tty.device;
            reference.board.iop.resolve_tty();
            fast.blocks.as_mut().unwrap().verify(Verify::Blocks, reference, quiet);
            let start = fast.frames;
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                while fast.frames < start + frames as i64 {
                    fast.advance();
                    if let Some(r) = fast.blocks.as_mut().unwrap().reference_mut() {
                        r.board.iop.resolve_tty();
                    }
                }
                fast.blocks.as_ref().unwrap().check_all(&fast.sys);
            }));
            let steps = fast.blocks.as_ref().unwrap().verified().unwrap_or(0);
            match r {
                Ok(()) => println!("{workload}: the verifier held over {frames} frames, {steps} steps compared"),
                Err(_) => println!("{workload}: the verifier FAILED after {steps} steps compared"),
            }
            println!("{}", skipped(&fast, fast.sys.board.now));
        }
        "state" => {
            let more: usize = rest[1].parse().expect("a count of frames");
            let mut plain = machine(workload, bios, market_name, image, "recompiler", false);
            let mut first = machine(workload, bios, market_name, image, "recompiler", true);
            for _ in 0..frames {
                plain.advance();
                first.advance();
                let _ = sound(&mut plain, &mut buf);
                let _ = sound(&mut first, &mut buf);
            }
            let mut second = machine(workload, bios, market_name, image, "recompiler", true);
            second.load_state(&state(&first)).expect("the state loads");
            let (mut pictures, mut sounds, mut states) = (0, 0, 0);
            for _ in 0..more {
                plain.advance();
                first.advance();
                second.advance();
                let p = sound(&mut plain, &mut buf);
                let _ = sound(&mut first, &mut buf);
                pictures += usize::from(plain.sys.board.pcrtc.picture.rgba != second.sys.board.pcrtc.picture.rgba);
                sounds += usize::from(p != sound(&mut second, &mut buf));
                let s = state(&plain);
                states += usize::from(s != state(&second)) + usize::from(s != state(&first));
            }
            println!("{workload}: a state at frame {frames} with idle skipping loaded, over {more} frames against the plain run: pictures differing {pictures}, sound {sounds}, states (loaded and carried) {states}");
            println!("{}", skipped(&second, second.sys.board.now));
        }
        other => panic!("no mode {other}"),
    }
}
