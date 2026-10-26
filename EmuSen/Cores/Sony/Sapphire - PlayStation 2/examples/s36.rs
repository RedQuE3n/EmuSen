//! Stage S36's runner (Sapphire_Native.md §44): the EE recompiler against the interpreter on whole machines.
//!
//! The workloads are S32's (`examples/bench.rs` on its branch): `boot`, the console started empty and the player's BIOS
//! booting to OSDSYS; `elf <program>`, an EE program entered by the fast boot; `disc <image>`, a disc in the drive and
//! the program its SYSTEM.CNF names entered by the fast boot; and `cost`, the project's own integer loop on a machine
//! with no BIOS. A BIOS is handed to the machine by path and never read here; nothing printed holds an address, an
//! instruction or memory.
//!
//! - `s36 equal <workload> <bios> <market> [image] <frames>`: an interpreted and a recompiled machine frame by frame, the
//!   picture and the sound after every frame and the state every 60, compared; prints the counts of differing frames.
//! - `s36 time <workload> <bios> <market> [image] <frames> <interpreter|recompiler|c1>`: one machine, each frame's
//!   `advance` timed; prints the mean and median milliseconds a frame, the frames a second and the load average around.
//! - `s36 verify <workload> <bios> <market> [image] <frames>`: the recompiled machine with the interpreter stepped beside
//!   it after every block (`Verify::Blocks`, quiet); prints whether it held and the steps compared.
//! - `s36 state <workload> <bios> <market> [image] <frames> <more>`: a recompiled machine runs `frames`, its state is
//!   loaded into a second recompiled machine with an empty cache, and both run `more` frames, compared after each.
//!
//! For `cost`, `<bios>` and `<market>` are `-`.

use std::time::Instant;

use emusen_native::core::{Core, Kept};
use emusen_native::ffi::StateMachine;
use sapphire::blocks::{Blocks, Verify};
use sapphire::machine::{Machine, Market};
use sapphire::media::{Media, Sectors};

const ORIGIN: u32 = 0x0010_0000;

/// The project's integer loop: arithmetic, the multipliers, shifts, compares and branches, a call, and one load and one
/// store a pass, as an EE program's inner loops are made.
const COST: &str = "
    lui s0, 0x20
    ori s2, zero, 1
    loop: jal routine
    addiu s2, s2, 3
    beq zero, zero, loop
    nop
    routine: ori t9, zero, 24
    inner: addu t0, s2, t9
    xor t1, t0, s2
    sll t2, t1, 3
    subu t3, t2, t0
    mult t4, t3, t1
    sltu t5, t4, t3
    bne t5, zero, skip
    daddu t6, t6, t4
    dsra t6, t6, 1
    skip: mflo t7
    andi t8, t7, 0xff
    addiu t9, t9, -1
    bgtz t9, inner
    or s3, s3, t8
    lw t1, 0(s0)
    addu t1, t1, s3
    jr ra
    sw t1, 0(s0)
";

fn market(s: &str) -> Market {
    match s {
        "europe" => Market::Europe,
        "japan" => Market::Japan,
        _ => Market::Americas,
    }
}

/// A machine for the workload, its program waiting for the fast boot where it has one.
fn machine(workload: &str, bios: &str, market_name: &str, image: Option<&str>) -> Machine {
    let ports = [Some("ps2.dualshock2"), None];
    if workload == "cost" {
        let words = emusen_mips::assemble(COST, ORIGIN, &emusen_mips::Dialect::R5900).expect("the loop assembles");
        let mut m = Machine::new(Media::read(&sapphire::media::program(&[0], ORIGIN)).unwrap(), Market::Americas, ports);
        let mut sys = sapphire::board::System::new(ORIGIN);
        sys.board.map.load(ORIGIN, &words);
        m.sys = Box::new(sys);
        m.booted = true;
        return m;
    }
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
    m
}

fn recompiled(mut m: Machine, fixed_cost: bool) -> Machine {
    let mut b = Blocks::from_env();
    b.fixed_cost |= fixed_cost;
    m.blocks = Some(Box::new(b));
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

fn stats(m: &Machine) -> String {
    match &m.blocks {
        Some(b) => {
            let (compiled, refused, bytes) = b.compiled();
            format!("{:?}; blocks compiled {compiled}, refused {refused}, code {bytes} bytes", b.stats)
        }
        None => "interpreter".into(),
    }
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
    match mode {
        "equal" => {
            let mut plain = machine(workload, bios, market_name, image);
            let mut fast = recompiled(machine(workload, bios, market_name, image), false);
            let (mut pictures, mut sounds, mut states, mut checked) = (0, 0, 0, 0);
            let started = Instant::now();
            for f in 0..frames {
                plain.advance();
                fast.advance();
                pictures += usize::from(plain.sys.board.pcrtc.picture.rgba != fast.sys.board.pcrtc.picture.rgba);
                sounds += usize::from(sound(&mut plain, &mut buf) != sound(&mut fast, &mut buf));
                if (f + 1) % 60 == 0 || f + 1 == frames {
                    checked += 1;
                    states += usize::from(state(&plain) != state(&fast));
                }
                if (f + 1) % 100 == 0 {
                    eprintln!("frame {} at {:.0} s: pictures differing {pictures}, sound {sounds}, states {states} of {checked}", f + 1, started.elapsed().as_secs_f64());
                }
            }
            println!("{workload}: over {frames} frames pictures differing {pictures}, sound differing {sounds}, states differing {states} of {checked}");
            println!("stopped: interpreter {:?}, recompiler {:?}; entered {:?} and {:?}", plain.stop, fast.stop, plain.entered, fast.entered);
            println!("{}", stats(&fast));
        }
        "time" => {
            let tier = rest[1].as_str();
            let m = machine(workload, bios, market_name, image);
            let mut m = match tier {
                "interpreter" => m,
                "recompiler" => recompiled(m, false),
                "c1" => recompiled(m, true),
                other => panic!("no tier {other}"),
            };
            let before = load();
            let mut times = Vec::with_capacity(frames);
            for _ in 0..frames {
                let t = Instant::now();
                m.advance();
                times.push(t.elapsed().as_secs_f64() * 1e3);
                let _ = sound(&mut m, &mut buf);
            }
            let mean = times.iter().sum::<f64>() / times.len() as f64;
            let mut sorted = times.clone();
            sorted.sort_by(|x, y| x.partial_cmp(y).unwrap());
            println!(
                "{workload} {tier}: {frames} frames, mean {mean:.3} ms, median {:.3} ms, fps {:.3}; load {before} before, {} after",
                sorted[sorted.len() / 2],
                1e3 / mean,
                load()
            );
            println!("{}", stats(&m));
        }
        "verify" => {
            let mut fast = recompiled(machine(workload, bios, market_name, image), false);
            // A fast-booted program is entered before the verifier is attached: the entry is the machine's, not a step.
            while fast.waiting() && fast.frames < frames as i64 {
                fast.advance();
            }
            let mut twin = machine(workload, bios, market_name, image);
            twin.load_state(&state(&fast)).expect("the twin loads the state");
            let quiet = workload != "cost";
            let mut reference = std::mem::replace(&mut twin.sys, Box::new(sapphire::board::System::new(0)));
            // What a state does not carry and a frame keeps: the exception counts (the hardware's reports) and the IOP's
            // console hook, whose answers to the program's writes are part of the run.
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
                Err(_) if quiet => println!("{workload}: the verifier FAILED after {steps} steps compared"),
                Err(e) => println!("{workload}: the verifier FAILED after {steps} steps: {:?}", e.downcast_ref::<String>()),
            }
            println!("{}", stats(&fast));
        }
        "state" => {
            let more: usize = rest[1].parse().expect("a count of frames");
            let mut first = recompiled(machine(workload, bios, market_name, image), false);
            for _ in 0..frames {
                first.advance();
                let _ = sound(&mut first, &mut buf);
            }
            let mut second = recompiled(machine(workload, bios, market_name, image), false);
            second.load_state(&state(&first)).expect("the state loads");
            second.frames = first.frames;
            let (mut pictures, mut sounds, mut states) = (0, 0, 0);
            for _ in 0..more {
                first.advance();
                second.advance();
                pictures += usize::from(first.sys.board.pcrtc.picture.rgba != second.sys.board.pcrtc.picture.rgba);
                sounds += usize::from(sound(&mut first, &mut buf) != sound(&mut second, &mut buf));
                states += usize::from(state(&first) != state(&second));
            }
            println!("{workload}: after a state at frame {frames} loaded into an empty cache, over {more} frames pictures differing {pictures}, sound {sounds}, states {states}");
            println!("carried: {}", stats(&first));
            println!("loaded:  {}", stats(&second));
        }
        other => panic!("no mode {other}"),
    }
}
